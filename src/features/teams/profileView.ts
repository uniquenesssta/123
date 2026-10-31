import { escapeHtml, formatPercent } from "../../components/format";
import { workspaceAnchorNavigation } from "../../ui/navigation";

import type {
  CoachListItem,
  FormationRecord,
  FormationUsageDistributionRecord,
  TeamDetail,
  TeamLineupPresetRecord,
  TeamMatchLineupHistoryItem,
  TeamProfileRecord,
  TeamSquadPlayer
} from "../../types";

import { availabilityLabels, currentLocalizedTeamName, tacticalStyleLabels, teamTypeLabels } from "./teamPresentation";
function numberValue(value: number | null | undefined): string {
  return value === null || value === undefined ? "" : String(value);
}

function squadGroup(
  players: TeamSquadPlayer[],
  group: string,
  title: string,
): string {
  const matches = players.filter((player) => {
    const position = player.position_code ?? "";
    if (group === "GK") return position.startsWith("GK");
    if (group === "DEF")
      return ["CB", "LB", "RB", "LWB", "RWB", "SW"].some((code) =>
        position.startsWith(code),
      );
    if (group === "MID")
      return ["DM", "CM", "AM", "LM", "RM"].some((code) =>
        position.startsWith(code),
      );
    if (group === "ATT")
      return ["LW", "RW", "ST", "CF", "SS"].some((code) =>
        position.startsWith(code),
      );
    return (
      position === "" ||
      ![
        "GK",
        "CB",
        "LB",
        "RB",
        "LWB",
        "RWB",
        "SW",
        "DM",
        "CM",
        "AM",
        "LM",
        "RM",
        "LW",
        "RW",
        "ST",
        "CF",
        "SS",
      ].some((code) => position.startsWith(code))
    );
  });
  if (matches.length === 0) return "";
  return `<section class="squad-group"><h4>${title}<span>${matches.length}</span></h4>${matches
    .map(
      (player) => `
    <button class="squad-player" data-action="open-player-profile-from-team" data-player-id="${escapeHtml(player.player_id)}">
      <b>${player.squad_number ?? "—"}</b><span><strong>${escapeHtml(player.player_name)}</strong><small>${escapeHtml(player.position_code ?? "位置待补全")} · ${escapeHtml(player.role_code ?? "角色待补全")} · ${escapeHtml(availabilityLabels[player.availability_status ?? "unknown"] ?? "未知")}</small></span><em>${player.ability_average === null ? "—" : player.ability_average.toFixed(1)}</em>
    </button>`,
    )
    .join("")}</section>`;
}

function profileValue(
  profile: TeamProfileRecord | null,
  key: keyof TeamProfileRecord,
): unknown {
  return profile?.[key] ?? null;
}


function formationOptions(formations: FormationRecord[], selected: string | null): string {
  return `<option value="">未设置</option>${formations
    .filter((item) => item.code !== "CUSTOM")
    .map((item) => `<option value="${escapeHtml(item.code)}" ${item.code === selected ? "selected" : ""}>${escapeHtml(item.code)} · ${escapeHtml(item.name)}</option>`)
    .join("")}`;
}

function formationDistributionRows(distribution: FormationUsageDistributionRecord | null): string {
  if (!distribution) return `<div class="empty-state compact"><strong>暂无阵型观察</strong><p>保存观察场数与使用次数后生成概率。</p></div>`;
  return `<div class="formation-probability-list">${distribution.entries.map((entry) => `<div class="formation-probability-row"><strong>${escapeHtml(entry.formation_code)}</strong><span>${entry.usage_count} 场</span><span>原始 ${formatPercent(entry.raw_probability)}</span><b>平滑 ${formatPercent(entry.smoothed_probability)}</b></div>`).join("")}</div>`;
}

function latestCoachFormation(history: FormationUsageDistributionRecord[], coachId: string): string {
  const found = history.find((item) => item.scope_type === "team_coach" && item.coach_id === coachId);
  if (!found) return "尚无球队 + 教练阵型观察";
  const primary = found.entries[0];
  return primary ? `${primary.formation_code} ${formatPercent(primary.smoothed_probability)}` : "尚无球队 + 教练阵型观察";
}

export function lineupPresetCards(
  presets: TeamLineupPresetRecord[],
  teamId: string,
  teamName: string,
): string {
  const active = presets.filter((preset) => preset.status === "active");
  const archived = presets.filter((preset) => preset.status === "archived");
  const cards = active.length
    ? active.map((preset) => `<article class="lineup-preset-card ${preset.is_default ? "default" : ""}">
        <div><span>${preset.is_default ? "默认方案" : "阵容预设"}</span><h4>${escapeHtml(preset.name)}</h4><p>${escapeHtml(preset.formation_code ?? "阵型未设置")} · 首发 ${preset.starter_count} · 共 ${preset.member_count} 人 · v${preset.version}</p></div>
        <div class="lineup-preset-card-actions"><button class="secondary tiny" data-action="open-team-lineup-preset-editor" data-preset-id="${escapeHtml(preset.id)}">编辑</button><button class="secondary tiny" data-action="duplicate-team-lineup-preset" data-preset-id="${escapeHtml(preset.id)}" data-preset-name="${escapeHtml(preset.name)}">复制</button><button class="ghost tiny danger" data-action="archive-team-lineup-preset" data-preset-id="${escapeHtml(preset.id)}" data-preset-name="${escapeHtml(preset.name)}">归档</button></div>
      </article>`).join("")
    : `<div class="empty-state compact"><strong>暂无常用阵容预设</strong><p>可以从球队名单自定义首发与替补，也可以从比赛双方阵容保存。</p></div>`;
  return `<details id="team-profile-presets" class="editor-details workspace-anchor-target" open><summary>常用阵容预设（${active.length}）</summary>
    <div class="lineup-preset-heading"><p class="field-note">预设属于球队；应用到比赛时会复制为本场独立阵容，不会反向修改预设。</p><div class="lineup-preset-heading-actions"><button class="secondary" data-action="open-team-lineup-preset-manager" data-team-id="${escapeHtml(teamId)}" data-team-name="${escapeHtml(teamName)}">管理 / 删除预设</button><button class="primary" data-action="open-team-lineup-preset-editor">新建阵容预设</button></div></div>
    <div class="lineup-preset-list">${cards}</div>
    ${archived.length ? `<details class="inline-details top-gap"><summary>已归档（${archived.length}）</summary><div class="history-list">${archived.map((preset) => `<div class="history-row"><strong>${escapeHtml(preset.name)}</strong><span>${escapeHtml(preset.formation_code ?? "阵型未设置")}</span><span>v${preset.version}</span><b>已归档</b></div>`).join("")}</div></details>` : ""}
  </details>`;
}

export function detailPanel(
  detail: TeamDetail | null,
  coaches: CoachListItem[],
  formations: FormationRecord[],
  lineupHistory: TeamMatchLineupHistoryItem[],
  lineupPresets: TeamLineupPresetRecord[],
): string {
  if (!detail)
    return `<section class="panel team-detail-panel"><div class="empty-state"><strong>选择一支球队</strong><p>这里会集中显示球队身份、阵容、战术档案、近期比赛和 AI 问答入口。</p></div></section>`;
  const team = detail.team;
  const profile = detail.profile;
  const localizedName = currentLocalizedTeamName(detail);
  const aliases =
    detail.names.length === 0
      ? '<span class="quiet">暂无别名</span>'
      : detail.names
          .map(
            (item) =>
              `<span class="tag">${escapeHtml(item.name)}${item.language_code ? ` · ${escapeHtml(item.language_code)}` : ""}</span>`,
          )
          .join("");
  const recent =
    detail.recent_matches.length === 0
      ? '<div class="empty-state compact"><strong>暂无比赛记录</strong></div>'
      : detail.recent_matches
          .map(
            (match) =>
              `<div class="team-match-row"><span>${new Date(match.kickoff_time).toLocaleDateString("zh-CN")}</span><strong>${match.venue_side === "home" ? "主" : "客"} vs ${escapeHtml(match.opponent_team_name)}</strong><b>${match.goals_for === null ? "未赛" : `${match.goals_for} : ${match.goals_against}`}</b></div>`,
          )
          .join("");
  const coachOptions = coaches
    .map(
      (coach) =>
        `<option value="${escapeHtml(coach.id)}">${escapeHtml(coach.canonical_name)}${coach.current_team_name ? ` · ${escapeHtml(coach.current_team_name)}` : ""}</option>`,
    )
    .join("");
  const coachHistory =
    detail.coach_periods.length === 0
      ? '<div class="empty-state compact"><strong>尚无教练任期</strong><p>先创建教练，再登记任期；主教练显示会由任期自动投影。</p></div>'
      : detail.coach_periods
          .map(
            (period) =>
              `<div class="history-row"><strong>${escapeHtml(period.coach_name)}</strong><span>${escapeHtml(period.role)}${period.is_interim ? " · 临时" : ""}</span><span>${escapeHtml(period.valid_from)} → ${escapeHtml(period.valid_to ?? "至今")}</span><b>${formatPercent(period.confidence)} · ${escapeHtml(latestCoachFormation(detail.formation_usage, period.coach_id))}</b></div>`,
          )
          .join("");
  const playerHistory =
    detail.player_periods.length === 0
      ? '<div class="empty-state compact"><strong>暂无球员履历</strong></div>'
      : detail.player_periods
          .slice(0, 30)
          .map(
            (period) =>
              `<button class="history-row history-button" data-action="open-player-profile-from-team" data-player-id="${escapeHtml(period.player_id)}"><strong>${escapeHtml(period.player_name)}</strong><span>${escapeHtml(period.registration_status)}${period.squad_number === null ? "" : ` · #${period.squad_number}`}</span><span>${escapeHtml(period.valid_from)} → ${escapeHtml(period.valid_to ?? "至今")}</span><b>${escapeHtml(period.season_name ?? "")}</b></button>`,
          )
          .join("");
  const lineupHistoryRows = lineupHistory.length
    ? lineupHistory.map((item) => `<button class="history-row history-button" data-action="open-lineup" data-lineup-id="${escapeHtml(item.lineup.id)}"><strong>${item.venue_side === "home" ? "主" : "客"} vs ${escapeHtml(item.opponent_team_name)}</strong><span>${escapeHtml(item.lineup.snapshot_type)} · ${escapeHtml(item.lineup.lineup_type)}</span><span>${escapeHtml(new Date(item.kickoff_time).toLocaleString())}</span><b>${item.lineup.model_eligible ? "模型可用" : escapeHtml(item.lineup.model_validation_status)}</b></button>`).join("")
    : '<div class="empty-state compact"><strong>暂无比赛阵容版本</strong></div>';

  return `<section id="team-profile-body" class="team-detail-panel" data-team-draft-id="${escapeHtml(detail.team.id)}" data-workspace-persist="false">
    <div class="team-profile-layout"><aside class="team-profile-directory"><h3>档案目录</h3>
    ${workspaceAnchorNavigation("球队档案", [
      { id: "team-profile-overview", label: "概览" },
      { id: "team-profile-identity", label: "基础身份" },
      { id: "team-profile-tactics", label: "档案与战术" },
      { id: "team-profile-formations", label: "阵型概率" },
      { id: "team-profile-coaches", label: "教练任期" },
      { id: "team-profile-players", label: "球员履历" },
      { id: "team-profile-presets", label: "阵容预设" },
      { id: "team-profile-lineups", label: "阵容历史" },
      { id: "team-profile-recent", label: "阵容与赛程" },
    ], "vertical")}
    </aside><div class="profile-section-stack">
    <details id="team-profile-overview" class="editor-details workspace-anchor-target"><summary>球队概览与能力</summary>
    <div class="team-detail-hero">
      <div><span>${escapeHtml(teamTypeLabels[profile?.team_type ?? "club"] ?? "球队")}</span><h2>${escapeHtml(team.canonical_name)}</h2><p>${escapeHtml(profile?.city ?? team.country_code ?? "国家/城市待补全")} · ${escapeHtml(profile?.stadium ?? "主场待补全")}</p></div>
      <div class="detail-actions"><button class="secondary" data-action="open-team-api-workspace" data-team-id="${escapeHtml(team.id)}">进入 AI 问答</button><button class="ghost danger-quiet" data-action="request-force-delete-team" data-team-id="${escapeHtml(team.id)}" title="永久删除球队及其关联球员、教练、比赛、评分、导入和 P4 历史">强制删除全部资料</button></div>
    </div>

    <div class="team-rating-grid">
      ${[
        ["进攻", profile?.attack_rating],
        ["中场", profile?.midfield_rating],
        ["防守", profile?.defence_rating],
        ["门将", profile?.goalkeeper_rating],
        ["声望", profile?.reputation],
      ]
        .map(
          ([label, value]) =>
            `<div><span>${label}</span><strong>${typeof value === "number" ? value.toFixed(1) : "—"}</strong></div>`,
        )
        .join("")}
    </div>

    </details>
    <details id="team-profile-identity" class="editor-details workspace-anchor-target" open><summary>球队基础身份</summary>
      <div class="compact-form three-column">
        <label class="field"><span>正式名称</span><input id="team-canonical-name" value="${escapeHtml(team.canonical_name)}"></label>
        <label class="field"><span>中文名称</span><input id="team-localized-name" value="${escapeHtml(localizedName)}" placeholder="可留空；留空保持现有中文名"><small class="field-note">保存后持续回显；空白不会生成“默认不存在”，也不会清除已有中文名。</small></label>
        <label class="field"><span>国家或地区</span><input id="team-country-code" value="${escapeHtml(team.country_code ?? "")}" placeholder="例如 KR / 韩国"></label>
        <label class="field"><span>短名称</span><input id="team-short-name" value="${escapeHtml(String(profileValue(profile, "short_name") ?? ""))}"></label>
      </div>
      <button class="primary" data-action="update-team" data-team-id="${escapeHtml(team.id)}">保存球队身份</button>
      <div class="tag-row">${aliases}</div>
      <div class="compact-form three-column top-gap"><label class="field"><span>新增其他名称</span><input id="team-alias-name" placeholder="英文、历史名称或其他语言名称"></label><label class="field"><span>名称语言</span><select id="team-alias-language"><option value="">请选择语言</option><option value="zh-CN">中文（简体）</option><option value="en">英文</option><option value="es">西班牙语</option><option value="pt">葡萄牙语</option><option value="fr">法语</option><option value="de">德语</option><option value="it">意大利语</option><option value="ko">韩语</option><option value="ja">日语</option><option value="other">其他</option></select></label><button class="secondary align-end" data-action="add-team-name" data-team-id="${escapeHtml(team.id)}">保存其他名称</button></div>
    </details>

    <details id="team-profile-tactics" class="editor-details workspace-anchor-target"><summary>球队档案与战术</summary>
      <p class="field-note">按足球管理游戏常用逻辑拆分为身份、阵容、战术和能力，但只把有来源或人工确认的数据写入数据库。</p>
      <div class="compact-form three-column">
        <label class="field"><span>球队类型</span><select id="team-profile-type">${Object.entries(
          teamTypeLabels,
        )
          .map(
            ([key, label]) =>
              `<option value="${key}" ${profile?.team_type === key ? "selected" : ""}>${label}</option>`,
          )
          .join("")}</select></label>
        <label class="field"><span>成立年份</span><input id="team-founded-year" type="number" min="1850" max="2100" value="${numberValue(profile?.founded_year)}"></label>
        <label class="field"><span>所在城市</span><input id="team-city" value="${escapeHtml(profile?.city ?? "")}"></label>
        <label class="field"><span>主场</span><input id="team-stadium" value="${escapeHtml(profile?.stadium ?? "")}"></label>
        <label class="field"><span>当前主教练</span><input value="${escapeHtml(profile?.head_coach ?? "尚未建立任期")}" readonly><small class="field-note">由当前教练任期自动生成，不在球队档案中直接修改。</small></label>
        <label class="field"><span>默认阵型</span><select id="team-default-formation">${formationOptions(formations, profile?.default_formation ?? null)}</select></label>
        <label class="field"><span>战术风格</span><select id="team-tactical-style">${Object.entries(
          tacticalStyleLabels,
        )
          .map(
            ([key, label]) =>
              `<option value="${key}" ${profile?.tactical_style === key ? "selected" : ""}>${label}</option>`,
          )
          .join("")}</select></label>
        ${[
          ["team-attack-rating", "进攻评分", profile?.attack_rating],
          ["team-midfield-rating", "中场评分", profile?.midfield_rating],
          ["team-defence-rating", "防守评分", profile?.defence_rating],
          ["team-goalkeeper-rating", "门将评分", profile?.goalkeeper_rating],
          ["team-reputation", "声望", profile?.reputation],
        ]
          .map(
            ([id, label, value]) =>
              `<label class="field"><span>${label}</span><input id="${id}" type="number" min="0" max="100" step="0.1" value="${numberValue(value as number | null)}"></label>`,
          )
          .join("")}
        <label class="field"><span>资料可信度</span><input id="team-profile-confidence" type="number" min="0" max="1" step="0.01" value="${profile?.data_confidence ?? 0.5}"></label>
        <label class="field span-3"><span>说明</span><textarea id="team-profile-notes" rows="3" placeholder="战术特点、数据来源限制或待核验事项">${escapeHtml(profile?.notes ?? "")}</textarea></label>
      </div>
      <button class="primary" data-action="save-team-profile" data-team-id="${escapeHtml(team.id)}">保存球队档案</button>
    </details>


    <details id="team-profile-formations" class="editor-details workspace-anchor-target"><summary>阵型目录与使用概率</summary>
      <p class="field-note">当前解析来源：${escapeHtml(detail.resolved_formation_distribution.source_label)}。填写观察场数和各阵型使用次数，客户端自动补齐未知次数、归一化并使用 α=3 平滑。</p>
      ${formationDistributionRows(detail.formation_usage[0] ?? null)}
      <div class="compact-form three-column top-gap">
        <label class="field"><span>作用域</span><select id="formation-scope-type"><option value="team">球队</option><option value="team_coach">球队 + 教练</option></select></label>
        <label class="field"><span>教练（联合作用域）</span><select id="formation-coach-id"><option value="">不指定</option>${coachOptions}</select></label>
        <label class="field"><span>观察窗口</span><select id="formation-window-preset"><option value="last_5">最近 5 场</option><option value="last_10" selected>最近 10 场</option><option value="last_20">最近 20 场</option><option value="current_season">当前赛季</option><option value="current_coach_term">当前教练任期</option><option value="custom">自定义日期</option></select></label>
        <label class="field"><span>开始日期（自定义）</span><input id="formation-window-start" type="date"></label>
        <label class="field"><span>结束日期（自定义）</span><input id="formation-window-end" type="date"></label>
        <label class="field"><span>观察场数</span><input id="formation-observed-matches" type="number" min="0" max="500" value="10"></label>
        <label class="field"><span>可信度</span><input id="formation-confidence" type="number" min="0" max="1" step="0.01" value="0.7"></label>
        <label class="field"><span>平滑参数 α</span><input id="formation-alpha" type="number" min="0.1" max="100" step="0.1" value="3"></label>
      </div>
      <div class="formation-count-grid">${formations.filter((item) => !["UNKNOWN", "CUSTOM"].includes(item.code)).map((item) => `<label class="field compact"><span>${escapeHtml(item.code)}</span><input class="formation-usage-count" data-formation-id="${escapeHtml(item.id)}" type="number" min="0" max="500" value="0"></label>`).join("")}</div>
      <button class="primary" data-action="save-formation-usage" data-team-id="${escapeHtml(team.id)}">保存阵型观察</button>
      <details class="inline-details top-gap"><summary>历史观察（${detail.formation_usage.length}）</summary><div class="history-list">${detail.formation_usage.map((item) => `<div class="history-row"><strong>${escapeHtml(item.scope_type)} · ${escapeHtml(item.window_preset)}</strong><span>${escapeHtml(item.window_start)} → ${escapeHtml(item.window_end)}</span><span>${item.observed_matches} 场 · α ${item.alpha}</span><b>${item.entries[0] ? `${escapeHtml(item.entries[0].formation_code)} ${formatPercent(item.entries[0].smoothed_probability)}` : "—"}</b></div>`).join("") || '<div class="empty-state compact"><strong>暂无历史观察</strong></div>'}</div></details>
    </details>

    <details id="team-profile-coaches" class="editor-details workspace-anchor-target"><summary>教练与任期历史</summary>
      <p class="field-note">更换主教练时可自动结束上一任期；历史任期始终保留。俱乐部、国家队和临时教练可以分别登记。</p>
      <div class="history-list">${coachHistory}</div>
      <div class="compact-form three-column top-gap">
        <label class="field"><span>教练</span><select id="team-coach-id"><option value="">选择教练</option>${coachOptions}</select></label>
        <label class="field"><span>职务</span><select id="team-coach-role"><option value="head_coach">主教练</option><option value="interim_head_coach">临时主教练</option><option value="caretaker">代理教练</option><option value="assistant_coach">助理教练</option><option value="other">其他</option></select></label>
        <label class="field"><span>开始日期</span><input id="team-coach-valid-from" type="date"></label>
        <label class="field"><span>结束日期</span><input id="team-coach-valid-to" type="date"></label>
        <label class="field"><span>可信度</span><input id="team-coach-confidence" type="number" min="0" max="1" step="0.01" value="1"></label>
        <label class="check-field"><input id="team-coach-interim" type="checkbox"><span>临时任期</span></label>
        <label class="check-field"><input id="team-coach-end-previous" type="checkbox" checked><span>自动结束同职务上一任期</span></label>
      </div>
      <button class="primary" data-action="add-team-coach-period" data-team-id="${escapeHtml(team.id)}">保存教练任期</button>
    </details>

    <details id="team-profile-players" class="editor-details workspace-anchor-target"><summary>完整球员效力履历</summary>
      <p class="field-note">共 ${detail.player_periods.length} 条历史关系；当前阵容仅是其中仍在有效期内的部分。</p>
      <div class="history-list">${playerHistory}</div>
    </details>

    ${lineupPresetCards(lineupPresets, team.id, team.canonical_name)}

    <details id="team-profile-lineups" class="editor-details workspace-anchor-target"><summary>比赛阵容版本链（${lineupHistory.length}）</summary><p class="field-note">按比赛时点保留预计、确认和实际阵容；旧版本不会物理删除。</p><div class="history-list">${lineupHistoryRows}</div></details>

    <div id="team-profile-recent" class="team-detail-grid workspace-anchor-target">
      <section class="subpanel"><div class="panel-heading"><div><span>当前阵容</span><h3>${detail.squad.length} 名球员</h3></div></div>
        <div class="squad-groups">${squadGroup(detail.squad, "GK", "门将")}${squadGroup(detail.squad, "DEF", "后卫")}${squadGroup(detail.squad, "MID", "中场")}${squadGroup(detail.squad, "ATT", "前锋")}${squadGroup(detail.squad, "OTHER", "待整理")}</div>
      </section>
      <section class="subpanel"><div class="panel-heading"><div><span>比赛历史</span><h3>最近 ${detail.recent_matches.length} 场</h3></div></div><div class="team-match-list">${recent}</div></section>
    </div>
    </div></div>
  </section>`;
}
