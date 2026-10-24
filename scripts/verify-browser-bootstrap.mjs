import { existsSync, readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { createRequire } from "node:module";
import assert from "node:assert/strict";

const root = resolve(import.meta.dirname, "..");
const read = (path) => readFileSync(resolve(root, path), "utf8").replaceAll("\r\n", "\n");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const requiredFiles = [
  "src/bootstrap/main.ts",
  "src/bootstrap/startApplication.ts",
  "src/bootstrap/createApplication.ts",
  "src/bootstrap/registerApplicationModules.ts",
  "src/bootstrap/applicationHandle.ts",
];
for (const path of requiredFiles) check(existsSync(resolve(root, path)), `缺少浏览器组合根文件：${path}`);

const indexHtml = read("index.html");
const bootstrapMain = read("src/bootstrap/main.ts");
const startApplication = read("src/bootstrap/startApplication.ts");
const createApplication = read("src/bootstrap/createApplication.ts");
const registerModules = read("src/bootstrap/registerApplicationModules.ts");
const applicationHandle = read("src/bootstrap/applicationHandle.ts");
const lifecycleFiles = ["mountHandle", "eventScope", "asyncScope", "disposable", "lifecycleError", "index"];
for (const name of lifecycleFiles) {
  const file = `src/app/lifecycle/${name}.ts`;
  check(existsSync(resolve(root, file)), `R12-02 缺少生命周期模块：${file}`);
  const source = read(file);
  check(!/from ["'][^"']*(?:features|pages|api\/client|bootstrap)[^"']*["']/.test(source), `R12-02 生命周期不得反向依赖业务：${file}`);
}
check(applicationHandle.includes("new MountHandle(") && applicationHandle.includes("new DisposableScope("), "R12-02 应用启动/销毁必须使用唯一生命周期实现");
check(!applicationHandle.includes("private startTask") && !applicationHandle.includes("destroyModuleSequence"), "R12-02 组合根不得保留第二套生命周期状态机");
const browserApplication = read("src/main.ts");
check(browserApplication.includes("new EventScope()") && browserApplication.includes("new MountHandle("), "R12-02 真实浏览器模块未使用生命周期基础设施");
check(!/window\.(?:setTimeout|setInterval)\(/.test(browserApplication), "R12-02 真实浏览器定时器必须归事件scope");
check(browserApplication.includes("openAiExampleRequests.invalidate()") && browserApplication.includes("request.isCurrent()"), "R12-02 示例解析必须丢弃过期结果");
check(registerModules.includes("runtime?.stop()") && applicationHandle.includes("module.stop?.()"), "R12-02 销毁请求未即时中断真实模块");
const diagnostics = read("src/diagnostics/searchableSelectDiagnostics.ts");
const moduleContract = JSON.parse(read("architecture/module-boundaries.json"));
const stateContract = JSON.parse(read("architecture/state-ownership.json"));

check(indexHtml.includes('src="/src/bootstrap/main.ts"'), "index.html 未切换到浏览器组合根");
check(!indexHtml.includes('src="/src/main.ts"'), "index.html 仍使用旧浏览器入口");
check(bootstrapMain.includes("startApplication"), "bootstrap/main.ts 未调用 startApplication");
check(startApplication.includes("renderStartupFailure"), "顶层启动失败未集中处理");
check(createApplication.includes('querySelector<HTMLDivElement>("#app")'), "createApplication 未拥有根节点解析");
check(registerModules.includes('import("../main")'), "模块注册未通过唯一浏览器应用实现入口");
check(applicationHandle.includes("class ApplicationHandle"), "缺少 ApplicationHandle 生命周期所有者");
check(applicationHandle.includes("[...this.startedModules].reverse()"), "ApplicationHandle 未按逆序销毁模块");
check(applicationHandle.includes("AggregateError"), "ApplicationHandle 未保留启动/销毁复合错误");
check(browserApplication.includes("createBrowserApplicationModule"), "旧 main.ts 未转换为可注册浏览器应用模块");
check(!browserApplication.includes("void initializeApplication().catch"), "旧 main.ts 仍自行启动应用");
check((browserApplication.match(/browserLifecycleController\.signal/g) ?? []).length >= 10, "浏览器全局监听器未纳入生命周期信号");
check(browserApplication.includes("workspaceState.destroy()"), "浏览器销毁未关闭工作区状态生命周期");
check(diagnostics.includes("signal?: AbortSignal"), "搜索选择器诊断监听器不支持生命周期解绑");
check(moduleContract.frontend?.entry?.owner === "src/bootstrap/main.ts", "模块边界契约的前端 owner 未切换");
check(moduleContract.frontend?.entry?.status === "active-composition-owner", "模块边界契约的前端组合根未激活");
const lifecycle = stateContract.states.find((state) => state.id === "browser.lifecycle");
check(lifecycle?.owner === "src/bootstrap/applicationHandle.ts::ApplicationHandle", "browser.lifecycle 唯一 owner 未切换");
check(lifecycle?.transition === null, "browser.lifecycle 仍保留未完成 transition");

for (const path of requiredFiles) {
  const source = read(path);
  check(!/from ["']\.\.\/pages\//.test(source), `${path} 不得直接导入 Feature 页面`);
}

// R12-01 composition boundary BEGIN
{
  const tokens = source => (source.match(/\/\/[^\n]*|\/\*[\s\S]*?\*\/|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|[A-Za-z_$][A-Za-z_0-9$]*|[^\s]/g) ?? [])
    .filter(token => !token.startsWith("//") && !token.startsWith("/*"));
  const code = source => tokens(source).join("");
  const count = (source, value) => source.split(value).length - 1;
  check(code(bootstrapMain) === code('import { startApplication } from "./startApplication"; void startApplication();'), "R12-01 唯一入口只能导入并实际调用一次startApplication，不能空启动或夹带其他职责");
  const modules = [
    ["src/bootstrap/main.ts", bootstrapMain, ["./startApplication"], []],
    ["src/bootstrap/startApplication.ts", startApplication, ["./createApplication", "./applicationHandle"], ["errorMessage", "escapeHtml", "renderStartupFailure", "startApplication"]],
    ["src/bootstrap/createApplication.ts", createApplication, ["./applicationHandle", "./registerApplicationModules"], ["createApplication"]],
    ["src/bootstrap/registerApplicationModules.ts", registerModules, ["./applicationHandle"], ["registerApplicationModules"]],
  ];
  for (const [file, source, imports, functions] of modules) {
    const actualImports = [...source.matchAll(/\bfrom\s*["']([^"']+)["']/g), ...source.matchAll(/\bimport\s*["']([^"']+)["']/g)].map(match => match[1]);
    check(JSON.stringify(actualImports) === JSON.stringify(imports), `R12-01 启动职责不得直接依赖Feature/服务/数据或另一个状态owner：${file}`);
    const dynamicImports = [...source.matchAll(/\bimport\s*\(\s*["']([^"']+)["']\s*\)/g)].map(match => match[1]);
    check(JSON.stringify(dynamicImports) === JSON.stringify(file.endsWith("registerApplicationModules.ts") ? ["../main"] : []) && (source.match(/\bimport\s*\(/g) ?? []).length === dynamicImports.length, `R12-01 动态依赖只能由注册模块加载原真实应用：${file}`);
    check(JSON.stringify([...source.matchAll(/\bfunction\s+([A-Za-z_$][A-Za-z_0-9$]*)\s*\(/g)].map(match => match[1])) === JSON.stringify(functions), `R12-01 启动模块的职责函数集合漂移：${file}`);
    for (const forbidden of ["fetch(", "XMLHttpRequest", "localStorage", "sessionStorage", "addEventListener(", "setTimeout(", "setInterval(", "api."]) check(!code(source).includes(forbidden), `R12-01 启动组合根不得吸收业务I/O或监听器生命周期：${file}/${forbidden}`);
  }
  const creation = code(createApplication);
  const creationOrder = ['documentRoot.querySelector<HTMLDivElement>("#app")', 'if(!root)thrownewError("缺少 #app 根节点")', "newApplicationHandle(root)", "registerApplicationModules(handle)", "returnhandle;"];
  let previous = -1;
  for (const step of creationOrder) { const index = creation.indexOf(step, previous + 1); check(index > previous, `R12-01 必须依次解析真实根、校验、创建唯一handle、注册、返回：${step}`); previous = index; }
  check(count(creation, "newApplicationHandle(") === 1 && count(creation, "registerApplicationModules(") === 1 && count(creation, "returnhandle;") === 1, "R12-01 应用工厂不得创建或注册多份handle");
  const registration = code(registerModules);
  check(JSON.stringify([...registerModules.matchAll(/\bimport\s*\(\s*["']([^"']+)["']\s*\)/g)].map(match => match[1])) === JSON.stringify(["../main"]), "R12-01 必须唯一延迟载入原真实应用，不能导入mock/副本或多个运行时");
  check(registration.indexOf("exportfunctionregisterApplicationModules") < registration.indexOf("letruntime:"), "R12-01 runtime必须归本次handle注册闭包，不得变为全局状态");
  const registrationOrder = ['name:"browser-application"', 'awaitimport("../main")', "module.createBrowserApplicationModule(handle.root)", "awaitruntime.start()", "awaitruntime?.destroy()", "runtime=null", "handle.register(browserApplication)"];
  previous = -1;
  for (const step of registrationOrder) { const index = registration.indexOf(step, previous + 1); check(index > previous, `R12-01 真实模块start/destroy必须按原顺序接通：${step}`); previous = index; }
  check(count(registration, "handle.register(") === 1 && count(registration, "createBrowserApplicationModule(") === 1 && count(registration, "awaitruntime.start()") === 1, "R12-01 单个handle只能注册/创建/启动一次真实应用");
  const startup = code(startApplication);
  const startupOrder = ["handle=createApplication()", "awaithandle.start()", "returnhandle;", "catch(error:unknown)", 'renderStartupFailure(handle?.root??document.querySelector<HTMLElement>("#app"),error)', "returnnull;"];
  previous = -1;
  for (const step of startupOrder) { const index = startup.indexOf(step, previous + 1); check(index > previous, `R12-01 顶层启动必须等待真实start、成功返回原handle、失败投影后返回null：${step}`); previous = index; }
  check(startup.includes('escapeHtml(errorMessage(error))') && startup.includes('if(!root)return;') && startup.includes("error.errors[0]"), "R12-01 顶层错误须保留原聚合首错、HTML转义及无根节点处理");
  const moduleScripts = [...indexHtml.matchAll(/<script\b[^>]*\btype\s*=\s*["']module["'][^>]*>/g)].map(match => match[0]);
  check(moduleScripts.length === 1 && moduleScripts[0].includes('src="/src/bootstrap/main.ts"'), "R12-01 index.html只能载入一个真实组合根入口");
  const collect = directory => readdirSync(resolve(root, directory), { withFileTypes: true }).flatMap(entry => {
    const file = `${directory}/${entry.name}`;
    return entry.isDirectory() ? collect(file) : entry.isFile() && entry.name.endsWith(".ts") ? [file] : [];
  });
  const startupSites = collect("src").flatMap(file => [...tokens(read(file)).filter(token => !token.startsWith('"') && !token.startsWith("'")).join(" ").matchAll(/\bstartApplication \(/g)].map(() => file)).sort();
  check(JSON.stringify(startupSites) === JSON.stringify(["src/bootstrap/main.ts", "src/bootstrap/startApplication.ts"]), "R12-01 startApplication只允许原定义与唯一入口调用，不能旁路或重复自启动");
}
// R12-01 composition boundary END

// R12-02 executes production modules in this existing verification entry. No runner or dependency added.
let lifecycleCases = 0;
async function verifyLifecycleBehavior() {
  const ts = createRequire(import.meta.url)("typescript");
  const cache = new Map();
  const load = path => {
    const file = resolve(root, path);
    if (cache.has(file)) return cache.get(file);
    const exports = {};
    cache.set(file, exports);
    const output = ts.transpileModule(read(file), {
      compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
    }).outputText;
    const require = specifier => {
      assert.ok(specifier.startsWith("."), `Production lifecycle dependency must be local: ${specifier}`);
      const target = resolve(file, "..", specifier);
      return load(existsSync(`${target}.ts`) ? `${target}.ts` : `${target}/index.ts`);
    };
    new Function("exports", "require", output)(exports, require);
    return exports;
  };
  const { MountHandle, EventScope, AsyncScope, DisposableScope, LifecycleError } = load("src/app/lifecycle/index.ts");
  const { ApplicationHandle } = load("src/bootstrap/applicationHandle.ts");
  const { WorkspaceStateStore } = load("src/app/viewState.ts");
  const deferred = () => { let resolve, reject; const promise = new Promise((a, b) => { resolve = a; reject = b; }); return { promise, resolve, reject }; };
  const test = async (name, run) => {
    try { await run(); lifecycleCases++; }
    catch (error) { failures.push(`R12-02 ${name}: ${error.stack ?? error}`); }
  };
  await test("start/destroy single promise and one cleanup", async () => {
    const calls = [];
    const handle = new MountHandle({ start() { calls.push("start"); }, stop() { calls.push("stop"); }, destroy() { calls.push("destroy"); } });
    const start = handle.start(); assert.equal(handle.start(), start); await start;
    const destroy = handle.destroy(); assert.equal(handle.destroy(), destroy); await destroy;
    assert.deepEqual(calls, ["start", "stop", "destroy"]);
    await assert.rejects(handle.start(), LifecycleError);
  });
  await test("destroy before start microtask", async () => {
    const calls = [];
    const handle = new MountHandle({ start() { calls.push("start"); }, destroy() { calls.push("destroy"); } });
    const start = handle.start(); await handle.destroy(); await start;
    assert.deepEqual(calls, ["destroy"]);
    await assert.rejects(handle.start(), LifecycleError);
  });
  await test("destroy waits for partial start, stop is immediate", async () => {
    const pending = deferred(), calls = [];
    const handle = new MountHandle({ async start() { calls.push("start"); await pending.promise; calls.push("settled"); }, stop() { calls.push("stop"); }, destroy() { calls.push("destroy"); } });
    const start = handle.start(); await Promise.resolve();
    const destroy = handle.destroy(); assert.deepEqual(calls, ["start", "stop"]);
    pending.resolve(); await Promise.all([start, destroy]);
    assert.deepEqual(calls, ["start", "stop", "settled", "destroy"]);
  });
  await test("failed start retains primary and every cleanup error", async () => {
    const primary = new Error("start"), cleanup = new Error("cleanup"); let released = 0;
    const handle = new MountHandle({ start() { throw primary; }, destroy() { released++; throw cleanup; } });
    await assert.rejects(handle.start(), error => error instanceof AggregateError && error.errors[0] === primary && error.errors[1] === cleanup);
    await assert.rejects(handle.destroy(), error => error.errors[0] === cleanup);
    assert.equal(released, 1);
  });
  await test("abort listener can reenter destroy without a second teardown task", async () => {
    let reentered, count = 0;
    const handle = new MountHandle({ start() {}, destroy() { count++; } });
    await handle.start();
    handle.signal.addEventListener("abort", () => { reentered = handle.destroy(); });
    const task = handle.destroy(); assert.equal(task, reentered); await task; assert.equal(count, 1);
  });
  await test("stop forbids future start but cleanup remains one-shot", async () => {
    let count = 0;
    const handle = new MountHandle({ start() { throw new Error("must not start"); }, destroy() { count++; } });
    handle.stop(); await assert.rejects(handle.start(), LifecycleError); await handle.destroy(); await handle.destroy();
    assert.equal(count, 1);
  });
  await test("disposal reverse order, failure continuation, unregister and closed registration", async () => {
    const calls = [], fail = new Error("release"); const scope = new DisposableScope();
    scope.defer(() => { calls.push("first"); });
    const unregister = scope.defer(() => { calls.push("removed"); }); unregister(); unregister();
    scope.defer(() => { calls.push("second"); throw fail; });
    scope.add({ dispose() { calls.push("third"); } });
    const task = scope.dispose(); assert.equal(scope.dispose(), task);
    assert.throws(() => scope.defer(() => {}), LifecycleError);
    await assert.rejects(task, error => error.errors[0] === fail);
    assert.deepEqual(calls, ["third", "second", "first"]);
  });
  await test("application starts once, reverse destroy, duplicate names and registration closure", async () => {
    const calls = [], app = new ApplicationHandle({});
    for (const name of ["a", "b"]) app.register({ name, start() { calls.push(`start:${name}`); }, destroy() { calls.push(`destroy:${name}`); } });
    assert.throws(() => app.register({ name: "a" }), /重复注册/);
    const start = app.start(); assert.equal(app.start(), start); await start;
    assert.throws(() => app.register({ name: "c" }), /启动后/);
    const destroy = app.destroy(); assert.equal(app.destroy(), destroy); await destroy;
    assert.deepEqual(calls, ["start:a", "start:b", "destroy:b", "destroy:a"]);
  });
  await test("application destroy stops pending start and never starts next module", async () => {
    const pending = deferred(), calls = [], app = new ApplicationHandle({});
    app.register({ name: "a", async start() { calls.push("start:a"); await pending.promise; }, stop() { calls.push("stop:a"); }, destroy() { calls.push("destroy:a"); } });
    app.register({ name: "b", start() { calls.push("start:b"); }, destroy() { calls.push("destroy:b"); } });
    const start = app.start(); await Promise.resolve(); const destroy = app.destroy();
    assert.deepEqual(calls, ["start:a", "stop:a"]); pending.resolve(); await Promise.all([start, destroy]);
    assert.deepEqual(calls, ["start:a", "stop:a", "destroy:a"]);
  });
  await test("partial failing application module is released before previous modules", async () => {
    const calls = [], app = new ApplicationHandle({}), primary = new Error("partial");
    app.register({ name: "a", start() {}, destroy() { calls.push("a"); } });
    app.register({ name: "b", start() { throw primary; }, destroy() { calls.push("b"); } });
    await assert.rejects(app.start(), error => error === primary); await app.destroy();
    assert.deepEqual(calls, ["b", "a"]);
  });
  await test("stop failure cannot skip other stops or resource release", async () => {
    const calls = [], app = new ApplicationHandle({});
    for (const name of ["a", "b"]) app.register({ name, start() {}, stop() { calls.push(`stop:${name}`); if (name === "b") throw new Error("stop"); }, destroy() { calls.push(`destroy:${name}`); } });
    await app.start(); await assert.rejects(app.destroy(), AggregateError);
    assert.deepEqual(calls, ["stop:b", "stop:a", "destroy:b", "destroy:a"]);
  });
  await test("latest request wins, old finish cannot finish a new request", async () => {
    const scope = new AsyncScope(), calls = [], old = scope.begin(), current = scope.begin();
    assert.ok(old.signal.aborted); assert.ok(current.id > old.id);
    assert.equal(old.commit(() => calls.push("old")), false); old.finish();
    assert.equal(current.commit(() => calls.push("current")), true);
    scope.destroy(); assert.equal(current.commit(() => calls.push("late")), false);
    assert.throws(() => scope.begin(), LifecycleError); assert.deepEqual(calls, ["current"]);
  });
  await test("apply errors remain visible when callback starts another request", async () => {
    const scope = new AsyncScope();
    await assert.rejects(scope.run(async () => 1, () => { scope.begin(); throw new Error("apply"); }), /apply/);
    scope.destroy();
  });
  await test("reversed async completion and stale errors are ignored", async () => {
    const scope = new AsyncScope(), older = deferred(), newer = deferred(), calls = [];
    const a = scope.run(() => older.promise, x => calls.push(x));
    const b = scope.run(() => newer.promise, x => calls.push(x));
    newer.resolve("new"); assert.equal(await b, true); older.reject(new Error("stale")); assert.equal(await a, false);
    await assert.rejects(scope.run(async () => { throw new Error("current"); }, () => {}), /current/);
    assert.deepEqual(calls, ["new"]);
  });
  await test("destroy while request pending drops successful completion", async () => {
    const scope = new AsyncScope(), pending = deferred(); let applied = false;
    const run = scope.run(() => pending.promise, () => { applied = true; }); scope.destroy(); pending.resolve(1);
    assert.equal(await run, false); assert.equal(applied, false);
  });
  await test("event release, once, external abort and closed listener registration", async () => {
    const target = new EventTarget(), scope = new EventScope(), external = new AbortController(); let count = 0;
    const off = scope.listen(target, "ping", () => count++);
    target.dispatchEvent(new Event("ping")); off(); off(); target.dispatchEvent(new Event("ping"));
    scope.listen(target, "ping", () => count++, { once: true });
    target.dispatchEvent(new Event("ping")); target.dispatchEvent(new Event("ping"));
    scope.listen(target, "ping", () => count++, { signal: external.signal }); external.abort();
    scope.listen(target, "ping", () => count++); scope.destroy(); scope.destroy(); target.dispatchEvent(new Event("ping"));
    assert.equal(count, 2); assert.throws(() => scope.listen(target, "ping", () => {}), LifecycleError);
  });
  await test("capture flag is retained for manual event removal", async () => {
    const calls = [], scope = new EventScope(), external = new AbortController(); external.abort();
    const target = { addEventListener(type, callback, options) { calls.push([type, options.capture]); }, removeEventListener(type, callback, capture) { calls.push([type, capture]); } };
    const off = scope.listen(target, "capture", () => {}, { capture: true }); off();
    scope.listen(target, "already-aborted", () => {}, { signal: external.signal }); scope.destroy();
    assert.deepEqual(calls, [["capture", true], ["capture", true]]);
  });
  const originals = Object.fromEntries(["setTimeout", "clearTimeout", "setInterval", "clearInterval", "requestAnimationFrame", "cancelAnimationFrame", "window"].map(k => [k, Object.getOwnPropertyDescriptor(globalThis, k)]));
  const scheduled = new Map(); let nextId = 0;
  try {
    for (const [set, clear] of [["setTimeout", "clearTimeout"], ["setInterval", "clearInterval"], ["requestAnimationFrame", "cancelAnimationFrame"]]) {
      globalThis[set] = callback => { const id = ++nextId; scheduled.set(id, callback); return id; };
      globalThis[clear] = id => { scheduled.delete(id); };
    }
    globalThis.window = { matchMedia: () => ({ matches: false }) };
    await test("all timer types cancelled, queued callbacks inert after destroy", async () => {
      const scope = new EventScope(); let count = 0;
      scope.setTimeout(() => count++, 1); scope.setInterval(() => count++, 1); scope.requestAnimationFrame(() => count++);
      const queued = [...scheduled.values()]; scope.destroy(); assert.equal(scheduled.size, 0);
      queued.forEach(callback => callback(0)); assert.equal(count, 0);
      assert.throws(() => scope.setTimeout(() => {}, 0), LifecycleError);
    });
    await test("manual timer cancellation and completed timeout removed from ownership", async () => {
      const scope = new EventScope(); let count = 0;
      const cancelled = scope.setTimeout(() => count++, 1); scope.clearTimeout(cancelled);
      const interval = scope.setInterval(() => count++, 1); scope.clearInterval(interval);
      const frame = scope.requestAnimationFrame(() => count++); scope.cancelAnimationFrame(frame);
      assert.equal(scheduled.size, 0);
      const id = scope.setTimeout(() => count++, 1), callback = scheduled.get(id); scheduled.delete(id); callback();
      scope.destroy(); assert.equal(count, 1); assert.equal(scheduled.size, 0);
    });
    await test("queued callback cannot fire after manual cancellation", async () => {
      const scope = new EventScope(); let count = 0;
      const timeout = scope.setTimeout(() => count++, 1), interval = scope.setInterval(() => count++, 1), frame = scope.requestAnimationFrame(() => count++);
      const queued = [...scheduled.values()]; scope.clearTimeout(timeout); scope.clearInterval(interval); scope.cancelAnimationFrame(frame);
      queued.forEach(callback => callback(0)); assert.equal(count, 0); scope.destroy();
    });
    await test("workspace late initialize does not clear, migrate, save or apply", async () => {
      const pending = deferred(), calls = [];
      const workspace = new WorkspaceStateStore({ read: () => pending.promise, save: async () => calls.push("save"), clear: async () => { calls.push("clear"); } });
      const initialize = workspace.initialize(); const destroy = workspace.destroy();
      pending.resolve({ schema_version: 1, global: {}, modules: {} }); await initialize; await destroy;
      assert.deepEqual(calls, []);
    });
    await test("workspace stale read failure does not trigger destructive recovery", async () => {
      const pending = deferred(); let cleared = false;
      const workspace = new WorkspaceStateStore({ read: () => pending.promise, save: async () => {}, clear: async () => { cleared = true; } });
      const initialize = workspace.initialize(); await workspace.destroy(); pending.reject(new Error("late")); await initialize;
      assert.equal(cleared, false);
    });
    await test("workspace restore frame is cancelled before focus or scroll can run", async () => {
      const workspace = new WorkspaceStateStore({ read: async () => {}, save: async () => {}, clear: async () => {} });
      let scrolls = 0; globalThis.window.scrollTo = () => { scrolls++; };
      workspace.restore("teams", { querySelectorAll: () => [] }); const queued = [...scheduled.values()];
      await workspace.destroy(); queued.forEach(callback => callback(0)); assert.equal(scrolls, 0); assert.equal(scheduled.size, 0);
    });
    await test("workspace final save once, no delayed save survives destruction", async () => {
      const calls = [], workspace = new WorkspaceStateStore({ read: async () => ({ schema_version: 1, global: { ui_revision: 5 }, modules: {} }), save: async doc => calls.push(doc), clear: async () => { throw new Error("unexpected"); } });
      await workspace.initialize(); workspace.setSidebarCollapsed(true); assert.equal(scheduled.size, 1);
      const destroy = workspace.destroy(); assert.equal(workspace.destroy(), destroy); await destroy;
      assert.equal(scheduled.size, 0); assert.equal(calls.length, 1); assert.equal(calls[0].global.sidebar_collapsed, true);
    });
  } finally {
    for (const [key, descriptor] of Object.entries(originals)) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor); else delete globalThis[key];
    }
  }
}
try { await verifyLifecycleBehavior(); } catch (error) { failures.push(`R12-02 生命周期验证无法执行：${error.stack ?? error}`); }

if (failures.length > 0) {
  console.error("浏览器组合根验证失败：");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}
console.log(`浏览器组合根验证通过：唯一入口、模块注册和生命周期边界；${lifecycleCases}项真实生命周期行为测试通过。`);
