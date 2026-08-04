import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
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
requireTrue(main.includes("modal.reset();\n  const request = navigation.begin(nextPage);"), "跨二级页面导航未清理旧右侧工作区历史");
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

console.log("右侧内联工作区、资源中心滚动与运行日志链路专项验证通过。");
