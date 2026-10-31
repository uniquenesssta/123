import { checkboxView } from "../ui/primitives/checkbox";
import { escapeHtml } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { icon } from "../ui/icons";
import { initials, teamTypeLabel } from "../components/footballText";
import { taskContextRibbon, taskPageHeader } from "../ui/patterns/workbench";
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
    <p class="field-note">球队归属固定；复制到比赛后为独立草稿，不反向修改此预设。</p><div class="lineup-preset-card-actions">
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
    ${taskPageHeader({ eyebrow: "资源", title: "球队阵容预设", description: "按球队管理首发、替补、阵型和角色；应用到比赛时生成独立副本。" })}
    <p class="field-note preset-page-note">保存、套用、归档与删除均作用于当前球队；比赛阵容独立保存。</p>
    <div class="button-row preset-page-actions">${pageActions}<button class="secondary" data-action="show-lineup-preset-rules">套用规则</button><button class="secondary" data-page="teams">球队中心</button></div>
    <section class="lineup-preset-workspace">
      <aside class="lineup-preset-directory panel" data-workspace-scroll-key="lineup-preset-team-directory" data-workspace-persist="false">
        <header class="preset-panel-heading"><h2>球队目录</h2><p>先选择归属球队</p></header>
        <label class="field"><span>名称或别名</span><input id="team-search" value="${escapeHtml(query.search ?? "")}" placeholder="名称或别名" title="支持中英文球队名称或别名的部分匹配"></label>
        <div class="lineup-preset-filter-grid">
          <label class="field"><span>球队类型</span><select id="team-filter-type"><option value="">全部类型</option>${teamTypeOptions.map(([value, label]) => `<option value="${value}" ${query.team_type === value ? "selected" : ""}>${label}</option>`).join("")}</select></label>
          <label class="field"><span>国家/地区</span><input id="team-filter-country" value="${escapeHtml(query.country_code ?? "")}" placeholder="例如 BR / GB"></label>
        </div>
        ${checkboxView({label:"仅显示活跃球队",checked:query.active_only??false,attributes:{id:"team-filter-active"}})}
        <div class="button-row compact preset-filter-actions"><button class="primary" data-action="search-teams">筛选</button><button class="secondary" data-action="clear-team-filters">清除</button></div>
        <div class="lineup-preset-team-list">${teamList(teamPage, selectedTeam?.team.id ?? null)}</div>
        <footer class="entity-directory-footer"><button class="secondary tiny" data-action="previous-team-page" ${pageNumber <= 1 ? "disabled" : ""}>上一页</button><span>第 ${pageNumber} 页</span><button class="secondary tiny" data-action="next-team-page" ${teamPage?.has_more ? "" : "disabled"}>下一页</button></footer>
      </aside>
      <main class="lineup-preset-main panel">
        <header class="preset-panel-heading"><h2>${escapeHtml(selectedTeam?.team.canonical_name ?? "阵容预设")}</h2><p>${selectedTeam ? "活动与归档分别展示；默认方案不等于本场首发已核验。" : "选择球队后显示该队预设，不沿用上一支球队的数据。"}</p></header>
        <div class="preset-results-scroll" data-workspace-scroll-key="lineup-preset-main">
        ${selectedTeam ? `          <div class="lineup-preset-overview"><div><span>活动预设</span><strong>${active.length}</strong><small>可用于比赛快速套用</small></div><div><span>默认方案</span><strong>${escapeHtml(defaultPreset?.name ?? "未设置")}</strong><small>${escapeHtml(defaultPreset?.formation_code ?? "阵型未设置")}</small></div><div><span>归档预设</span><strong>${archived.length}</strong><small>保留历史，不参与套用</small></div></div>
          <section class="lineup-preset-page-section"><div class="lineup-preset-section-heading"><div><span>活动方案</span><h3>常用阵容预设</h3></div><small>${active.length} 套</small></div><div class="lineup-preset-list">${active.length ? active.map(presetCard).join("") : `<div class="empty-state compact"><strong>暂无活动预设</strong><p>点击“新建阵容预设”，从当前球队名单中选择11名首发和替补。</p></div>`}</div></section>
          ${archived.length ? `<section class="lineup-preset-page-section archived"><div class="lineup-preset-section-heading"><div><span>历史方案</span><h3>已归档预设</h3></div><small>${archived.length} 套</small></div><div class="lineup-preset-list">${archived.map(presetCard).join("")}</div></section>` : ""}` : `<div class="preset-selection-empty"><strong>请选择一支球队</strong><p>新建预设需先确定球队，旧球队的方案不会保留在这里。</p></div>`}
        </div>
      </main>
    </section>
  </section>`;
}
