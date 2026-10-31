import type { WorkspaceLayoutMode, WorkspaceTabState } from "../../app/workspace";
import { inlineDatabaseSetup } from "../../components/databaseSetup";
import { ageFromBirthDate, availabilityLabel, detailPlayerName, displayPlayerName, initials, playerStatusLabel, positionLabel, preferredFootLabel } from "../../components/footballText";
import { escapeHtml, formatPercent } from "../../components/format";
import type {
  BootstrapResponse,
  PlayerCatalogReferenceData,
  PlayerDetail,
  PlayerListPage,
  PlayerListQuery,
  PlayerNavigationContext,
  SpreadsheetImportPreview,
} from "../../types";
import { icon } from "../../ui/icons";
import { paginationView, workspaceSectionNavigation } from "../../ui/navigation";
import { entityDetailView, entityDirectoryView, workbenchView } from "../../ui/patterns";
import { taskContextRibbon, taskPageHeader } from "../../ui/patterns/workbench";
import { detailPanel } from "./playerProfileView";
import { spreadsheetPanel } from "./playerWorkbookView";
export function playerTableRows(
  page: PlayerListPage | null,
  selectedId: string | null,
  selectedIds: ReadonlySet<string>,
): string {
  if (!page) return `<tr><td colspan="8"><div class="entity-table-empty"><strong>正在载入球员</strong><span>名单准备完成后会显示在这里。</span></div></td></tr>`;
  if (page.items.length === 0) return `<tr><td colspan="8"><div class="entity-table-empty"><strong>没有匹配球员</strong><span>调整搜索词、球队或位置筛选。</span></div></td></tr>`;
  return page.items.map((player) => {
    const name = displayPlayerName(player);
    const status = availabilityLabel(player.availability_status);
    const statusTone = player.availability_status && ["unavailable", "injured", "suspended", "doubtful"].includes(player.availability_status) ? "warning" : "positive";
    return `<tr class="${selectedId === player.id ? "active" : ""}" data-player-id="${escapeHtml(player.id)}">
      <td><label class="entity-table-check"><input type="checkbox" class="player-select-checkbox" data-player-id="${escapeHtml(player.id)}" ${selectedIds.has(player.id) ? "checked" : ""}><span></span></label></td>
      <td><button id="player-directory-${escapeHtml(player.id)}" class="entity-table-person" data-action="open-player" data-player-id="${escapeHtml(player.id)}"><span class="entity-avatar player-avatar">${escapeHtml(initials(name.primary))}</span><span><strong>${escapeHtml(name.primary)}</strong>${name.secondary ? `<small>${escapeHtml(name.secondary)}</small>` : `<small>${escapeHtml(player.nationality_code ?? "国籍未设置")}</small>`}</span></button></td>
      <td><span class="position-chip">${escapeHtml(positionLabel(player.primary_position_code))}</span></td>
      <td><span class="table-main-text">${escapeHtml(player.current_team_name ?? "未登记球队")}</span></td>
      <td>${escapeHtml(ageFromBirthDate(player.date_of_birth))}</td>
      <td><strong class="rating-value">${player.ability_average === null ? "—" : player.ability_average.toFixed(1)}</strong>${player.ability_confidence === null ? "" : `<small class="table-subtext">${formatPercent(player.ability_confidence)}</small>`}</td>
      <td><span class="status-chip ${statusTone}">${escapeHtml(status)}</span></td>
      <td><div class="table-row-actions"><button class="table-row-action primary-link" data-action="open-player-profile" data-player-id="${escapeHtml(player.id)}" aria-label="打开${escapeHtml(name.primary)}完整档案">档案</button></div></td>
    </tr>`;
  }).join("");
}
export function playerQuickInspector(detail: PlayerDetail): string {
  const name = detailPlayerName(detail);
  const primaryPosition = detail.positions.find((item) => item.is_primary === true) ?? detail.positions[0];
  const positionCode = typeof primaryPosition?.position_code === "string" ? primaryPosition.position_code : null;
  const availability = detail.availability[0];
  const availabilityStatus = typeof availability?.status === "string" ? availability.status : null;
  const availabilityReason = typeof availability?.reason === "string" ? availability.reason : null;
  const currentTeam = detail.team_periods.find((item) => item.valid_to === null) ?? detail.team_periods[0];
  const ability = detail.ability_profile;
  return `<div class="entity-inspector-content">
    <div class="inspector-identity"><span class="entity-avatar player-avatar large">${escapeHtml(initials(name.primary))}</span><div><span>球员速览</span><h2>${escapeHtml(name.primary)}</h2>${name.secondary ? `<p>${escapeHtml(name.secondary)}</p>` : `<p>${escapeHtml(detail.player.nationality_code ?? "国籍未设置")}</p>`}</div></div>
    <div class="inspector-facts three"><div><span>位置</span><strong>${escapeHtml(positionLabel(positionCode))}</strong></div><div><span>年龄</span><strong>${escapeHtml(ageFromBirthDate(detail.player.date_of_birth))}</strong></div><div><span>惯用脚</span><strong>${escapeHtml(preferredFootLabel(detail.player.preferred_foot))}</strong></div></div>
    <div class="inspector-score-row"><div><span>综合能力</span><strong>${ability?.average_value?.toFixed(1) ?? "—"}</strong><small>${ability ? `${ability.dimension_count} 个维度` : "暂无能力观察"}</small></div><div><span>当前状态</span><strong>${escapeHtml(availabilityLabel(availabilityStatus as never))}</strong><small>${escapeHtml(availabilityReason ?? "无补充说明")}</small></div></div>
    <dl class="inspector-description"><div><dt>当前球队</dt><dd>${escapeHtml(currentTeam?.team_name ?? "未登记")}</dd></div><div><dt>身高</dt><dd>${detail.player.height_cm ? `${detail.player.height_cm} cm` : "未设置"}</dd></div><div><dt>档案状态</dt><dd>${escapeHtml(playerStatusLabel(detail.player.status))}</dd></div><div><dt>动态标签</dt><dd>${detail.dynamic_tags.length} 项有效</dd></div></dl>
    <div class="inspector-section"><div class="inspector-section-title"><strong>能力与标签</strong><span>${detail.dynamic_tags.length + (ability?.dimension_count ?? 0)} 项</span></div><div class="inspector-tag-cloud">${detail.dynamic_tags.slice(0, 6).map((tag) => `<span>${escapeHtml(tag.label ?? tag.tag_name)}</span>`).join("") || `<span class="muted-chip">暂无动态标签</span>`}</div></div>
    <div class="inspector-actions"><button class="primary" data-action="select-workspace-section" data-section-id="profile">编辑完整档案</button><button class="secondary" data-action="open-player-api-workspace" data-player-id="${escapeHtml(detail.player.id)}">AI 问答</button></div>
  </div>`;
}
export function playerTaskWorkspace(
  section: string,
  state: BootstrapResponse,
  references: PlayerCatalogReferenceData | null,
  selectedPlayer: PlayerDetail | null,
  preview: SpreadsheetImportPreview | null,
  navigationContext: PlayerNavigationContext | null,
): string {
  if (section === "directory") return "";
  let title = "";
  let content = "";
  if (section === "profile") {
    title = selectedPlayer ? `${detailPlayerName(selectedPlayer).primary} · 完整档案` : "球员完整档案";
    content = detailPanel(selectedPlayer, references, state);
  } else if (section === "workbook") {
    title = "球员资料工作包";
    content = spreadsheetPanel(preview);
  } else {
    title = "新增球员";
    content = `<section class="panel task-create-panel"><div class="task-form-heading"><span>基础身份</span><h3>创建后继续补充球队履历、位置和能力观察</h3></div><div class="form-grid two-column-form clean-form"><label class="field"><span>球员正式姓名</span><input id="new-player-name"></label><label class="field"><span>出生日期</span><input id="new-player-birth" type="date"></label><label class="field"><span>国籍</span><input id="new-player-nationality"></label><label class="field"><span>身高（cm）</span><input id="new-player-height" type="number" min="100" max="240"></label><label class="field"><span>惯用脚</span><select id="new-player-foot"><option value="unknown">未知</option><option value="right">右脚</option><option value="left">左脚</option><option value="both">双脚</option></select></label></div><div class="workflow-actions"><button class="primary" data-action="create-player">创建球员</button></div></section>`;
  }
  const returnAction = section === "profile" && navigationContext?.origin_page === "teams"
    ? `<button class="secondary" data-action="return-to-source-team-profile" data-team-id="${escapeHtml(navigationContext.team_id)}">返回${escapeHtml(navigationContext.team_name)}完整档案</button>`
    : section === "profile" && navigationContext?.origin_page === "lineups"
      ? `<button class="secondary" data-action="return-to-lineup-workspace" data-return-section="${escapeHtml(navigationContext.return_section ?? "chain")}">返回比赛阵容</button>`
      : `<button class="secondary" data-action="select-workspace-section" data-section-id="directory">返回球员目录</button>`;
  return entityDetailView({title,context:"球队与人员",actions:returnAction,content});
}
export function playersPage(
  state: BootstrapResponse,
  references: PlayerCatalogReferenceData | null,
  playerPage: PlayerListPage | null,
  selectedPlayer: PlayerDetail | null,
  query: PlayerListQuery,
  spreadsheetPreview: SpreadsheetImportPreview | null,
  selectedPlayerIds: ReadonlySet<string>,
  _tabs: readonly WorkspaceTabState[],
  _activeTabId: string | null,
  _layoutMode: WorkspaceLayoutMode,
  _moduleSidebarCollapsed: boolean,
  inspectorCollapsed: boolean,
  activeSection: string,
  pageNumber = 1,
  navigationContext: PlayerNavigationContext | null = null,
): string {
  if (!state.data.database_configured) {
    return `<section class="task-empty-workspace">${taskPageHeader({ eyebrow: "球队与人员", title: "球员浏览与管理", description: "连接数据库后维护球员身份、球队履历、位置、可用性、能力和动态标签。", status: { label: "等待数据库", tone: "warning" } })}${taskContextRibbon([{ label: "当前状态", value: "数据库未连接", note: "连接成功后自动加载球员目录", tone: "warning" }])}${inlineDatabaseSetup("连接数据服务以维护球员", "连接成功后本页会自动加载球员目录。", state.connection_error)}</section>`;
  }
  const positions = references?.positions ?? [];
  const teams = references?.teams ?? [];
  const section = ["directory", "profile", "workbook", "create"].includes(activeSection) ? activeSection : "directory";
  const selectedCount = selectedPlayerIds.size;
  const selectedTeamOption = teams.find((team) => team.id === query.team_id) ?? null;
  const activeFilterCount = [query.team_id, query.position_code, query.availability_status, query.player_status].filter(Boolean).length;
  const sourceContext = navigationContext?.team_id === query.team_id ? navigationContext : null;
  const pageActions = `<button class="secondary" data-action="refresh-player-catalog">刷新名单</button>`;
  const sectionNav = workspaceSectionNavigation([
    { id: "directory", index: "01", label: "球员目录", description: "筛选、名单和快速检查", badge: `${playerPage?.items.length ?? 0}` },
    { id: "profile", index: "02", label: "完整档案", description: selectedPlayer ? detailPlayerName(selectedPlayer).primary : "选择球员后开放", disabled: !selectedPlayer },
    { id: "workbook", index: "03", label: "球员工作包", description: "导出、预检和批量导入" },
    { id: "create", index: "04", label: "新增球员", description: "创建基础身份" },
  ], section);
  const contextRibbon = taskContextRibbon([
    { label: "当前球员", value: selectedPlayer ? detailPlayerName(selectedPlayer).primary : "尚未选择", note: selectedPlayer ? `${selectedPlayer.team_periods[0]?.team_name ?? "暂无球队"} · 完整档案按需打开` : "单击名单显示速览，主动打开完整档案", tone: selectedPlayer ? "success" : "neutral" },
    { label: "名单结果", value: `${playerPage?.items.length ?? 0} 名球员`, note: `第 ${pageNumber} 页 · ${activeFilterCount} 项筛选` },
  ]);
  const directoryWorkspace = entityDirectoryView({entity:"players",className:"player-browser",content:`
      <aside class="entity-filter-panel panel master-pane" data-workspace-panel="players-filter" data-workspace-persist="false">
        <div class="entity-directory-header"><div><strong>筛选球员</strong><small>按球队、位置与状态查找</small></div><button class="icon-button" data-action="refresh-player-catalog" title="刷新球员">${icon("refresh")}</button></div>
        <section class="entity-filter-groups">
          <div class="entity-filter-group">${sourceContext ? `<div class="player-source-prefill"><span>从球队页带入</span><strong>${escapeHtml(sourceContext.team_name)}</strong><small>已自动选中，可直接修改或清除</small></div>` : ""}<label class="entity-filter-field"><span>当前所属球队</span><select id="player-filter-team"><option value="">不限球队</option>${teams.map((team) => `<option value="${escapeHtml(team.id)}" ${query.team_id === team.id ? "selected" : ""}>${escapeHtml(team.canonical_name)}</option>`).join("")}</select></label></div>
          <div class="entity-filter-group"><label class="entity-filter-field"><span>主要位置</span><select id="player-filter-position"><option value="">不限位置</option>${positions.map((position) => `<option value="${escapeHtml(position.code)}" ${query.position_code === position.code ? "selected" : ""}>${escapeHtml(positionLabel(position.code))}</option>`).join("")}</select></label></div>
          <div class="entity-filter-group"><label class="entity-filter-field"><span>当前可用状态</span><select id="player-filter-availability"><option value="">不限可用状态</option>${["available","doubtful","injured","suspended","rested","returning","unknown"].map((value) => `<option value="${value}" ${query.availability_status === value ? "selected" : ""}>${escapeHtml(availabilityLabel(value as never))}</option>`).join("")}</select></label></div>
          <div class="entity-filter-group"><label class="entity-filter-field"><span>球员档案状态</span><select id="player-filter-status"><option value="">不限档案状态</option>${["active","inactive","retired","unknown"].map((value) => `<option value="${value}" ${query.player_status === value ? "selected" : ""}>${escapeHtml(playerStatusLabel(value))}</option>`).join("")}</select></label></div>
        </section>
        <div class="entity-active-filters"><b>${activeFilterCount ? `${activeFilterCount} 项已应用` : "当前未限制"}</b>${query.team_id ? `<span>${escapeHtml(selectedTeamOption?.canonical_name ?? sourceContext?.team_name ?? "指定球队")}</span>` : ""}${query.position_code ? `<span>${escapeHtml(positionLabel(query.position_code))}</span>` : ""}${query.availability_status ? `<span>${escapeHtml(availabilityLabel(query.availability_status as never))}</span>` : ""}${query.player_status ? `<span>${escapeHtml(playerStatusLabel(query.player_status))}</span>` : ""}</div>
        <div class="entity-filter-actions"><button class="primary" data-action="search-players">应用筛选</button><button class="secondary" data-action="clear-player-filters">清除全部</button></div>
      </aside>
      <main class="entity-main panel detail-pane" data-workspace-scroll-key="players-main">
        <div class="player-list-heading"><h2>球员名单</h2><p>单击球员查看速览；完整资料在档案中维护。</p></div><div class="player-list-toolbar"><label class="entity-search wide" data-workspace-persist="false">${icon("search")}<input id="player-search" value="${escapeHtml(query.search ?? "")}" placeholder="支持中文名、原名或别名的部分匹配"><button data-action="search-players">搜索</button></label><div class="entity-list-actions"><button class="secondary" data-action="toggle-workspace-pane" data-pane="inspector" ${selectedPlayer ? "" : "disabled"}>球员速览</button></div></div><p class="player-result-count">${query.search ? `搜索“${escapeHtml(query.search)}” · ` : ""}${playerPage?.items.length ?? 0} 条当前结果</p>
        <div class="entity-table-wrap player-table-wrap"><table class="entity-data-table player-directory-table"><thead><tr><th><label class="entity-table-check"><input id="player-select-all" type="checkbox"><span></span></label></th><th>球员</th><th>位置</th><th>当前球队</th><th>年龄</th><th>能力</th><th>状态</th><th>操作</th></tr></thead><tbody>${playerTableRows(playerPage, selectedPlayer?.player.id ?? null, selectedPlayerIds)}</tbody></table></div>
        <footer class="entity-main-footer"><span>${selectedCount ? `已选择 ${selectedCount} 名球员` : "点击球员查看右侧速览"}</span>${paginationView({page:pageNumber,summary:playerPage?`本页 ${playerPage.items.length} 条结果`:"尚未载入",hasMore:playerPage?.has_more??false,previousAction:"previous-player-page",nextAction:"next-player-page",label:"球员目录分页"})}</footer>
      </main>
      `,inspector:{collapsed:inspectorCollapsed,content:`<aside class="entity-inspector panel inspector-pane" data-workspace-panel="players-inspector"><button class="entity-inspector-close icon-button" data-action="toggle-workspace-pane" data-pane="inspector" aria-label="关闭速览">×</button>${selectedPlayer ? playerQuickInspector(selectedPlayer) : `<div class="entity-inspector-empty"><span class="empty-orbit">${icon("shield")}</span><strong>选择一名球员</strong><span>速览不会离开当前名单。</span></div>`}</aside>
    `},selection:selectedCount ? `<div class="entity-selection-bar"><strong>已选 ${selectedCount} 名球员</strong><span>可批量打开、归档或删除空对象</span><button class="secondary" data-action="open-selected-players">打开</button><button class="secondary" data-action="bulk-archive-players">归档</button><button class="danger" data-action="bulk-delete-players">批量删除空对象</button></div>` : ""});
  const activeWorkspace = section === "directory"
    ? directoryWorkspace
    : playerTaskWorkspace(section, state, references, selectedPlayer, spreadsheetPreview, navigationContext);
  return workbenchView({className: "entity-page entity-page-players task-page core-workspace-page core-player-workspace", heading: taskPageHeader({ eyebrow: "球员中心", title: "球员浏览与管理", description: "目录、完整档案、工作包和新增球员保持在同一页面；档案内部按基础资料、位置角色、履历、状态和能力继续分层。", status: { label: section === "profile" ? "正在编辑完整档案" : sourceContext ? "球队筛选已带入" : selectedPlayer ? "球员速览已就绪" : "等待选择球员", tone: section === "profile" || sourceContext || selectedPlayer ? "success" : "neutral" }, actions: "" }), context: contextRibbon, navigation: sectionNav + pageActions, content: activeWorkspace});
}
