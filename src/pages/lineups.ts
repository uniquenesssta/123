import { escapeHtml } from "../components/format";
import { availabilityLabel, displayPlayerName, positionLabel } from "../components/footballText";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { icon } from "../components/icons";
import { workspaceAnchorNavigation, workspaceSectionNavigation } from "../components/workspace";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import type {
  BootstrapResponse,
  CoachListItem,
  CompetitionRecord,
  FormationRecord,
  LineupRecord,
  MatchLineupChain,
  PairedLineupBuilderState,
  PlayerCatalogReferenceData,
  SpreadsheetImportPreview,
  TeamOption,
  TeamLineupPresetRecord,
} from "../types";

function matchStatusLabel(status: string): string {
  return ({
    scheduled: "已排期",
    live: "进行中",
    finished: "已结束",
    postponed: "延期",
    cancelled: "取消",
  } as Record<string, string>)[status] ?? "状态未知";
}

function lineupTypeLabel(type: string): string {
  return ({ expected: "预计阵容", confirmed: "确认阵容", actual: "实际阵容" } as Record<string, string>)[type] ?? "阵容";
}

function spreadsheetEntityLabel(type: string): string {
  return ({
    team: "球队",
    player: "球员",
    player_name: "球员名称",
    player_position: "球员位置",
    player_team_period: "球员所属球队",
    player_ability: "球员能力",
    player_availability: "球员状态",
    player_dynamic_tag: "球员动态标签",
    external_entity_id: "外部数据关联",
    match: "比赛",
    lineup: "阵容",
    lineup_player: "阵容球员",
  } as Record<string, string>)[type] ?? "其他数据";
}

function spreadsheetStatusLabel(status: string): string {
  return ({
    ready_add: "待新增",
    ready_update: "待更新",
    conflict: "需要选择",
    error: "格式错误",
    skip: "已跳过",
    imported: "已导入",
  } as Record<string, string>)[status] ?? "待处理";
}

function lineupMatchLabel(lineup: LineupRecord, matches: PlayerCatalogReferenceData["managed_matches"]): string {
  const match = matches.find((item) => item.id === lineup.match_id);
  return match ? `${match.home_team_name} vs ${match.away_team_name}` : "历史比赛";
}

function chainTeam(team: MatchLineupChain["home"]): string {
  const selected = team.versions.find((item) => item.id === team.selected_lineup_id);
  const versions = team.versions.map((item) => `
    <button class="history-row history-button" data-action="open-lineup" data-lineup-id="${escapeHtml(item.id)}">
      <strong>${escapeHtml(item.snapshot_type)} · ${escapeHtml(lineupTypeLabel(item.lineup_type))}</strong>
      <span>${escapeHtml(item.status)} · ${escapeHtml(item.model_validation_status)}</span>
      <span>${escapeHtml(new Date(item.captured_at).toLocaleString())}</span>
      <b>${item.model_eligible ? "可进入模型" : "不可进入模型"}</b>
    </button>`).join("");
  return `<article class="subpanel chain-team-card">
    <div class="panel-heading compact"><div><span>${team.team_side === "home" ? "主队" : "客队"}</span><h3>${escapeHtml(team.team_name)}</h3></div><b>${selected ? "模型已选" : "阻断"}</b></div>
    ${selected ? `<div class="lineup-chain-summary"><strong>${escapeHtml(selected.snapshot_type)} · ${escapeHtml(lineupTypeLabel(selected.lineup_type))} · ${escapeHtml(selected.formation_code ?? selected.formation ?? "未绑定阵型")}</strong><span>${selected.starter_count} 首发 / ${selected.player_count} 人 · ${escapeHtml(selected.coach_name ?? "教练未绑定")}</span><div class="tag-row">${selected.players.map((player) => `<button class="tag" data-action="open-player-from-lineup" data-player-id="${escapeHtml(player.player_id)}" data-team-id="${escapeHtml(team.team_id)}" data-team-name="${escapeHtml(team.team_name)}" data-return-section="chain">${escapeHtml(player.player_name)}${player.is_starter ? " · 首发" : ""}</button>`).join("")}</div></div>` : `<div class="blocking-note">${team.blocking_issues.map(escapeHtml).join("；")}</div>`}
    <div class="chain-version-list">${versions || '<div class="empty-state compact"><strong>暂无版本</strong></div>'}</div>
  </article>`;
}

function lineupChainPanel(chain: MatchLineupChain | null): string {
  if (!chain) return `<section class="panel"><div class="empty-inline">选择比赛和时间窗口后点击“检查模型链路”。</div></section>`;
  return `<section class="panel">
    <div class="panel-heading"><div><span>比赛 → 双方阵容 → 球员 → P4 输入</span><h2>${escapeHtml(chain.match_record.home_team_name)} vs ${escapeHtml(chain.match_record.away_team_name)}</h2></div><b class="${chain.ready_for_model ? "success-text" : "warning-text"}">${chain.ready_for_model ? "可进入模型" : "冻结阻断"}</b></div>
    <p class="field-note">${escapeHtml(chain.snapshot_type)} · ${chain.data_window_start_time ? `窗口 ${escapeHtml(new Date(chain.data_window_start_time).toLocaleString())} 至 ${escapeHtml(new Date(chain.data_cutoff_time).toLocaleString())}` : `任意赛前时间，当前截止 ${escapeHtml(new Date(chain.data_cutoff_time).toLocaleString())}`}</p>
    ${chain.blocking_issues.length ? `<div class="blocking-note">${chain.blocking_issues.map(escapeHtml).join("；")}</div>` : `<div class="completion-note"><b>双方阵容链路已闭合</b><span>可以直接返回正式推演。</span></div>`}
    <div class="two-column">${chainTeam(chain.home)}${chainTeam(chain.away)}</div>
    ${chain.ready_for_model ? '<div class="workflow-actions"><button class="primary" data-action="continue-lineup-prediction">返回正式推演</button></div>' : ""}
  </section>`;
}

function teamTypeLabel(team: TeamOption): string {
  return ({ national: "国家队", club: "俱乐部", reserve: "预备队", youth: "青年队", women: "女足", other: "未分类" } as Record<string, string>)[team.team_type] ?? "未分类";
}

function teamOptions(teams: TeamOption[], references: PlayerCatalogReferenceData | null, selectedId = ""): string {
  const membershipMap = new Map<string, string[]>();
  for (const item of references?.season_team_memberships ?? []) {
    const current = membershipMap.get(item.team_id) ?? [];
    current.push(item.season_id);
    membershipMap.set(item.team_id, current);
  }
  return teams.map((team) => `<option value="${escapeHtml(team.id)}" data-team-type="${escapeHtml(team.team_type)}" data-season-ids="${escapeHtml((membershipMap.get(team.id) ?? []).join(","))}" ${team.id === selectedId ? "selected" : ""}>${escapeHtml(team.canonical_name)} · ${escapeHtml(teamTypeLabel(team))}</option>`).join("");
}

type CompetitionScope = "national" | "club" | "other";

function metadataText(competition: CompetitionRecord, key: string): string {
  const value = competition.metadata?.[key];
  return typeof value === "string" ? value : "";
}

function competitionScope(competition: CompetitionRecord): CompetitionScope {
  const explicit = metadataText(competition, "scope").toLowerCase();
  if (explicit === "national" || explicit === "club") return explicit;
  const text = `${competition.code} ${competition.name} ${metadataText(competition, "official_name")}`.toLowerCase();
  if (/world cup|世界杯|euro|欧洲杯|nations|国家联赛|qualif|预选|copa america|美洲杯|asian cup|亚洲杯|afcon|非洲杯|gold cup|金杯|national/.test(text)) return "national";
  if (/league|联赛|cup|杯|champions|libertadores|俱乐部|superliga|premier|bundesliga|serie|liga|ligue/.test(text)) return "club";
  return "other";
}

function competitionRegion(competition: CompetitionRecord): string {
  const confederation = metadataText(competition, "confederation").toUpperCase();
  const code = competition.country_code?.toUpperCase() ?? "";
  const confederationCode = confederation || (["FIFA", "UEFA", "CONMEBOL", "CONCACAF", "AFC", "CAF", "OFC"].includes(code) ? code : "");
  const labels: Record<string, string> = {
    FIFA: "全球 / FIFA",
    UEFA: "欧洲 / UEFA",
    CONMEBOL: "南美洲 / CONMEBOL",
    CONCACAF: "中北美及加勒比 / CONCACAF",
    AFC: "亚洲 / AFC",
    CAF: "非洲 / CAF",
    OFC: "大洋洲 / OFC",
  };
  if (labels[confederationCode]) return labels[confederationCode];
  const explicit = metadataText(competition, "menu_region") || metadataText(competition, "region");
  if (/世界|全球|洲际/.test(explicit)) return explicit;
  if (["INT", "WORLD"].includes(code)) return labels.FIFA;
  return "其他地区 / 未指定足联";
}

function competitionCountryLabel(competition: CompetitionRecord): string {
  const region = metadataText(competition, "region") || metadataText(competition, "menu_region");
  const confederation = competitionRegion(competition);
  if (!region || region === confederation || /洲际|全球|世界/.test(region)) return "";
  return region;
}

function competitionRegionOrder(region: string): number {
  const order = ["全球 / FIFA", "欧洲 / UEFA", "南美洲 / CONMEBOL", "中北美及加勒比 / CONCACAF", "亚洲 / AFC", "非洲 / CAF", "大洋洲 / OFC", "其他地区 / 未指定足联"];
  const index = order.indexOf(region);
  return index >= 0 ? index : 100;
}

function competitionHierarchy(state: BootstrapResponse, selectedId = ""): string {
  const competitions = state.data.competitions.filter((item) => item.is_active);
  const selected = competitions.find((item) => item.id === selectedId) ?? null;
  const selectedScope = selected ? competitionScope(selected) : "";
  const selectedRegion = selected ? competitionRegion(selected) : "";
  const scopes: Array<[CompetitionScope, string]> = [["national", "国家队赛事"], ["club", "俱乐部赛事"], ["other", "其他与自定义"]];
  const regions = Array.from(new Set(competitions.map(competitionRegion))).sort((a, b) => competitionRegionOrder(a) - competitionRegionOrder(b) || a.localeCompare(b, "zh-CN"));
  const options = competitions
    .sort((a, b) => {
      const left = Number(a.metadata?.sort_order ?? 9999);
      const right = Number(b.metadata?.sort_order ?? 9999);
      return left - right || a.name.localeCompare(b.name, "zh-CN");
    })
    .map((competition) => {
      const country = competitionScope(competition) === "club" ? competitionCountryLabel(competition) : "";
      const label = country ? `${country} · ${competition.name}` : competition.name;
      const searchText = [
        competition.code,
        competition.name,
        metadataText(competition, "official_name"),
        metadataText(competition, "aliases"),
        competition.country_code ?? "",
        country,
        competitionRegion(competition),
      ].filter(Boolean).join(" ");
      return `<option value="${escapeHtml(competition.id)}" data-search="${escapeHtml(searchText)}" data-kind="${escapeHtml(competition.competition_kind)}" data-scope="${competitionScope(competition)}" data-region="${escapeHtml(competitionRegion(competition))}" ${competition.id === selectedId ? "selected" : ""}>${escapeHtml(label)}</option>`;
    })
    .join("");
  return `<div class="hierarchy-selector competition-hierarchy" data-hierarchy="competition">
    <label class="field hierarchy-level"><span>1级 · 参赛体系</span><select id="new-match-competition-scope" data-searchable-select data-search-placeholder="输入或选择参赛体系"><option value="">选择国家队或俱乐部</option>${scopes.map(([value, label]) => `<option value="${value}" ${value === selectedScope ? "selected" : ""}>${label}</option>`).join("")}</select><small>国家队与俱乐部赛事严格分开。</small></label>
    <label class="field hierarchy-level"><span>2级 · 地区/足联</span><select id="new-match-competition-region" data-searchable-select data-search-placeholder="输入地区或足联" ${selectedScope ? "" : "disabled"}><option value="">${selectedScope ? "选择地区或足联" : "先选择参赛体系"}</option>${regions.map((region) => `<option value="${escapeHtml(region)}" ${region === selectedRegion ? "selected" : ""}>${escapeHtml(region)}</option>`).join("")}</select><small>只显示 FIFA、UEFA、AFC 等地区/足联，不按国家分层。</small></label>
    <label class="field hierarchy-level"><span>3级 · 具体赛事</span><select id="new-match-competition" data-searchable-select data-search-placeholder="输入赛事名称、代码或别名" ${selectedRegion ? "" : "disabled"}><option value="">${selectedRegion ? "选择具体赛事" : "先选择地区或足联"}</option>${options}</select><small>俱乐部体系不混入国家队赛事，反之亦然。</small></label>
  </div>`;
}

function seasonOptions(state: BootstrapResponse, selectedId = ""): string {
  return state.data.seasons.map((season) => `<option value="${escapeHtml(season.id)}" data-competition-id="${escapeHtml(season.competition_id)}" data-starts-on="${escapeHtml(season.starts_on ?? "")}" data-ends-on="${escapeHtml(season.ends_on ?? "")}" ${season.id === selectedId ? "selected" : ""}>${escapeHtml(season.name)} · ${escapeHtml(season.status)}</option>`).join("");
}

function stageOptions(state: BootstrapResponse, selectedId = ""): string {
  return state.data.stages.map((stage) => `<option value="${escapeHtml(stage.id)}" data-competition-id="${escapeHtml(stage.competition_id)}" data-season-id="${escapeHtml(stage.season_id)}" ${stage.id === selectedId ? "selected" : ""}>${escapeHtml(stage.name)}</option>`).join("");
}

function roundOptions(state: BootstrapResponse, selectedId = ""): string {
  return state.data.rounds.map((round) => `<option value="${escapeHtml(round.id)}" data-stage-id="${escapeHtml(round.stage_id)}" ${round.id === selectedId ? "selected" : ""}>${escapeHtml(round.name)}</option>`).join("");
}

function matchListItem(
  match: PlayerCatalogReferenceData["managed_matches"][number],
  selectedId: string | null,
): string {
  return `<article class="match-list-item ${match.id === selectedId ? "active" : ""}" data-context-kind="match" data-match-id="${escapeHtml(match.id)}" data-match-label="${escapeHtml(`${match.home_team_name} vs ${match.away_team_name}`)}">
    <button class="match-list-open" data-action="select-managed-match" data-match-id="${escapeHtml(match.id)}">
      <span>${escapeHtml(match.competition_name ?? "未绑定赛事")}</span>
      <strong>${escapeHtml(match.home_team_name)} <i>vs</i> ${escapeHtml(match.away_team_name)}</strong>
      <small>${escapeHtml(new Date(match.kickoff_time).toLocaleString("zh-CN"))} · ${escapeHtml(matchStatusLabel(match.status))}</small>
    </button>
    <button class="icon-button danger subtle match-list-delete" data-action="request-delete-match" data-match-id="${escapeHtml(match.id)}" data-match-label="${escapeHtml(`${match.home_team_name} vs ${match.away_team_name}`)}" title="删除比赛">×</button>
  </article>`;
}

function matchList(matches: PlayerCatalogReferenceData["managed_matches"], selectedId: string | null): string {
  if (matches.length === 0) return `<div class="empty-state"><strong>暂无比赛</strong><span>在右侧创建第一场比赛。</span></div>`;
  const current = matches.filter((match) => !["finished", "cancelled"].includes(match.status));
  const history = matches.filter((match) => ["finished", "cancelled"].includes(match.status));
  const selectedIsHistory = history.some((match) => match.id === selectedId);
  const currentMarkup = current.length
    ? `<div class="match-list-section-label"><span>当前比赛</span><b>${current.length}</b></div>${current.map((match) => matchListItem(match, selectedId)).join("")}`
    : `<div class="entity-table-empty"><strong>暂无待进行比赛</strong><span>已结束比赛收纳在历史记录中。</span></div>`;
  const historyMarkup = history.length
    ? `<details class="match-history-group" ${selectedIsHistory ? "open" : ""}><summary><span>历史比赛</span><b>${history.length}</b></summary><div class="match-history-items">${history.map((match) => matchListItem(match, selectedId)).join("")}</div></details>`
    : "";
  return `${currentMarkup}${historyMarkup}`;
}

function matchEditor(state: BootstrapResponse, references: PlayerCatalogReferenceData | null, selectedId: string | null): string {
  const matches = references?.managed_matches ?? [];
  const teams = references?.teams ?? [];
  const match = matches.find((item) => item.id === selectedId) ?? null;
  const isEditing = Boolean(match);
  return `<section class="match-editor">
    <div class="workspace-section-heading">
      <div><span>${isEditing ? "比赛详情" : "新建比赛"}</span><h2>${match ? `${escapeHtml(match.home_team_name)} vs ${escapeHtml(match.away_team_name)}` : "按赛事层级创建比赛"}</h2><p>使用1级、2级、3级菜单定位赛事；赛季按赛事时区和开球地本地日期自动判断。</p></div>
      ${match ? `<div class="button-row"><button class="secondary" data-action="open-match-lineups" data-match-id="${escapeHtml(match.id)}">编排双方阵容</button><button class="secondary" data-action="open-match-prediction" data-match-id="${escapeHtml(match.id)}">进入推演</button><button class="danger" data-action="request-delete-match" data-match-id="${escapeHtml(match.id)}" data-match-label="${escapeHtml(`${match.home_team_name} vs ${match.away_team_name}`)}">删除比赛</button></div>` : ""}
    </div>
    ${workspaceAnchorNavigation("比赛编辑", [
      { id: "match-editor-competition", label: "赛事与赛季" },
      { id: "match-editor-teams", label: "双方球队" },
      { id: "match-editor-status", label: "状态与场地" },
      { id: "match-editor-actions", label: "保存与补录" },
    ])}
    <section class="panel match-context-form" data-workspace-persist="false">
      <input id="managed-match-id" type="hidden" value="${escapeHtml(match?.id ?? "")}"><input id="managed-match-external-key" type="hidden" value="${escapeHtml(match?.external_key ?? "")}">
      <div id="match-editor-competition" class="step-title workspace-anchor-target"><span>1</span><div><strong>赛事与赛季</strong><small>赛事按参赛体系、地区和具体赛事三级定位；未找到赛季时保存比赛会自动创建。</small></div></div>
      ${competitionHierarchy(state, match?.competition_id ?? "")}
      <div class="form-grid three-column clean-form">
        <label class="field"><span>赛季</span><select id="new-match-season"><option value="">按本地开球日期自动匹配/创建</option>${seasonOptions(state, match?.season_id ?? "")}</select></label>
        <label class="field"><span>参赛体系</span><select id="new-match-team-scope"><option value="auto">按赛事自动</option><option value="national">国家队</option><option value="club">俱乐部</option><option value="all">全部球队</option></select></label>
        <label class="field"><span>开球时间</span><input id="new-match-kickoff" type="datetime-local" value="${match ? escapeHtml(new Date(new Date(match.kickoff_time).getTime() - new Date(match.kickoff_time).getTimezoneOffset() * 60000).toISOString().slice(0, 16)) : ""}"></label>
      </div>
      <div class="form-grid two-column-form clean-form"><label class="field"><span>阶段</span><select id="new-match-stage"><option value="">自动或不限定</option>${stageOptions(state, match?.stage_id ?? "")}</select></label><label class="field"><span>轮次</span><select id="new-match-round"><option value="">自动或不限定</option>${roundOptions(state, match?.round_id ?? "")}</select></label></div>
      <div id="match-editor-teams" class="step-title workspace-anchor-target"><span>2</span><div><strong>双方球队</strong><small>优先显示当前赛季已注册球队；无注册数据时按国家队或俱乐部类型过滤。</small></div></div>
      <div class="match-sides-grid"><label class="field team-select-card home"><span>主队</span><select id="new-match-home-team"><option value="">选择主队</option>${teamOptions(teams, references, match?.home_team_id ?? "")}</select></label><div class="match-versus">VS</div><label class="field team-select-card away"><span>客队</span><select id="new-match-away-team"><option value="">选择客队</option>${teamOptions(teams, references, match?.away_team_id ?? "")}</select></label></div>
      <div id="match-team-filter-note" class="field-note">选择具体赛事后将自动筛选对应球队。</div>
      <div id="match-editor-status" class="form-grid three-column clean-form workspace-anchor-target"><label class="field"><span>状态</span><select id="new-match-status"><option value="scheduled" ${match?.status === "scheduled" ? "selected" : ""}>已排期</option><option value="live" ${match?.status === "live" ? "selected" : ""}>进行中</option><option value="finished" ${match?.status === "finished" ? "selected" : ""}>已结束</option><option value="postponed" ${match?.status === "postponed" ? "selected" : ""}>延期</option><option value="cancelled" ${match?.status === "cancelled" ? "selected" : ""}>取消</option></select></label><label class="field field-wide"><span>场地</span><input id="new-match-venue" value="${escapeHtml(match?.venue ?? "")}"></label></div>
      <div id="match-editor-actions" class="workflow-actions workspace-anchor-target"><button class="secondary" data-action="complete-workflow" data-target-page="rules" data-target-section="competitions" data-return-reason="补充赛事或赛季后返回比赛创建">缺少赛事或赛季？前往补充</button><button class="secondary" data-action="complete-workflow" data-target-page="teams" data-target-section="directory" data-return-reason="补充球队后返回比赛创建">缺少球队？前往球队中心</button><button class="primary" data-action="create-match">${match ? "保存比赛修改" : "创建比赛"}</button></div>
    </section>
  </section>`;
}

function formationLevel1(formation: FormationRecord): string {
  const first = formation.code.split("-")[0];
  return ["3", "4", "5"].includes(first) ? `${first}后卫体系` : "其他/自定义";
}

function formationLevel2(formation: FormationRecord): string {
  const parts = formation.code.split("-");
  return parts.length >= 2 ? `${parts[0]}-${parts[1]}结构` : "其他结构";
}

function formationHierarchy(references: PlayerCatalogReferenceData | null, selectedId: string, selectedCode: string, side: "home" | "away"): string {
  const formations = (references?.formations ?? []).filter((item) => item.code !== "CUSTOM");
  const selected = formations.find((item) => item.id === selectedId || (!selectedId && item.code === selectedCode)) ?? null;
  const level1 = selected ? formationLevel1(selected) : "";
  const level2 = selected ? formationLevel2(selected) : "";
  const firstLevels = Array.from(new Set(formations.map(formationLevel1)));
  const secondLevels = Array.from(
    new Map(
      formations.map((formation) => [
        `${formationLevel1(formation)}|${formationLevel2(formation)}`,
        { level1: formationLevel1(formation), level2: formationLevel2(formation) },
      ]),
    ).values(),
  );
  return `<div class="hierarchy-selector formation-hierarchy" data-hierarchy="formation" data-lineup-side="${side}">
    <label class="field hierarchy-level"><span>1级 · 防线体系</span><select id="paired-${side}-formation-level1" data-search-placeholder="输入防线体系"><option value="">选择三/四/五后卫</option>${firstLevels.map((item) => `<option value="${escapeHtml(item)}" ${item === level1 ? "selected" : ""}>${escapeHtml(item)}</option>`).join("")}</select></label>
    <label class="field hierarchy-level"><span>2级 · 中场结构</span><select id="paired-${side}-formation-level2" data-search-placeholder="输入中场结构"><option value="">选择结构</option>${secondLevels.map((item) => `<option value="${escapeHtml(item.level2)}" data-level1="${escapeHtml(item.level1)}" ${item.level2 === level2 && item.level1 === level1 ? "selected" : ""}>${escapeHtml(item.level2)}</option>`).join("")}</select></label>
    <label class="field hierarchy-level"><span>3级 · 具体阵型</span><select id="paired-${side}-formation-id" data-search-placeholder="输入阵型代码或名称"><option value="">保留自由文本</option>${formations.map((item) => `<option value="${escapeHtml(item.id)}" data-code="${escapeHtml(item.code)}" data-level1="${escapeHtml(formationLevel1(item))}" data-level2="${escapeHtml(formationLevel2(item))}" ${item.id === selectedId || (!selectedId && item.code === selectedCode) ? "selected" : ""}>${escapeHtml(item.code)} · ${escapeHtml(item.name)}</option>`).join("")}</select></label>
    <input id="paired-${side}-formation" type="hidden" value="${escapeHtml(selectedCode)}">
  </div>`;
}

function coachOptions(coaches: CoachListItem[], teamId: string, selectedId: string): string {
  const current = coaches.filter((coach) => coach.current_team_id === teamId);
  const others = coaches.filter((coach) => coach.current_team_id !== teamId);
  const currentOptions = current.map((coach) => `<option value="${escapeHtml(coach.id)}" ${coach.id === selectedId ? "selected" : ""}>${escapeHtml(coach.canonical_name)} · ${escapeHtml(coach.current_role ?? "教练")}</option>`).join("");
  const otherOptions = others.map((coach) => `<option value="${escapeHtml(coach.id)}" ${coach.id === selectedId ? "selected" : ""}>${escapeHtml(coach.canonical_name)}${coach.current_team_name ? ` · ${escapeHtml(coach.current_team_name)}` : ""}</option>`).join("");
  return `<option value="">不绑定</option>${current.length ? `<optgroup label="当前球队教练">${currentOptions}</optgroup>` : '<option value="" disabled>当前球队尚未关联教练</option>'}${otherOptions ? `<optgroup label="其他教练（手动选择）">${otherOptions}</optgroup>` : ""}`;
}

function positionSelectOptions(
  references: PlayerCatalogReferenceData | null,
  selected: string | null,
): string {
  return `<option value="">自动</option>${(references?.positions ?? [])
    .map(
      (item) =>
        `<option value="${escapeHtml(item.code)}" ${item.code === selected ? "selected" : ""}>${escapeHtml(item.code)} · ${escapeHtml(item.name)}</option>`,
    )
    .join("")}`;
}

function compactLineupRows(
  selected: PairedLineupBuilderState["home"]["players"],
  references: PlayerCatalogReferenceData | null,
  side: "home" | "away",
): string {
  if (selected.length === 0) {
    return `<div class="empty-state compact"><strong>本次阵容尚为空</strong><span>从上方选择球员、身份和位置后加入。</span></div>`;
  }
  return `<div class="balanced-lineup-list">${selected
    .map(
      (item, index) => `<article class="balanced-lineup-row" data-lineup-builder-row data-lineup-side="${side}" data-player-id="${escapeHtml(item.player_id)}">
        <span class="lineup-order">${index + 1}</span>
        <div class="balanced-lineup-player"><strong>${escapeHtml(item.player_name)}</strong><small>${item.player_secondary_name ? `${escapeHtml(item.player_secondary_name)} · ` : ""}${escapeHtml(availabilityLabel(item.availability_status))}</small></div>
        <select class="balanced-lineup-role" data-lineup-field="is_starter" aria-label="身份"><option value="true" ${item.is_starter ? "selected" : ""}>首发</option><option value="false" ${!item.is_starter ? "selected" : ""}>替补</option></select>
        <select class="balanced-lineup-position" data-lineup-field="position_code" aria-label="位置">${positionSelectOptions(references, item.position_code)}</select>
        <button class="ghost tiny balanced-lineup-settings" data-action="open-lineup-player-settings" data-lineup-side="${side}" data-player-id="${escapeHtml(item.player_id)}" title="角色、分钟与概率设置">设置</button>
        <input type="hidden" data-lineup-field="role_code" value="${escapeHtml(item.role_code ?? "")}">
        <input type="hidden" data-lineup-field="expected_minutes" value="${item.expected_minutes ?? (item.is_starter ? 90 : 20)}">
        <input type="hidden" data-lineup-field="starting_probability" value="${item.starting_probability ?? (item.is_starter ? 1 : 0)}">
        <input type="hidden" data-lineup-field="bench_order" value="${item.bench_order ?? ""}">
        <input type="hidden" data-lineup-field="shirt_number" value="${item.shirt_number ?? ""}">
        <input type="checkbox" class="visually-hidden" data-lineup-field="membership_override" ${item.membership_override ? "checked" : ""}>
        <button class="ghost tiny danger" data-action="remove-lineup-player" data-lineup-side="${side}" data-player-id="${escapeHtml(item.player_id)}" title="移除">×</button>
      </article>`,
    )
    .join("")}</div>`;
}

function lineupCandidateSelect(
  current: PairedLineupBuilderState["home"],
  references: PlayerCatalogReferenceData | null,
  side: "home" | "away",
): string {
  const selectedIds = new Set(current.players.map((item) => item.player_id));
  const available = current.candidates.filter((item) => !selectedIds.has(item.id));
  const options = available
    .map((item) => {
      const name = displayPlayerName(item);
      const bilingualName = name.secondary ? `${name.primary}（${name.secondary}）` : name.primary;
      const searchText = [
        item.canonical_name,
        item.localized_name ?? "",
        item.alternate_name ?? "",
        item.normalized_name,
        item.current_team_name ?? "",
        item.primary_position_code ?? "",
        positionLabel(item.primary_position_code),
        availabilityLabel(item.availability_status),
      ].filter(Boolean).join(" ");
      return `<option value="${escapeHtml(item.id)}" data-position="${escapeHtml(item.primary_position_code ?? "")}" data-search="${escapeHtml(searchText)}">${escapeHtml(bilingualName)} · ${escapeHtml(positionLabel(item.primary_position_code))} · ${escapeHtml(availabilityLabel(item.availability_status))}</option>`;
    })
    .join("");
  return `<div class="balanced-lineup-add">
    <label class="field"><span>选择球员</span><select id="paired-${side}-candidate" data-search-placeholder="输入中文名、原文名、位置或状态"><option value="">${available.length ? `从 ${available.length} 名可选球员中选择` : "没有可添加球员"}</option>${options}</select></label>
    <label class="field"><span>身份</span><select id="paired-${side}-candidate-role"><option value="starter">首发</option><option value="substitute">替补</option></select></label>
    <label class="field"><span>位置</span><select id="paired-${side}-candidate-position">${positionSelectOptions(references, null)}</select></label>
    <button class="primary" data-action="add-selected-lineup-player" data-lineup-side="${side}" ${available.length ? "" : "disabled"}>加入本次阵容</button>
  </div>`;
}

function lineupPresetControls(
  side: "home" | "away",
  presets: TeamLineupPresetRecord[],
  currentPlayerCount: number,
): string {
  const options = presets.map((preset) => `<option value="${escapeHtml(preset.id)}" data-search="${escapeHtml(`${preset.name} ${preset.formation_code ?? ""} ${preset.coach_name ?? ""}`)}" ${preset.is_default ? 'data-default="true"' : ""}>${preset.is_default ? "★ " : ""}${escapeHtml(preset.name)} · ${escapeHtml(preset.formation_code ?? "阵型未设置")} · ${preset.starter_count}+${Math.max(0, preset.member_count - preset.starter_count)}</option>`).join("");
  return `<section class="lineup-preset-quickbar">
    <div><span>球队常用阵容</span><strong>${presets.length ? `${presets.length} 个可用预设` : "尚未保存预设"}</strong></div>
    <label class="field compact"><span>应用已保存阵容</span><select id="paired-${side}-preset" data-search-placeholder="输入预设名称、阵型或教练"><option value="">选择阵容预设</option>${options}</select></label>
    <div class="lineup-preset-quickbar-actions"><button class="secondary" data-action="preview-apply-lineup-preset" data-lineup-side="${side}" ${presets.length ? "" : "disabled"}>预览套用</button><button class="secondary" data-action="open-lineup-preset-manager" data-lineup-side="${side}">管理预设</button><button class="secondary" data-action="save-current-lineup-as-preset" data-lineup-side="${side}" ${currentPlayerCount >= 11 ? "" : "disabled"}>保存当前阵容</button></div>
  </section>`;
}

function lineupSideCard(side: "home" | "away", builder: PairedLineupBuilderState, references: PlayerCatalogReferenceData | null, coaches: CoachListItem[], presets: TeamLineupPresetRecord[]): string {
  const current = builder[side];
  const starters = current.players.filter((item) => item.is_starter).length;
  const currentCoachCount = coaches.filter((coach) => coach.current_team_id === current.team_id).length;
  return `<article id="lineup-builder-${side}" class="paired-lineup-side balanced ${side} workspace-anchor-target">
    <div class="paired-lineup-header"><div><span>${side === "home" ? "主队" : "客队"}</span><h2>${escapeHtml(current.team_name || "等待选择比赛")}</h2></div><div class="lineup-count"><strong>${current.players.length}</strong><span>首发 ${starters} / 11</span></div></div>
    ${lineupPresetControls(side, presets, current.players.length)}
    ${formationHierarchy(references, current.formation_id, current.formation, side)}
    <div class="form-grid three-column clean-form lineup-team-metadata balanced-metadata">
      <label class="field"><span>球队教练</span><select id="paired-${side}-coach" data-search-placeholder="输入教练姓名或球队">${coachOptions(coaches, current.team_id, current.coach_id)}</select></label>
      <label class="field"><span>本队数据可信度（0–1）</span><input id="paired-${side}-quality" type="number" min="0" max="1" step="0.01" value="${current.quality_score}"></label>
      <label class="field"><span>当前首发</span><input value="${starters} / 11" readonly></label>
    </div>
    ${currentCoachCount ? "" : `<div class="inline-assist"><button class="secondary tiny" data-action="complete-workflow" data-target-page="teams" data-target-section="profile" data-return-reason="补充${escapeHtml(current.team_name)}教练后返回双方阵容">当前球队没有有效教练，前往补充</button></div>`}
    ${lineupCandidateSelect(current, references, side)}
    <div class="balanced-lineup-heading"><div><strong>本次阵容</strong><span>角色、分钟、概率和号码在高级设置中维护</span></div><button class="ghost tiny" data-action="clear-paired-lineup-side" data-lineup-side="${side}">清空</button></div>
    ${compactLineupRows(current.players, references, side)}
  </article>`;
}

function pairedBuilderView(
  builder: PairedLineupBuilderState,
  references: PlayerCatalogReferenceData | null,
  presets: Record<"home" | "away", TeamLineupPresetRecord[]>,
  coaches: CoachListItem[],
): string {
  const matches = references?.managed_matches ?? [];
  return `<section class="panel paired-lineup-workflow" data-workspace-persist="false">
    ${workspaceAnchorNavigation("双方阵容", [
      { id: "lineup-builder-context", label: "版本设置" },
      { id: "lineup-builder-home", label: "主队阵容" },
      { id: "lineup-builder-away", label: "客队阵容" },
      { id: "lineup-builder-submit", label: "检查与提交" },
    ])}
    <div id="lineup-builder-context" class="paired-lineup-common workspace-anchor-target"><div class="step-title"><span>1</span><div><strong>比赛与统一版本</strong><small>双方共用比赛、阵容类型、数据窗口、记录时间和来源；球队可信度分别填写。</small></div></div>
      <div class="form-grid four-column clean-form"><label class="field field-wide"><span>比赛</span><select id="paired-lineup-match" data-search-placeholder="输入球队、赛事或开球时间"><option value="">选择比赛</option>${matches.map((match) => `<option value="${escapeHtml(match.id)}" data-home-team="${escapeHtml(match.home_team_id)}" data-home-name="${escapeHtml(match.home_team_name)}" data-away-team="${escapeHtml(match.away_team_id)}" data-away-name="${escapeHtml(match.away_team_name)}" ${match.id === builder.match_id ? "selected" : ""}>${escapeHtml(match.home_team_name)} vs ${escapeHtml(match.away_team_name)} · ${escapeHtml(new Date(match.kickoff_time).toLocaleString())}</option>`).join("")}</select></label><label class="field"><span>阵容类型</span><select id="paired-lineup-type"><option value="expected" ${builder.lineup_type === "expected" ? "selected" : ""}>预计阵容</option><option value="confirmed" ${builder.lineup_type === "confirmed" ? "selected" : ""}>确认阵容</option><option value="actual" ${builder.lineup_type === "actual" ? "selected" : ""}>实际阵容</option></select></label><label class="field"><span>数据窗口</span><select id="paired-lineup-snapshot"><option value="T-N" ${builder.snapshot_type === "T-N" ? "selected" : ""}>T-N · 任意赛前时间</option><option value="T-24h" ${builder.snapshot_type === "T-24h" ? "selected" : ""}>T-24h · 24小时以内</option><option value="T-6h" ${builder.snapshot_type === "T-6h" ? "selected" : ""}>T-6h · 6小时以内</option><option value="T-1h" ${builder.snapshot_type === "T-1h" ? "selected" : ""}>T-1h · 1小时以内</option></select></label><label class="field"><span>记录时间</span><input id="paired-lineup-captured-at" type="datetime-local" value="${escapeHtml(builder.captured_at)}"></label><label class="field field-wide"><span>来源网址</span><input id="paired-lineup-source-urls" value="${escapeHtml(builder.source_urls)}" placeholder="多个网址用分号分隔"></label></div>
    </div>
    ${builder.match_id ? (() => {
      const homeStarters = builder.home.players.filter((item) => item.is_starter).length;
      const awayStarters = builder.away.players.filter((item) => item.is_starter).length;
      const submitReady = homeStarters === 11 && awayStarters === 11;
      const readinessMessage = submitReady
        ? "双方均已满足 11 名首发，可以一次提交。"
        : `还需补齐：${builder.home.team_name || "主队"} ${Math.max(0, 11 - homeStarters)} 名首发；${builder.away.team_name || "客队"} ${Math.max(0, 11 - awayStarters)} 名首发。`;
      return `<div class="paired-lineup-board">${lineupSideCard("home", builder, references, coaches, presets.home)}<div class="paired-lineup-vs"><span>VS</span><small>同场提交</small></div>${lineupSideCard("away", builder, references, coaches, presets.away)}</div><div id="lineup-builder-submit" class="lineup-submit-readiness workspace-anchor-target ${submitReady ? "ready" : "blocked"}"><strong>${submitReady ? "阵容可以提交" : "阵容尚未完整"}</strong><span>${escapeHtml(readinessMessage)}</span></div><div class="workflow-actions"><button class="secondary" data-action="inspect-paired-lineup-chain">检查双方链路</button><button class="primary large" data-action="create-lineup-pair" ${submitReady ? "" : "disabled"}>同时提交双方阵容</button></div>`;
    })() : `<div class="empty-state"><strong>先选择比赛</strong><span>选择后会同时加载主队和客队名单。</span></div>`}
  </section>`;
}

function workbookPreview(exchangePreview: SpreadsheetImportPreview | null): string {
  if (!exchangePreview) return "";
  return `<div class="spreadsheet-preview"><div class="preview-summary"><div><span>文件</span><strong>${escapeHtml(exchangePreview.source_file_name)}</strong></div><div class="preview-count add"><span>待新增</span><strong>${exchangePreview.counts.ready_add}</strong></div><div class="preview-count update"><span>待更新</span><strong>${exchangePreview.counts.ready_update}</strong></div><div class="preview-count conflict"><span>冲突</span><strong>${exchangePreview.counts.conflict}</strong></div><div class="preview-count error"><span>错误</span><strong>${exchangePreview.counts.error}</strong></div></div><div class="spreadsheet-table-wrap"><table class="spreadsheet-table"><thead><tr><th>工作表</th><th>行</th><th>类型</th><th>状态</th><th>说明/处理</th></tr></thead><tbody>${exchangePreview.rows.slice(0, 80).map((row) => `<tr class="status-${escapeHtml(row.status)}"><td>${escapeHtml(row.sheet_name)}</td><td>${row.row_number}</td><td>${escapeHtml(spreadsheetEntityLabel(row.entity_type))}</td><td>${escapeHtml(spreadsheetStatusLabel(row.status))}</td><td>${row.status === "conflict" ? `<div class="conflict-actions">${row.conflict_candidates.map((candidate) => `<button class="tiny secondary" data-action="resolve-match-import-conflict" data-row-id="${escapeHtml(row.id)}" data-entity-id="${escapeHtml(candidate.entity_id)}">${escapeHtml(candidate.display_name)}</button>`).join("")}<button class="tiny ghost" data-action="skip-match-import-conflict" data-row-id="${escapeHtml(row.id)}">跳过</button></div>` : escapeHtml(row.message ?? "—")}</td></tr>`).join("")}</tbody></table></div><div class="preview-footer"><button class="primary" data-action="commit-match-import" ${exchangePreview.counts.conflict + exchangePreview.counts.error > 0 ? "disabled" : ""}>确认写入数据库</button><button class="secondary" data-action="show-match-import-json">查看全部预检结果</button></div></div>`;
}

export function lineupsPage(
  state: BootstrapResponse,
  references: PlayerCatalogReferenceData | null,
  lineups: LineupRecord[],
  exchangePreview: SpreadsheetImportPreview | null,
  pairedBuilder: PairedLineupBuilderState,
  presets: Record<"home" | "away", TeamLineupPresetRecord[]>,
  coaches: CoachListItem[],
  lineupChain: MatchLineupChain | null,
  selectedManagedMatchId: string | null,
  _moduleSidebarCollapsed: boolean,
  _inspectorCollapsed: boolean,
  activeSection: string,
): string {
  if (!state.data.database_configured) return `<section class="task-empty-workspace">${taskPageHeader({ eyebrow: "比赛中心", title: "比赛、阵容与模型输入", description: "连接数据库后在同一工作区管理赛事、比赛、双方阵容和模型输入门禁。", status: { label: "等待数据库", tone: "warning" } })}${taskContextRibbon([{ label: "当前状态", value: "数据库未连接", note: "连接成功后自动加载赛事、球队、比赛和球员名单", tone: "warning" }])}${inlineDatabaseSetup("连接数据服务以管理比赛与阵容", "连接成功后本页会自动加载赛事、球队、比赛和球员名单。", state.connection_error)}</section>`;
  const matches = references?.managed_matches ?? [];
  const section = ["matches", "builder", "chain", "history", "workbook"].includes(activeSection) ? activeSection : "matches";
  const sectionNav = workspaceSectionNavigation([
    { id: "matches", index: "01", label: "比赛管理", description: "赛事层级、赛季与比赛列表", badge: `${matches.length}` },
    { id: "builder", index: "02", label: "双方阵容", description: "主客队并排编排并一次提交", badge: `${pairedBuilder.home.players.length + pairedBuilder.away.players.length}` },
    { id: "chain", index: "03", label: "模型链路", description: "检查双方输入与阻断" },
    { id: "history", index: "04", label: "阵容历史", description: "查看版本与修订记录", badge: `${lineups.length}` },
    { id: "workbook", index: "05", label: "Excel 工作包", description: "批量导入、导出与预检" },
  ], section);

  const matchDirectoryClass = matches.length <= 4 ? " compact-directory" : "";
  const matchView = `<section class="workspace-module-view ${section === "matches" ? "active" : ""}" data-workspace-section="matches"><div class="match-browser-layout"><aside class="panel match-browser-sidebar master-pane${matchDirectoryClass}"><div class="panel-heading"><div><span>比赛目录</span><h2>已创建比赛</h2></div><button class="primary tiny" data-action="new-managed-match">新建</button></div><label class="field"><span>搜索比赛</span><input id="managed-match-search" placeholder="球队或赛事名称"></label><div class="match-list" data-workspace-scroll-key="lineups-match-list">${matchList(matches, selectedManagedMatchId)}</div><small class="field-note">点击打开；右键或点击 × 删除。</small></aside><main class="panel match-browser-detail detail-pane" data-workspace-scroll-key="lineups-match-detail">${matchEditor(state, references, selectedManagedMatchId)}</main></div></section>`;
  const builderView = `<section class="workspace-module-view ${section === "builder" ? "active" : ""}" data-workspace-section="builder" data-workspace-scroll-key="lineups-builder"><div class="workspace-section-heading"><div><span>双方阵容</span><h2>主队和客队相对编排</h2><p>两侧分别维护阵型、教练、可信度、首发和替补，确认后在一个事务中同时提交。</p></div><div class="section-status-pill">${pairedBuilder.home.players.length + pairedBuilder.away.players.length} 人</div></div>${pairedBuilderView(pairedBuilder, references, presets, coaches)}</section>`;
  const chainView = `<section class="workspace-module-view ${section === "chain" ? "active" : ""}" data-workspace-section="chain" data-workspace-scroll-key="lineups-chain"><div class="workspace-section-heading"><div><span>模型链路</span><h2>检查当前比赛的有效输入</h2><p>根据所选时间窗口读取双方最新有效阵容，并显示可操作的阻断原因。</p></div><div class="button-row"><button class="secondary" data-action="inspect-paired-lineup-chain">重新检查</button>${lineupChain?.ready_for_model ? '<button class="primary" data-action="continue-lineup-prediction">进入正式推演</button>' : ""}</div></div>${lineupChainPanel(lineupChain)}</section>`;
  const historyView = `<section class="workspace-module-view ${section === "history" ? "active" : ""}" data-workspace-section="history" data-workspace-scroll-key="lineups-history"><div class="workspace-section-heading"><div><span>阵容历史</span><h2>版本与修订记录</h2><p>未被正式运行引用的版本可删除；已引用版本会归档并从列表隐藏。</p></div><div class="section-status-pill">${lineups.length} 条</div></div><section class="panel table-panel"><div class="lineup-table header balanced-history"><span>比赛</span><span>球队</span><span>类型</span><span>阵型</span><span>人数</span><span>时间</span><span>操作</span></div>${lineups.length === 0 ? `<div class="empty-state"><strong>暂无阵容</strong><span>先创建比赛并同时提交双方阵容。</span></div>` : lineups.map((lineup) => `<article class="lineup-table row balanced-history" data-context-kind="lineup" data-lineup-id="${escapeHtml(lineup.id)}" data-lineup-label="${escapeHtml(`${lineup.team_name} ${lineup.snapshot_type}`)}"><button class="lineup-history-open" data-action="open-lineup" data-lineup-id="${escapeHtml(lineup.id)}"><span>${escapeHtml(lineupMatchLabel(lineup, matches))}</span><span>${escapeHtml(lineup.team_name)}</span><span>${escapeHtml(lineup.snapshot_type)} · ${escapeHtml(lineupTypeLabel(lineup.lineup_type))}</span><span>${escapeHtml(lineup.formation_code ?? lineup.formation ?? "—")}</span><span>${lineup.starter_count} / ${lineup.player_count} · ${lineup.model_eligible ? "可用" : "阻断"}</span><span>${escapeHtml(new Date(lineup.captured_at).toLocaleString())}</span></button><button class="ghost tiny danger" data-action="request-remove-lineup-history" data-lineup-id="${escapeHtml(lineup.id)}" data-lineup-label="${escapeHtml(`${lineup.team_name} ${lineup.snapshot_type}`)}">删除</button></article>`).join("")}</section></section>`;
  const workbookView = `<section class="workspace-module-view ${section === "workbook" ? "active" : ""}" data-workspace-section="workbook" data-workspace-scroll-key="lineups-workbook"><div class="workspace-section-heading"><div><span>Excel 工作包</span><h2>比赛、阵容与俱乐部关系批量维护</h2><p>新模板支持国家队关系与俱乐部关系并存，旧的1248球员资料无需删除重导。</p></div></div><section class="panel spreadsheet-panel match-exchange-panel"><div class="spreadsheet-actions"><article><strong>空白比赛模板</strong><span>录入比赛、双方阵容、球员状态和动态标签。</span><button class="secondary" data-action="export-match-template">导出模板</button></article><article><strong>导出当前比赛</strong><label class="field"><span>比赛</span><select id="exchange-match-id"><option value="">选择比赛</option>${matches.map((match) => `<option value="${escapeHtml(match.id)}">${escapeHtml(match.home_team_name)} vs ${escapeHtml(match.away_team_name)}</option>`).join("")}</select></label><button class="secondary" data-action="export-match-data">导出表格</button></article><article><strong>导出分析资料</strong><span>生成包含比赛、阵容和球员信息的资料包。</span><button class="secondary" data-action="export-ai-match-package">导出分析资料包</button></article><article><strong>导入比赛与阵容</strong><label class="field"><span>导入模式</span><select id="match-import-mode"><option value="add_and_update">新增并更新</option><option value="add_only">仅新增</option></select></label><div class="button-row"><button class="primary" data-action="preview-match-import">导入表格</button><button class="secondary" data-action="preview-ai-match-import">导入分析建议包</button></div></article></div>${workbookPreview(exchangePreview)}</section></section>`;

  const selectedMatch = matches.find((item) => item.id === pairedBuilder.match_id) ?? null;
  const chainReady = lineupChain?.ready_for_model === true;
  const contextRibbon = taskContextRibbon([
    { label: "当前比赛", value: selectedMatch ? `${selectedMatch.home_team_name} vs ${selectedMatch.away_team_name}` : "尚未选择", note: selectedMatch ? new Date(selectedMatch.kickoff_time).toLocaleString() : "在比赛管理中选择或新建比赛", tone: selectedMatch ? "accent" : "neutral" },
    { label: "数据窗口", value: pairedBuilder.snapshot_type, note: `${pairedBuilder.home.players.length + pairedBuilder.away.players.length} 名阵容球员` },
    { label: "模型输入门禁", value: chainReady ? "双方输入已就绪" : lineupChain ? "存在阻断项" : "尚未检查", note: chainReady ? "可以进入正式推演" : lineupChain?.blocking_issues[0] ?? "完成阵容后检查链路", tone: chainReady ? "success" : lineupChain ? "warning" : "neutral" },
  ]);
  return `<section class="task-page core-workspace-page core-match-workspace">
  ${taskPageHeader({ eyebrow: "比赛中心", title: "比赛、阵容与模型输入", description: "一级和二级导航负责定位业务，比赛创建、双方阵容、模型链路与历史在当前页面内完成。", status: { label: chainReady ? "模型输入已就绪" : selectedMatch ? "比赛上下文已载入" : "等待选择比赛", tone: chainReady ? "success" : selectedMatch ? "accent" : "neutral" }, actions: `<button class="secondary" data-action="refresh-lineups">${icon("refresh")}<span>刷新</span></button>` })}
  ${contextRibbon}
  <div class="core-local-navigation">${sectionNav}</div>
  <div class="core-workspace-stage"><section class="balanced-workspace lineups-balanced-workspace master-detail-workspace">
    <main class="balanced-workspace-main">${matchView}${builderView}${chainView}${historyView}${workbookView}</main>
  </section></div>
  </section>`;
}
