import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import ts from "typescript";

// Runs the production modules from the existing workspace verification entry.
export async function verifyWorkspaceBehavior(root) {
  const cache = new Map();
  function load(path) {
    const file = resolve(root, path);
    if (cache.has(file)) return cache.get(file);
    const exports = {};
    cache.set(file, exports);
    const output = ts.transpileModule(readFileSync(file, "utf8"), {
      compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
    }).outputText;
    new Function("exports", "require", output)(exports, specifier => {
      assert.ok(specifier.startsWith("."), `Workspace dependency must be local: ${specifier}`);
      const target = resolve(dirname(file), specifier);
      return load(existsSync(`${target}.ts`) ? `${target}.ts` : `${target}/index.ts`);
    });
    return exports;
  }
  const { WorkspaceCoordinator } = load("src/app/workspace/index.ts");
  const { normalizeWorkspaceDocument } = load("src/app/workspace/workspaceVersion.ts");
  const empty = () => ({ schema_version: 1, global: { sidebar_collapsed: false, ui_revision: 5 }, modules: {} });
  const deferred = () => { let resolve, reject; const promise = new Promise((a, b) => { resolve = a; reject = b; }); return { promise, resolve, reject }; };
  const tick = () => new Promise(resolve => setImmediate(resolve));
  const passed = [];
  async function test(name, run) { await run(); passed.push(name); }
  function fixture(overrides = {}, options = {}) {
    const calls = [], errors = [];
    const adapter = { read: async () => empty(), save: async document => calls.push(document), clear: async () => empty(), ...overrides };
    return { workspace: new WorkspaceCoordinator(adapter, { onError: failure => errors.push(failure), ...options }), calls, errors };
  }
  await test("malformed fields normalize without crashing or admitting prototype keys", async () => {
    const raw = JSON.parse('{"schema_version":1,"global":{"ui_revision":5},"modules":{"teams":{"controls":{"__proto__":{"value":"bad"},"draft":{"value":"ok"},"api_key":{"value":"secret"}},"tabs":"invalid","selected_object_ids":"invalid","internal_scrolls":{"bad":{"top":-9}},"scroll_y":-8,"layout_mode":"unknown"},"__proto__":{}}}');
    const { document } = normalizeWorkspaceDocument(raw);
    assert.equal(document.modules.teams.controls.draft.value, "ok");
    assert.equal(Object.hasOwn(document.modules, "__proto__"), false);
    assert.deepEqual(Object.keys(document.modules.teams.controls), ["draft"]);
    assert.deepEqual(document.modules.teams.tabs, []);
    assert.deepEqual(document.modules.teams.selected_object_ids, []);
    assert.equal(document.modules.teams.scroll_y, 0);
    assert.equal(document.modules.teams.layout_mode, "detail");
  });
  await test("legacy UI revision migration retains drafts and selection and resets layout only", async () => {
    const { document, migrated } = normalizeWorkspaceDocument({ schema_version: 1, global: { ui_revision: 4, sidebar_collapsed: true }, modules: { teams: { controls: { draft: { value: "keep" } }, selected_object_ids: ["team-1"], inspector_collapsed: false, panel_widths: { inspector: 400 }, active_section: "old" } } });
    assert.equal(migrated, true); assert.equal(document.global.sidebar_collapsed, false);
    assert.equal(document.modules.teams.controls.draft.value, "keep");
    assert.deepEqual(document.modules.teams.selected_object_ids, ["team-1"]);
    assert.equal(document.modules.teams.inspector_collapsed, true);
    assert.deepEqual(document.modules.teams.panel_widths, {});
    assert.equal(document.modules.teams.active_section, null);
  });
  await test("future schema and UI revision are rejected without destructive downgrade", async () => {
    assert.throws(() => normalizeWorkspaceDocument({ ...empty(), schema_version: 2 }), /版本/);
    assert.throws(() => normalizeWorkspaceDocument({ ...empty(), global: { ui_revision: 6 } }), /版本/);
  });
  await test("one state owner returns isolated deep snapshots and caps tabs", async () => {
    const f = fixture(); await f.workspace.initialize();
    const patch = { controls: { draft: { value: "keep" } }, tabs: Array.from({ length: 9 }, (_, i) => ({ id: `t${i}`, label: `T${i}` })), internal_scrolls: { body: { left: 4, top: 8 } } };
    f.workspace.patchModule("teams", patch); patch.controls.draft.value = "changed";
    const snapshot = f.workspace.module("teams"); snapshot.controls.draft.value = "changed-again"; snapshot.internal_scrolls.body.top = 999;
    assert.equal(f.workspace.module("teams").controls.draft.value, "keep");
    assert.equal(f.workspace.module("teams").internal_scrolls.body.top, 8);
    assert.equal(f.workspace.module("teams").tabs.length, 6);
    await f.workspace.destroy();
  });
  await test("initialize is idempotent while pending and after successful recovery", async () => {
    const pending = deferred(); let reads = 0;
    const f = fixture({ read: () => { reads++; return pending.promise; } });
    const a = f.workspace.initialize(), b = f.workspace.initialize(); assert.equal(a, b);
    pending.resolve(empty()); await a; await f.workspace.initialize(); assert.equal(reads, 1); await f.workspace.destroy();
  });
  await test("read failure preserves disk and blocks default-state writes until explicit retry", async () => {
    let fail = true, clears = 0;
    const f = fixture({ read: async () => { if (fail) throw new Error("unreadable"); return empty(); }, clear: async () => { clears++; return empty(); } });
    await f.workspace.initialize(); f.workspace.patchModule("teams", { active_section: "local" });
    await assert.rejects(f.workspace.flush(), /读取/);
    assert.equal(clears, 0); assert.equal(f.calls.length, 0); assert.equal(f.errors[0].operation, "read");
    fail = false; await f.workspace.initialize();
    assert.equal(f.workspace.module("teams").active_section, "local"); await f.workspace.destroy(); assert.equal(f.calls.length, 1);
  });
  await test("late initialization success and failure cannot apply, clear or save after stop", async () => {
    for (const fail of [false, true]) {
      const pending = deferred(); let clears = 0;
      const f = fixture({ read: () => pending.promise, clear: async () => { clears++; return empty(); } });
      const started = f.workspace.initialize(), ended = f.workspace.destroy();
      if (fail) pending.reject(new Error("late")); else pending.resolve({ ...empty(), global: { ui_revision: 4 } });
      await started; await ended; assert.equal(clears, 0); assert.equal(f.calls.length, 0); assert.equal(f.errors.length, 0);
    }
  });
  await test("migration save failure retains recovered state and reports a save error", async () => {
    let fail = true;
    const f = fixture({ read: async () => ({ schema_version: 1, global: {}, modules: { teams: { active_tab_id: "keep" } } }), save: async () => { if (fail) throw new Error("disk full"); } });
    await f.workspace.initialize(); assert.equal(f.workspace.module("teams").active_tab_id, "keep");
    assert.equal(f.errors[0].operation, "save"); fail = false; await f.workspace.destroy();
  });
  await test("overlapping flushes serialize and the latest state is saved last", async () => {
    const first = deferred(), writes = [];
    const f = fixture({ save: async doc => { writes.push(doc.modules.teams.active_section); if (writes.length === 1) await first.promise; } });
    await f.workspace.initialize(); f.workspace.patchModule("teams", { active_section: "old" }); const a = f.workspace.flush(); await tick();
    f.workspace.patchModule("teams", { active_section: "new" }); const b = f.workspace.flush(); await tick(); assert.deepEqual(writes, ["old"]);
    first.resolve(); await Promise.all([a, b]); assert.deepEqual(writes, ["old", "new"]); await f.workspace.destroy();
  });
  await test("clear-all is ordered after in-flight writes and invalidates queued old saves", async () => {
    const first = deferred(), calls = [];
    const f = fixture({ save: async doc => { calls.push(`save:${doc.modules.teams?.active_section ?? "empty"}`); if (calls.length === 1) await first.promise; }, clear: async () => { calls.push("clear"); return empty(); } });
    await f.workspace.initialize(); f.workspace.patchModule("teams", { active_section: "old" }); const a = f.workspace.flush(); await tick();
    f.workspace.patchModule("teams", { active_section: "queued" }); const b = f.workspace.flush(); const clearing = f.workspace.clearAll();
    first.resolve(); await Promise.all([a, b, clearing]); assert.deepEqual(calls, ["save:old", "clear"]);
    assert.equal(f.workspace.module("teams").active_section, null); await f.workspace.destroy(); assert.deepEqual(calls, ["save:old", "clear"]);
  });
  await test("edits during clear survive its late response and are persisted after clear", async () => {
    const pending = deferred(), writes = [];
    const f = fixture({ clear: () => pending.promise, save: async doc => writes.push(doc) });
    await f.workspace.initialize(); const clearing = f.workspace.clearAll(); await tick();
    f.workspace.patchModule("players", { active_section: "new-edit" }); pending.resolve(empty()); await clearing; await f.workspace.flush();
    assert.equal(f.workspace.module("players").active_section, "new-edit"); assert.equal(writes.at(-1).modules.players.active_section, "new-edit"); await f.workspace.destroy();
  });
  await test("failed clear leaves the current state intact and a later save remains possible", async () => {
    const f = fixture({ clear: async () => { throw new Error("clear failure"); } }); await f.workspace.initialize(); f.workspace.patchModule("teams", { active_section: "keep" });
    await assert.rejects(f.workspace.clearAll(), /clear failure/); assert.equal(f.workspace.module("teams").active_section, "keep");
    await f.workspace.flush(); assert.equal(f.calls.at(-1).modules.teams.active_section, "keep"); await f.workspace.destroy();
  });
  await test("failed save remains dirty and retries through the same serialized channel", async () => {
    let fail = true;
    const f = fixture({ save: async doc => { if (fail) throw new Error("save failure"); f.calls.push(doc); } });
    await f.workspace.initialize(); f.workspace.patchModule("teams", { active_section: "keep" }); await assert.rejects(f.workspace.flush(), /save failure/);
    fail = false; await f.workspace.flush(); assert.equal(f.calls[0].modules.teams.active_section, "keep"); await f.workspace.destroy();
  });
  await test("current-page reset preserves other modules and global navigation", async () => {
    const f = fixture(); await f.workspace.initialize(); f.workspace.setSidebarCollapsed(true);
    f.workspace.patchModule("teams", { active_section: "reset" }); f.workspace.patchModule("players", { active_section: "keep" }); f.workspace.clear("teams");
    assert.equal(f.workspace.module("teams").active_section, null); assert.equal(f.workspace.module("players").active_section, "keep"); assert.equal(f.workspace.sidebarCollapsed(), true); await f.workspace.destroy();
  });
  await test("destroy is idempotent, saves once and freezes later state mutations", async () => {
    const f = fixture(); await f.workspace.initialize(); f.workspace.setSidebarCollapsed(true);
    const a = f.workspace.destroy(); assert.equal(a, f.workspace.destroy()); await a;
    f.workspace.setSidebarCollapsed(false); f.workspace.patchModule("teams", { active_section: "late" }); f.workspace.clear("players");
    assert.equal(f.workspace.sidebarCollapsed(), true); assert.equal(f.workspace.module("teams").active_section, null); assert.equal(f.calls.length, 1);
  });
  await test("database reset clear then stop prevents unload from recreating stale references", async () => {
    let clears = 0;
    const f = fixture({ clear: async () => { clears++; return empty(); } });
    await f.workspace.initialize(); f.workspace.patchModule("teams", { selected_object_ids: ["deleted-team"] });
    await f.workspace.clearAll(); f.workspace.stop(); f.workspace.patchModule("teams", { selected_object_ids: ["deleted-team"] });
    await f.workspace.flush(); await f.workspace.destroy();
    assert.equal(clears, 1); assert.equal(f.calls.length, 0); assert.deepEqual(f.workspace.module("teams").selected_object_ids, []);
  });
  console.log(`R12-07 Workspace: ${passed.length} state/version/persistence/concurrency/lifecycle cases PASS`);
  return passed;
}
