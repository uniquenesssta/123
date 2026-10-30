import {readProjectSource} from "./styles/projectSource.mjs";
import {fileURLToPath as stylesheetFilePath} from "node:url";
const stylesheetRoot=stylesheetFilePath(new URL("../",import.meta.url));
import { readStylesheet } from "./tokens/tokenStyles.mjs";
import assert from "node:assert/strict";
import { writeFileSync, existsSync, mkdirSync, copyFileSync, readdirSync, mkdtempSync } from "node:fs";
import { resolve, dirname, relative, join } from "node:path";
import { tmpdir } from "node:os";
import { createRequire } from "node:module";
import vm from "node:vm";
import { resolveChromiumExecutable, captureHtmlScreenshot } from "./task-ui-screenshot-tools.mjs";

const root = resolve(import.meta.dirname, "..");
const require = createRequire(import.meta.url);
const ts = require("typescript");
const read = file => readProjectSource(stylesheetRoot,resolve(root, file), "utf8");
const source = ["warning-dto/warningDto", "warning-store/warningStore"].map(file => read(`src/app/warnings/${file}.ts`)).join("\n")
  .replace(/^import .*?from ["'][^"']+["'];?\s*$/gm, "").replaceAll("export ", "");
const runtime = ts.transpileModule(source + "\nglobalThis.warningModules = { WarningStore, safeWarningText, warningDateGroups };", { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None } }).outputText;
const context = vm.createContext({ Date, Object, Map, Set });
vm.runInContext(runtime, context);
const { WarningStore, safeWarningText, warningDateGroups } = context.warningModules;
let now = new Date("2026-10-24T23:59:00+08:00").getTime();
const store = new WarningStore(() => now);
const input = { key: "connection", source: "数据服务", severity: "error", title: "连接失败", message: "请检查连接", target: "database" };
const first = store.record(input);
assert.equal(store.unreadCount, 1);
assert.ok(Object.isFrozen(first));
store.markAllRead();
assert.equal(store.unreadCount, 0);
now += 120_000;
store.record(input);
assert.equal(store.records.length, 1);
assert.equal(store.records[0].occurrences, 2);
assert.equal(store.unreadCount, 1);
assert.equal(store.records[0].firstSeenAt, first.firstSeenAt);
assert.notEqual(store.records[0].lastSeenAt, first.lastSeenAt);
store.observe({ ...input, key: "bootstrap" });
store.markAllRead();
store.observe({ ...input, key: "bootstrap" });
assert.equal(store.unreadCount, 0, "rendering an unchanged source must not create new occurrences");
assert.equal(store.records.find(item => item.key === "bootstrap").occurrences, 1);
store.resolve("bootstrap");
assert.equal(store.records.find(item => item.key === "bootstrap").resolved, true);
store.observe({ ...input, key: "bootstrap" });
assert.equal(store.records.find(item => item.key === "bootstrap").resolved, false);
assert.equal(store.unreadCount, 1);
store.record({ ...input, key: "yesterday", occurredAt: new Date(now - 86_400_000).toISOString() });
const groups = warningDateGroups(store.records, now);
assert.equal(groups[0].label, "今天");
assert.equal(groups[1].label, "昨天");
store.record({ ...input, key: "invalid-date", occurredAt: "invalid" });
assert.ok(Number.isFinite(Date.parse(store.records[0].lastSeenAt)));
const notice = store.notify("已完成", "success");
assert.equal(notice.autoDismiss, true);
store.notify("重要问题", "danger");
assert.equal(store.dismissNotice(notice.id), false, "stale timeout must not dismiss replacement");
assert.equal(store.notice.tone, "danger");
assert.equal(store.notice.autoDismiss, false);
store.notify("查看详情", "info", { label: "问题日志", target: "logs" });
assert.equal(store.notice.autoDismiss, false);
assert.equal(store.beginTask(), true);
assert.equal(store.beginTask(), false, "global task gate remains exclusive");
store.dismissNotice(store.notice.id);
assert.equal(store.taskActive, true, "dismissing feedback must not cancel the task");
store.endTask();
assert.equal(store.taskActive, false);
const secret = 'postgres://user:secret@localhost/db password=hidden api_key="secret-key" Bearer token-value\n at privateStack (a.ts:1)';
const safe = safeWarningText(secret);
assert.ok(!/secret|hidden|secret-key|token-value|privateStack/.test(safe), safe);
assert.ok(!safeWarningText('{"api_key":"sk-proj-testcredential123","password":"json-password"}').includes("testcredential123"));
assert.ok(!safeWarningText("请求失败 sk-proj-barecredential123").includes("barecredential123"));
for (let index = 0; index < 260; index++) store.record({ ...input, key: `bounded:${index}` });
assert.equal(store.records.length, 200, "warning history remains bounded");
console.log("R12-06 Warning Store: identity, recurrence/read state, dates, recovery, task gate, stale notice, redaction and bounded history PASS");

const main = read("src/main.ts");
assert.ok(main.includes("warningCenter.runTask(operation)") && !/let busy\b|querySelector[^\n]*#(?:busy|toast)/.test(main), "main must use the sole warning/task owner");
assert.ok(!read("src/app/shell/application-shell/applicationShellView.ts").includes('id="toast"'), "Shell must not recreate the feedback slot on redraw");
assert.ok(!read("src/components/databaseSetup.ts").includes('class="alert error"'), "global connection errors must use the unified center");
for (const file of ["src/styles/app.css", "src/styles/layout.css", "src/styles/visualSystem.css"]) assert.ok(!/\.(?:toast|task-activity|busy)[ .:{]/.test(read(file)), `obsolete global feedback CSS remains in ${file}`);
assert.ok(!/setInterval|api\/client|pages\//.test(read("src/app/warnings/warning-center/warningCenterController.ts")), "warning center must not poll or import feature/transport code");
const modules = {};
function bundle(file) {
  if (modules[file]) return;
  const output = ts.transpileModule(read(file), { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }).outputText;
  modules[file] = { output, imports: {} };
  for (const [, specifier] of output.matchAll(/require\("([^"]+)"\)/g)) {
    assert.ok(specifier.startsWith("."), `browser fixture dependency must be local: ${specifier}`);
    const base = resolve(root, dirname(file), specifier);
    if (specifier.endsWith(".svg?raw")) {
      const target = relative(root, base).replaceAll("\\", "/");
      modules[file].imports[specifier] = target;
      modules[target] = { output: `exports.default=${JSON.stringify(readProjectSource(stylesheetRoot,base.replace(/\?raw$/, ""), "utf8"))}`, imports: {} };
      continue;
    }
    const target = relative(root, existsSync(`${base}.ts`) ? `${base}.ts` : join(base, "index.ts")).replaceAll("\\", "/");
    modules[file].imports[specifier] = target; bundle(target);
  }
}
const entry = "src/app/warnings/__tests__/warningBrowserFixture.ts";
bundle(entry);
const browserBundle = `const modules=${JSON.stringify(modules)};const cache={};function load(id){if(cache[id])return cache[id];const record=modules[id],exports={};cache[id]=exports;new Function('exports','require',record.output)(exports,s=>load(record.imports[s]));return exports;}`;
const styles = [...main.matchAll(/^import "(\.\/[^"\n]+\.css)";/gm)].map(match => match[1].replace("./", "src/")).map(file => readStylesheet(root, file).replace(/url\("([^"\n]+\.svg)"\)/g, (_match, asset) => {
  const bytes = readProjectSource(stylesheetRoot,resolve(root, dirname(file), asset));
  assert.ok(bytes.length > 0, `empty local icon asset ${asset}`);
  return `url("data:image/svg+xml;base64,${bytes.toString("base64")}")`;
})).join("\n");
const browser = resolveChromiumExecutable();
assert.ok(browser, "Chromium/Edge/Chrome is required for warning center runtime checks");
const directory = mkdtempSync(join(tmpdir(), "football-warning-center-"));
console.log(`Warning Center fixture files: ${directory}`);
const results = [];
for (const [width, height] of [[960, 600], [1440, 900]]) for (const theme of ["light", "dark"]) {
  const name = `warning-${width}x${height}-${theme}`;
  const htmlPath = join(directory, `${name}.html`);
  writeFileSync(htmlPath, `<!doctype html><html lang="zh-CN" data-theme="${theme}"><head><meta charset="utf-8"><style>${styles}</style></head><body><div id="warning-test-root"></div><script>${browserBundle.replaceAll("</script", "<\\/script")}try{window.warningFixture=load(${JSON.stringify(entry)}).installWarningFixture(${JSON.stringify(theme)});}catch(error){window.warningFixtureError=String(error.stack);}</script></body></html>`);
  const outputPath = join(directory, `${name}.png`);
  const result = await captureHtmlScreenshot({ browser, htmlPath, outputPath, width, height, async exercise(cdp) {
    await cdp.send("Page.bringToFront");
    const evaluate = async expression => {
      const value = await cdp.send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
      if (value.exceptionDetails) throw new Error(value.exceptionDetails.exception?.description ?? value.exceptionDetails.text);
      return value.result?.value;
    };
    const key = async (key, code, vk, modifiers = 0) => {
      await cdp.send("Input.dispatchKeyEvent", { type: "keyDown", key, code, text: key === "Enter" ? "\r" : undefined, unmodifiedText: key === "Enter" ? "\r" : undefined, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk, modifiers });
      await cdp.send("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode: vk, modifiers });
    };
    const initialError = await evaluate("window.warningFixtureError");
    assert.ok(!initialError, initialError);
    await evaluate(`(async()=>{const f=window.warningFixture;f.expect(await f.secondTask,'second operation is rejected');f.finishTask();await f.pending;f.expect(!f.controller.taskActive&&document.querySelector('.ui-task-activity').hidden&&document.querySelector('.ui-task-activity').getAttribute('aria-hidden')==='true','completed task hides activity and ARIA');f.button('open-warning-center').focus();})();`);
    await key("Enter", "Enter", 13);
    await evaluate("window.warningFixture.expect(!!document.querySelector('dialog[open]'),'native Enter opens warning dialog')");
    await key("Tab", "Tab", 9);
    await evaluate("window.warningFixture.expect(document.activeElement===document.querySelector('dialog .ui-icon-button'),'native Tab wraps last to first')");
    await key("Tab", "Tab", 9, 8);
    await evaluate("window.warningFixture.expect(document.activeElement===document.querySelector('dialog [autofocus]'),'native Shift+Tab wraps first to last')");
    await key("Escape", "Escape", 27);
    await evaluate("window.warningFixture.expect(!document.querySelector('dialog[open]')&&document.activeElement===window.warningFixture.button('open-warning-center'),'native Escape closes and restores focus')");
    await evaluate(`window.warningFixture.controller.notify('新的成功通知','success','验收');window.warningFixture.expect(getComputedStyle(document.querySelector('.ui-toast .ui-icon-check')).maskImage.startsWith('url("data:'),'exact downloaded success icon is rendered');`);
    if (width === 960 && theme === "light") {
      await evaluate("document.querySelector('.ui-toast button').focus()");
      await evaluate("new Promise(resolve=>setTimeout(resolve,3400))");
      await evaluate("window.warningFixture.expect(!!document.querySelector('.ui-toast'),'keyboard focus pauses auto-dismiss beyond 3200ms');document.querySelector('#warning-draft').focus()");
      await evaluate("new Promise(resolve=>setTimeout(resolve,3300))");
      await evaluate("window.warningFixture.expect(!document.querySelector('.ui-toast'),'focus leave resumes remaining duration');window.warningFixture.controller.notify('鼠标悬停提示','info','验收')");
      const rect = await evaluate("(()=>{const r=document.querySelector('.ui-toast').getBoundingClientRect();return {x:r.x+30,y:r.y+20}})()");
      await cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: rect.x, y: rect.y });
      await evaluate("new Promise(resolve=>setTimeout(resolve,3400))");
      await evaluate("window.warningFixture.expect(!!document.querySelector('.ui-toast'),'native hover pauses auto-dismiss')");
      await cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: 400, y: 100 });
      await evaluate("window.warningFixture.controller.notify('后来的持续错误','danger','验收');new Promise(resolve=>setTimeout(resolve,3300))");
      await evaluate("window.warningFixture.expect(document.querySelector('.ui-toast').textContent.includes('后来的持续错误'),'old timer cannot hide replacement persistent notice')");
    }
    await evaluate("window.warningFixture.controller.dismissNotice();window.warningFixture.addLongList();window.warningFixture.controller.open()");
    await evaluate("window.warningFixture.expect(document.querySelector('.warning-center-content').scrollHeight>document.querySelector('.warning-center-content').clientHeight,'long list has independent body scrolling')");
    const bounds = await evaluate("(()=>{const r=document.querySelector('.warning-center-content').getBoundingClientRect();return {x:r.x+100,y:r.y+70}})()");
    await cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: bounds.x, y: bounds.y });
    await cdp.send("Input.dispatchMouseEvent", { type: "mouseWheel", x: bounds.x, y: bounds.y, deltaX: 0, deltaY: 240 });
    await evaluate("new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))");
    await evaluate(`(()=>{const f=window.warningFixture;const dialog=document.querySelector('dialog'),body=document.querySelector('.warning-center-content'),footer=document.querySelector('.ui-dialog-actions');f.expect(body.scrollTop>0&&window.scrollY===0,'native wheel scrolls dialog body only');f.expect(footer.getBoundingClientRect().bottom<=innerHeight-10&&dialog.scrollWidth<=dialog.clientWidth+2,'footer stays visible and long text does not overflow');const draft=document.querySelector('#warning-draft').value;const read=f.button('read-all-warnings');read.focus();read.click();f.expect(f.controller.unreadCount===0&&document.activeElement===dialog.querySelector('[autofocus]'),'read-all keeps focus on an enabled control');f.button('warning-open-logs').click();f.expect(f.navigation.at(-1)==='logs'&&document.querySelector('#warning-draft').value===draft,'logs action has a real adapter destination and preserves draft');f.controller.notify('告警与任务同时存在','danger','验收');f.task=f.controller.runTask(()=>new Promise(resolve=>f.complete=resolve));const toast=document.querySelector('.ui-toast').getBoundingClientRect(),activity=document.querySelector('.ui-task-activity').getBoundingClientRect();f.expect(Math.abs(activity.top-toast.bottom-8)<1,'toast and task activity have accepted 8px separation');})();`);
    const interactionChecks = await evaluate("[...window.warningFixture.checks]");
    await evaluate(`(async()=>{const f=window.warningFixture;f.complete();await f.task;let late;const pending=f.controller.runTask(()=>new Promise(resolve=>late=resolve));f.controller.destroy();f.controller.destroy();f.controller.start();late();await pending;f.expect(!document.querySelector('.warning-center-root'),'destroy releases DOM/listeners/timers and late task completion cannot recreate it');window.warningFixture=load(${JSON.stringify(entry)}).installWarningFixture(${JSON.stringify(theme)});window.warningFixture.finishTask();await window.warningFixture.pending;window.warningFixture.finalSpecimen();})();`);
    return { passed: true, checks: [...interactionChecks, "destroy releases resources and ignores late task completion"] };
  }});
  assert.ok(result?.passed, name);
  results.push({ name, ...result });
  writeFileSync(join(directory, "warning-results.json"), JSON.stringify(results, null, 2));
  console.log(`${name}: ${result.checks.length} actual warning/notice/task/dialog checks PASS`);
}
const evidenceDirectory = resolve(root, "..", "logs", `r12-06-warning-${Date.now()}`);
mkdirSync(evidenceDirectory, { recursive: true });
for (const name of readdirSync(directory).filter(name => name.endsWith(".png") || name === "warning-results.json")) copyFileSync(join(directory, name), join(evidenceDirectory, name));
console.log(`R12-06 browser evidence retained: ${directory}; copied to ${evidenceDirectory}`);
