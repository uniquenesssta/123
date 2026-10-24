import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import ts from "typescript";

/** Exercises production transport and the unchanged typed API facade with an injected native boundary. */
export async function verifyTransportBehavior(root) {
  const passed = [];
  const tick = () => new Promise(resolve => setImmediate(resolve));
  const deferred = () => { let resolve, reject; const promise = new Promise((a, b) => { resolve = a; reject = b; }); return { promise, resolve, reject }; };
  const test = async (name, run) => { await run(); passed.push(name); };
  const fixture = (options = {}) => {
    const cache = new Map(), calls = [], logs = [], issues = [];
    const native = async (command, args) => {
      calls.push([command, args]);
      if (command === "record_runtime_operation") { if (options.logFailure) throw new Error("log unavailable"); logs.push(args.draft); return null; }
      if (command === "record_issue") { issues.push(args.draft); if (options.issueFailure) throw new Error("issue unavailable"); return null; }
      return options.business ? options.business(command, args) : { ok: command };
    };
    function load(file) {
      file = resolve(root, file);
      if (cache.has(file)) return cache.get(file);
      const exports = {}; cache.set(file, exports);
      const output = ts.transpileModule(readFileSync(file, "utf8"), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText;
      new Function("exports", "require", output)(exports, specifier => {
        if (specifier === "@tauri-apps/api/core") return { invoke: native };
        if (specifier === "@tauri-apps/plugin-dialog") return { open: async () => null, save: async () => null };
        assert.ok(specifier.startsWith("."), `Unexpected production dependency: ${specifier}`);
        const target = resolve(dirname(file), specifier);
        return load(existsSync(`${target}.ts`) ? `${target}.ts` : `${target}/index.ts`);
      });
      return exports;
    }
    const platform = load("src/platform/tauri/index.ts");
    const transport = new platform.InvokeTransport(native);
    return { platform, transport, load, calls, logs, issues };
  };
  await test("typed command names exactly preserve all 171 frozen commands and order", async () => {
    const f = fixture(), { COMMAND_NAMES } = f.load("src/platform/tauri/commandName.ts");
    const contract = JSON.parse(readFileSync(resolve(root, "architecture/command-contract.json"), "utf8"));
    assert.deepEqual(COMMAND_NAMES, contract.commands_in_registration_order); assert.equal(new Set(COMMAND_NAMES).size, 171);
    await assert.rejects(f.transport.invoke("rogue_command"), /命令/); assert.equal(f.calls.length, 0);
  });
  await test("success preserves argument and response identity and emits one matching trace pair", async () => {
    const response = { id: "team", nested: { value: 1 } }, args = { teamId: "team" }, f = fixture({ business: async () => response });
    assert.equal(await f.transport.invoke("read_team", args), response);
    assert.equal(f.calls.find(([command]) => command === "read_team")[1], args);
    assert.deepEqual(f.logs.map(log => log.phase), ["started", "completed"]);
    assert.equal(f.logs[0].trace_id, f.logs[1].trace_id); assert.match(f.logs[0].trace_id, /^ipc-/); assert.equal(f.transport.pendingCount, 0);
  });
  await test("null undefined arrays and primitives pass through without response envelopes", async () => {
    for (const value of [null, undefined, 0, false, "text", [1, 2]]) {
      const f = fixture({ business: async () => value }); assert.equal(await f.transport.invoke("bootstrap"), value);
    }
  });
  await test("ordinary backend failures keep the original message and logged-error marker", async () => {
    const original = new Error("database failed"), f = fixture({ business: async () => { throw original; } });
    await assert.rejects(f.transport.invoke("read_team"), error => error.message === original.message && error.name === "LoggedInvokeError" && error.cause === original && f.platform.issueWasLogged(error));
    assert.deepEqual(f.logs.map(log => log.phase), ["started", "failed"]); assert.equal(f.issues.length, 1); assert.equal(f.issues[0].operation, "read_team"); assert.equal(f.transport.pendingCount, 0);
  });
  await test("string and non-Error rejection messages preserve existing normalization", async () => {
    for (const value of ["backend text", null, { code: "existing" }]) {
      const f = fixture({ business: async () => { throw value; } }); await assert.rejects(f.transport.invoke("read_team"), error => error.message === String(value));
    }
  });
  await test("bootstrap failures retain their existing no-issue-recording exception", async () => {
    const f = fixture({ business: async () => { throw new Error("bootstrap failed"); } });
    await assert.rejects(f.transport.invoke("bootstrap"), /bootstrap failed/); assert.equal(f.issues.length, 0); assert.equal(f.logs.at(-1).phase, "failed");
  });
  await test("logging failures never replace a success or the original backend failure", async () => {
    const success = fixture({ logFailure: true }); await success.transport.invoke("bootstrap");
    const failure = fixture({ logFailure: true, issueFailure: true, business: async () => { throw new Error("original"); } });
    await assert.rejects(failure.transport.invoke("read_team"), /original/); assert.equal(failure.transport.pendingCount, 0);
  });
  await test("issue recording failures cannot recurse into another issue call", async () => {
    const f = fixture({ issueFailure: true }); await assert.rejects(f.transport.invoke("record_issue", { draft: {} }), /issue unavailable/);
    assert.equal(f.issues.length, 1); assert.equal(f.logs.filter(log => log.phase === "failed").length, 1);
  });
  await test("request IDs remain unique for concurrent calls with independent lifetimes", async () => {
    const a = deferred(), b = deferred(), f = fixture({ business: command => command === "read_team" ? a.promise : b.promise });
    const one = f.transport.invoke("read_team"), two = f.transport.invoke("read_player"); await tick();
    assert.equal(f.transport.pendingCount, 2); assert.notEqual(f.logs[0].trace_id, f.logs[1].trace_id);
    b.resolve("new"); assert.equal(await two, "new"); assert.equal(f.transport.pendingCount, 1); a.resolve("old"); assert.equal(await one, "old"); assert.equal(f.transport.pendingCount, 0);
  });
  await test("an already aborted request dispatches no command or logging side effect", async () => {
    const f = fixture(), abort = new AbortController(); abort.abort();
    await assert.rejects(f.transport.invoke("read_team", {}, { signal: abort.signal }), f.platform.isInvokeCancelled); assert.equal(f.calls.length, 0); assert.equal(f.transport.pendingCount, 0);
  });
  await test("logical cancellation rejects promptly and drops late native success without another terminal", async () => {
    const pending = deferred(), abort = new AbortController(), f = fixture({ business: () => pending.promise });
    const request = f.transport.invoke("read_team", {}, { signal: abort.signal }); await tick(); abort.abort();
    await assert.rejects(request, f.platform.isInvokeCancelled); assert.equal(f.transport.pendingCount, 0); pending.resolve({ late: true }); await tick();
    assert.deepEqual(f.logs.map(log => log.phase), ["started", "failed"]); assert.equal(f.logs[1].context.cancelled, true); assert.equal(f.issues.length, 0);
  });
  await test("late native rejection after cancellation is consumed without an issue or unhandled rejection", async () => {
    const pending = deferred(), abort = new AbortController(), f = fixture({ business: () => pending.promise });
    const request = f.transport.invoke("read_player", {}, { signal: abort.signal }); await tick(); abort.abort(); await assert.rejects(request, f.platform.isInvokeCancelled);
    pending.reject(new Error("late native failure")); await tick(); assert.equal(f.issues.length, 0); assert.equal(f.logs.filter(log => log.phase !== "started").length, 1);
  });
  await test("cancellation while start logging is pending prevents the business dispatch", async () => {
    const gate = deferred(), calls = [], f = fixture();
    const transport = new f.platform.InvokeTransport(async (command, args) => { calls.push(command); if (command === "record_runtime_operation" && args.draft.phase === "started") await gate.promise; });
    const abort = new AbortController(), request = transport.invoke("read_team", {}, { signal: abort.signal }); await tick(); abort.abort();
    await assert.rejects(request, f.platform.isInvokeCancelled); gate.resolve(); await tick(); assert.ok(!calls.includes("read_team")); assert.equal(transport.pendingCount, 0);
  });
  await test("stop cancels all active results once and rejects later invocations without dispatch", async () => {
    const pending = deferred(), f = fixture({ business: () => pending.promise }); const one = f.transport.invoke("read_team"), two = f.transport.invoke("read_player"); await tick();
    f.transport.stop(); f.transport.stop(); await assert.rejects(one, f.platform.isInvokeCancelled); await assert.rejects(two, f.platform.isInvokeCancelled);
    const before = f.calls.length; await assert.rejects(f.transport.invoke("bootstrap"), f.platform.isInvokeCancelled); await tick(); assert.equal(f.calls.length, before); assert.equal(f.transport.pendingCount, 0);
    pending.resolve(null); await tick(); assert.equal(f.logs.filter(log => log.phase === "completed").length, 0);
  });
  await test("completion before a later abort keeps the successful result and terminal pair", async () => {
    const f = fixture(), abort = new AbortController(); await f.transport.invoke("read_team", {}, { signal: abort.signal }); abort.abort(); await tick();
    assert.deepEqual(f.logs.map(log => log.phase), ["started", "completed"]); assert.equal(f.transport.pendingCount, 0);
  });
  await test("trace summaries redact credentials and omit message bodies without mutating arguments", async () => {
    const f = fixture(), args = { api_key: "private-key", prompt: "private-prompt", nested: { authorization: "Bearer private-bearer", url: "postgresql://user:private-password@localhost/db" }, error: "token=private-token sk-privatevalue123456789" };
    await f.transport.invoke("read_team", args); const log = JSON.stringify(f.logs);
    for (const secret of ["private-key", "private-prompt", "private-bearer", "private-password", "private-token", "sk-privatevalue123456789"]) assert.ok(!log.includes(secret), secret);
    assert.equal(args.api_key, "private-key"); assert.equal(f.calls.find(([command]) => command === "read_team")[1], args); assert.ok(log.includes("contentOmitted"));
  });
  await test("trace limits and cycles cannot turn a successful result into a transport error", async () => {
    const value = { count: 2, ids: ["a", "b"] }; value.self = value;
    const f = fixture({ business: async () => value }); assert.equal(await f.transport.invoke("read_team", { body: "x".repeat(5000), cyclic: value }), value);
    assert.ok(JSON.stringify(f.logs).length < 10000); assert.equal(f.issues.length, 0);
    const hostile = Object.defineProperty({}, "bad", { enumerable: true, get() { throw new Error("getter"); } });
    const g = fixture({ business: async () => hostile }); assert.equal(await g.transport.invoke("read_team"), hostile); assert.equal(g.issues.length, 0);
  });
  await test("typed API facade and UI diagnostics use the sole production adapter", async () => {
    const f = fixture(), client = f.load("src/api/client.ts"); await client.api.readTeam("t1");
    assert.deepEqual(f.calls.find(([command]) => command === "read_team")[1], { teamId: "t1" });
    const trace = client.recordUiOperation("ui_action", "test", { content: "private" }); client.recordUiOperationFailure("test", new Error("failure"), trace); client.recordFrontendDiagnostic("diagnostic", {}); await tick();
    assert.match(trace, /^ui-/); assert.ok(f.logs.some(log => log.phase === "ui_action")); assert.ok(f.logs.some(log => log.phase === "diagnostic"));
    f.platform.stopTauriTransport(); const count = f.calls.length; client.recordFrontendDiagnostic("late"); await tick(); assert.equal(f.calls.length, count);
  });
  await test("local cancellation never invokes the distinct backend AI cancellation command", async () => {
    const f = fixture(), abort = new AbortController(); abort.abort(); await assert.rejects(f.transport.invoke("read_team", {}, { signal: abort.signal }), f.platform.isInvokeCancelled);
    assert.ok(!f.calls.some(([command]) => command === "cancel_api_workspace_request"));
    const client = f.load("src/api/client.ts"); await client.api.cancelApiWorkspaceRequest("business-request-id");
    assert.deepEqual(f.calls.find(([command]) => command === "cancel_api_workspace_request")[1], { requestId: "business-request-id" });
  });
  await test("AbortSignal registrations are released exactly once on success failure cancel and stop", async () => {
    for (const mode of ["success", "failure", "cancel", "stop"]) {
      const pending = deferred(), f = fixture({ business: () => pending.promise });
      const signal = new EventTarget(); signal.aborted = false; let added = 0, removed = 0;
      const add = signal.addEventListener.bind(signal), remove = signal.removeEventListener.bind(signal);
      signal.addEventListener = (...args) => { added++; add(...args); }; signal.removeEventListener = (...args) => { removed++; remove(...args); };
      const request = f.transport.invoke("read_team", {}, { signal }); await tick();
      if (mode === "success") { pending.resolve(1); await request; }
      else if (mode === "failure") { pending.reject(new Error("failure")); await assert.rejects(request, /failure/); }
      else { if (mode === "cancel") { signal.aborted = true; signal.dispatchEvent(new Event("abort")); } else f.transport.stop(); await assert.rejects(request, f.platform.isInvokeCancelled); pending.resolve(null); }
      await tick(); assert.equal(added, 1); assert.equal(removed, 1); assert.equal(f.transport.pendingCount, 0);
    }
  });
  await test("duplicate request IDs cannot replace an active lease", async () => {
    const f = fixture(), { CancellationRegistry } = f.load("src/platform/tauri/cancellationRegistry.ts"), registry = new CancellationRegistry();
    const lease = registry.begin("same"), pending = deferred(), result = lease.wait(pending.promise);
    assert.throws(() => registry.begin("same"), /ID 已在使用/); assert.equal(registry.size, 1); registry.stop(); await assert.rejects(result, f.platform.isInvokeCancelled); pending.resolve(null); assert.equal(registry.size, 0);
  });
  await test("cancelling one concurrent call leaves the other result deliverable", async () => {
    const pending = deferred(), f = fixture({ business: () => pending.promise }), abort = new AbortController();
    const cancelled = f.transport.invoke("read_team", {}, { signal: abort.signal }), live = f.transport.invoke("read_player"); await tick(); abort.abort();
    await assert.rejects(cancelled, f.platform.isInvokeCancelled); assert.equal(f.transport.pendingCount, 1); pending.resolve("live"); assert.equal(await live, "live"); assert.equal(f.transport.pendingCount, 0);
  });
  await test("cancelling during terminal logging drops delivery and cannot create a second terminal", async () => {
    const gate = deferred(), logs = [], f = fixture();
    const transport = new f.platform.InvokeTransport(async (command, args) => { if (command === "record_runtime_operation") { logs.push(args.draft); if (args.draft.phase === "completed") await gate.promise; } else return "result"; });
    const abort = new AbortController(), request = transport.invoke("read_team", {}, { signal: abort.signal }); await tick(); abort.abort(); await assert.rejects(request, f.platform.isInvokeCancelled); gate.resolve(); await tick();
    assert.deepEqual(logs.map(log => log.phase), ["started", "completed"]); assert.equal(transport.pendingCount, 0);
  });
  await test("error logging redacts secrets while preserving the public rejection message", async () => {
    const message = "Bearer private-secret token=private-token", f = fixture({ business: async () => { throw new Error(message); } });
    await assert.rejects(f.transport.invoke("read_team"), error => error.message === message);
    assert.ok(!JSON.stringify([f.logs, f.issues]).includes("private-secret")); assert.ok(!JSON.stringify([f.logs, f.issues]).includes("private-token"));
  });
  console.log(`R12-08 Tauri Transport: ${passed.length} production request/response/error/trace/cancellation cases PASS`);
  return passed;
}
