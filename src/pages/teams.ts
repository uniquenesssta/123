import { escapeHtml, formatPercent } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { icon } from "../components/icons";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import { workspaceAnchorNavigation, workspaceSectionNavigation } from "../components/workspace";
import { ageFromBirthDate, availabilityLabel, detailPlayerName, displayPlayerName, initials, positionLabel, preferredFootLabel, teamTypeLabel } from "../components/footballText";
import type { WorkspaceLayoutMode, WorkspaceTabState } from "../app/viewState";
import type {
  BootstrapResponse,
  CoachListItem,
  FormationRecord,
  FormationUsageDistributionRecord,
  TeamDetail,
  TeamListPage,
  TeamListQuery,
  TeamProfileRecord,
  TeamSquadPlayer,
  TeamMatchLineupHistoryItem,
  TeamPackageImportPreview,
  TeamLineupPresetRecord,
  PlayerDetail,
} from "../types";

const teamTypeLabels: Record<string, string> = {
  club: "俱乐部一线队",
  national: "国家/地区代表队",
  reserve: "预备队",
  youth: "青年队",
  women: "女子队",
  other: "其他",
};

const tacticalStyleLabels: Record<string, string> = {
  balanced: "均衡",
  possession: "控球组织",
  direct: "直接推进",
  counter: "快速反击",
  pressing: "高位压迫",
  defensive: "防守优先",
  custom: "自定义",
};

const availabilityLabels: Record<string, string> = {
  available: "可用",
  unavailable: "不可出场",
  doubtful: "存疑",
  injured: "伤病",
  suspended: "停赛",
  rested: "轮休",
  returning: "恢复中",
  unknown: "未知",
};

function numberValue(value: number | null | undefined): string {
  return value === null || value === undefined ? "" : String(value);
}

function currentLocalizedTeamName(detail: TeamDetail): string {
  const records = detail.names.filter((item) => {
    const language = item.language_code?.toLowerCase() ?? "";
    return ["zh-cn", "zh-hans", "zh"].includes(language) || /[一-龥]/u.test(item.name);
  });
  records.sort((left, right) => {
    const languageRank = (language: string | null) => ["zh-cn", "zh-hans", "zh"].includes(language?.toLowerCase() ?? "") ? 1 : 0;
    const byLanguage = languageRank(right.language_code) - languageRank(left.language_code);
    if (byLanguage !== 0) return byLanguage;
    return (right.valid_from ?? "").localeCompare(left.valid_from ?? "") || right.id.localeCompare(left.id);
  });
  return records[0]?.name ?? "";
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

function lineupPresetCards(
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

function detailPanel(
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

  return `<section id="team-profile-overview" class="panel team-detail-panel workspace-anchor-target">
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
    ])}
    <div class="profile-section-stack">
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
    </div>
  </section>`;
}


function teamPackagePanel(preview: TeamPackageImportPreview | null): string {
  const teamCounts = preview?.team_preview?.counts;
  const playerCounts = preview?.player_preview?.counts;
  const blocking =
    (teamCounts?.conflict ?? 0) +
    (teamCounts?.error ?? 0) +
    (playerCounts?.conflict ?? 0) +
    (playerCounts?.error ?? 0);
  const ready =
    (teamCounts?.ready_add ?? 0) +
    (teamCounts?.ready_update ?? 0) +
    (teamCounts?.ready_end_previous ?? 0) +
    (playerCounts?.ready_add ?? 0) +
    (playerCounts?.ready_update ?? 0) +
    (playerCounts?.ready_end_previous ?? 0);
  const rows = [
    ...(preview?.team_preview?.rows.map((row) => ({ scope: "team", row })) ?? []),
    ...(preview?.player_preview?.rows.map((row) => ({ scope: "player", row })) ?? []),
  ].slice(0, 100);
  const statusLabel: Record<string, string> = {
    ready_add: "待新增",
    ready_update: "待更新",
    ready_end_previous: "新增并结束旧记录",
    conflict: "冲突",
    error: "错误",
    skip: "跳过",
    imported: "已导入",
  };
  const coverage = preview?.coverage;
  const p4Ready = Boolean(coverage && coverage.blockers.length === 0 && coverage.readiness_score >= 70 && blocking === 0);
  const readinessClass = p4Ready ? "ready" : coverage ? "warning" : "idle";
  return `<section class="panel spreadsheet-panel team-package-panel">
    <div class="panel-heading"><div><span>统一导入入口</span><h2>球队完整资料包 → P4 输入链路</h2><p>一份 Excel 同时承载球队、球队多语言名称、全部球员、球员多语言名称、基础能力、动态评分、教练历史和阵型分布。</p></div></div>
    <div class="team-package-entry-grid">
      <article class="team-package-primary-action">
        <span class="package-step">主要操作</span>
        <h3>导入完整资料包</h3>
        <p>仅接受 football.team-package.v1 完整资料包。新模板包含“球队名称”和“球员名称”工作表，可维护中文名、英文名、主显示名与历史名称；球队月度文件请使用右侧“球队月度工作包”。</p>
        <label class="field"><span>导入模式</span><select id="team-package-import-mode"><option value="add_and_update">新增并更新</option><option value="add_only">仅新增</option></select></label>
        <button class="primary" data-action="preview-team-package-import">选择 Excel 并统一预检</button>
      </article>
      <article class="team-package-secondary-action">
        <span class="package-step">辅助</span>
        <h3>标准空白资料包</h3>
        <p>仅在需要新建填报文件时导出。日常操作以导入为主，不再要求分别维护球队表和球员表。</p>
        <button class="secondary" data-action="export-team-package-template">导出标准模板</button>
        <button class="ghost" data-page="workbooks">打开球队月度工作包</button>
      </article>
    </div>
    ${preview && coverage ? `<div class="team-package-readiness ${readinessClass}">
      <div class="readiness-score"><span>P4 输入就绪度</span><strong>${coverage.readiness_score}</strong><small>/ 100</small></div>
      <div class="readiness-state"><b>${p4Ready ? "数据结构已达到 P4 输入门槛" : "仍有缺口或预检阻断"}</b><span>${escapeHtml(preview.source_file_name)}</span></div>
      <div class="readiness-metrics">
        <div><span>球队</span><strong>${coverage.team_count}</strong></div>
        <div><span>球员</span><strong>${coverage.player_count}</strong></div>
        <div><span>教练</span><strong>${coverage.coach_count}</strong></div>
        <div><span>阵型记录</span><strong>${coverage.formation_usage_count}</strong></div>
        <div><span>能力观察</span><strong>${coverage.player_ability_count}</strong></div>
        <div><span>动态标签</span><strong>${coverage.player_dynamic_tag_count}</strong></div>
        <div><span>默认角色</span><strong>${coverage.player_role_count}</strong></div>
      </div>
    </div>
    <div class="package-message-grid">
      ${coverage.blockers.length ? `<section class="package-message blocker"><strong>必须处理</strong>${coverage.blockers.map((item) => `<span>${escapeHtml(item)}</span>`).join("")}</section>` : `<section class="package-message ok"><strong>结构阻断</strong><span>没有发现资料包结构性阻断。</span></section>`}
      ${coverage.warnings.length ? `<section class="package-message warning"><strong>建议补全</strong>${coverage.warnings.map((item) => `<span>${escapeHtml(item)}</span>`).join("")}</section>` : `<section class="package-message ok"><strong>完整度提示</strong><span>核心球队、阵容、能力和动态数据均已覆盖。</span></section>`}
    </div>
    <div class="preview-summary"><div><span>待处理总数</span><strong>${ready}</strong></div><div class="preview-count add"><span>球队链新增</span><strong>${teamCounts?.ready_add ?? 0}</strong></div><div class="preview-count update"><span>球员链新增</span><strong>${playerCounts?.ready_add ?? 0}</strong></div><div class="preview-count conflict"><span>冲突</span><strong>${(teamCounts?.conflict ?? 0) + (playerCounts?.conflict ?? 0)}</strong></div><div class="preview-count error"><span>错误</span><strong>${(teamCounts?.error ?? 0) + (playerCounts?.error ?? 0)}</strong></div></div>
    <div class="spreadsheet-table-wrap"><table class="spreadsheet-table"><thead><tr><th>链路</th><th>工作表</th><th>行</th><th>类型</th><th>状态</th><th>说明</th></tr></thead><tbody>${rows.map(({ scope, row }) => `<tr class="status-${escapeHtml(row.status)}"><td>${scope === "team" ? "球队/教练" : "球员/评分"}</td><td>${escapeHtml(row.sheet_name)}</td><td>${row.row_number}</td><td>${escapeHtml(row.entity_type)}</td><td>${escapeHtml(statusLabel[row.status] ?? row.status)}</td><td>${escapeHtml(row.message ?? "")}${row.conflict_candidates.length ? `<div class="candidate-list">${row.conflict_candidates.map((candidate) => `<button class="candidate-choice" data-action="resolve-team-package-conflict" data-package-scope="${scope}" data-row-id="${escapeHtml(row.id)}" data-entity-id="${escapeHtml(candidate.entity_id)}"><strong>${escapeHtml(candidate.display_name)}</strong>${candidate.detail ? `<small>${escapeHtml(candidate.detail)}</small>` : ""}</button>`).join("")}<button class="candidate-skip" data-action="skip-team-package-conflict" data-package-scope="${scope}" data-row-id="${escapeHtml(row.id)}">跳过这一行</button></div>` : ""}</td></tr>`).join("")}</tbody></table></div>
    <div class="button-row"><button class="primary" data-action="commit-team-package-import" ${blocking > 0 || ready === 0 || coverage.blockers.length > 0 ? "disabled" : ""}>确认导入完整资料包</button><button class="secondary" data-action="show-team-package-preview-json">查看完整预检</button><button class="secondary" data-action="export-team-package-preview-json">导出完整预检 JSON</button>${blocking > 0 ? `<span class="blocking-note">仍有 ${blocking} 条冲突或错误；冲突可直接选择，格式错误需修正 Excel 后重新预检。</span>` : ""}</div>
    ` : `<div class="empty-state team-package-empty"><strong>尚未导入球队完整资料包</strong><p>优先选择已经整理好的 Excel。预检不会写入数据库，并会先检查 P4 所需的球队、阵容、能力、动态评分、教练与阵型覆盖情况。</p></div>`}
  </section>`;
}


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
  return `<section class="entity-task-workspace"><header><div><span>球队与人员</span><h2>${escapeHtml(title)}</h2></div><button class="secondary" data-action="select-workspace-section" data-section-id="directory">返回球队目录</button></header><div class="entity-task-body">${content}</div></section>`;
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
  const teamDirectoryPanel = `<aside class="entity-directory panel master-pane" data-workspace-panel="teams-directory" data-workspace-persist="false">
        <div class="entity-directory-header"><div><span>球队目录</span><strong>${teamPage?.items.length ?? 0} 支当前结果</strong></div><button class="icon-button" data-action="refresh-team-catalog" title="刷新球队">${icon("refresh")}</button></div>
        <label class="entity-search">${icon("search")}<input id="team-search" value="${escapeHtml(query.search ?? "")}" placeholder="支持中英文球队名称或别名的部分匹配"><button data-action="search-teams">搜索</button></label>
        <section class="entity-filter-groups compact">
          <div class="entity-filter-group"><header><span>球队身份</span><small>区分国家队、俱乐部及梯队</small></header><label class="entity-filter-field"><span>球队类型</span><select id="team-filter-type"><option value="">全部类型</option>${Object.entries(teamTypeLabels).map(([value, label]) => `<option value="${value}" ${query.team_type === value ? "selected" : ""}>${escapeHtml(label)}</option>`).join("")}</select></label></div>
          <div class="entity-filter-group"><header><span>地域归属</span><small>按球队登记的国家或地区代码筛选</small></header><label class="entity-filter-field"><span>国家/地区代码</span><input id="team-filter-country" value="${escapeHtml(query.country_code ?? "")}" placeholder="不限，例如 KR / GB / BR"></label></div>
          <div class="entity-filter-group"><header><span>档案范围</span><small>停用球队默认不参与日常浏览</small></header><label class="entity-toggle wide"><input id="team-filter-active" type="checkbox" ${query.active_only ? "checked" : ""}><span><strong>仅显示活跃球队</strong><small>关闭后同时显示停用和归档球队</small></span></label></div>
        </section>
        <div class="entity-active-filters"><b>${activeFilterCount} 项筛选</b>${activeFilterSummary}</div>
        <div class="entity-filter-actions directory-actions"><button class="primary" data-action="search-teams">应用筛选</button><button class="secondary" data-action="clear-team-filters">清除</button></div>
        <div class="entity-directory-list" data-workspace-scroll-key="teams-directory-list">${teamDirectoryItems(teamPage, selectedTeam?.team.id ?? null, selectedIds)}</div>
        <footer class="entity-directory-footer"><button class="secondary tiny" data-action="previous-team-page" ${pageNumber <= 1 ? "disabled" : ""}>上一页</button><span>第 ${pageNumber} 页</span><button class="secondary tiny" data-action="next-team-page" ${teamPage?.has_more ? "" : "disabled"}>下一页</button></footer>
      </aside>`;
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
    ? `<section class="entity-browser master-detail-workspace team-detail-workspace ${inspectorCollapsed ? "inspector-collapsed" : "inspector-open"}" data-entity-browser="teams">${teamDetailFilterPanel}${teamDetailPanel}${teamInspectorPanel}</section>`
    : `<section class="entity-browser master-detail-workspace team-directory-only" data-entity-browser="teams">${teamDirectoryPanel}</section>`;
  const selectionBar = selectedCount && !selectedTeam ? `<div class="entity-selection-bar"><strong>已选 ${selectedCount} 支球队</strong><span>仅无任何业务或历史引用的球队可永久删除</span><button class="secondary" data-action="open-selected-teams">打开</button><button class="secondary" data-action="bulk-archive-teams">归档</button><button class="danger" data-action="bulk-delete-teams">永久删除（无引用）</button></div>` : "";
  const directoryStage = `${directoryWorkspace}${selectionBar}`;
  const activeWorkspace = section === "directory"
    ? directoryStage
    : teamTaskWorkspace(section, selectedTeam, coaches, formations, lineupHistory, lineupPresets, packagePreview);
  return `<section class="entity-page entity-page-teams task-page core-workspace-page core-team-workspace">
    ${taskPageHeader({ eyebrow: "球队中心", title: "球队与阵容资源中心", description: "目录、完整档案、资料工作包和新增资料都在当前页面完成；档案内部继续按身份、战术、教练、预设和历史分层。", status: { label: section === "profile" ? "正在编辑完整档案" : selectedTeam ? "球队上下文已就绪" : "等待选择球队", tone: section === "profile" || selectedTeam ? "success" : "neutral" }, actions: pageActions })}
    ${contextRibbon}
    <div class="core-local-navigation">${sectionNav}</div>
    <div class="core-workspace-stage">${activeWorkspace}</div>
  </section>`;
}
