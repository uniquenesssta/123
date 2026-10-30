import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { createRequire } from "node:module";

/** Runs real Feature classes through their transport/Session boundary ports in the existing reset gate. */
export async function verifyDatabaseWorkflow(root) {
  const ts = createRequire(import.meta.url)("typescript"), cache = new Map();
  function load(file) {
    file = resolve(root, file);
    if (cache.has(file)) return cache.get(file);
    const text = readFileSync(file.replace(/\?raw$/, ""), "utf8");
    if (file.endsWith(".svg?raw")) return { default: text };
    const exports = {}; cache.set(file, exports);
    const code = ts.transpileModule(text, { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText;
    new Function("exports", "require", code)(exports, specifier => {
      assert.ok(specifier.startsWith("."));
      const target = resolve(dirname(file), specifier);
      return load(specifier.endsWith(".svg?raw") ? target : existsSync(target + ".ts") ? target + ".ts" : target + "/index.ts");
    });
    return exports;
  }
  const { DatabaseWorkflow } = load("src/features/database/databaseWorkflow.ts");
  const { DatabaseFeature, databasePage } = load("src/features/database/index.ts");
  const deferred = () => { let resolve; const promise = new Promise(r => { resolve = r; }); return { promise, resolve }; };
  const options = name => ({ connection_url: "postgres://boundary:secret@host/" + name, max_connections: 10, connect_timeout_seconds: 10 });
  const snapshot = name => ({ data: { database_configured: !!name, database_health: name ? { database_name: name, server_version: "16", latency_ms: 4, database_size_bytes: 1024, migration_count: 46 } : null, stats: null, database_url: "postgres://user:do-not-display@host/test", models: [] }, connection_error: null, config_path: "boundary.json" });
  function fixture(overrides = {}) {
    const calls = []; let current = snapshot("a_test");
    const ports = {
      snapshot: () => current,
      configure: async input => { calls.push("configure:" + input.connection_url.split("/").at(-1)); return {}; },
      disconnect: async () => { calls.push("disconnect"); },
      reset: async name => { calls.push("reset:" + name); return {}; },
      refresh: async () => { calls.push("refresh"); current = snapshot("b_test"); return true; },
      completeReset: async () => { calls.push("cleanup"); },
      invalidateConfirmation: () => calls.push("dismiss"),
      ...overrides,
    };
    return { calls, ports, flow: new DatabaseWorkflow(ports), replace: value => { current = value; } };
  }
  let cases = 0;
  const test = async (name, run) => { try { await run(); cases++; } catch (cause) { throw new Error("Database workflow: " + name, { cause }); } };
  await test("slow A cannot activate after queued B or publish an old summary", async () => {
    const gate = deferred(), order = []; let active = null;
    const f = fixture({ configure: async value => { const name = value.connection_url.split("/").at(-1); order.push(name); if (name === "a") await gate.promise; active = name; return {}; }, refresh: async () => { order.push("summary:" + active); return true; } });
    const a = f.flow.connect(options("a")); await Promise.resolve();
    const b = f.flow.connect(options("b")); await Promise.resolve();
    assert.deepEqual(order, ["a"]); gate.resolve();
    assert.equal(await a, false); assert.equal(await b, true);
    assert.deepEqual(order, ["a", "b", "summary:b"]);
  });
  await test("queued obsolete requests are skipped and do not write", async () => {
    const f = fixture(); const a = f.flow.connect(options("a")), b = f.flow.connect(options("b"));
    assert.equal(await a, false); assert.equal(await b, true); assert.deepEqual(f.calls, ["configure:b", "refresh"]);
  });
  await test("obsolete failure does not fail the current request", async () => {
    const gate = deferred(); let first = true;
    const f = fixture({ configure: async () => { if (first) { first = false; await gate.promise; throw Error("A failed"); } return {}; } });
    const a = f.flow.connect(options("a")); await Promise.resolve(); const b = f.flow.connect(options("b")); gate.resolve();
    assert.equal(await a, false); assert.equal(await b, true);
  });
  await test("disconnect is ordered behind an in-flight configure", async () => {
    const gate = deferred(), f = fixture({ configure: async () => { f.calls.push("start"); await gate.promise; return {}; } });
    const a = f.flow.connect(options("a")); await Promise.resolve(); const b = f.flow.disconnect();
    gate.resolve(); await a; await b; assert.deepEqual(f.calls, ["start", "disconnect", "refresh"]);
  });
  await test("connection changes invalidate old reset even with the same database name", async () => {
    const f = fixture(), ticket = f.flow.prepareReset();
    await f.flow.connect(options("a_test"));
    await assert.rejects(f.flow.reset(ticket, ticket.databaseName), /状态已变化/);
    assert.ok(f.calls.includes("dismiss") && !f.calls.some(x => x.startsWith("reset:")));
  });
  await test("a refreshed snapshot invalidates a prepared reset", async () => {
    const f = fixture(), ticket = f.flow.prepareReset(); f.replace(snapshot("a_test"));
    await assert.rejects(f.flow.reset(ticket, "a_test"), /状态已变化/); assert.equal(f.calls.length, 0);
  });
  await test("pending mutation and mismatched names cannot authorize reset", async () => {
    const f = fixture(), ticket = f.flow.prepareReset();
    await assert.rejects(f.flow.reset(ticket, "other_test"), /名称不匹配/);
    const pending = f.flow.connect(options("a")); assert.throws(() => f.flow.prepareReset(), /尚未完成/); await pending;
    assert.ok(!f.calls.some(x => x.startsWith("reset:")));
  });
  await test("reset locks duplicate submissions and connection switches through cleanup", async () => {
    const gate = deferred(), f = fixture({ completeReset: async () => { await gate.promise; f.calls.push("cleanup"); } }), ticket = f.flow.prepareReset();
    const reset = f.flow.reset(ticket, "a_test"); await Promise.resolve();
    await assert.rejects(f.flow.reset(ticket, "a_test"), /重复提交/);
    assert.throws(() => f.flow.disconnect(), /暂不能切换/); assert.throws(() => f.flow.connect(options("b")), /暂不能切换/);
    gate.resolve(); assert.equal(await reset, true); assert.deepEqual(f.calls, ["reset:a_test", "cleanup"]);
  });
  await test("uncertain reset never cleans local data or automatically retries", async () => {
    const f = fixture({ reset: async () => { f.calls.push("reset"); throw Error("response lost"); } }), ticket = f.flow.prepareReset();
    await assert.rejects(f.flow.reset(ticket, "a_test"), /核查数据库实际状态/);
    assert.deepEqual(f.calls, ["reset"]); assert.throws(() => f.flow.prepareReset(), /上次清空结果/);
    await f.flow.connect(options("b")); assert.equal(f.flow.prepareReset().databaseName, "b_test");
  });
  await test("stop drops queued changes and late cleanup", async () => {
    const gate = deferred(), f = fixture({ reset: async () => { f.calls.push("reset"); await gate.promise; return {}; } });
    const reset = f.flow.reset(f.flow.prepareReset(), "a_test"); f.flow.stop(); gate.resolve();
    assert.equal(await reset, false); assert.deepEqual(f.calls, ["reset"]); assert.throws(() => f.flow.disconnect(), /已停止/);
  });
  await test("failed connection clears only secret input and retains safe draft controls", async () => {
    const f = fixture({ configure: async () => { throw Error("failed"); } }), fields = { "database-url": { value: options("a").connection_url }, "max-connections": { value: "15" }, "connect-timeout": { value: "12" } };
    const feature = new DatabaseFeature({ ...f.ports, busy: f => f(), render() { throw Error("failed request rendered success"); }, notify() { throw Error("failed request notified success"); }, confirm() {} });
    await assert.rejects(feature.connect({ querySelector: selector => fields[selector.slice(1)] }), /failed/);
    assert.equal(fields["database-url"].value, ""); assert.equal(fields["max-connections"].value, "15"); assert.equal(fields["connect-timeout"].value, "12");
  });
  await test("read-only view distinguishes unknown from zero and redacts URLs", () => {
    const input = snapshot(null), html = databasePage(input); assert.ok(html.includes("—") && !html.includes("do-not-display"));
    input.data.stats = { players: 0, matches: 0, model_runs: 0, large_counts_are_estimates: false };
    assert.ok(databasePage(input).includes('class="ui-stat-value">0<'));
  });
  await test("configuration success with failed summary read cannot authorize old identity", async () => {
    const f = fixture({ refresh: async () => { throw Error("summary unavailable"); } });
    await assert.rejects(f.flow.connect(options("b")), /summary unavailable/);
    assert.throws(() => f.flow.prepareReset(), /连接状态未确认/);
    assert.ok(!f.calls.some(x => x.startsWith("reset:")));
  });
  await test("duplicate in-flight connection is submitted only once", async () => {
    const gate = deferred(); let writes = 0;
    const f = fixture({ configure: async () => { writes++; await gate.promise; return {}; } });
    const first = f.flow.connect(options("a")); await Promise.resolve();
    assert.equal(await f.flow.connect(options("a")), false); gate.resolve();
    assert.equal(await first, true); assert.equal(writes, 1);
  });
  await test("stop prevents queued connection writes and late refresh", async () => {
    const gate = deferred(); let writes = 0;
    const f = fixture({ configure: async () => { writes++; await gate.promise; return {}; } });
    const first = f.flow.connect(options("a")); await Promise.resolve(); const second = f.flow.connect(options("b"));
    f.flow.stop(); gate.resolve(); assert.equal(await first, false); assert.equal(await second, false);
    assert.equal(writes, 1); assert.equal(f.calls.length, 0);
  });

  console.log(`R13-02 database workflow: ${cases} production behavior cases PASS`);
}
