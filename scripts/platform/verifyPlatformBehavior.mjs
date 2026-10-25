import assert from "node:assert/strict";
import { readFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import ts from "typescript";

/** Production modules, injected browser/native boundaries; no alternate implementation. */
export async function verifyPlatformBehavior(root) {
  const passed = [];
  const test = async (name, run) => { await run(); passed.push(name); };
  const fixture = (dialog = {}) => {
    const cache = new Map(), calls = [];
    function load(file) {
      file = resolve(root, file);
      if (cache.has(file)) return cache.get(file);
      const exports = {}; cache.set(file, exports);
      const code = ts.transpileModule(readFileSync(file, "utf8"), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText;
      new Function("exports", "require", code)(exports, specifier => {
        if (specifier === "@tauri-apps/api/core") return { invoke: async () => null };
        if (specifier === "@tauri-apps/plugin-dialog") return Object.fromEntries(["open", "save"].map(name => [name, async options => { calls.push([name, options]); return dialog[name]?.(options) ?? null; }]));
        assert.ok(specifier.startsWith("."), `Unexpected dependency: ${specifier}`);
        const target = resolve(dirname(file), specifier);
        return load(existsSync(`${target}.ts`) ? `${target}.ts` : `${target}/index.ts`);
      });
      return exports;
    }
    return { load, calls };
  };
  const memory = () => {
    const values = new Map(), removed = [];
    return { values, removed, getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value), removeItem: key => { removed.push(key); values.delete(key); } };
  };
  const timers = () => {
    let sequence = 0;
    const jobs = new Map(), stale = [];
    return { jobs, stale, setTimeout(fn, ms) { jobs.set(++sequence, { fn, ms }); stale.push(fn); return sequence; }, clearTimeout(id) { jobs.delete(id); }, fire() { const batch = [...jobs.values()]; jobs.clear(); for (const job of batch) job.fn(); } };
  };
  await test("open one file preserves filters/path and native path bytes", async () => {
    const f = fixture({ open: () => "C:\\中文 目录\\file.xlsx" });
    const filters = [{ name: "Excel 工作簿", extensions: ["xlsx"] }];
    assert.equal(await f.load("src/platform/files/index.ts").openFile({ filters, defaultPath: "C:\\目录" }), "C:\\中文 目录\\file.xlsx");
    assert.deepEqual(f.calls, [["open", { filters, defaultPath: "C:\\目录", multiple: false, directory: false }]]);
  });
  await test("save preserves options including default filename", async () => {
    const f = fixture({ save: () => "C:\\out.zip" }), options = { defaultPath: "package.zip", filters: [{ name: "交换包", extensions: ["zip"] }] };
    assert.equal(await f.load("src/platform/files/index.ts").saveFile(options), "C:\\out.zip"); assert.deepEqual(f.calls, [["save", options]]);
  });
  await test("directory selection has single nonrecursive scope", async () => {
    const f = fixture({ open: () => "C:\\folder" }); assert.equal(await f.load("src/platform/files/index.ts").selectDirectory({ defaultPath: "C:\\" }), "C:\\folder");
    assert.deepEqual(f.calls[0], ["open", { defaultPath: "C:\\", multiple: false, directory: true, recursive: false }]);
  });
  await test("native cancel and unexpected array remain null without writes", async () => {
    for (const result of [null, ["a"], undefined]) {
      const f = fixture({ open: () => result, save: () => result }), files = f.load("src/platform/files/index.ts");
      assert.equal(await files.openFile(), null); assert.equal(await files.saveFile(), null); assert.equal(await files.selectDirectory(), null);
    }
  });
  await test("file permission failure keeps original rejection", async () => {
    const error = new Error("native denied"), f = fixture({ open: () => { throw error; }, save: () => { throw error; } }), files = f.load("src/platform/files/index.ts");
    await assert.rejects(files.openFile(), e => e === error); await assert.rejects(files.saveFile(), e => e === error);
  });
  await test("all nine existing API dialog methods preserve native options", async () => {
    const f = fixture({ open: () => "chosen", save: () => "chosen" }), api = f.load("src/api/client.ts").api;
    const cases = [["chooseIssueLogExportFile", "save", "问题日志报告", "txt"], ["chooseExcelImportFile", "open", "Excel 工作簿", "xlsx"], ["chooseExcelExportFile", "save", "Excel 工作簿", "xlsx"], ["chooseJsonExportFile", "save", "JSON 文件", "json"], ["chooseZipImportFile", "open", "AI 比赛交换包", "zip"], ["chooseZipExportFile", "save", "AI 比赛交换包", "zip"], ["chooseAiAnalysisExportFile", "save", "AI 分析包", "zip"], ["chooseAiAnalysisResponseFile", "open", "AI 分析回包", "zip"], ["chooseAiAnalysisResponseTemplateFile", "save", "AI 分析回包模板", "zip"]];
    for (const [method, kind, name, extension] of cases) { assert.equal(await api[method]("unchanged.default"), "chosen"); const expected = { filters: [{ name, extensions: [extension] }], ...(kind === "save" ? { defaultPath: "unchanged.default" } : { multiple: false, directory: false }) }; assert.deepEqual(f.calls.at(-1), [kind, expected]); }
  });
  const { PreferenceStore, preferenceKey, VersionedJsonStore } = fixture().load("src/platform/storage/index.ts");
  await test("preference key namespace preserves existing identifiers", () => {
    for (const suffix of ["theme", "team-query", "player-query", "player-navigation-context", "last-analysis-package-id", "database-reset-complete"]) assert.equal(preferenceKey(suffix), `football-model-platform.${suffix}`);
    assert.throws(() => preferenceKey("")); assert.throws(() => preferenceKey(" bad"));
  });
  await test("raw preferences retain exact theme/package/flag bytes", () => {
    const disk = memory(), store = new PreferenceStore(() => disk);
    store.setItem("theme", "dark"); store.setItem("flag", "1"); assert.equal(store.getItem("theme"), "dark"); assert.equal(store.getItem("flag"), "1"); store.removeItem("flag"); assert.equal(store.getItem("flag"), null);
  });
  await test("legacy object JSON is read without rewriting or metadata", () => {
    const disk = memory(), store = new PreferenceStore(() => disk), raw = '{ "search": "中文", "limit": 50 }'; disk.values.set("query", raw);
    assert.deepEqual(store.readObject("query"), { search: "中文", limit: 50 }); assert.equal(disk.values.get("query"), raw); assert.deepEqual(disk.removed, []);
  });
  await test("corrupt JSON removes only the offending preference", () => {
    const disk = memory(), store = new PreferenceStore(() => disk); disk.values.set("bad", "{bad"); disk.values.set("good", "keep");
    assert.equal(store.readObject("bad"), null); assert.deepEqual(disk.removed, ["bad"]); assert.equal(disk.values.get("good"), "keep");
  });
  await test("valid primitive/array JSON stays preserved and unavailable as object", () => {
    for (const raw of ["null", "[]", '"text"', "12", "false", ""]) { const disk = memory(), store = new PreferenceStore(() => disk); disk.values.set("key", raw); assert.equal(store.readObject("key"), null); assert.equal(disk.values.get("key"), raw); }
  });
  await test("storage unavailable read falls back; strict writes keep original failure", () => {
    const error = new Error("Storage denied"), store = new PreferenceStore(() => { throw error; }); assert.equal(store.readObject("query"), null); assert.throws(() => store.setItem("x", "y"), e => e === error); assert.throws(() => store.getItem("x"), e => e === error);
  });
  await test("JSON serialization/quota failures do not replace previous bytes", () => {
    const disk = memory(), store = new PreferenceStore(() => disk); disk.values.set("key", "before"); const cyclic = {}; cyclic.self = cyclic;
    assert.throws(() => store.writeJson("key", cyclic)); assert.equal(disk.values.get("key"), "before");
    const error = new Error("QuotaExceeded"); const denied = new PreferenceStore(() => ({ ...disk, setItem() { throw error; } })); assert.throws(() => denied.writeJson("key", { value: 1 }), e => e === error);
  });
  const codec = { version: 2, decode: v => typeof v?.name === "string" ? v : null, migrate: (v, version) => version === 1 && typeof v?.label === "string" ? { name: v.label } : null };
  await test("explicit versioned migration validates before one atomic write", () => {
    const disk = memory(), store = new VersionedJsonStore(new PreferenceStore(() => disk)); disk.values.set("key", '{"version":1,"value":{"label":"legacy"}}');
    assert.deepEqual(store.read("key", codec), { status: "migrated", value: { name: "legacy" } }); assert.deepEqual(JSON.parse(disk.values.get("key")), { version: 2, value: { name: "legacy" } });
    assert.equal(store.read("key", codec).status, "loaded");
  });
  await test("future version and invalid migration retain original bytes", () => {
    for (const raw of ['{"version":3,"value":{"name":"future"}}', '{"version":1,"value":{"wrong":true}}', "{broken"]) {
      const disk = memory(), store = new VersionedJsonStore(new PreferenceStore(() => disk)); disk.values.set("key", raw); assert.equal(store.read("key", codec).value, null); assert.equal(disk.values.get("key"), raw); assert.deepEqual(disk.removed, []);
    }
  });
  await test("migration exception and write denial preserve the source", () => {
    const disk = memory(), raw = '{"version":1,"value":{"label":"legacy"}}'; disk.values.set("key", raw);
    const store = new VersionedJsonStore(new PreferenceStore(() => disk)); assert.equal(store.read("key", { ...codec, migrate() { throw Error("migration failed"); } }).status, "failed"); assert.equal(disk.values.get("key"), raw);
    const denied = new VersionedJsonStore(new PreferenceStore(() => ({ ...disk, setItem() { throw Error("quota"); } }))); assert.equal(denied.read("key", codec).status, "failed"); assert.equal(disk.values.get("key"), raw);
  });
  const diagnostics = fixture().load("src/platform/diagnostics/index.ts");
  await test("diagnostics redact secrets without mutating business values", () => {
    const value = { password: "secret-value", content: "private message", link: "postgres://user:pass@server/db", nested: { token: "Bearer supersecret" } }, before = structuredClone(value), summary = JSON.stringify(diagnostics.summarizeLogValue(value));
    for (const secret of ["secret-value", "private message", "user:pass", "supersecret"]) assert.ok(!summary.includes(secret)); assert.deepEqual(value, before);
  });
  await test("diagnostics remain bounded and nonthrowing with cycles/getters", () => {
    const value = {}; value.self = value; assert.doesNotThrow(() => diagnostics.summarizeLogValue(value));
    assert.equal(diagnostics.summarizeLogValue(Object.defineProperty({}, "bad", { enumerable: true, get() { throw Error("getter"); } })), "[UNAVAILABLE]");
    assert.equal(diagnostics.summarizeLogValue(Array.from({ length: 100 }, (_, i) => i)).sample.length, 8); assert.equal(diagnostics.summarizeLogValue("x".repeat(600)).preview.length, 512);
  });
  const { ClipboardAdapter } = fixture().load("src/platform/clipboard/index.ts");
  await test("clipboard writes exact Unicode text with correct receiver", async () => {
    const writes = [], native = { writeText(text) { assert.equal(this, native); writes.push(text); return Promise.resolve(); } }; await new ClipboardAdapter(() => native).writeText("中文\n⚽"); assert.deepEqual(writes, ["中文\n⚽"]);
  });
  await test("clipboard permissions/unavailable reject and never report success", async () => {
    const error = new Error("NotAllowedError"); await assert.rejects(new ClipboardAdapter(() => ({ writeText: () => Promise.reject(error) })).writeText("text"), e => e === error);
    await assert.rejects(new ClipboardAdapter(() => undefined).writeText("text"));
  });
  const { RequestSequence, DebouncedTask, wallTime, monotonicTime } = fixture().load("src/platform/timing/index.ts");
  await test("clock uses milliseconds and monotonic durations", () => { const before = Date.now(); assert.ok(wallTime() >= before); assert.ok(Number.isFinite(monotonicTime())); });
  await test("request sequence isolates old results and independent channels", () => {
    const home = new RequestSequence(), away = new RequestSequence(), id = home.next(), other = away.next(); assert.equal(home.isCurrent(id), true); home.next(); assert.equal(home.isCurrent(id), false); assert.equal(away.isCurrent(other), true); home.invalidate(); assert.equal(home.isCurrent(id + 1), false);
  });
  await test("debounce coalesces into latest callback at original delay", () => {
    const scheduler = timers(), task = new DebouncedTask(scheduler), seen = []; task.schedule(() => seen.push(1), 320); task.schedule(() => seen.push(2), 320); assert.equal(scheduler.jobs.size, 1); assert.equal([...scheduler.jobs.values()][0].ms, 320); scheduler.fire(); assert.deepEqual(seen, [2]); assert.equal(task.pending, false);
  });
  await test("cancelled timer callback cannot run after replacement or disposal", () => {
    const scheduler = timers(), task = new DebouncedTask(scheduler), seen = []; task.schedule(() => seen.push(1), 250); const stale = scheduler.stale[0]; task.schedule(() => seen.push(2), 250); stale(); assert.deepEqual(seen, []); task.cancel(); for (const fn of scheduler.stale) fn(); assert.deepEqual(seen, []); task.schedule(() => seen.push(3), 250); task.destroy(); task.schedule(() => seen.push(4), 250); scheduler.fire(); assert.deepEqual(seen, []); assert.equal(scheduler.jobs.size, 0);
  });
  await test("reentrant debounce keeps the replacement pending", () => {
    const scheduler = timers(), task = new DebouncedTask(scheduler), seen = []; task.schedule(() => { seen.push(1); task.schedule(() => seen.push(2), 10); }, 10); scheduler.fire(); assert.equal(task.pending, true); scheduler.fire(); assert.deepEqual(seen, [1, 2]);
  });
  console.log(`R12-09 Platform: ${passed.length} production files/storage/diagnostics/clipboard/timing cases PASS`);
  return passed;
}
