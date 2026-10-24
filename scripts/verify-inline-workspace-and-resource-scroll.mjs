import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import os from "node:os";
import ts from "typescript";
import { captureHtmlScreenshot, decodePng, resolveChromiumExecutable } from "./task-ui-screenshot-tools.mjs";

const root = process.cwd();
const read = (file) =>
  fs.readFileSync(path.join(root, file), "utf8").replace(/\r\n?/g, "\n");
const requireTrue = (condition, message) => {
  if (!condition) throw new Error(message);
};

const modal = read("src/app/modal.ts");
const shell = read("src/app/shell.ts");
const main = read("src/main.ts");
const teams = read("src/pages/teams.ts");
const panelCss = read("src/styles/workspacePanels.css");
const entityCss = read("src/styles/entityCenter.css");
const runtimeLog = read("src-tauri/src/runtime_log.rs");

requireTrue(!modal.includes("modal-backdrop"), "详情控制器仍在生成遮罩弹窗");
requireTrue(!modal.includes("close-modal") && !modal.includes("confirm-modal") && !main.includes("close-modal") && !main.includes("confirm-modal"), "右侧工作区仍暴露旧弹窗动作语义");
requireTrue(modal.includes('class="workspace-detail-page'), "详情控制器未切换为右侧完整工作区");
requireTrue(modal.includes("back(): void") && modal.includes("forward(): void"), "右侧工作区缺少返回/前进历史");
requireTrue(modal.includes("captureCurrentState(): void") && modal.includes("restoreCurrentState") && modal.includes("bodyScrollTop"), "右侧工作区返回/前进未保留表单草稿与滚动位置");
requireTrue(modal.includes("restore(): void") && main.includes("modal.restore();"), "页面重绘后未恢复右侧工作区");
requireTrue(main.includes('case "workspace-history-back"') && main.includes('case "workspace-history-forward"'), "返回/前进按钮未接入事件链");
requireTrue(/navigation\.navigate\(nextPage, loadNavigationPage,\s*\{\s*started\(\)\s*\{\s*modal\.reset\(\);/.test(main), "跨二级页面导航未在切换开始清理旧右侧工作区历史");
requireTrue(shell.includes('data-action="workspace-history-back"') && shell.includes('data-action="workspace-history-forward"'), "顶部栏未提供返回/前进按钮");
requireTrue(shell.indexOf('id="modal-root"') < shell.indexOf("</main>"), "右侧工作区根节点未放在主内容区域内");
requireTrue(panelCss.includes(".main-content.workspace-panel-open > .page-container") && panelCss.includes("display: none !important"), "打开右侧工作区时未替换原页面内容");
requireTrue(panelCss.includes(".workspace-detail-body") && panelCss.includes("overflow: auto"), "右侧完整工作区内容不可滚动");

requireTrue(teams.includes("team-directory-only") && teams.includes("team-detail-workspace"), "球队目录与球队详情仍混在同一三栏界面");
requireTrue(teams.includes('data-action="return-team-directory"'), "球队详情缺少返回球队目录入口");
requireTrue(main.includes('case "return-team-directory"'), "返回球队目录按钮未接入状态清理链");
requireTrue(entityCss.includes(".core-workspace-stage > .entity-browser") && entityCss.includes("height: 100%"), "球队/球员主工作区未获得明确可滚动高度");
requireTrue(entityCss.includes(".entity-table-wrap") && entityCss.includes("overflow: auto !important"), "球队/球员表格未强制启用内部滚动");
requireTrue(entityCss.includes(".player-table-wrap") && entityCss.includes("scrollbar-gutter: stable"), "球员名单未锁定独立滚动与稳定滚动条");
requireTrue(entityCss.includes(".team-directory-only") && entityCss.includes("grid-template-columns: minmax(0, 1fr) !important"), "球队目录未切换为独立全宽列表");
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
  read("src/components/format.ts"), read("src/app/router/routeHistory.ts"), modal,
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
const controller=new ModalController();
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
