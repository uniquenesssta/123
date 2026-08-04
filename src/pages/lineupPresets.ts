import { escapeHtml } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { icon } from "../components/icons";
import { initials, teamTypeLabel } from "../components/footballText";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import type {
  BootstrapResponse,
  TeamDetail,
  TeamLineupPresetRecord,
  TeamListPage,
  TeamListQuery,
} from "../types";

const teamTypeOptions: ReadonlyArray<readonly [string, string]> = [
  ["club", "俱乐部一线队"],
  ["national", "国家/地区代表队"],
  ["reserve", "预备队"],
  ["youth", "青年队"],
  ["women", "女子队"],
  ["other", "其他"],
];

function teamList(
  teamPage: TeamListPage | null,
  selectedTeamId: string | null,
): string {
  if (!teamPage) {
    return `<div class="lineup-preset-directory-empty"><strong>正在载入球队</strong><span>球队目录准备完成后会显示在这里。</span></div>`;
  }
  if (teamPage.items.length === 0) {
    return `<div class="lineup-preset-directory-empty"><strong>没有匹配球队</strong><span>调整搜索条件后重新查询。</span></div>`;
  }
  return teamPage.items.map((team) => `<button
      class="lineup-preset-team-item ${team.id === selectedTeamId ? "active" : ""}"
      data-action="select-lineup-preset-team"
      data-team-id="${escapeHtml(team.id)}"
    >
      <span class="entity-avatar team-avatar">${escapeHtml(initials(team.canonical_name))}</span>
      <span class="lineup-preset-team-copy"><strong>${escapeHtml(team.canonical_name)}</strong><small>${escapeHtml(teamTypeLabel(team.team_type))} · ${escapeHtml(team.country_code ?? "地区未设置")}</small></span>
      <span class="lineup-preset-team-meta"><b>${team.current_player_count}</b><small>球员</small></span>
    </button>`).join("");
}

function presetCard(preset: TeamLineupPresetRecord): string {
  return `<article class="lineup-preset-card lineup-preset-page-card ${preset.is_default ? "default" : ""} ${preset.status}">
    <div>
      <span>${preset.status === "archived" ? "已归档" : preset.is_default ? "默认方案" : "阵容预设"}</span>
      <h4>${escapeHtml(preset.name)}</h4>
      <p>${escapeHtml(preset.formation_code ?? "阵型未设置")} · 首发 ${preset.starter_count} · 共 ${preset.member_count} 人 · v${preset.version}</p>
      <small>${escapeHtml(preset.coach_name ?? "未绑定教练")} · ${escapeHtml(preset.usage_context || "general")}</small>
    </div>
    <div class="lineup-preset-card-actions">
      ${preset.status === "active" ? `<button class="secondary tiny" data-action="open-team-lineup-preset-editor" data-preset-id="${escapeHtml(preset.id)}">编辑</button><button class="secondary tiny" data-action="duplicate-team-lineup-preset" data-preset-id="${escapeHtml(preset.id)}" data-preset-name="${escapeHtml(preset.name)}">复制</button><button class="ghost tiny danger" data-action="archive-team-lineup-preset" data-preset-id="${escapeHtml(preset.id)}" data-preset-name="${escapeHtml(preset.name)}">归档</button>` : `<button class="ghost tiny danger" data-action="request-delete-team-lineup-preset" data-preset-id="${escapeHtml(preset.id)}" data-preset-name="${escapeHtml(preset.name)}" data-team-id="${escapeHtml(preset.team_id)}" data-team-name="${escapeHtml(preset.team_name)}" data-preset-status="${escapeHtml(preset.status)}" data-member-count="${preset.member_count}">永久删除</button>`}
    </div>
  </article>`;
}

export function lineupPresetsPage(
  state: BootstrapResponse,
  teamPage: TeamListPage | null,
  selectedTeam: TeamDetail | null,
  presets: TeamLineupPresetRecord[],
  query: TeamListQuery,
  pageNumber = 1,
): string {
  if (!state.data.database_configured) {
    return `<section class="task-empty-workspace">${taskPageHeader({ eyebrow: "资源", title: "球队阵容预设", description: "连接数据库后统一维护球队常用首发、替补、阵型和战术角色。", status: { label: "等待数据库", tone: "warning" } })}${taskContextRibbon([{ label: "当前状态", value: "数据库未连接", note: "连接成功后自动加载球队与预设", tone: "warning" }])}${inlineDatabaseSetup("连接数据服务以维护阵容预设", "预设属于球队，套用到比赛后会生成独立的本场阵容。", state.connection_error)}</section>`;
  }

  const active = presets.filter((preset) => preset.status === "active");
  const archived = presets.filter((preset) => preset.status === "archived");
  const defaultPreset = active.find((preset) => preset.is_default) ?? null;
  const pageActions = `<button class="secondary" data-action="refresh-lineup-preset-page">${icon("refresh")}<span>刷新</span></button><button class="primary" data-action="open-team-lineup-preset-editor" ${selectedTeam ? "" : "disabled"}>新建阵容预设</button>`;

  return `<section class="lineup-preset-page task-page module-workspace-page">
    ${taskPageHeader({ eyebrow: "资源", title: "球队阵容预设", description: "按球队集中管理常用首发、替补、阵型、教练和角色；应用到比赛时复制为独立阵容，不会反向修改预设。", status: { label: selectedTeam ? `${selectedTeam.team.canonical_name} 已选择` : "等待选择球队", tone: selectedTeam ? "success" : "neutral" }, actions: pageActions })}
    ${taskContextRibbon([
      { label: "当前球队", value: selectedTeam?.team.canonical_name ?? "尚未选择", note: selectedTeam ? `${teamTypeLabel(selectedTeam.profile?.team_type)} · ${selectedTeam.team.country_code ?? "地区未设置"}` : "从左侧目录选择球队", tone: selectedTeam ? "accent" : "neutral" },
      { label: "活动预设", value: `${active.length} 套`, note: defaultPreset ? `默认：${defaultPreset.name}` : "尚未设置默认方案", tone: active.length ? "success" : "neutral" },
      { label: "归档预设", value: `${archived.length} 套`, note: "归档后不进入比赛快速套用" },
    ])}
    <section class="lineup-preset-workspace">
      <aside class="lineup-preset-directory panel" data-workspace-scroll-key="lineup-preset-team-directory" data-workspace-persist="false">
        <div class="entity-directory-header"><div><span>球队目录</span><strong>${teamPage?.items.length ?? 0} 支当前结果</strong></div><button class="icon-button" data-action="refresh-lineup-preset-page" title="刷新球队">${icon("refresh")}</button></div>
        <label class="entity-search">${icon("search")}<input id="team-search" value="${escapeHtml(query.search ?? "")}" placeholder="支持中英文球队名称或别名的部分匹配"><button data-action="search-teams">搜索</button></label>
        <div class="lineup-preset-filter-grid">
          <label class="field"><span>球队类型</span><select id="team-filter-type"><option value="">全部类型</option>${teamTypeOptions.map(([value, label]) => `<option value="${value}" ${query.team_type === value ? "selected" : ""}>${label}</option>`).join("")}</select></label>
          <label class="field"><span>国家/地区</span><input id="team-filter-country" value="${escapeHtml(query.country_code ?? "")}" placeholder="例如 BR / GB"></label>
        </div>
        <label class="entity-toggle compact"><input id="team-filter-active" type="checkbox" ${query.active_only ? "checked" : ""}><span><strong>仅显示活跃球队</strong></span></label>
        <div class="entity-filter-actions directory-actions"><button class="primary" data-action="search-teams">应用筛选</button><button class="secondary" data-action="clear-team-filters">清除</button></div>
        <div class="lineup-preset-team-list">${teamList(teamPage, selectedTeam?.team.id ?? null)}</div>
        <footer class="entity-directory-footer"><button class="secondary tiny" data-action="previous-team-page" ${pageNumber <= 1 ? "disabled" : ""}>上一页</button><span>第 ${pageNumber} 页</span><button class="secondary tiny" data-action="next-team-page" ${teamPage?.has_more ? "" : "disabled"}>下一页</button></footer>
      </aside>
      <main class="lineup-preset-main panel" data-workspace-scroll-key="lineup-preset-main">
        ${selectedTeam ? `<header class="lineup-preset-main-header"><div><span>当前球队</span><h2>${escapeHtml(selectedTeam.team.canonical_name)}</h2><p>${escapeHtml(teamTypeLabel(selectedTeam.profile?.team_type))} · ${selectedTeam.squad.length} 名当前球员</p></div><div class="button-row compact"><button class="secondary" data-page="teams">打开球队中心</button><button class="secondary" data-action="open-team-lineup-preset-manager" data-team-id="${escapeHtml(selectedTeam.team.id)}" data-team-name="${escapeHtml(selectedTeam.team.canonical_name)}">管理 / 删除</button><button class="primary" data-action="open-team-lineup-preset-editor">新建阵容预设</button></div></header>
          <div class="lineup-preset-overview"><div><span>活动预设</span><strong>${active.length}</strong><small>可用于比赛快速套用</small></div><div><span>默认方案</span><strong>${escapeHtml(defaultPreset?.name ?? "未设置")}</strong><small>${escapeHtml(defaultPreset?.formation_code ?? "阵型未设置")}</small></div><div><span>归档预设</span><strong>${archived.length}</strong><small>保留历史，不参与套用</small></div></div>
          <section class="lineup-preset-page-section"><div class="lineup-preset-section-heading"><div><span>活动方案</span><h3>常用阵容预设</h3></div><small>${active.length} 套</small></div><div class="lineup-preset-list">${active.length ? active.map(presetCard).join("") : `<div class="empty-state compact"><strong>暂无活动预设</strong><p>点击“新建阵容预设”，从当前球队名单中选择11名首发和替补。</p></div>`}</div></section>
          ${archived.length ? `<section class="lineup-preset-page-section archived"><div class="lineup-preset-section-heading"><div><span>历史方案</span><h3>已归档预设</h3></div><small>${archived.length} 套</small></div><div class="lineup-preset-list">${archived.map(presetCard).join("")}</div></section>` : ""}` : `<div class="entity-main-empty"><span class="empty-orbit">${icon("shield")}</span><strong>请选择一支球队</strong><p>选择球队后，这里会集中显示活动预设、默认方案和归档历史。</p></div>`}
      </main>
    </section>
  </section>`;
}
