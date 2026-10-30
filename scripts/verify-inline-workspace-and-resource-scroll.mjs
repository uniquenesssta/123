import {readProjectSource} from "./styles/projectSource.mjs";
import {fileURLToPath as stylesheetFilePath} from "node:url";
const stylesheetRoot=stylesheetFilePath(new URL("../",import.meta.url));
import { readStylesheet } from "./tokens/tokenStyles.mjs";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import os from "node:os";
import ts from "typescript";
import { captureHtmlScreenshot, decodePng, resolveChromiumExecutable } from "./task-ui-screenshot-tools.mjs";

const root = process.cwd();
const read = (file) =>
  readProjectSource(stylesheetRoot,path.join(root, file), "utf8").replace(/\r\n?/g, "\n");
const requireTrue = (condition, message) => {
  if (!condition) throw new Error(message);
};

const modal = read("src/app/modal/workspacePanelController.ts");
const shell = JSON.parse(read("contracts/global-visual-system-contract.json")).shell_source_entries.map(read).join("\n");
const main = read("src/main.ts");
const teams = read("src/pages/teams.ts");
const panelCss = read("src/styles/workspacePanels.css") + read("src/ui/layout/content-region/contentRegion.css");
const entityCss = read("src/styles/entityCenter.css");
const runtimeLog = read("src-tauri/src/runtime_log.rs");

requireTrue(!modal.includes("modal-backdrop"), "详情控制器仍在生成遮罩弹窗");
requireTrue(!modal.includes("close-modal") && !modal.includes("confirm-modal") && !main.includes("close-modal") && !main.includes("confirm-modal"), "右侧工作区仍暴露旧弹窗动作语义");
requireTrue(modal.includes('class="workspace-detail-page'), "详情控制器未切换为右侧完整工作区");
requireTrue(modal.includes("back(): void") && modal.includes("forward(): void"), "右侧工作区缺少返回/前进历史");
requireTrue(modal.includes("captureCurrentState(): void") && modal.includes("restoreCurrentState") && modal.includes("bodyScrollTop"), "右侧工作区返回/前进未保留表单草稿与滚动位置");
requireTrue(modal.includes("restore(): void") && main.includes("modal.restore();"), "页面重绘后未恢复右侧工作区");
const shellController = read("src/app/shell/application-shell/applicationShellController.ts");
requireTrue(main.includes("shellController?.handleAction(action)") && shellController.includes('case "workspace-history-back"') && shellController.includes('case "workspace-history-forward"'), "返回/前进按钮未接入唯一Shell事件链");
requireTrue(/async function navigateTo[\s\S]*?navigation\.navigate\(nextPage,[\s\S]*?started\(\)\s*\{\s*modal\.reset\(\);/.test(main), "跨二级页面导航未在切换开始清理旧右侧工作区历史");
requireTrue(shell.includes('action: "workspace-history-back"') && shell.includes('iconButtonView({ action: "workspace-history-forward"'), "顶部栏未提供返回/前进按钮");
requireTrue(read("src/app/shell/page-frame/pageFrameView.ts").includes("mainFrameView(topbar, contentRegionView(options.content, options.navigationPending, options.navigationFailed))") && read("src/ui/layout/content-region/contentRegionView.ts").includes('id="modal-root"'), "右侧工作区根节点未放在主内容区域内");
requireTrue(panelCss.includes(".main-content.workspace-panel-open > .page-container") && panelCss.includes("display: none !important"), "打开右侧工作区时未替换原页面内容");
requireTrue(panelCss.includes(".workspace-detail-body") && panelCss.includes("overflow: auto"), "右侧完整工作区内容不可滚动");

requireTrue(teams.includes("team-directory-only") && teams.includes("team-detail-workspace"), "球队目录与球队详情仍混在同一三栏界面");
requireTrue(teams.includes('data-action="return-team-directory"'), "球队详情缺少返回球队目录入口");
requireTrue(main.includes('case "return-team-directory"'), "返回球队目录按钮未接入状态清理链");
requireTrue(entityCss.includes(".core-workspace-stage > .entity-browser") && entityCss.includes("height: 100%"), "球队/球员主工作区未获得明确可滚动高度");
requireTrue(entityCss.includes(".entity-table-wrap") && /overflow:\s*auto(?:\s*!important)?\s*;/.test(entityCss), "球队/球员表格缺少内部滚动");
requireTrue(entityCss.includes(".player-table-wrap") && entityCss.includes("scrollbar-gutter: stable"), "球员名单未锁定独立滚动与稳定滚动条");
requireTrue(/\.team-directory-only\s*\{[^}]*grid-template-columns:\s*minmax\(0,\s*1fr\)/s.test(entityCss), "球队目录未切换为独立全宽列表");
requireTrue(entityCss.includes('.app-shell[data-current-page="teams"] .team-detail-workspace.inspector-collapsed') && entityCss.includes("grid-column: 1 / -1"), "选中球队后主名单未恢复为全宽工作区");

requireTrue(runtimeLog.includes('"operation_started" | "operation_completed" | "operation_failed"'), "运行日志仍可能压缩命令开始/完成/失败配对");
requireTrue(runtimeLog.includes("operation_lifecycle_events_keep_trace_pairs"), "运行日志缺少链路配对回归测试");
requireTrue(main.includes("api.listFormations(false),"), "球队目录未统一加载完整阵型目录");
requireTrue(!main.includes("formationCatalog = await api.listFormations(false);"), "导入完成后仍存在重复且可能悬挂的阵型目录请求");

requireTrue(main.indexOf("modal.capture();") < main.indexOf("app.innerHTML = renderShell("), "主题或Shell重绘前没有捕获当前详情草稿");
requireTrue(modal.includes("new RouteHistory<WorkspacePanelEntry>()") && !modal.includes("historyIndex"), "详情仍保留第二套历史索引");

// Execute actual detail/history/format modules in the existing Windows browser fixture.
const browser = resolveChromiumExecutable();
if (!browser) throw new Error("未找到Chromium、Chrome或Edge，无法验证详情历史输入与滚动");
const source = [
  read("src/components/format.ts"), read("src/app/router/routeHistory.ts"), read("src/app/modal/visualRecordView.ts"), read("src/app/modal/workspacePanelTypes.ts"), modal,
  read("src/platform/timing/requestSequence.ts"),
  ...["storageKey", "preferenceStore"].map(name => read(`src/platform/storage/${name}.ts`)),
  ...["lifecycleError", "asyncScope", "mountHandle"].map(name => read(`src/app/lifecycle/${name}.ts`)),
  ...["sessionPersistence", "themeState", "bootstrapState", "appSessionState", "appSession"].map(name => read(`src/app/session/${name}.ts`)),
]
  .join("\n").replace(/^import\s+.*?from\s+["'][^"']+["'];?\s*$/gm, "").replaceAll("export ", "");
const javascript = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None } }).outputText;
const html = `<!doctype html><html><head><meta charset="utf-8"><style>
body{margin:0;font:14px system-ui}.test-marker{position:fixed;left:0;top:0;width:20px;height:20px;background:red;z-index:10}
.workspace-detail-body{height:130px;overflow:auto}.content{height:1200px}.workspace-panel-open>.page-container{display:none}
</style></head><body><div class="test-marker"></div><div id="host"></div><script>${javascript}
const host=document.querySelector('#host');
const shell=()=>'<main class="main-content"><button data-action="workspace-history-back" disabled>返回</button><button data-action="workspace-history-forward" disabled>前进</button><div class="page-container">基础页面</div><div id="modal-root"></div></main>';
const markup=()=>'<div class="content"><input id="draft"><input id="checked" type="checkbox"><select id="multi" multiple><option value="a">A</option><option value="b">B</option></select><input id="password" type="password"><input id="api-key"><input id="private-draft" data-workspace-persist="false"><input id="file" type="file"></div>';
const controller=new WorkspacePanelController();
const preferences=new Map([[SESSION_THEME_KEY,'dark']]);
const session=new AppSession({loadBootstrap:async()=>({data:{recent_runs:[]}}),loadRecentRuns:async()=>[],applyTheme(theme){document.documentElement.dataset.theme=theme;document.documentElement.style.colorScheme=theme;}},new SessionPersistence(()=>({getItem:key=>preferences.get(key)??null,setItem(key,value){preferences.set(key,value)}})));
const expect=(ok,message)=>{if(!ok)throw new Error(message)};
const field=id=>document.getElementById(id);
const body=()=>document.querySelector('.workspace-detail-body');
const historyButtons=()=>Array.from(document.querySelectorAll('[data-action="workspace-history-back"], [data-action="workspace-history-forward"]'));
try{
  expect(session.theme==='dark' && document.documentElement.dataset.theme==='dark' && document.documentElement.style.colorScheme==='dark','initial session theme');
  host.innerHTML=shell(); controller.restore();
  expect(historyButtons().every(button=>button.disabled),'empty boundaries');
  controller.showHtml('详情A','测试',markup());
  field('draft').value='未保存输入A'; field('checked').checked=true; field('multi').options[1].selected=true;
  field('password').value='private'; field('api-key').value='private'; field('private-draft').value='private';
  field('draft').focus(); body().scrollTop=437;
  controller.showHtml('详情B','测试',markup()); field('draft').value='未保存输入B'; body().scrollTop=219;
  controller.back();
  expect(field('draft').value==='未保存输入A' && field('checked').checked && field('multi').options[1].selected,'back draft');
  expect(body().scrollTop===437 && document.activeElement===field('draft'),'back scroll/focus');
  expect(!field('password').value && !field('api-key').value && !field('private-draft').value,'sensitive snapshot');
  controller.forward(); expect(field('draft').value==='未保存输入B' && body().scrollTop===219,'forward draft/scroll');
  field('draft').value='主题重绘前的输入'; field('draft').focus(); body().scrollTop=311;
  session.toggleTheme(); controller.capture(); host.innerHTML=shell(); controller.restore();
  expect(field('draft').value==='主题重绘前的输入' && body().scrollTop===311 && document.activeElement===field('draft'),'theme redraw');
  expect(session.theme==='light' && document.documentElement.style.colorScheme==='light' && preferences.get(SESSION_THEME_KEY)==='light','session theme persistence');
  controller.back(); controller.back(); expect(!field('draft') && !document.querySelector('.page-container').hasAttribute('aria-hidden'),'base page');
  controller.forward(); expect(field('draft').value==='未保存输入A','base forward');
  controller.showHtml('详情C','测试',markup()); controller.forward(); expect(document.querySelector('h2').textContent==='详情C','branch truncate');
  controller.reset(); expect(historyButtons().every(button=>button.disabled) && !field('draft'),'navigation reset');
  session.stop(); void session.destroy();
  document.querySelector('.test-marker').style.background='#00ff00'; document.body.dataset.testResult='pass';
}catch(error){document.body.dataset.error=String(error); document.body.insertAdjacentHTML('beforeend','<pre>'+escapeHtml(error.stack)+'</pre>');}
</script></body></html>`;
const directory = fs.mkdtempSync(path.join(os.tmpdir(), "football-detail-history-"));
const htmlPath = path.join(directory, "history.html"), outputPath = path.join(directory, "history.png");
fs.writeFileSync(htmlPath, html);
await captureHtmlScreenshot({ browser, htmlPath, outputPath, width: 700, height: 400 });
const image = decodePng(outputPath), offset = (5 * image.width + 5) * image.channels;
requireTrue(image.pixels[offset] < 40 && image.pixels[offset+1] > 220 && image.pixels[offset+2] < 40, `详情历史浏览器验证失败，检查保留的${htmlPath}与${outputPath}`);
console.log(`右侧内联工作区、资源滚动与日志验证通过；实际浏览器详情A/B/基础页、输入/多选/滚动/焦点、真实会话主题初始化/切换/保存与重绘、敏感排除、分支清栈通过。产物保留：${directory}`);

// R12-05: the same browser entry also mounts the production Shell module tree.
const modules = {};
function bundle(file) {
  if (modules[file]) return;
  const output = ts.transpileModule(read(file), { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }).outputText;
  modules[file] = { output, imports: {} };
  for (const [, specifier] of output.matchAll(/require\("([^"]+)"\)/g)) {
    requireTrue(specifier.startsWith("."), `Shell fixture dependency must be local: ${specifier}`);
    const base = path.resolve(root, path.dirname(file), specifier);
    if (specifier.endsWith(".svg?raw")) {
      const target = path.relative(root, base).replaceAll("\\", "/");
      modules[file].imports[specifier] = target;
      modules[target] = { output: `exports.default=${JSON.stringify(readProjectSource(stylesheetRoot,base.replace(/\?raw$/, ""), "utf8"))}`, imports: {} };
      continue;
    }
    const target = path.relative(root, fs.existsSync(`${base}.ts`) ? `${base}.ts` : path.join(base, "index.ts")).replaceAll("\\", "/");
    modules[file].imports[specifier] = target;
    bundle(target);
  }
}
const entry = "src/app/shell/__tests__/shellBrowserFixture.ts";
bundle(entry);
const shellBundle = `const shellModules=${JSON.stringify(modules)};const shellCache={};function shellRequire(id){if(shellCache[id])return shellCache[id];const record=shellModules[id],exports={};shellCache[id]=exports;new Function('exports','require',record.output)(exports,s=>shellRequire(record.imports[s]));return exports;}`;
const mainStyles = [...main.matchAll(/^import "(\.\/[^"\n]+\.css)";/gm)].map(match => match[1].replace("./", "src/"));
const shellStyles = mainStyles.map(file => readStylesheet(root, file).replace(/url\("([^"\n]+\.svg)"\)/g, (_match, asset) => {
  const bytes = readProjectSource(stylesheetRoot,path.resolve(root, path.dirname(file), asset));
  return `url("data:image/svg+xml;base64,${bytes.toString("base64")}")`;
})).join("\n");
const shellResults = [];
for (const [width, height] of [[960, 600], [1440, 900], [2560, 1440]]) {
  for (const theme of ["light", "dark"]) for (const collapsed of [false, true]) {
    const name = `shell-${width}x${height}-${theme}-${collapsed ? "collapsed" : "expanded"}`;
    const fixturePath = path.join(directory, `${name}.html`);
    fs.writeFileSync(fixturePath, `<!doctype html><html lang="zh-CN" data-theme="${theme}"><head><meta charset="utf-8"><style>${shellStyles}\nbody{margin:0}</style></head><body><div id="shell-host"></div><script>${shellBundle.replaceAll("</script", "<\\/script")}
window.shellFixtureReady=shellRequire(${JSON.stringify(entry)}).installShellFixture(${JSON.stringify(theme)},${collapsed}).then(result=>{window.shellFixture=result;return {passed:true,checks:result.checks}}).catch(error=>({passed:false,error:String(error.stack)}));</script></body></html>`);
    const result = await captureHtmlScreenshot({ browser, htmlPath: fixturePath, outputPath: path.join(directory, `${name}.png`), width, height,
      async exercise(cdp) {
        const response = await cdp.send("Runtime.evaluate", { expression: "window.shellFixtureReady", awaitPromise: true, returnByValue: true });
        const result = response.result?.value ?? { passed: false, error: response.exceptionDetails?.text ?? "no fixture result" };
        if (!result.passed) return result;
        await cdp.send("Page.bringToFront");
        await cdp.send("Runtime.evaluate", { expression: "document.querySelector('#shell-draft').focus();document.querySelector('#shell-draft').select()" });
        await cdp.send("Input.insertText", { text: "真实浏览器输入" });
        const typed = await cdp.send("Runtime.evaluate", { expression: "document.querySelector('#shell-draft').value", returnByValue: true });
        if (typed.result?.value !== "真实浏览器输入") return { passed: false, error: "native text input" };
        result.checks.push("native text input");
        // This point remains inside the workspace and the native headless window at every viewport.
        await cdp.send("Runtime.evaluate", { expression: `window.shellWheelResult = new Promise(resolve => {
          const page = document.querySelector('.page-container');
          const snapshot = () => ({top:page.scrollTop,height:page.scrollHeight,client:page.clientHeight,windowY:window.scrollY});
          const timer = setTimeout(() => resolve(snapshot()), 1000);
          page.addEventListener('scroll', () => requestAnimationFrame(() => { clearTimeout(timer); resolve(snapshot()); }), {once:true});
        });` });
        await cdp.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: 400, y: 300 });
        await cdp.send("Input.dispatchMouseEvent", { type: "mouseWheel", x: 400, y: 300, deltaX: 0, deltaY: 180 });
        const scrolled = await cdp.send("Runtime.evaluate", { expression: "window.shellWheelResult", awaitPromise: true, returnByValue: true });
        if (!(scrolled.result?.value?.top > 0) || scrolled.result.value.windowY !== 0) return { passed: false, error: `native workspace wheel scroll: ${JSON.stringify(scrolled.result?.value)}` };
        result.checks.push("native workspace wheel scroll");
        // Trusted browser keyboard events exercise the native button activation path.
        for (const [key, code, vk] of [["Enter", "Enter", 13], [" ", "Space", 32]]) {
          await cdp.send("Runtime.evaluate", { expression: "Array.from(document.querySelectorAll('[data-action=\"toggle-global-sidebar\"]')).find(button=>!button.closest('[hidden]')).focus()" });
          await cdp.send("Input.dispatchKeyEvent", { type: "keyDown", key, code, text: key === "Enter" ? "\r" : " ", unmodifiedText: key === "Enter" ? "\r" : " ", windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk });
          await cdp.send("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode: vk, nativeVirtualKeyCode: vk });
          const state = await cdp.send("Runtime.evaluate", { expression: "({collapsed:document.querySelector('.secondary-sidebar').hidden,focus:document.activeElement?.dataset?.action,draft:document.querySelector('#shell-draft').value})", returnByValue: true });
          const expectedCollapsed = key === "Enter" ? !collapsed : collapsed;
          if (state.result?.value?.collapsed !== expectedCollapsed) return { passed: false, error: `native ${code} activation failed: ${JSON.stringify(state.result?.value)}` };
          if (state.result?.value?.draft !== "真实浏览器输入") return { passed: false, error: `native ${code} lost typed draft` };
          result.checks.push(`native ${code} activation`);
        }
        // Tab gives the browser a real keyboard modality for the visible focus ring.
        await cdp.send("Runtime.evaluate", { expression: "document.querySelector('.primary-nav-item').focus()" });
        await cdp.send("Input.dispatchKeyEvent", { type: "keyDown", key: "Tab", code: "Tab", windowsVirtualKeyCode: 9 });
        await cdp.send("Input.dispatchKeyEvent", { type: "keyUp", key: "Tab", code: "Tab", windowsVirtualKeyCode: 9 });
        const focus = await cdp.send("Runtime.evaluate", { expression: "({visible:document.activeElement.matches(':focus-visible'),width:getComputedStyle(document.activeElement).outlineWidth})", returnByValue: true });
        if (!focus.result?.value?.visible || focus.result.value.width !== "1px") return { passed: false, error: "native keyboard focus ring" };
        result.checks.push("native Tab focus ring 1px");
        await cdp.send("Runtime.evaluate", { expression: "document.querySelector('[data-action=\"reset-current-workspace\"]').focus()" });
        await cdp.send("Input.dispatchKeyEvent", { type: "keyDown", key: "Enter", code: "Enter", text: "\r", windowsVirtualKeyCode: 13 });
        await cdp.send("Input.dispatchKeyEvent", { type: "keyUp", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13 });
        const dialog = await cdp.send("Runtime.evaluate", { expression: "({open:!!document.querySelector('dialog:modal'),width:document.querySelector('dialog')?.getBoundingClientRect().width,cancel:document.activeElement.matches('footer [data-confirmation-action=cancel]')})", returnByValue: true });
        if (!dialog.result?.value?.open || dialog.result.value.width !== 480 || !dialog.result.value.cancel) return { passed: false, error: `native reset dialog: ${JSON.stringify(dialog.result?.value)}` };
        result.checks.push("native reset dialog 480 and Cancel focus");
        for (let index = 0; index < 3; index++) {
          await cdp.send("Input.dispatchKeyEvent", { type: "keyDown", key: "Tab", code: "Tab", windowsVirtualKeyCode: 9 });
          await cdp.send("Input.dispatchKeyEvent", { type: "keyUp", key: "Tab", code: "Tab", windowsVirtualKeyCode: 9 });
          const inside = await cdp.send("Runtime.evaluate", { expression: "document.querySelector('dialog').contains(document.activeElement)", returnByValue: true });
          if (!inside.result?.value) return { passed: false, error: "native dialog focus escaped" };
        }
        result.checks.push("native dialog Tab focus confinement");
        const dialogScreenshot = await cdp.send("Page.captureScreenshot", { format: "png", fromSurface: true, captureBeyondViewport: false });
        fs.writeFileSync(path.join(directory, `${name}-reset-confirmation.png`), Buffer.from(dialogScreenshot.data, "base64"));
        await cdp.send("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 });
        await cdp.send("Input.dispatchKeyEvent", { type: "keyUp", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 });
        const cancelled = await cdp.send("Runtime.evaluate", { expression: "({closed:!document.querySelector('dialog:modal'),draft:document.querySelector('#shell-draft').value,focus:document.activeElement.dataset.action})", returnByValue: true });
        if (!cancelled.result?.value?.closed || cancelled.result.value.draft !== "真实浏览器输入" || cancelled.result.value.focus !== "reset-current-workspace") return { passed: false, error: `native dialog Escape: ${JSON.stringify(cancelled.result?.value)}` };
        result.checks.push("native Escape cancels reset and restores focus/draft");
        await cdp.send("Runtime.evaluate", { expression: "const probe=document.createElement('div');probe.id='motion-probe';probe.innerHTML=shellRequire('src/app/shell/topbar/serviceStatusView.ts').serviceStatusView(true,true);document.querySelector('.topbar').append(probe)" });
        for (const [value, expected] of [["no-preference", "fip-spinner"], ["reduce", "none"]]) {
          await cdp.send("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value }] });
          const motion = await cdp.send("Runtime.evaluate", { expression: "getComputedStyle(document.querySelector('#motion-probe .ui-icon-spinner')).animationName", returnByValue: true });
          if (motion.result?.value !== expected) return { passed: false, error: `reduced-motion ${value}` };
          result.checks.push(`spinner motion ${value}`);
        }
        await cdp.send("Runtime.evaluate", { expression: "document.querySelector('#motion-probe').remove();document.querySelector('#shell-draft').value='初始输入';document.querySelector('.page-container').scrollTop=0" });
        await cdp.send("Runtime.evaluate", { expression: "document.activeElement.blur(); window.shellFixture.finish()" });
        return result;
      },
    });
    shellResults.push({ name, ...result });
    fs.writeFileSync(path.join(directory, "shell-results.json"), JSON.stringify(shellResults, null, 2));
    requireTrue(result?.passed, `${name}: ${result?.error}. See ${fixturePath}`);
    console.log(`${name}: ${result.checks.length} Shell layout/interaction/lifecycle checks passed`);
  }
}
console.log(`R12-05真实Shell浏览器验证通过：${shellResults.length}个深浅主题/折叠/桌面视口组合，保留截图和结构化结果：${directory}`);
// Reuse the existing Windows artifact location (../logs/**); no workflow or runner added.
const evidenceDirectory = path.resolve(root, "..", "logs", `r12-05-shell-${Date.now()}`);
fs.mkdirSync(evidenceDirectory, { recursive: true });
for (const file of fs.readdirSync(directory).filter(name => name.endsWith(".png") || name === "shell-results.json")) {
  fs.copyFileSync(path.join(directory, file), path.join(evidenceDirectory, file));
}
console.log(`Shell/确认弹窗证据已复制到原Windows日志产物目录：${evidenceDirectory}`);
