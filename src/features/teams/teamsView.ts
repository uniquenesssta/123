import { inlineDatabaseSetup } from "../../components/databaseSetup";
import { escapeHtml, formatPercent } from "../../components/format";
import { icon } from "../../ui/icons";
import { paginationView, workspaceSectionNavigation } from "../../ui/navigation";
import { entityDetailView, entityDirectoryView, workbenchView } from "../../ui/patterns";
import { taskContextRibbon, taskPageHeader } from "../../ui/patterns/workbench";

import type { WorkspaceLayoutMode, WorkspaceTabState } from "../../app/workspace";
import { ageFromBirthDate, availabilityLabel, detailPlayerName, displayPlayerName, initials, positionLabel, preferredFootLabel, teamTypeLabel } from "../../components/footballText";
import type {
  BootstrapResponse,
  CoachListItem,
  FormationRecord,
  PlayerDetail,
  TeamDetail,
  TeamLineupPresetRecord,
  TeamListPage,
  TeamListQuery,
  TeamMatchLineupHistoryItem,
  TeamPackageImportPreview
} from "../../types";

import { teamPackagePanel } from "./packageView";
import { detailPanel } from "./profileView";
import { tacticalStyleLabels, teamTypeLabels } from "./teamPresentation";
function teamDirectoryItems(
  page: TeamListPage | null,
  selectedTeamId: string | null,
  selectedIds: ReadonlySet<string>,
): string {
  if (!page) return `<div class="entity-directory-empty"><strong>正在载入球队</strong><span>目录准备完成后会显示在这里。</span></div>`;
  if (page.items.length === 0) return `<div class="entity-directory-empty"><strong>没有匹配球队</strong><span>调整搜索词或筛选条件。</span></div>`;
  return page.items.map((team) => {
    const completion = Math.max(0, Math.min(100, Math.round((team.profile_confidence ?? 0) * 55 + Math.min(team.current_player_count / 26, 1) * 30 + (team.current_coach_name ? 15 : 0))));
    return `<article class="entity-directory-item ${team.id === selectedTeamId ? "active" : ""}" data-team-id="${escapeHtml(team.id)}">
      <label class="entity-select-check" title="加入批量操作"><input type="checkbox" class="team-select-checkbox" data-team-id="${escapeHtml(team.id)}" ${selectedIds.has(team.id) ? "checked" : ""}><span></span></label>
      <button class="entity-directory-open" data-action="open-team" data-team-id="${escapeHtml(team.id)}">
        <span class="entity-avatar team-avatar">${escapeHtml(initials(team.canonical_name))}</span>
        <span class="entity-directory-copy"><strong>${escapeHtml(team.canonical_name)}</strong><small>${escapeHtml(teamTypeLabel(team.team_type))} · ${escapeHtml(team.country_code ?? "地区未设置")}</small></span>
        <span class="entity-directory-metrics"><b>${team.current_player_count}</b><small>球员</small></span>
      </button>
      <div class="entity-directory-foot team-directory-foot"><span class="team-directory-coach" title="${escapeHtml(team.current_coach_name ?? "教练待补")}">${escapeHtml(team.current_coach_name ?? "教练待补")}</span><div class="directory-foot-actions"><span class="completion-chip"><i style="--completion:${completion}%"></i>${completion}%</span><button class="team-directory-profile-action" data-action="open-team-profile" data-team-id="${escapeHtml(team.id)}">档案</button></div></div>
    </article>`;
  }).join("");
}

function teamRosterRows(detail: TeamDetail, selectedPlayerId: string | null): string {
  if (detail.squad.length === 0) {
    return `<tr><td colspan="6"><div class="entity-table-empty"><strong>当前没有有效阵容</strong><span>可通过完整资料包或球员履历补充。</span></div></td></tr>`;
  }
  return detail.squad.map((player) => {
    const name = displayPlayerName(player);
    const availability = availabilityLabel(player.availability_status);
    const statusTone = player.availability_status && ["unavailable", "injured", "suspended", "doubtful"].includes(player.availability_status) ? "warning" : "positive";
    return `<tr class="${selectedPlayerId === player.player_id ? "active" : ""}" data-player-id="${escapeHtml(player.player_id)}">
      <td><button class="entity-table-person" data-action="preview-player-from-team" data-player-id="${escapeHtml(player.player_id)}"><span class="entity-avatar player-avatar">${escapeHtml(initials(name.primary))}</span><span><strong>${escapeHtml(name.primary)}</strong>${name.secondary ? `<small>${escapeHtml(name.secondary)}</small>` : `<small>球员资料</small>`}</span></button></td>
      <td><span class="position-chip">${escapeHtml(positionLabel(player.position_code))}</span>${player.role_code ? `<small class="table-cell-note">${escapeHtml(player.role_code)}</small>` : ""}</td>
      <td>${player.squad_number ?? "—"}</td>
      <td><span class="status-chip ${statusTone}">${escapeHtml(availability)}</span></td>
      <td><strong class="rating-value">${player.ability_average === null ? "—" : player.ability_average.toFixed(1)}</strong></td>
      <td><div class="table-row-actions"><button class="table-row-action primary-link" data-action="open-player-profile-from-team" data-player-id="${escapeHtml(player.player_id)}">完整档案</button></div></td>
    </tr>`;
  }).join("");
}

function playerInspector(detail: PlayerDetail): string {
  const name = detailPlayerName(detail);
  const position = detail.positions.find((item) => item.is_primary === true) ?? detail.positions[0];
  const positionCode = typeof position?.position_code === "string" ? position.position_code : null;
  const availability = detail.availability[0];
  const availabilityStatus = typeof availability?.status === "string" ? availability.status : null;
  const currentTeam = detail.team_periods.find((item) => item.valid_to === null) ?? detail.team_periods[0];
  return `<div class="entity-inspector-content">
    <div class="inspector-identity">
      <span class="entity-avatar player-avatar large">${escapeHtml(initials(name.primary))}</span>
      <div><span>球员速览</span><h2>${escapeHtml(name.primary)}</h2>${name.secondary ? `<p>${escapeHtml(name.secondary)}</p>` : ""}</div>
    </div>
    <div class="inspector-facts three"><div><span>位置</span><strong>${escapeHtml(positionLabel(positionCode))}</strong></div><div><span>年龄</span><strong>${escapeHtml(ageFromBirthDate(detail.player.date_of_birth))}</strong></div><div><span>惯用脚</span><strong>${escapeHtml(preferredFootLabel(detail.player.preferred_foot))}</strong></div></div>
    <div class="inspector-score-row"><div><span>综合能力</span><strong>${detail.ability_profile?.average_value?.toFixed(1) ?? "—"}</strong><small>${detail.ability_profile ? `${detail.ability_profile.dimension_count} 个维度` : "暂无能力观察"}</small></div><div><span>当前状态</span><strong>${escapeHtml(availabilityLabel(availabilityStatus as never))}</strong><small>${escapeHtml(currentTeam?.team_name ?? "未登记球队")}</small></div></div>
    <dl class="inspector-description"><div><dt>国籍</dt><dd>${escapeHtml(detail.player.nationality_code ?? "未设置")}</dd></div><div><dt>身高</dt><dd>${detail.player.height_cm ? `${detail.player.height_cm} cm` : "未设置"}</dd></div><div><dt>动态标签</dt><dd>${detail.dynamic_tags.length} 项有效</dd></div><div><dt>球队履历</dt><dd>${detail.team_periods.length} 段</dd></div></dl>
    <div class="inspector-actions"><button class="primary" data-action="open-player-profile-from-team" data-player-id="${escapeHtml(detail.player.id)}">打开完整球员档案</button><button class="secondary" data-action="open-player-api-workspace" data-player-id="${escapeHtml(detail.player.id)}">AI 问答</button></div>
  </div>`;
}

function teamInspector(detail: TeamDetail): string {
  const profile = detail.profile;
  const latestMatches = detail.recent_matches.slice(0, 4);
  return `<div class="entity-inspector-content">
    <div class="inspector-identity"><span class="entity-avatar team-avatar large">${escapeHtml(initials(detail.team.canonical_name))}</span><div><span>球队速览</span><h2>${escapeHtml(detail.team.canonical_name)}</h2><p>${escapeHtml(teamTypeLabel(profile?.team_type))} · ${escapeHtml(detail.team.country_code ?? "地区未设置")}</p></div></div>
    <div class="inspector-facts three"><div><span>球员</span><strong>${detail.squad.length}</strong></div><div><span>阵型</span><strong>${escapeHtml(profile?.default_formation ?? "未设置")}</strong></div><div><span>可信度</span><strong>${formatPercent(profile?.data_confidence ?? 0)}</strong></div></div>
    <dl class="inspector-description"><div><dt>主教练</dt><dd>${escapeHtml(detail.coach_periods[0]?.coach_name ?? profile?.head_coach ?? "待补")}</dd></div><div><dt>主场</dt><dd>${escapeHtml(profile?.stadium ?? "未设置")}</dd></div><div><dt>战术风格</dt><dd>${escapeHtml(tacticalStyleLabels[profile?.tactical_style ?? "balanced"] ?? "未设置")}</dd></div><div><dt>阵型观察</dt><dd>${detail.formation_usage.length} 组</dd></div></dl>
    <div class="inspector-section"><div class="inspector-section-title"><strong>近期比赛</strong><span>${latestMatches.length} 场</span></div>${latestMatches.length ? `<div class="inspector-match-list">${latestMatches.map((match) => `<div><span>${escapeHtml(new Date(match.kickoff_time).toLocaleDateString("zh-CN"))}</span><strong>${match.venue_side === "home" ? "主" : "客"} vs ${escapeHtml(match.opponent_team_name)}</strong><b>${match.goals_for === null ? "未赛" : `${match.goals_for}:${match.goals_against}`}</b></div>`).join("")}</div>` : `<p class="inspector-muted">暂无近期比赛记录</p>`}</div>
    <div class="inspector-actions"><button class="primary" data-action="open-team-profile" data-team-id="${escapeHtml(detail.team.id)}">打开完整球队档案</button><button class="secondary" data-action="open-team-api-workspace" data-team-id="${escapeHtml(detail.team.id)}">AI 问答</button></div>
  </div>`;
}

function teamTaskWorkspace(
  section: string,
  selectedTeam: TeamDetail | null,
  coaches: CoachListItem[],
  formations: FormationRecord[],
  lineupHistory: TeamMatchLineupHistoryItem[],
  lineupPresets: TeamLineupPresetRecord[],
  packagePreview: TeamPackageImportPreview | null,
): string {
  if (section === "directory") return "";
  let title = "";
  let content = "";
  if (section === "profile") {
    title = selectedTeam ? `${selectedTeam.team.canonical_name} · 完整档案` : "球队完整档案";
    content = detailPanel(selectedTeam, coaches, formations, lineupHistory, lineupPresets);
  } else if (section === "workbook") {
    title = "球队完整资料包";
    content = teamPackagePanel(packagePreview);
  } else {
    title = "新增球队或教练";
    content = `<section class="task-form-grid"><section class="subpanel"><div class="task-form-heading"><span>球队</span><h3>创建基础球队身份</h3></div><label class="field"><span>球队正式名称</span><input id="new-team-name"></label><label class="field"><span>国家或地区</span><input id="new-team-country"></label><button class="primary" data-action="create-team">创建球队</button></section><section class="subpanel"><div class="task-form-heading"><span>教练</span><h3>创建教练身份</h3></div><label class="field"><span>教练姓名</span><input id="new-coach-name"></label><label class="field"><span>国籍</span><input id="new-coach-nationality"></label><button class="primary" data-action="create-coach">创建教练</button></section></section>`;
  }
  return entityDetailView({title,context:"球队与人员",actions:`<button class="secondary" data-action="select-workspace-section" data-section-id="directory">返回球队目录</button>`,content});
}

export function teamsPage(
  state: BootstrapResponse,
  teamPage: TeamListPage | null,
  selectedTeam: TeamDetail | null,
  selectedRosterPlayer: PlayerDetail | null,
  query: TeamListQuery,
  selectedIds: ReadonlySet<string>,
  coaches: CoachListItem[],
  formations: FormationRecord[],
  packagePreview: TeamPackageImportPreview | null,
  lineupHistory: TeamMatchLineupHistoryItem[],
  lineupPresets: TeamLineupPresetRecord[],
  _tabs: readonly WorkspaceTabState[],
  _activeTabId: string | null,
  _layoutMode: WorkspaceLayoutMode,
  _moduleSidebarCollapsed: boolean,
  inspectorCollapsed: boolean,
  activeSection: string,
  pageNumber = 1,
): string {
  if (!state.data.database_configured) {
    return `<section class="task-empty-workspace">${taskPageHeader({ eyebrow: "球队与人员", title: "球队与阵容资源中心", description: "连接数据库后统一维护球队、完整阵容、评分、动态状态、教练和阵型。", status: { label: "等待数据库", tone: "warning" } })}${taskContextRibbon([{ label: "当前状态", value: "数据库未连接", note: "连接成功后自动加载球队目录", tone: "warning" }])}${inlineDatabaseSetup("连接数据服务以维护球队", "连接成功后本页会自动加载球队目录。", state.connection_error)}</section>`;
  }
  const section = ["directory", "profile", "workbook", "create"].includes(activeSection) ? activeSection : "directory";
  const selectedCount = selectedIds.size;
  const profile = selectedTeam?.profile ?? null;
  const selectedPlayerId = selectedRosterPlayer?.player.id ?? null;
  const rosterUnavailable = selectedTeam?.squad.filter((item) => item.availability_status && item.availability_status !== "available").length ?? 0;
  const activeFilterCount = [query.team_type, query.country_code, query.active_only ? "active" : null].filter(Boolean).length;
  const activeFilterSummary = [
    query.team_type ? `类型：${teamTypeLabel(query.team_type)}` : null,
    query.country_code ? `地区：${query.country_code}` : null,
    query.active_only ? "状态：仅活跃" : "状态：包含停用",
  ].filter(Boolean).map((item) => `<span>${escapeHtml(String(item))}</span>`).join("");
  const pageActions = `<button class="secondary" data-action="refresh-team-catalog">${icon("refresh")}<span>刷新目录</span></button>${selectedTeam ? `<button class="primary" data-action="open-team-profile" data-team-id="${escapeHtml(selectedTeam.team.id)}">打开完整档案</button>` : `<button class="primary" data-action="select-workspace-section" data-section-id="create">新增球队</button>`}`;
  const sectionNav = workspaceSectionNavigation([
    { id: "directory", index: "01", label: "球队目录", description: "筛选、浏览和阵容速览", badge: `${teamPage?.items.length ?? 0}` },
    { id: "profile", index: "02", label: "完整档案", description: selectedTeam ? selectedTeam.team.canonical_name : "选择球队后开放", disabled: !selectedTeam },
    { id: "workbook", index: "03", label: "资料工作包", description: "导出、预检和批量导入" },
    { id: "create", index: "04", label: "新增资料", description: "创建球队或教练" },
  ], section);
  const contextRibbon = taskContextRibbon([
    { label: "当前球队", value: selectedTeam?.team.canonical_name ?? "尚未选择", note: selectedTeam ? `${teamTypeLabel(profile?.team_type)} · ${selectedTeam.team.country_code ?? "地区未设置"}` : "从左侧目录选择球队", tone: selectedTeam ? "accent" : "neutral" },
    { label: "目录结果", value: `${teamPage?.items.length ?? 0} 支球队`, note: `第 ${pageNumber} 页 · ${activeFilterCount} 项筛选` },
    { label: "当前阵容", value: selectedTeam ? `${selectedTeam.squad.length} 名球员` : "等待球队", note: selectedTeam ? `${rosterUnavailable} 人需要关注` : "选择球队后保持目录与阵容同屏", tone: rosterUnavailable > 0 ? "warning" : selectedTeam ? "success" : "neutral" },
  ]);
  const teamDirectoryPanel = `<div class="team-catalogue-layout"><aside class="entity-directory panel master-pane" data-workspace-panel="teams-directory" data-workspace-persist="false">
        <div class="entity-directory-header"><div><span>球队目录</span><strong>${teamPage?.items.length ?? 0} 支当前结果</strong></div><button class="icon-button" data-action="refresh-team-catalog" title="刷新球队">${icon("refresh")}</button></div>
        <label class="entity-search">${icon("search")}<input id="team-search" value="${escapeHtml(query.search ?? "")}" placeholder="支持中英文球队名称或别名的部分匹配"><button data-action="search-teams">搜索</button></label>
        <section class="entity-filter-groups compact">
          <div class="entity-filter-group"><header><span>球队身份</span><small>区分国家队、俱乐部及梯队</small></header><label class="entity-filter-field"><span>球队类型</span><select id="team-filter-type"><option value="">全部类型</option>${Object.entries(teamTypeLabels).map(([value, label]) => `<option value="${value}" ${query.team_type === value ? "selected" : ""}>${escapeHtml(label)}</option>`).join("")}</select></label></div>
          <div class="entity-filter-group"><header><span>地域归属</span><small>按球队登记的国家或地区代码筛选</small></header><label class="entity-filter-field"><span>国家/地区代码</span><input id="team-filter-country" value="${escapeHtml(query.country_code ?? "")}" placeholder="不限，例如 KR / GB / BR"></label></div>
          <div class="entity-filter-group"><header><span>档案范围</span><small>停用球队默认不参与日常浏览</small></header><label class="entity-toggle wide"><input id="team-filter-active" type="checkbox" ${query.active_only ? "checked" : ""}><span><strong>仅显示活跃球队</strong><small>关闭后同时显示停用和归档球队</small></span></label></div>
        </section>
        <div class="entity-active-filters"><b>${activeFilterCount} 项筛选</b>${activeFilterSummary}</div>
        <div class="entity-filter-actions directory-actions"><button class="primary" data-action="search-teams">应用筛选</button><button class="secondary" data-action="clear-team-filters">清除</button></div>
      </aside><section class="team-catalogue-results"><h2>当前 ${teamPage?.items.length ?? 0} 支球队 · 第 ${pageNumber} 页</h2>
        <div class="entity-directory-list" data-workspace-scroll-key="teams-directory-list">${teamDirectoryItems(teamPage, selectedTeam?.team.id ?? null, selectedIds)}</div>
        <footer class="entity-directory-footer">${paginationView({page:pageNumber,summary:teamPage?`本页 ${teamPage.items.length} 条结果`:"尚未载入",hasMore:teamPage?.has_more??false,previousAction:"previous-team-page",nextAction:"next-team-page",label:"球队目录分页"})}</footer>
      </section></div>`;
  const teamDetailFilterPanel = selectedTeam ? `<aside class="entity-filter-panel panel team-detail-filter master-pane" data-workspace-panel="teams-filter" data-workspace-persist="false">
        <div class="entity-directory-header"><div><span>筛选器</span><strong>${activeFilterCount ? `${activeFilterCount} 项已应用` : "全部球队"}</strong></div><button class="icon-button" data-action="refresh-team-catalog" title="刷新球队">${icon("refresh")}</button></div>
        <label class="entity-search">${icon("search")}<input id="team-search" value="${escapeHtml(query.search ?? "")}" placeholder="支持中英文球队名称或别名的部分匹配"><button data-action="search-teams-from-detail">搜索</button></label>
        <section class="entity-filter-groups compact">
          <div class="entity-filter-group"><header><span>球队身份</span><small>区分国家队、俱乐部及梯队</small></header><label class="entity-filter-field"><span>球队类型</span><select id="team-filter-type"><option value="">全部类型</option>${Object.entries(teamTypeLabels).map(([value, label]) => `<option value="${value}" ${query.team_type === value ? "selected" : ""}>${escapeHtml(label)}</option>`).join("")}</select></label></div>
          <div class="entity-filter-group"><header><span>地域归属</span><small>按球队登记的国家或地区代码筛选</small></header><label class="entity-filter-field"><span>国家/地区代码</span><input id="team-filter-country" value="${escapeHtml(query.country_code ?? "")}" placeholder="不限，例如 KR / GB / BR"></label></div>
          <div class="entity-filter-group"><header><span>档案范围</span><small>停用球队默认不参与日常浏览</small></header><label class="entity-toggle wide"><input id="team-filter-active" type="checkbox" ${query.active_only ? "checked" : ""}><span><strong>仅显示活跃球队</strong><small>关闭后同时显示停用和归档球队</small></span></label></div>
        </section>
        <div class="entity-active-filters"><b>${activeFilterCount} 项筛选</b>${activeFilterSummary}</div>
        <div class="entity-filter-actions directory-actions"><button class="primary" data-action="search-teams-from-detail">应用并查看结果</button><button class="secondary" data-action="clear-team-filters-from-detail">清除并查看全部</button></div>
        <div class="team-detail-filter-note"><span>当前球队</span><strong>${escapeHtml(selectedTeam.team.canonical_name)}</strong><small>筛选条件应用后返回球队目录；当前详情不会作为重复目录项显示。</small></div>
      </aside>` : "";
  const teamDetailPanel = selectedTeam ? `<main class="entity-main panel detail-pane" data-workspace-scroll-key="teams-main">
        <header class="team-resource-header"><div class="team-resource-title"><span class="entity-avatar team-avatar xlarge">${escapeHtml(initials(selectedTeam.team.canonical_name))}</span><div><span>${escapeHtml(teamTypeLabel(profile?.team_type))}</span><h2>${escapeHtml(selectedTeam.team.canonical_name)}</h2><p>${escapeHtml(selectedTeam.team.country_code ?? "地区未设置")} · ${escapeHtml(profile?.city ?? "城市未设置")}</p></div></div><div class="team-resource-actions"><button class="secondary" data-action="return-team-directory">返回球队目录</button><button class="secondary" data-action="toggle-workspace-pane" data-pane="inspector">球队速览</button><button class="secondary" data-action="open-team-lineup-preset-manager" data-team-id="${escapeHtml(selectedTeam.team.id)}" data-team-name="${escapeHtml(selectedTeam.team.canonical_name)}">阵容预设</button><button class="secondary" data-action="open-team-profile" data-team-id="${escapeHtml(selectedTeam.team.id)}">完整档案</button><button class="primary" data-action="open-team-api-workspace" data-team-id="${escapeHtml(selectedTeam.team.id)}">AI 问答</button></div></header>
        <div class="team-resource-stats"><div><span>当前球员</span><strong>${selectedTeam.squad.length}</strong><small>${rosterUnavailable} 人需关注</small></div><div><span>主教练</span><strong>${escapeHtml(selectedTeam.coach_periods[0]?.coach_name ?? profile?.head_coach ?? "待补")}</strong><small>${escapeHtml(selectedTeam.coach_periods[0]?.role ?? "未登记任期")}</small></div><div><span>默认阵型</span><strong>${escapeHtml(profile?.default_formation ?? "未设置")}</strong><small>${escapeHtml(tacticalStyleLabels[profile?.tactical_style ?? "balanced"] ?? "战术待补")}</small></div><div><span>数据可信度</span><strong>${formatPercent(profile?.data_confidence ?? 0)}</strong><small>${profile?.updated_at ? `更新于 ${escapeHtml(new Date(profile.updated_at).toLocaleDateString("zh-CN"))}` : "资料待完善"}</small></div></div>
        <section class="entity-table-section"><div class="entity-table-toolbar"><div><span>当前阵容</span><h3>球员名单</h3></div><div class="entity-table-summary"><span>${selectedTeam.squad.length} 人</span><span>${rosterUnavailable} 人异常</span></div></div><div class="entity-table-wrap"><table class="entity-data-table roster-table"><thead><tr><th>球员</th><th>位置</th><th>号码</th><th>状态</th><th>能力</th><th></th></tr></thead><tbody>${teamRosterRows(selectedTeam, selectedPlayerId)}</tbody></table></div></section>
      </main>` : "";
  const teamInspectorPanel = selectedTeam ? `<aside class="entity-inspector panel inspector-pane" data-workspace-panel="teams-inspector"><button class="entity-inspector-close icon-button" data-action="toggle-workspace-pane" data-pane="inspector" aria-label="关闭速览">×</button>${selectedRosterPlayer ? playerInspector(selectedRosterPlayer) : teamInspector(selectedTeam)}</aside>` : "";
  const directoryWorkspace = selectedTeam
    ? entityDirectoryView({entity:"teams",className:"team-detail-workspace",content:teamDetailFilterPanel+teamDetailPanel,inspector:{content:teamInspectorPanel,collapsed:inspectorCollapsed}})
    : entityDirectoryView({entity:"teams",className:"team-directory-only",content:teamDirectoryPanel});
  const selectionBar = selectedCount && !selectedTeam ? `<div class="entity-selection-bar"><strong>已选 ${selectedCount} 支球队</strong><span>仅无任何业务或历史引用的球队可永久删除</span><button class="secondary" data-action="open-selected-teams">打开</button><button class="secondary" data-action="bulk-archive-teams">归档</button><button class="danger" data-action="bulk-delete-teams">永久删除（无引用）</button></div>` : "";
  const directoryStage = `${directoryWorkspace}${selectionBar}`;
  const activeWorkspace = section === "directory"
    ? directoryStage
    : teamTaskWorkspace(section, selectedTeam, coaches, formations, lineupHistory, lineupPresets, packagePreview);
  return workbenchView({className: "entity-page entity-page-teams task-page core-workspace-page core-team-workspace", heading: taskPageHeader({ eyebrow: "球队中心", title: selectedTeam ? `${selectedTeam.team.canonical_name} · ${section === "profile" ? "完整档案" : "球队阵容"}` : "球队与阵容资源中心", description: "目录、完整档案、资料工作包和新增资料保持同一页面上下文。", status: { label: section === "profile" ? "正在编辑完整档案" : selectedTeam ? "球队上下文已就绪" : "等待选择球队", tone: section === "profile" || selectedTeam ? "success" : "neutral" }, actions: pageActions }), context: contextRibbon, navigation: sectionNav, content: activeWorkspace});
}
