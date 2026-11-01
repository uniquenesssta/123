import { competitionKindOptions } from "../components/competition";
import { escapeHtml } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { formalPredictionView,predictionExecutionBar } from "../features/prediction/formalPredictionView";
import type { PredictionFeature } from "../features/prediction/predictionFeature";

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

export function predictionPage(
  state: BootstrapResponse,
  references: PlayerCatalogReferenceData | null,
  selectedMatchId: string | null,
  p4Workspace: P4MatchWorkspace | null,
  p4TaskWorkspace: P4TaskWorkspace | null,
  _lineupChain: MatchLineupChain | null,
  readiness: MatchPredictionReadiness | null,
  selectedSnapshot: LineupSnapshotType,
  selectedModelFamily: PredictionModelFamily,
  _moduleSidebarCollapsed: boolean,
  _inspectorCollapsed: boolean,
  activeSection: string,
  workflow?:PredictionFeature["view"],
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
  const activeReadiness = readiness?.match_id === activeMatchId
    && readiness.snapshot_type === selectedSnapshot
    && readiness.model_family === selectedModelFamily
    ? readiness
    : null;
  const section = ["formal", "p4", "history", "simulation"].includes(activeSection) ? activeSection : "formal";
  const sectionNav=`<nav class="prediction-section-nav" aria-label="推演工作区">${[["formal","正式推演"],["p4","P4 研究与收敛"],["history","推演历史"],["simulation","临时演练"]].map(([id,label])=>`<button class="${section===id?"primary":"secondary"}" data-action="select-workspace-section" data-section-id="${id}" aria-current="${section===id?"page":"false"}">${label}</button>`).join("")}</nav>`;
  const snapshotOptions=(["T-N","T-24h","T-6h","T-1h"] as const).map((type)=>`<option value="${type}" ${type===selectedSnapshot?"selected":""}>${type==="T-N"?"T-N · 任意赛前时间的最新数据":type+" · 开球前 "+type.slice(2,-1)+" 小时窗口"}</option>`).join("");
  const selectedPackage=productionPackages.find(item=>item.id===workflow?.context.explicit_rule_package_id)??defaultProductionPackage;
  const formalOptions={enabled:state.data.database_configured&&matches.length>0,matchOptions,modelOptions:modelFamilyOptions,snapshotOptions,ruleOptions:`<option value="">自动选择当前入口规则</option>${explicitPackages}`,parameter:selectedPackage?.parameter_version??"由实际提供器返回，不在仓库中",guide:modelGuide,family:selectedFamily,view:workflow??{readiness:activeReadiness,phase:activeReadiness?"ready" as const:"unchecked" as const,busy:false,canFormal:activeReadiness?.can_run_formal??true,canShadow:activeReadiness?.can_run_shadow??false}};
  const formalView=formalPredictionView(formalOptions);
  const simulationView = `<section class="workspace-module-view ${section === "simulation" ? "active" : ""}" data-workspace-section="simulation"><div class="workspace-section-heading"><div><span>临时演练</span><h2>不保存比赛的快速模拟</h2><p>公开仓库只保留外部模型入口；运行演练需要接入私有或独立部署的 ModelProvider。</p></div></div><section class="panel stacked"><div class="simulation-context-grid"><label class="field"><span>模型</span><select id="simulation-model-family">${modelFamilyOptions}</select></label><label class="field"><span>赛事</span><select id="competition-id"><option value="">按赛事类型自动选择</option>${competitionOptions}</select></label><label class="field"><span>赛季</span><select id="season-id"><option value="">不限定</option>${seasonOptions}</select></label><label class="field"><span>阶段</span><select id="stage-id"><option value="">不限定</option>${stageOptions}</select></label><label class="field"><span>赛事类型</span><select id="competition-kind">${competitionKindOptions("custom")}</select></label><label class="field"><span>数据窗口</span><select id="snapshot-type"><option value="T-N" selected>T-N · 任意赛前时间</option><option value="T-24h">T-24h · 24 小时窗口</option><option value="T-6h">T-6h · 6 小时窗口</option><option value="T-1h">T-1h · 1 小时窗口</option></select></label></div><div class="simulation-match-grid"><label class="field simulation-kickoff"><span>开球时间</span><input id="simple-kickoff" type="datetime-local" value="${escapeHtml(localInputValue(match.kickoff_time))}"></label><article class="simulation-team-card home"><span>主队</span><label class="field"><span>球队名称</span><input id="simple-home-name" value="${escapeHtml(String(teamA.name ?? ""))}" placeholder="输入主队名称"></label></article><div class="simulation-versus">VS</div><article class="simulation-team-card away"><span>客队</span><label class="field"><span>球队名称</span><input id="simple-away-name" value="${escapeHtml(String(teamB.name ?? ""))}" placeholder="输入客队名称"></label></article></div><div class="notice-card"><strong>外部模型未捆绑</strong><span>入口、路由和数据准备链保留；模型算法、参数、Profile 与固定回归资产不在此仓库中。</span></div><div class="workflow-actions"><button class="secondary" data-action="preview-route">检查模型</button><button class="primary" data-action="execute-prediction">运行演练</button></div></section></section>`;
  const contextSummary=`${activeMatch?escapeHtml(activeMatch.home_team_name+" vs "+activeMatch.away_team_name):"尚未选择比赛"} · ${escapeHtml(selectedModelFamily.toUpperCase())} · ${escapeHtml(selectedSnapshot)} · ${activeReadiness?escapeHtml(readinessLevelLabel(activeReadiness.level)):"尚未检查"}`;
  return `<section class="prediction-workspace core-workspace-page core-prediction-workspace"><header class="prediction-page-heading"><h1>P4 / P7 外部模型入口</h1><p>正式数据准备、研究与收敛、历史和临时演练；真实模型由独立 ModelProvider 提供。</p></header>${state.data.database_configured ? "" : inlineDatabaseSetup("连接数据服务以使用正式推演", "临时演练仍可使用；连接成功后可创建比赛、推演并查看记录。", state.connection_error)}
  <p class="prediction-context-summary">${contextSummary}</p><div class="core-local-navigation">${sectionNav}</div>
  <div class="core-workspace-stage"><main class="prediction-viewport" data-workspace-scroll-key="prediction-main">${section==="formal"?formalView:""}<section class="workspace-module-view ${section === "p4" ? "active" : ""}" data-workspace-section="p4"><div class="workspace-section-heading"><div><span>P4 研究与收敛</span><h2>正式运行与重新校准分开管理</h2><p>正式运行使用已验证参数；新增历史仅用于后续回测、漂移监测与收敛。</p></div></div>${p4WorkbenchMarkup(state.data.database_configured, state.data.rule_packages, references?.managed_matches ?? [], p4Workspace, p4TaskWorkspace)}</section><section class="workspace-module-view ${section === "history" ? "active" : ""}" data-workspace-section="history"><div class="workspace-section-heading"><div><span>推演历史</span><h2>最近的正式推演</h2><p>保留比赛、模型、窗口、比分和胜平负；技术血缘按需打开。</p></div><button class="secondary" data-action="refresh-runs">刷新</button></div><section class="panel prediction-history-panel">${runHistoryMarkup(state.data.recent_runs, true)}</section></section>${simulationView}</main></div>${section==="formal"?predictionExecutionBar(formalOptions):""}
  </section>`;
}
