import { escapeHtml } from "../components/format";
import { icon } from "../components/icons";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { workspacePaneToggle, workspaceSectionNavigation } from "../components/workspace";
import type { BootstrapResponse, SpreadsheetImportPreview } from "../types";

function summary(preview: SpreadsheetImportPreview | null): string {
  if (!preview) return `<div class="workbook-empty"><span>尚未预检</span><small>导出模板、填写后选择文件。</small></div>`;
  const c = preview.counts;
  return `<div class="workbook-preview-summary"><strong>${escapeHtml(preview.source_file_name)}</strong><span>新增 ${c.ready_add}</span><span>更新 ${c.ready_update}</span><span class="${c.conflict ? "warning-text" : ""}">冲突 ${c.conflict}</span><span class="${c.error ? "danger" : ""}">错误 ${c.error}</span></div>`;
}

function workbookCard(
  id: string,
  index: string,
  eyebrow: string,
  title: string,
  description: string,
  modeId: string,
  preview: SpreadsheetImportPreview | null,
  actions: { exportTemplate: string; exportData: string; preview: string; commit: string; details: string },
): string {
  const ready = Boolean(preview && preview.counts.conflict + preview.counts.error === 0);
  return `<section id="${id}" class="panel workbook-card">
    <div class="workbook-card-header"><span class="workbook-index">${index}</span><div><span>${eyebrow}</span><h2>${title}</h2><p>${description}</p></div><span class="workbook-state ${preview ? (ready ? "ready" : "blocked") : "idle"}">${preview ? (ready ? "可提交" : "需处理") : "未预检"}</span></div>
    <div class="workbook-flow">
      <article><span>步骤 1</span><strong>导出</strong><p>空白模板用于新建；现有数据用于月度更新。</p><div class="button-row"><button class="secondary" data-action="${actions.exportTemplate}">空白模板</button><button class="secondary" data-action="${actions.exportData}">现有数据</button></div></article>
      <article><span>步骤 2</span><strong>预检</strong><p>选择导入模式并检查匹配、冲突和错误。</p><label class="field"><span>导入模式</span><select id="${modeId}"><option value="add_and_update">新增并更新</option><option value="add_only">仅新增</option></select></label><button class="primary" data-action="${actions.preview}">选择文件并预检</button></article>
      <article><span>步骤 3</span><strong>确认提交</strong><p>仅在不存在冲突和错误时允许写入数据库。</p><div class="button-row"><button class="primary" data-action="${actions.commit}" ${ready ? "" : "disabled"}>确认提交</button><button class="ghost" data-action="${actions.details}" ${preview ? "" : "disabled"}>查看明细</button></div></article>
    </div>
    ${summary(preview)}
  </section>`;
}

export function workbooksPage(
  state: BootstrapResponse,
  teamPreview: SpreadsheetImportPreview | null,
  playerPreview: SpreadsheetImportPreview | null,
  matchPreview: SpreadsheetImportPreview | null,
  moduleSidebarCollapsed: boolean,
  inspectorCollapsed: boolean,
  activeSection: string,
): string {
  if (!state.data.database_configured) {
    return `<section class="page-heading simple-heading"><div><p class="eyebrow">Excel 工作包</p><h1>可审阅的数据维护入口</h1><p>连接数据库后导出、预检和提交球队、球员、比赛与阵容工作簿。</p></div></section>${inlineDatabaseSetup("连接数据服务以使用工作包", "导入必须先预检，再由用户确认提交。", state.connection_error)}`;
  }
  const section = ["team", "player", "match"].includes(activeSection) ? activeSection : "team";
  const sectionNavigation = workspaceSectionNavigation([
    { id: "team", index: "01", label: "球队月度", description: "球队、教练、阵型与观察", badge: teamPreview ? "已预检" : undefined },
    { id: "player", index: "02", label: "球员月度", description: "身份、履历、位置与状态", badge: playerPreview ? "已预检" : undefined },
    { id: "match", index: "03", label: "比赛与阵容", description: "比赛、四个数据窗口与阵容版本", badge: matchPreview ? "已预检" : undefined },
  ], section);
  const cards = {
    team: workbookCard("workbook-team", "01", "球队月度", "球队、教练、阵型与观察", "更新球队身份、教练任期、阵型概率和球队能力观察。", "team-spreadsheet-import-mode", teamPreview, { exportTemplate: "export-team-template", exportData: "export-team-data", preview: "preview-team-import", commit: "commit-team-import", details: "show-team-import-preview-json" }),
    player: workbookCard("workbook-player", "02", "球员月度", "身份、履历、位置与状态", "更新球员身份、球队履历、位置、状态和能力观察。", "spreadsheet-import-mode", playerPreview, { exportTemplate: "export-player-template", exportData: "export-player-data", preview: "preview-player-import", commit: "commit-player-import", details: "show-import-preview-json" }),
    match: workbookCard("workbook-match", "03", "比赛与阵容", "比赛、四个数据窗口与阵容版本", "维护比赛、阵容快照、首发替补和模型输入版本。", "match-import-mode", matchPreview, { exportTemplate: "export-match-template", exportData: "export-match-data", preview: "preview-match-import", commit: "commit-match-import", details: "show-match-import-json" }),
  } as const;

  return `<section class="page-heading simple-heading entity-page-heading"><div><p class="eyebrow">Excel 工作包</p><h1>数据导入与导出</h1><p>左侧选择数据类型，右侧只显示当前工作包的导出、预检与提交步骤。</p></div><button class="icon-button ${inspectorCollapsed ? "" : "active"}" data-action="toggle-workspace-pane" data-pane="inspector" title="${inspectorCollapsed ? "打开" : "关闭"}导入规则">${icon("panel-right")}</button></section>
  <section class="workspace-grid workbooks-workspace ${inspectorCollapsed ? "inspector-collapsed" : ""}" data-module-sidebar-state="${moduleSidebarCollapsed ? "legacy-collapsed" : "fixed"}">
    <aside class="panel module-sidebar" data-workspace-panel="workbooks-sidebar"><div class="panel-heading"><div><span>工作包目录</span><h2>数据维护</h2></div></div>${sectionNavigation}<div class="module-sidebar-summary"><span>已完成预检</span><strong>${[teamPreview, playerPreview, matchPreview].filter(Boolean).length}</strong><small>共 3 类工作包</small></div></aside>
    <main class="workspace-main" data-workspace-scroll-key="workbooks-main"><section class="workspace-module-view active" data-workspace-section="${section}">${cards[section as keyof typeof cards]}</section></main>
    <aside class="panel workspace-inspector" data-workspace-panel="workbooks-inspector">${workspacePaneToggle("inspector", inspectorCollapsed)}<div class="panel-heading"><div><span>导入规则</span><h2>先预检，再提交</h2></div></div><ul class="inspector-list"><li>空白默认不修改。</li><li>clear 才会显式清空。</li><li>冲突和错误必须先解决。</li><li>重复文件不会重复写入。</li></ul></aside>
  </section>`;
}
