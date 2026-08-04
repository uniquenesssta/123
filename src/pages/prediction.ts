import { competitionKindOptions } from "../components/competition";
import { escapeHtml } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { workspaceAnchorNavigation, workspaceSectionNavigation } from "../components/workspace";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import { runHistoryMarkup } from "./runs";
import { p4WorkbenchMarkup } from "./p4Workbench";
import type {
  BootstrapResponse,
  CompetitionKind,
  P4MatchWorkspace,
  P4TaskWorkspace,
  PlayerCatalogReferenceData,
  MatchLineupChain,
  MatchPredictionReadiness,
  LineupSnapshotType,
  PredictionModelFamily,
} from "../types";

function readRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function localInputValue(value: unknown): string {
  if (typeof value !== "string" || value.length === 0) return "";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "";
  const local = new Date(date.getTime() - date.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 16);
}

function modelFamily(modelId: string): "p4" | "p7" {
  return modelId.toLowerCase().startsWith("p7") ? "p7" : "p4";
}

function compareVersion(left: string, right: string): number {
  return left.localeCompare(right, undefined, { numeric: true, sensitivity: "base" });
}

function modelOptions(state: BootstrapResponse, selectedModelId: string): string {
  const latestById = new Map<string, (typeof state.data.models)[number]>();
  for (const model of state.data.models) {
    if (!/^(p4|p7)(_|$)/i.test(model.model_id)) continue;
    const existing = latestById.get(model.model_id);
    if (!existing || compareVersion(model.engine_version, existing.engine_version) > 0) {
      latestById.set(model.model_id, model);
    }
  }
  const renderGroup = (family: "p4" | "p7") => {
    const models = [...latestById.values()]
      .filter((model) => modelFamily(model.model_id) === family)
      .sort((left, right) => {
        const leftGeneric = left.model_id === family ? 0 : 1;
        const rightGeneric = right.model_id === family ? 0 : 1;
        return leftGeneric - rightGeneric || left.display_name.localeCompare(right.display_name, "zh-CN");
      });
    if (!models.length) return "";
    return `<optgroup label="${family.toUpperCase()} · 最新已注册模型">${models.map((model) => {
      const generic = model.model_id === family ? " · 推荐：按赛事自动匹配" : "";
      const searchText = `${model.model_id} ${model.display_name} ${family} ${model.supported_competitions.join(" ")}`;
      return `<option value="${escapeHtml(model.model_id)}" data-search="${escapeHtml(searchText)}" ${model.model_id === selectedModelId ? "selected" : ""}>${escapeHtml(model.display_name)}${generic} · v${escapeHtml(model.engine_version)}</option>`;
    }).join("")}</optgroup>`;
  };
  return `${renderGroup("p4")}${renderGroup("p7")}`;
}

const competitionKindLabels: Record<CompetitionKind, string> = {
  league: "联赛",
  group_stage: "小组赛",
  knockout_single_leg: "单回合淘汰赛",
  knockout_two_leg: "两回合淘汰赛",
  friendly: "友谊赛",
  custom: "自定义赛事",
};

function exactModelForKind(family: "p4" | "p7", kind: CompetitionKind): string {
  const suffixes: Record<CompetitionKind, string> = {
    league: "league",
    group_stage: "group_stage",
    knockout_single_leg: "knockout_90",
    knockout_two_leg: "knockout_two_leg_90",
    friendly: "friendly",
    custom: "",
  };
  return suffixes[kind] ? `${family}_${suffixes[kind]}` : family;
}

function modelSelectionGuide(modelId: string, displayName: string, kind: CompetitionKind): string {
  const family = modelFamily(modelId);
  const generic = modelId === family;
  const expectedExact = exactModelForKind(family, kind);
  const matchesKind = generic || modelId === expectedExact || kind === "custom";
  const familyAdvice = family === "p4"
    ? "P4 是当前默认正式推演系列，适合日常正式输出。"
    : "P7 可正式运行，更适合明确需要 P7 或进行 P4/P7 对照时使用。";
  const routeAdvice = generic
    ? `系统会根据当前“${competitionKindLabels[kind]}”自动匹配对应的 90 分钟模型和最新生产规则。`
    : `当前手动锁定具体模型；只有比赛确实属于“${competitionKindLabels[kind]}”时才建议保持此选择。`;
  const status = generic ? "推荐默认" : matchesKind ? "赛制匹配" : "需要复核";
  return `<aside class="model-selection-guide ${matchesKind ? "recommended" : "warning"}">
    <div class="model-guide-current"><span>模型选择建议</span><strong>${escapeHtml(displayName)}</strong><b>${status}</b></div>
    <div><span>当前比赛类型</span><strong>${escapeHtml(competitionKindLabels[kind])}</strong><small>${escapeHtml(routeAdvice)}</small></div>
    <div><span>${family.toUpperCase()} 定位</span><strong>${family === "p4" ? "正式主推" : "对照与指定运行"}</strong><small>${escapeHtml(familyAdvice)}</small></div>
    <div><span>怎么选</span><strong>${generic ? "不确定时保持自动匹配" : "明确赛制时才手动锁定"}</strong><small>联赛、小组赛、单回合、两回合和友谊赛应使用各自对应模型；不要跨赛制强行套用。</small></div>
  </aside>`;
}


function readinessLevelLabel(level: MatchPredictionReadiness["level"]): string {
  const labels: Record<MatchPredictionReadiness["level"], string> = {
    formal_ready: "可正式推演",
    ready_with_warnings: "可推演，有警告",
    shadow_only: "仅允许影子推演",
    blocked: "禁止推演",
  };
  return labels[level];
}

function readinessStatusLabel(status: MatchPredictionReadiness["checks"][number]["status"]): string {
  return status === "passed" ? "通过" : status === "warning" ? "警告" : "阻断";
}

function predictionReadinessMarkup(
  readiness: MatchPredictionReadiness | null,
  lineupChain: MatchLineupChain | null,
  hasMatch: boolean,
): string {
  if (!hasMatch) return "";
  if (!readiness) {
    return `<section class="panel prediction-readiness-panel unchecked"><div class="panel-heading"><div><span>赛前数据完整度门禁</span><h2>尚未检查正式输入</h2></div><b>未检查</b></div><p>一次检查会核对比赛身份、数据窗口、双方阵容、首发门将、位置与角色、球队历史、模型路由和输入质量。</p><div class="button-row"><button class="secondary" data-action="check-prediction-lineup-chain">检查数据完整度</button>${lineupChain && !lineupChain.ready_for_model ? '<button class="primary" data-action="prepare-prediction-lineups">补齐双方阵容</button>' : ""}</div></section>`;
  }
  const levelClass = readiness.level.replaceAll("_", "-");
  const fingerprint = readiness.input_manifest_sha256?.slice(0, 16) ?? "尚未生成";
  const messages = readiness.blockers.length > 0 ? readiness.blockers : readiness.warnings;
  const messageClass = readiness.blockers.length > 0 ? "blocking-note" : "completion-note";
  return `<section class="panel prediction-readiness-panel ${levelClass}">
    <div class="prediction-readiness-heading"><div><span class="eyebrow">赛前数据完整度门禁</span><h2>${escapeHtml(readinessLevelLabel(readiness.level))}</h2><p>${escapeHtml(readiness.snapshot_type)} · 检查于 ${escapeHtml(new Date(readiness.assessed_at).toLocaleString("zh-CN"))}</p></div><div class="prediction-readiness-score"><strong>${readiness.score}</strong><span>/ 100</span><b>${escapeHtml(readinessLevelLabel(readiness.level))}</b></div></div>
    <div class="prediction-readiness-checks">${readiness.checks.map((check) => `<article class="prediction-readiness-check ${check.status}"><div><span>${escapeHtml(check.label)}</span><b>${check.score}/${check.weight}</b></div><strong>${escapeHtml(readinessStatusLabel(check.status))}</strong><p>${escapeHtml(check.summary)}</p>${check.details.length ? `<small>${check.details.map(escapeHtml).join("；")}</small>` : ""}</article>`).join("")}</div>
    ${messages.length ? `<div class="${messageClass}"><b>${readiness.blockers.length ? "必须处理" : "允许推演，但需关注"}</b><span>${messages.map(escapeHtml).join("；")}</span></div>` : '<div class="completion-note"><b>全部通过</b><span>当前输入满足正式推演标准。</span></div>'}
    <div class="prediction-audit-strip"><div><span>数据截止</span><strong>${readiness.data_cutoff_at ? escapeHtml(new Date(readiness.data_cutoff_at).toLocaleString("zh-CN")) : "未生成"}</strong></div><div><span>输入指纹</span><code title="${escapeHtml(readiness.input_manifest_sha256 ?? "")}">${escapeHtml(fingerprint)}</code></div><div><span>审计版本</span><strong>${escapeHtml(readiness.audit_version)}</strong></div></div>
    <div class="button-row"><button class="secondary" data-action="check-prediction-lineup-chain">重新检查完整度</button>${lineupChain && !lineupChain.ready_for_model ? '<button class="primary" data-action="prepare-prediction-lineups">补齐双方阵容</button>' : ""}${readiness.can_run_shadow && !readiness.can_run_formal ? '<button class="primary" data-action="run-shadow-prediction-match">运行影子推演</button>' : ""}</div>
  </section>`;
}

export function predictionPage(
  state: BootstrapResponse,
  references: PlayerCatalogReferenceData | null,
  selectedMatchId: string | null,
  p4Workspace: P4MatchWorkspace | null,
  p4TaskWorkspace: P4TaskWorkspace | null,
  lineupChain: MatchLineupChain | null,
  readiness: MatchPredictionReadiness | null,
  selectedSnapshot: LineupSnapshotType,
  selectedModelFamily: PredictionModelFamily,
  _moduleSidebarCollapsed: boolean,
  _inspectorCollapsed: boolean,
  activeSection: string,
): string {
  const matches = references?.managed_matches ?? references?.upcoming_matches ?? [];
  const competitionOptions = state.data.competitions.map((competition) => `<option value="${escapeHtml(competition.id)}" data-kind="${competition.competition_kind}">${escapeHtml(competition.name)}</option>`).join("");
  const seasonOptions = state.data.seasons.map((item) => `<option value="${escapeHtml(item.id)}" data-competition-id="${escapeHtml(item.competition_id)}">${escapeHtml(item.name)}</option>`).join("");
  const stageOptions = state.data.stages.map((item) => `<option value="${escapeHtml(item.id)}" data-season-id="${escapeHtml(item.season_id)}" data-competition-id="${escapeHtml(item.competition_id)}" data-kind="${item.stage_kind}">${escapeHtml(item.name)}</option>`).join("");
  const modelFamilyOptions = modelOptions(state, selectedModelFamily);
  const selectedFamily = modelFamily(selectedModelFamily);
  const selectedExactModel = selectedModelFamily === selectedFamily ? null : selectedModelFamily;
  const match = readRecord(state.data.default_match);
  const teamA = readRecord(match.team_a);
  const teamB = readRecord(match.team_b);
  const activeMatchId = selectedMatchId ?? matches[0]?.id ?? null;
  const activeMatch = matches.find((item) => item.id === activeMatchId) ?? null;
  const activeStage = activeMatch?.stage_id
    ? state.data.stages.find((item) => item.id === activeMatch.stage_id) ?? null
    : null;
  const activeCompetition = activeMatch?.competition_id
    ? state.data.competitions.find((item) => item.id === activeMatch.competition_id) ?? null
    : null;
  const activeCompetitionKind = activeStage?.stage_kind ?? activeCompetition?.competition_kind ?? "custom";
  const selectedModel = [...state.data.models]
    .filter((model) => model.model_id === selectedModelFamily)
    .sort((left, right) => compareVersion(right.engine_version, left.engine_version))[0] ?? null;
  const modelGuide = modelSelectionGuide(
    selectedModelFamily,
    selectedModel?.display_name ?? `${selectedFamily.toUpperCase()} 模型`,
    activeCompetitionKind,
  );
  const activeModelPackages = [...state.data.rule_packages]
    .filter((item) => (
      item.status === "active"
      && modelFamily(item.model_id) === selectedFamily
      && (!selectedExactModel || item.model_id === selectedExactModel)
    ))
    .sort((left, right) => new Date(right.created_at).getTime() - new Date(left.created_at).getTime());
  const productionPackages = Array.from(new Map(activeModelPackages.map((item) => [item.package_key, item])).values())
    .sort((left, right) => Number(right.competition_kind === activeCompetitionKind) - Number(left.competition_kind === activeCompetitionKind) || right.priority - left.priority);
  const defaultProductionPackage = productionPackages.find((item) => item.competition_kind === activeCompetitionKind) ?? productionPackages[0] ?? null;
  const recommendedPackages = productionPackages.filter((item) => item.competition_kind === activeCompetitionKind);
  const alternativePackages = productionPackages.filter((item) => item.competition_kind !== activeCompetitionKind);
  const packageOption = (item: (typeof productionPackages)[number]) => `<option value="${escapeHtml(item.id)}" data-competition-kind="${escapeHtml(item.competition_kind)}">${escapeHtml(item.display_name)} · ${escapeHtml(item.version)}</option>`;
  const explicitPackages = `${recommendedPackages.length ? `<optgroup label="当前赛制推荐">${recommendedPackages.map(packageOption).join("")}</optgroup>` : ""}${alternativePackages.length ? `<optgroup label="其他 ${selectedFamily.toUpperCase()} 生产规则（手动覆盖）">${alternativePackages.map(packageOption).join("")}</optgroup>` : ""}`;
  const matchOptions = matches.map((item) => `<option value="${escapeHtml(item.id)}" ${item.id === activeMatchId ? "selected" : ""}>${escapeHtml(item.home_team_name)} vs ${escapeHtml(item.away_team_name)} · ${escapeHtml(new Date(item.kickoff_time).toLocaleString())}</option>`).join("");
  const activeLineupChain = lineupChain?.match_record.id === activeMatchId && lineupChain.snapshot_type === selectedSnapshot
    ? lineupChain
    : null;
  const activeReadiness = readiness?.match_id === activeMatchId
    && readiness.snapshot_type === selectedSnapshot
    && readiness.model_family === selectedModelFamily
    ? readiness
    : null;
  const readinessMarkup = predictionReadinessMarkup(activeReadiness, activeLineupChain, Boolean(activeMatch));
  const section = ["formal", "p4", "history", "simulation"].includes(activeSection) ? activeSection : "formal";
  const sectionNav = workspaceSectionNavigation([
    { id: "formal", index: "01", label: "正式推演", description: "选择比赛、时点与正式输出", badge: `${matches.length}` },
    { id: "p4", index: "02", label: "P4 研究与收敛", description: "三个计划窗口、按需 T-N 与来源审计", badge: `${p4Workspace?.tasks.length ?? 0}` },
    { id: "history", index: "03", label: "推演历史", description: "查看最近正式运行", badge: `${state.data.recent_runs.length}` },
    { id: "simulation", index: "04", label: "临时演练", description: "不保存的快速模拟" },
  ], section);
  const formalView = `<section class="workspace-module-view ${section === "formal" ? "active" : ""}" data-workspace-section="formal">
    <div class="workspace-section-heading"><div><span>正式推演入口</span><h2>选择比赛并准备外部模型输入</h2><p>比赛、模型入口、规则包和数据窗口保持可见；公开仓库不包含模型算法、私有参数或固定回归资产。</p></div><span class="status-pill ${state.data.database_configured ? "online" : "offline"}">${state.data.database_configured ? "数据链可用 · 外部模型待接入" : "等待数据库"}</span></div>
    ${workspaceAnchorNavigation("正式推演", [
      { id: "prediction-form-setup", label: "推演设置" },
      { id: "prediction-form-readiness", label: "完整度门禁" },
      { id: "prediction-result", label: "概率结果" },
      { id: "route-preview", label: "模型判定" },
    ])}
    <section id="prediction-form-setup" class="panel prediction-control-panel focus-panel workspace-anchor-target" data-workspace-persist="false">
      <div class="prediction-control-copy"><span class="step-badge">1</span><div><span class="eyebrow">正式推演设置</span><h2>选择比赛、模型入口和数据窗口</h2><p>P4/P7 的入口、路由与数据准备链已保留；执行必须接入私有或独立部署的 ModelProvider，未接入时返回明确的不可用错误。</p></div></div>
      <div class="prediction-control-grid three-column-form">
        <label class="field prediction-match-field"><span>比赛</span><select id="prediction-match-id" ${matches.length === 0 ? "disabled" : ""}>${matches.length === 0 ? `<option value="">暂无可选比赛</option>` : matchOptions}</select></label>
        <label class="field"><span>计算模型</span><select id="prediction-model-family" data-search-placeholder="输入 P4、P7、联赛、淘汰赛或友谊赛">${modelFamilyOptions}</select><small class="field-note">按模型 ID 去重，仅显示当前注册的最新引擎版本；通用模型自动匹配赛制，具体模型会锁定路由。</small></label>
        <label class="field"><span>数据窗口</span><select id="prediction-stored-snapshot"><option value="T-N" ${selectedSnapshot === "T-N" ? "selected" : ""}>T-N · 任意赛前时间的最新数据</option><option value="T-24h" ${selectedSnapshot === "T-24h" ? "selected" : ""}>T-24h · 开球前 24 小时窗口</option><option value="T-6h" ${selectedSnapshot === "T-6h" ? "selected" : ""}>T-6h · 开球前 6 小时窗口</option><option value="T-1h" ${selectedSnapshot === "T-1h" ? "selected" : ""}>T-1h · 开球前 1 小时窗口</option></select><small class="field-note">固定窗口读取窗口开始后、当前安全截止前的最新有效数据；T-N 读取任意赛前时间的最新安全数据。</small></label>
      </div>
      ${modelGuide}
      <div class="notice-card"><strong>外部模型未捆绑</strong><span>公开仓库仅提供接口壳、路由与数据准备能力；模型算法、参数、Profile 和固定回归资产由外部提供器负责。</span></div>
      <div class="prediction-secondary-controls balanced-prediction-rule-row">
        <label class="field"><span>外部提供器规则入口</span><select id="explicit-rule-package-id"><option value="">自动选择当前入口规则</option>${explicitPackages}</select><small>只显示所选模型系列的公开入口规则；真实参数与执行策略不在仓库中。</small></label>
        <label class="field"><span>参数版本</span><input value="${escapeHtml(defaultProductionPackage?.parameter_version ?? "等待规则路由")}" readonly><small>${defaultProductionPackage ? `${escapeHtml(defaultProductionPackage.model_version)} · 规则 ${escapeHtml(defaultProductionPackage.version)}` : "当前赛事暂无匹配的生产规则"}</small></label>
        <div class="field action-field"><span>提供器连通检查</span><button class="secondary" data-action="dry-run">检查 ${selectedFamily.toUpperCase()} 外部入口</button></div>
      </div>
      <div class="balanced-model-facts">
        <div><span>当前生产规则</span><strong>${escapeHtml(defaultProductionPackage?.display_name ?? "等待自动路由")}</strong></div>
        <div><span>当前模型</span><strong>${escapeHtml(selectedModelFamily.toUpperCase())}</strong></div>
        <div><span>数据窗口</span><strong>${escapeHtml(selectedSnapshot)}</strong></div>
        <div><span>模型运行状态</span><strong>外部 Provider 未捆绑</strong></div>
      </div>
      <div class="prediction-control-actions"><div>${matches.length === 0 ? `<strong>还没有比赛</strong><span>前往比赛中心按赛事、赛季和双方球队创建后返回。</span>` : activeReadiness?.can_run_formal ? `<strong>赛前完整度已通过</strong><span>${activeReadiness.score}/100 · 输入指纹 ${escapeHtml(activeReadiness.input_manifest_sha256?.slice(0, 12) ?? "待生成")}。</span>` : activeReadiness ? `<strong>${escapeHtml(readinessLevelLabel(activeReadiness.level))}</strong><span>${escapeHtml(activeReadiness.blockers[0] ?? activeReadiness.warnings[0] ?? "请处理检查结果")}</span>` : `<strong>运行前自动执行完整度门禁</strong><span>也可先单独检查，查看每项分数、阻塞原因和输入指纹。</span>`}</div>${matches.length === 0 ? `<button class="secondary" data-action="complete-workflow" data-target-page="lineups" data-target-section="matches" data-return-reason="创建比赛后返回正式推演">前往比赛中心</button>` : ""}<button class="secondary" data-action="preview-stored-route" ${state.data.database_configured && matches.length > 0 ? "" : "disabled"}>查看模型判定</button><button class="primary large" data-action="${activeReadiness?.can_run_shadow && !activeReadiness.can_run_formal ? "run-shadow-prediction-match" : "calculate-prediction-match"}" ${state.data.database_configured && matches.length > 0 ? "" : "disabled"}>${activeReadiness?.can_run_shadow && !activeReadiness.can_run_formal ? `运行影子 ${selectedFamily.toUpperCase()}` : `检查并运行 ${selectedFamily.toUpperCase()}`}</button></div>
    </section>
    <div id="prediction-form-readiness" class="workspace-anchor-target">${readinessMarkup}</div>
    <div class="prediction-output-layout"><article class="panel result-panel prediction-result-panel workspace-anchor-target" id="prediction-result"><div class="empty-state prediction-empty"><span class="empty-icon">◎</span><strong>等待推演结果</strong><span>完成后显示最可能比分、胜平负、双方进球和大小球。</span></div></article><article class="panel route-preview prediction-route-panel workspace-anchor-target" id="route-preview"><div class="panel-heading"><div><span>模型判定</span><h2>为什么使用这个模型</h2></div></div><div class="empty-state compact"><strong>尚未检查</strong><span>点击“查看模型判定”后显示赛事、规则和模型的匹配过程。</span></div></article></div>
  </section>`;
  const simulationView = `<section class="workspace-module-view ${section === "simulation" ? "active" : ""}" data-workspace-section="simulation"><div class="workspace-section-heading"><div><span>临时演练</span><h2>不保存比赛的快速模拟</h2><p>公开仓库只保留外部模型入口；运行演练需要接入私有或独立部署的 ModelProvider。</p></div></div><section class="panel stacked"><div class="simulation-context-grid"><label class="field"><span>模型</span><select id="simulation-model-family">${modelFamilyOptions}</select></label><label class="field"><span>赛事</span><select id="competition-id"><option value="">按赛事类型自动选择</option>${competitionOptions}</select></label><label class="field"><span>赛季</span><select id="season-id"><option value="">不限定</option>${seasonOptions}</select></label><label class="field"><span>阶段</span><select id="stage-id"><option value="">不限定</option>${stageOptions}</select></label><label class="field"><span>赛事类型</span><select id="competition-kind">${competitionKindOptions("custom")}</select></label><label class="field"><span>数据窗口</span><select id="snapshot-type"><option value="T-N" selected>T-N · 任意赛前时间</option><option value="T-24h">T-24h · 24 小时窗口</option><option value="T-6h">T-6h · 6 小时窗口</option><option value="T-1h">T-1h · 1 小时窗口</option></select></label></div><div class="simulation-match-grid"><label class="field simulation-kickoff"><span>开球时间</span><input id="simple-kickoff" type="datetime-local" value="${escapeHtml(localInputValue(match.kickoff_time))}"></label><article class="simulation-team-card home"><span>主队</span><label class="field"><span>球队名称</span><input id="simple-home-name" value="${escapeHtml(String(teamA.name ?? ""))}" placeholder="输入主队名称"></label></article><div class="simulation-versus">VS</div><article class="simulation-team-card away"><span>客队</span><label class="field"><span>球队名称</span><input id="simple-away-name" value="${escapeHtml(String(teamB.name ?? ""))}" placeholder="输入客队名称"></label></article></div><div class="notice-card"><strong>外部模型未捆绑</strong><span>入口、路由和数据准备链保留；模型算法、参数、Profile 与固定回归资产不在此仓库中。</span></div><div class="workflow-actions"><button class="secondary" data-action="preview-route">检查模型</button><button class="primary" data-action="execute-prediction">运行演练</button></div></section></section>`;
  const contextRibbon = taskContextRibbon([
    { label: "当前比赛", value: activeMatch ? `${activeMatch.home_team_name} vs ${activeMatch.away_team_name}` : "尚未选择", note: activeMatch ? new Date(activeMatch.kickoff_time).toLocaleString() : "先在正式推演中选择比赛", tone: activeMatch ? "accent" : "neutral" },
    { label: "正式模型", value: selectedModelFamily.toUpperCase(), note: defaultProductionPackage ? `${defaultProductionPackage.display_name} · ${defaultProductionPackage.version}` : "等待生产规则路由" },
    { label: "数据窗口", value: selectedSnapshot, note: activeReadiness?.data_cutoff_at ? `截止 ${new Date(activeReadiness.data_cutoff_at).toLocaleString()}` : "等待完整度检查" },
    { label: "赛前门禁", value: activeReadiness ? readinessLevelLabel(activeReadiness.level) : "尚未检查", note: activeReadiness ? `${activeReadiness.score}/100 · ${activeReadiness.input_manifest_sha256?.slice(0, 12) ?? "未生成指纹"}` : "检查比赛、阵容、历史、路由与输入质量", tone: activeReadiness?.can_run_formal ? "success" : activeReadiness ? "warning" : "neutral" },
  ]);
  return `<section class="task-page core-workspace-page core-prediction-workspace">
  ${taskPageHeader({ eyebrow: "赛事推演", title: "P4 / P7 外部模型入口", description: "正式数据准备、研究工作台、历史和临时演练保持在同一页面；真实模型由外部 ModelProvider 提供。", status: { label: state.data.database_configured ? "数据链可用 · 外部模型待接入" : "等待数据库", tone: "warning" } })}${state.data.database_configured ? "" : inlineDatabaseSetup("连接数据服务以使用正式推演", "临时演练仍可使用；连接成功后可创建比赛、推演并查看记录。", state.connection_error)}
  ${contextRibbon}
  <div class="core-local-navigation">${sectionNav}</div>
  <div class="core-workspace-stage"><section class="balanced-workspace prediction-balanced-workspace master-detail-workspace">
    <main class="balanced-workspace-main" data-workspace-scroll-key="prediction-main">${formalView}<section class="workspace-module-view ${section === "p4" ? "active" : ""}" data-workspace-section="p4"><div class="workspace-section-heading"><div><span>P4 研究与收敛</span><h2>正式运行与重新校准分开管理</h2><p>正式运行使用已验证参数；新增历史仅用于后续回测、漂移监测与收敛。</p></div></div>${p4WorkbenchMarkup(state.data.database_configured, state.data.rule_packages, references?.managed_matches ?? [], p4Workspace, p4TaskWorkspace)}</section><section class="workspace-module-view ${section === "history" ? "active" : ""}" data-workspace-section="history"><div class="workspace-section-heading"><div><span>推演历史</span><h2>最近的正式推演</h2><p>保留比赛、模型、窗口、比分和胜平负；技术血缘按需打开。</p></div><button class="secondary" data-action="refresh-runs">刷新</button></div><section class="panel prediction-history-panel">${runHistoryMarkup(state.data.recent_runs, true)}</section></section>${simulationView}</main>
  </section></div>
  </section>`;
}

