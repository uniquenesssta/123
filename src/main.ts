import {
api,
issueWasLogged,
recordUiOperation,
recordUiOperationFailure,
} from "./api/client";
import { ContextMenuAdapter } from "./app/context-menu";
import { AsyncScope,EventScope,MountHandle } from "./app/lifecycle";
import { matchReviewWorkflowAllows } from "./app/matchReviewWorkflow";
import { ModalController,type ModalConfirmationOptions } from "./app/modal";
import { NavigationContext,PRIMARY_NAVIGATION,Router,isRoute,type NavigationRequest } from "./app/router";
import { AppSession } from "./app/session";
import { ApplicationShellController,pageTitle,renderShell } from "./app/shell";
import { WarningCenterController } from "./app/warnings";
import { WorkspaceCoordinator,type WorkspaceLayoutMode,type WorkspaceTabState } from "./app/workspace";
import { bulkArchiveSummary } from "./components/entityDeletion";
import { escapeHtml,formatPercent } from "./components/format";
import { enhanceSearchableSelects,refreshSearchableSelects } from "./components/searchableSelect";
import {
fetchAnalysisCenter,
fetchLineups,

fetchReviewCenter,
fetchReviewLineups,
type AnalysisCenterLoadResult,
type LineupsLoadResult,

type ReviewCenterLoadResult,
} from "./controllers/pageLoaders";
import { bindSearchableSelectDiagnostics } from "./diagnostics/searchableSelectDiagnostics";
import { activateSectionPanel } from "./ui/navigation";
import { architecturePage } from "./features/architecture";
import { CompetitionFeature,fitCatalogueRegions,selectCatalogueRegion,clearRulesCatalogueFilters,filterRulesCompetitionCatalogue,selectRulesDirectoryLevel } from "./features/competitions";
import { dashboardPage } from "./features/dashboard";
import { DatabaseFeature,databasePage,databaseTechnical } from "./features/database";
import { createLineupsFeature } from "./features/lineups";
import type { LineupSide } from "./features/lineups/lineupContext";
import { emptyPairedLineupSide } from "./features/lineups/lineupState";
import { PlayersFeature,playersPage } from "./features/players";
import { RulesFeature,routeCard,routeDetailBody,rulesPage } from "./features/rules";
import { TeamsFeature,teamsPage } from "./features/teams";
import { analyticsPage } from "./pages/analytics";
import { apiWorkspacePage } from "./pages/apiWorkspace";
import { lineupPresetsPage } from "./pages/lineupPresets";
import { lineupsPage } from "./pages/lineups";
import { logsPage } from "./pages/logs";
import { openAiPage } from "./pages/openai";
import { predictionPage } from "./pages/prediction";
import { releasePage } from "./pages/release";
import { reviewPage } from "./pages/review";
import { runsPage } from "./pages/runs";
import { workbooksPage } from "./pages/workbooks";
import { clipboard } from "./platform/clipboard";
import { localPreferences,preferenceKey,sessionPreferences } from "./platform/storage";
import { stopTauriTransport } from "./platform/tauri";
import { DebouncedTask } from "./platform/timing";
import "./styles/index.css";
import type {
AbilityUpdateCandidateRecord,
AiAnalysisResponsePreview,
AiAnalysisSuggestionRecord,
AnalyticsOverview,
ApiWorkspacePreset,
ApiWorkspaceSessionDetail,
ApiWorkspaceSessionRecord,
BackgroundJob,
CoachListItem,
CompetitionKind,
EnqueueJobDraft,
EvidenceScoringDecisionDraft,
FormationRecord,
IssueLogEntry,
IssueSeverity,
LineupRecord,
LineupSnapshotType,
MatchLineupChain,
MatchPredictionReadiness,
MatchRecord,
MatchReviewDetail,
MatchReviewDraft,
MatchReviewPackagePreview,
MatchReviewPackageWorkflowRecord,
MatchReviewSummary,
OpenAiApiExampleParseResult,
OpenAiApiProtocol,
OpenAiProfileDraft,
OpenAiProfilesState,
P4MatchWorkspace,
P4TaskWorkspace,
Page,
ParameterLifecycleReadinessRequest,
ParameterPromotionRequest,
ParameterRollbackRequest,
ParameterTuningCandidateRecord,
ParameterTuningDecisionDraft,
ParameterTuningDraft,
PlayerListQuery,
PlayerMatchObservationDraft,
PlayerNavigationContext,
PostmatchMonitoringRequest,
PostmatchOverview,
PostmatchSettlementRecord,
PredictionCommand,
PredictionExecution,
PredictionModelFamily,
ReleaseAcceptanceRequest,
ReleaseAcceptanceRun,
ReleaseAcceptanceRunSummary,
ResolveP4ConflictCommand,
ReviewableMatch,
RouteDecision,
RoutePreviewCommand,
TeamListQuery
} from "./types";
import { KeyboardShortcutController,motionScrollBehavior } from "./ui/accessibility";
import { focusValidationTarget } from "./ui/forms/validation-summary";
import { CommandMenuController } from "./ui/overlays/command-menu";

const browserLifecycleController = new EventScope();
const openAiExampleRequests = new AsyncScope();

const DATABASE_RESET_COMPLETE_KEY = preferenceKey("database-reset-complete");
const ANALYSIS_PACKAGE_ID_KEY = preferenceKey("last-analysis-package-id");
const TEAM_QUERY_KEY = preferenceKey("team-query");
const PLAYER_QUERY_KEY = preferenceKey("player-query");
const PLAYER_NAV_CONTEXT_KEY = preferenceKey("player-navigation-context");

let app: HTMLDivElement;

const session = new AppSession({
  loadBootstrap: () => api.bootstrap(),
  loadRecentRuns: () => api.listRecentRuns(100),
  applyTheme(theme) {
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.colorScheme = theme;
  },
});
const databaseFeature = new DatabaseFeature({
  snapshot: () => session.bootstrap,
  configure: options => api.configureDatabase(options),
  disconnect: () => api.disconnectDatabase(),
  reset: confirmation => api.resetDatabase(confirmation),
  refresh: () => session.refresh(),
  busy: operation => runBusy(operation),
  render: () => render(),
  notify: message => toast(message, "success"),
  confirm: (...args) => modal.confirm(...args),
  invalidateConfirmation: () => modal.close(),
  async completeReset() {
    await workspaceState.clearAll();
    workspaceState.stop();
    localPreferences.removeItem(TEAM_QUERY_KEY);
    localPreferences.removeItem(PLAYER_QUERY_KEY);
    localPreferences.removeItem(PLAYER_NAV_CONTEXT_KEY);
    sessionPreferences.setItem(DATABASE_RESET_COMPLETE_KEY, "1");
    window.location.reload();
  },
});
const rulesFeature = new RulesFeature({
  snapshot: () => session.bootstrap,
  registerRulePackage: draft => api.registerRulePackage(draft),
  createBinding: draft => api.createCompetitionBinding(draft),
  previewRoute: command => api.previewRoute(command),
  refresh: () => session.refresh(),
  busy: operation => runBusy(operation),
  render: () => render({ preserveForm: true }),
  active: () => !browserLifecycleController.signal.aborted && navigation.page === "rules",
  notify: message => toast(message, "success"),
});
const competitionFeature = new CompetitionFeature({
  snapshot: () => session.bootstrap,
  createCompetition: draft => api.createCompetition(draft),
  createSeason: draft => api.createSeason(draft),
  createStage: draft => api.createStage(draft),
  createRound: draft => api.createRound(draft),
  deleteCompetition: id => api.deleteCompetition(id),
  refresh: () => session.refresh(),
  busy: operation => runBusy(operation),
  render: () => render({ preserveForm: true }),
  active: () => !browserLifecycleController.signal.aborted && navigation.page === "rules",
  invalidateRoutes: () => rulesFeature.invalidatePreview(),
  notify: message => toast(message, "success"),
  confirm: (...args) => modal.confirm(...args),
  showPath: body => showHtmlModal("赛事规则与模型路径", "目录级说明", body),
});

const teamsFeature = new TeamsFeature({
  api,
  initialQuery: localPreferences.readObject<TeamListQuery>(TEAM_QUERY_KEY),
  snapshot: () => session.bootstrap,
  page: () => navigation.page,
  root: () => app,
  busy: operation => runBusy(operation),
  persistQuery: query => localPreferences.writeJson(TEAM_QUERY_KEY, query),
  publishOptions(coaches, formations) { coachList = coaches; formationCatalog = formations; },
  publishCoaches: coaches => { coachList = coaches; },
  refreshPlayerReferences: async () => { const snapshot = session.bootstrap; const references = await api.playerCatalogReferenceData(); if (snapshot === session.bootstrap) playersFeature.catalog.setReferences(references); },
  refreshAll: () => refreshBootstrapAndCatalog(),
  clearPlayerInspector: () => { playersFeature.catalog.adopt(null); },
  render: () => render({ preserveForm: true }),
  notify: (message, kind) => toast(message, kind),
  setSection(section, inspectorCollapsed) { workspaceState.patchModule("teams", { active_section: section, ...(inspectorCollapsed !== undefined ? { inspector_collapsed: inspectorCollapsed } : {}), ...(section === "directory" && !teamsFeature.catalog.detail ? { active_tab_id: null } : {}) }); },
  appendTab: (id, label) => appendWorkspaceTab("teams", { id, label }),
  activeTab: page => workspaceState.module(page).active_tab_id,
  setPresetTab: id => workspaceState.patchModule("lineup_presets", { active_tab_id: id }),
  navigateTeams: () => navigateTo("teams"),
  confirm: (...args) => modal.confirm(...args),
  showHtml: (...args) => showHtmlModal(...args),
  closePanel: () => closeModal(),
  async afterDelete(result, missing) {
    const page = navigation.page, activeId = removeWorkspaceObjects("teams", [...missing, ...result.deleted_ids]), revision = teamsFeature.catalog.revision;
    await session.refresh();
    if (page !== navigation.page || revision !== teamsFeature.catalog.revision) return false;
    if (!await teamsFeature.catalog.load({ resetCursor: true, retainId: activeId, isCurrent: () => navigation.page === page })) return false;
    const current = teamsFeature.catalog.capture(), references = await api.playerCatalogReferenceData();
    if (!current() || page !== navigation.page) return false;
    playersFeature.catalog.setReferences(references); persistWorkspaceSelection("teams"); render({ preserveForm: true }); return true;
  },
  async removeMissing(ids) {
    const page = navigation.page, activeId = removeWorkspaceObjects("teams", ids), revision = teamsFeature.catalog.revision;
    await session.refresh(); if (page !== navigation.page || revision !== teamsFeature.catalog.revision) return;
    if (await teamsFeature.catalog.load({ resetCursor: true, retainId: activeId, isCurrent: () => navigation.page === page })) { persistWorkspaceSelection("teams"); render({ preserveForm: true }); }
  },
  async afterForceDelete(result) {
    const page = navigation.page, activeId = removeWorkspaceObjects("teams", [result.team_id]);
    removeWorkspaceObjects("players", result.deleted_player_ids); playersFeature.imports.clear(); lineupsFeature.imports.clear();
    const revision = teamsFeature.catalog.revision;
    await session.refresh(); if (page !== navigation.page || revision !== teamsFeature.catalog.revision) return false;
    if (!await teamsFeature.catalog.load({ resetCursor: true, retainId: activeId, isCurrent: () => navigation.page === page })) return false;
    const current = teamsFeature.catalog.capture();
    if (!await playersFeature.load(true, () => current() && page === navigation.page)) return false;
    persistWorkspaceSelection("teams"); persistWorkspaceSelection("players"); render({ preserveForm: true }); return true;
  },
  async afterArchive(result) {
    const page = navigation.page, activeId = removeWorkspaceObjects("teams", [...result.archived_ids, ...result.already_archived_ids]), revision = teamsFeature.catalog.revision;
    await session.refresh(); if (page !== navigation.page || revision !== teamsFeature.catalog.revision) return;
    if (!await teamsFeature.catalog.load({ resetCursor: true, retainId: activeId, isCurrent: () => navigation.page === page })) return;
    const current = teamsFeature.catalog.capture(), references = await api.playerCatalogReferenceData();
    if (!current() || page !== navigation.page) return;
    playersFeature.catalog.setReferences(references); persistWorkspaceSelection("teams"); render({ preserveForm: true });
    showHtmlModal("球队批量归档结果", "保留历史关系", bulkArchiveSummary(result, "支球队"));
  },
});


const playersFeature = new PlayersFeature({
 api,initialQuery:localPreferences.readObject<PlayerListQuery>(PLAYER_QUERY_KEY),initialSource:localPreferences.readObject<PlayerNavigationContext>(PLAYER_NAV_CONTEXT_KEY),snapshot:()=>session.bootstrap,page:()=>navigation.page,root:()=>app,
 busy:operation=>runBusy(operation),persistQuery:query=>localPreferences.writeJson(PLAYER_QUERY_KEY,query),persistSource:source=>{if(source)localPreferences.writeJson(PLAYER_NAV_CONTEXT_KEY,source);else localPreferences.removeItem(PLAYER_NAV_CONTEXT_KEY);},
 render:()=>render({preserveForm:true}),notify:(message,kind)=>toast(message,kind),section:(active_section,inspector_collapsed)=>workspaceState.patchModule("players",{active_section,...(inspector_collapsed!==undefined?{inspector_collapsed}:{})}),appendTab:(id,label)=>appendWorkspaceTab("players",{id,label}),refreshAll:()=>refreshBootstrapAndCatalog(),confirm:(...args)=>modal.confirm(...args),showHtml:(...args)=>showHtmlModal(...args),showResult:(title,result)=>showModal(title,result),
 async afterMutation(ids){const page=navigation.page,active=removeWorkspaceObjects("players",ids),revision=playersFeature.catalog.revision;await session.refresh();if(page!==navigation.page||revision!==playersFeature.catalog.revision)return false;
 if(!await playersFeature.load(true,()=>page===navigation.page))return false;const current=playersFeature.catalog.capture();
 if(teamsFeature.catalog.list!==null){await teamsFeature.load(true);if(!current()||page!==navigation.page)return false;}
 persistWorkspaceSelection("players");if(active&&playersFeature.catalog.detail?.player.id===active)appendWorkspaceTab("players",{id:active,label:playersFeature.catalog.detail.player.canonical_name});render({preserveForm:true});return true;}
});

const navigation = new Router();
const workflowNavigation = new NavigationContext();
let lastPredictionResult: unknown = null;
let coachList: CoachListItem[] = [];
let formationCatalog: FormationRecord[] = [];
let selectedPredictionReadiness: MatchPredictionReadiness | null = null;
let reviewableMatches: ReviewableMatch[] = [];
let recentMatchReviews: MatchReviewSummary[] = [];
let selectedReviewMatchId: string | null = null;
let reviewLineups: LineupRecord[] = [];
let selectedMatchReview: MatchReviewDetail | null = null;
let matchReviewPackagePreview: MatchReviewPackagePreview | null = null;
let matchReviewPackageWorkflow: MatchReviewPackageWorkflowRecord | null = null;
let selectedReviewSettlement: PostmatchSettlementRecord | null = null;
let analyticsOverview: AnalyticsOverview | null = null;
let analysisJobs: BackgroundJob[] = [];
let aiAnalysisResponsePreview: AiAnalysisResponsePreview | null = null;
let aiAnalysisResponsePath: string | null = null;
let aiAnalysisSuggestions: AiAnalysisSuggestionRecord[] = [];
let analysisAbilityCandidates: AbilityUpdateCandidateRecord[] = [];
let lastAiAnalysisPackageId: string | null = localPreferences.getItem(ANALYSIS_PACKAGE_ID_KEY);

const lineupsFeature = createLineupsFeature({
recordUiOperation,recordUiOperationFailure,pageTitle,
get references(){return playersFeature.catalog.references;},
setReferences:value=>playersFeature.catalog.setReferences(value),
get coachList(){return coachList;}, set coachList(v){coachList=v;},
localDateTimeInputValue:(...args)=>localDateTimeInputValue(...args),
api,files:api,
render:(...args)=>render(...args),
get formationCatalog(){return formationCatalog;}, set formationCatalog(v){formationCatalog=v;},
toast:(...args)=>toast(...args),
get teamDetail(){return teamsFeature.catalog.detail;},get teamPresets(){return teamsFeature.catalog.presets;},setTeamPresets:(id,presets)=>teamsFeature.catalog.setPresets(id,presets),refreshTeam:async()=>{await teamsFeature.catalog.refreshSelected();},
showHtmlModal:(...args)=>showHtmlModal(...args),
value:(...args)=>value(...args),
nullableValue:(...args)=>nullableValue(...args),
nullableNumber:(...args)=>nullableNumber(...args),
checked:(...args)=>checked(...args),
runBusy:(...args)=>runBusy(...args),
showConfirmation:(...args)=>showConfirmation(...args),
closeModal:(...args)=>closeModal(...args),
localDateTimeToIso:(...args)=>localDateTimeToIso(...args),
get lastPredictionResult(){return lastPredictionResult;}, set lastPredictionResult(v){lastPredictionResult=v;},
get workspaceState(){ return workspaceState; },
session,
filterSelectOptions:(...args)=>filterSelectOptions(...args),
get app(){ return app; },
refreshBootstrapAndCatalog:(...args)=>refreshBootstrapAndCatalog(...args),
navigation,
recordClientIssue:(...args)=>recordClientIssue(...args),
userFacingError:(...args)=>userFacingError(...args),
get selectedReviewMatchId(){return selectedReviewMatchId;}, set selectedReviewMatchId(v){selectedReviewMatchId=v;},
loadReviewLineups:(...args)=>loadReviewLineups(...args),
missingLineupTeam:(...args)=>missingLineupTeam(...args)
});

interface WorkflowContinuation {
  readonly reason: string;
  readonly matchId: string | null;
  readonly snapshotType: LineupSnapshotType | null;
}

let workflowContinuation: WorkflowContinuation | null = null;
let parameterTuningCandidates: ParameterTuningCandidateRecord[] = [];
let postmatchOverview: PostmatchOverview = { settlement_count: 0, pending_evidence_count: 0, scored_evidence_count: 0, settlements: [], evidence_queue: [], provider_scores: [], drift_runs: [] };
let releaseAcceptanceRuns: ReleaseAcceptanceRunSummary[] = [];
let selectedReleaseAcceptanceRun: ReleaseAcceptanceRun | null = null;
let issueLogs: IssueLogEntry[] = [];
let openAiProfiles: OpenAiProfilesState | null = null;
let selectedOpenAiProfileId: string | null = null;
let creatingOpenAiProfile = false;
const openAiApiExampleTimer = new DebouncedTask(browserLifecycleController);
let openAiApiExampleLastParsed = "";
let selectedP4MatchId: string | null = null;
let selectedPredictionSnapshot: LineupSnapshotType = "T-N";
let selectedPredictionModelFamily: PredictionModelFamily = "p4";
let p4MatchWorkspace: P4MatchWorkspace | null = null;
let p4TaskWorkspace: P4TaskWorkspace | null = null;
let apiWorkspacePresets: ApiWorkspacePreset[] = [];
let apiWorkspaceSessions: ApiWorkspaceSessionRecord[] = [];
let apiWorkspaceDetail: ApiWorkspaceSessionDetail | null = null;
let apiWorkspaceMatches: MatchRecord[] = [];
let selectedApiWorkspacePresetKey = "plain_chat";
let selectedApiWorkspaceProfileId = "";
let selectedApiWorkspaceMatchId: string | null = null;
let apiWorkspaceDraftMessage = "";
let apiWorkspaceSending = false;
let apiWorkspacePendingMessage: {
  content: string;
  started_at: string;
  session_id: string | null;
} | null = null;
let selectedApiWorkspaceContextEntityType: "team" | "player" | null = null;
let selectedApiWorkspaceContextEntityId: string | null = null;
let selectedApiWorkspaceContextEntityLabel: string | null = null;
let apiWorkspaceSessionSearch = "";
let apiWorkspaceIncludeContext = false;
let apiWorkspaceActiveRequestId: string | null = null;
const workspaceState = new WorkspaceCoordinator<Page>({
  read: () => api.readWorkspaceState(),
  save: (document) => api.saveWorkspaceState(document),
  clear: () => api.clearWorkspaceState(),
}, {
  onError({ operation, error }) {
    recordClientIssue(error, `工作区 / ${operation === "read" ? "读取状态" : operation === "clear" ? "清除状态" : "保存状态"}`);
    warningCenter?.record({
      key: `workspace:${operation}`, source: "工作区", severity: "warning", target: "logs",
      title: operation === "read" ? "工作区恢复失败" : operation === "clear" ? "工作区未能清除" : "工作区尚未保存",
      message: operation === "read" ? "原状态文件已保留，本次暂用临时工作区。请查看问题日志，修复后重新启动。" : "当前操作未完成，本次工作区仍保留。请查看问题日志后重试。",
    });
  },
});
const modal = new ModalController(error => { recordClientIssue(error, "确认操作");return userFacingError(error); });
const contextMenu = new ContextMenuAdapter();
const commandMenu = new CommandMenuController(document,error => { recordClientIssue(error,"快速导航");return userFacingError(error); });
const globalShortcuts = new KeyboardShortcutController(document,[{key:"k",run:openCommandMenu,allowInEditable:true}]);
function openCommandMenu(): void {
  contextMenu.close();
  commandMenu.open(PRIMARY_NAVIGATION.flatMap(module => module.items.map(item => ({id:item.page,label:item.label,description:item.description,keywords:module.label+" "+item.page,run:async()=>{
    if(item.page===navigation.page)return;
    if(item.page==="players")playersFeature.directEntry();
    await navigateTo(item.page);
  }}))));
}
let shellController: ApplicationShellController | null = null;
let warningCenter: WarningCenterController | null = null;
let renderedPage: Page | null = null;

interface RenderOptions {
  readonly preserveForm?: boolean;
}

function renderWorkflowContinuationBanner(): string {
  if (!workflowContinuation || !workflowNavigation.target || navigation.page === workflowNavigation.target.page) return "";
  return `<section class="workflow-continuation-banner" role="status"><div><span>正在补齐前置资料</span><strong>${escapeHtml(workflowContinuation.reason)}</strong><small>完成当前资料后可返回原工作流，比赛和时间窗口不会丢失。</small></div><div class="button-row"><button class="secondary" data-action="workflow-return">返回原任务</button><button class="ghost" data-action="workflow-cancel">取消返回</button></div></section>`;
}

function workflowSection(pageName: Page): string | null {
  return workspaceState.module(pageName).active_section ?? null;
}

async function startWorkflowCompletion(button: HTMLElement): Promise<void> {
  const targetPage = button.dataset.targetPage as Page | undefined;
  if (!targetPage) throw new Error("缺少补录目标页面");
  workflowNavigation.set({ page: navigation.page, section: workflowSection(navigation.page) });
  workflowContinuation = {
    reason: button.dataset.returnReason ?? "补齐缺失资料后返回",
    matchId: lineupsFeature.state.pairedLineupBuilder.match_id || lineupsFeature.state.selectedManagedMatchId,
    snapshotType: lineupsFeature.state.pairedLineupBuilder.snapshot_type,
  };
  const targetSection = button.dataset.targetSection ?? null;
  if (targetSection) workspaceState.patchModule(targetPage, { active_section: targetSection });
  await navigateTo(targetPage);
}

async function returnToWorkflow(): Promise<void> {
  const continuation = workflowContinuation;
  const returnTarget = workflowNavigation.take();
  if (!continuation || !returnTarget) return;
  workflowContinuation = null;
  if (returnTarget.section) {
    workspaceState.patchModule(returnTarget.page, {
      active_section: returnTarget.section,
    });
  }
  if (continuation.matchId) {
    lineupsFeature.state.selectedManagedMatchId = continuation.matchId;
    selectedP4MatchId = continuation.matchId;
    lineupsFeature.state.pairedLineupBuilder = {
      ...lineupsFeature.state.pairedLineupBuilder,
      match_id: continuation.matchId,
      snapshot_type: continuation.snapshotType ?? lineupsFeature.state.pairedLineupBuilder.snapshot_type,
    };
  }
  await navigateTo(returnTarget.page);
  if (returnTarget.page === "lineups" && continuation.matchId) {
    lineupsFeature.actions.resetPairedBuilderForMatch(continuation.matchId, false);
    await lineupsFeature.actions.loadBothPairedLineupSides();
    render({ preserveForm: true });
  }
}

function render(options: RenderOptions = {}): void {
  if (browserLifecycleController.signal.aborted) return;
  rulesFeature.invalidatePreview();
  const shellFocus = shellController?.captureFocus() ?? null;
  teamsFeature.drafts.capture(app);
  playersFeature.drafts.capture(app);
  modal.capture();
  if (options.preserveForm === true && renderedPage === "lineups") {
    lineupsFeature.actions.capturePairedLineupFromDom();
  }
  if (options.preserveForm === true && renderedPage === "review") {
    lineupsFeature.actions.captureLineupFormFromDom();
  }
  if (!session.bootstrap) {
    app.innerHTML = `<div class="boot-screen"><div class="spinner"></div><strong>正在启动平台</strong></div>`;
    renderedPage = null;
    return;
  }
  if (renderedPage) {
    const previousPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
    workspaceState.capture(renderedPage, previousPageRoot, options.preserveForm === true);
  }

  let content = "";
  switch (navigation.page) {
    case "dashboard":
      content = dashboardPage(session.bootstrap);
      break;
    case "database":
      content = databasePage(session.bootstrap, workspaceState.module("database").active_section);
      break;
    case "rules": {
      const workspace = workspaceState.module("rules");
      content = rulesPage(session.bootstrap, rulesFeature.packages.pending, workspace.active_section ?? "catalog", rulesFeature.packages.reading);
      break;
    }
    case "players": {
      const workspace = workspaceState.module("players");
      content = playersPage(
        session.bootstrap, playersFeature.catalog.references, playersFeature.catalog.list, playersFeature.catalog.detail, playersFeature.catalog.query, playersFeature.imports.preview, playersFeature.catalog.selectedIds,
        workspace.tabs, workspace.active_tab_id, workspace.layout_mode, workspace.module_sidebar_collapsed, workspace.inspector_collapsed,
        workspace.active_section ?? "directory", playersFeature.catalog.pageNumber, playersFeature.catalog.source,
      );
      break;
    }
    case "teams": {
      const workspace = workspaceState.module("teams");
      content = teamsPage(
        session.bootstrap, teamsFeature.catalog.list, teamsFeature.catalog.detail, playersFeature.catalog.detail, teamsFeature.catalog.query, teamsFeature.catalog.selectedIds, coachList, formationCatalog,
        teamsFeature.imports.package, teamsFeature.catalog.lineups, teamsFeature.catalog.presets, workspace.tabs, workspace.active_tab_id,
        workspace.layout_mode, workspace.module_sidebar_collapsed, workspace.inspector_collapsed,
        workspace.active_section ?? "directory", teamsFeature.catalog.pageNumber,
      );
      break;
    }
    case "lineup_presets": {
      content = lineupPresetsPage(
        session.bootstrap, teamsFeature.catalog.list, teamsFeature.catalog.detail, teamsFeature.catalog.presets, teamsFeature.catalog.query, teamsFeature.catalog.pageNumber,
      );
      break;
    }
    case "workbooks": {
      const workspace = workspaceState.module("workbooks");
      content = workbooksPage(session.bootstrap, teamsFeature.imports.monthly, playersFeature.imports.preview, lineupsFeature.imports.preview, workspace.module_sidebar_collapsed, workspace.inspector_collapsed, workspace.active_section ?? "team");
      break;
    }
    case "lineups": {
      const workspace = workspaceState.module("lineups");
      content = lineupsPage(
        session.bootstrap,
        playersFeature.catalog.references,
        lineupsFeature.state.lineupRecords,
        lineupsFeature.imports.preview,
        lineupsFeature.state.pairedLineupBuilder,
        lineupsFeature.state.pairedLineupPresets,
        coachList,
        lineupsFeature.state.selectedMatchLineupChain,
        lineupsFeature.state.selectedManagedMatchId,
        workspace.module_sidebar_collapsed,
        workspace.inspector_collapsed,
        workspace.active_section ?? "matches",
      );
      break;
    }
    case "prediction": {
      const workspace = workspaceState.module("prediction");
      content = predictionPage(
        session.bootstrap,
        playersFeature.catalog.references,
        selectedP4MatchId,
        p4MatchWorkspace,
        p4TaskWorkspace,
        lineupsFeature.state.selectedMatchLineupChain,
        selectedPredictionReadiness,
        selectedPredictionSnapshot,
        selectedPredictionModelFamily,
        workspace.module_sidebar_collapsed,
        workspace.inspector_collapsed,
        workspace.active_section ?? "formal",
      );
      break;
    }
    case "review": {
      const workspace = workspaceState.module("review");
      const storedStep = Number((workspace.active_section ?? "").replace("step-", ""));
      content = reviewPage(
        session.bootstrap,
        reviewableMatches,
        selectedReviewMatchId,
        reviewLineups,
        selectedMatchReview,
        matchReviewPackagePreview,
        matchReviewPackageWorkflow,
        selectedReviewSettlement,
        recentMatchReviews,
        playersFeature.catalog.references,
        lineupsFeature.state.lineupPlayerCandidates,
        lineupsFeature.state.lineupBuilderPlayers,
        lineupsFeature.state.lineupBuilderForm,
        Number.isInteger(storedStep) && storedStep >= 1 && storedStep <= 9 ? storedStep : null,
      );
      break;
    }
    case "analytics":
      content = analyticsPage(
        session.bootstrap,
        analyticsOverview,
        analysisJobs,
        aiAnalysisResponsePreview,
        aiAnalysisSuggestions,
        analysisAbilityCandidates,
        session.bootstrap.data.competitions,
        parameterTuningCandidates,
        postmatchOverview,
        lastAiAnalysisPackageId,
      );
      break;
    case "runs":
      content = runsPage(session.bootstrap);
      break;
    case "logs":
      content = logsPage(issueLogs);
      break;
    case "openai":
      content = openAiPage(
        openAiProfiles,
        selectedOpenAiProfileId,
        creatingOpenAiProfile,
      );
      break;
    case "api_workspace":
      content = apiWorkspacePage(
        session.bootstrap,
        apiWorkspacePresets,
        apiWorkspaceSessions,
        apiWorkspaceDetail,
        openAiProfiles,
        apiWorkspaceMatches,
        selectedApiWorkspacePresetKey,
        selectedApiWorkspaceProfileId,
        selectedApiWorkspaceMatchId,
        apiWorkspaceDraftMessage,
        apiWorkspaceSending,
        apiWorkspacePendingMessage,
        selectedApiWorkspaceContextEntityType,
        selectedApiWorkspaceContextEntityLabel,
        apiWorkspaceSessionSearch,
        apiWorkspaceIncludeContext,
        apiWorkspaceActiveRequestId,
        workspaceState.module("api_workspace").module_sidebar_collapsed,
        workspaceState.module("api_workspace").inspector_collapsed,
        workspaceState.module("api_workspace").active_section ?? "chat",
      );
      break;
    case "release": {
      const workspace = workspaceState.module("release");
      content = releasePage(
        session.bootstrap,
        releaseAcceptanceRuns,
        selectedReleaseAcceptanceRun,
        workspace.module_sidebar_collapsed,
        workspace.inspector_collapsed,
        workspace.active_section ?? "overview",
      );
      break;
    }
    case "architecture":
      content = architecturePage(workspaceState.module("architecture").active_section);
      break;
  }
  content = renderWorkflowContinuationBanner() + content;
  app.innerHTML = renderShell({
    state: session.bootstrap,
    page: navigation.page,
    theme: session.theme,
    content,
    navigationPending: navigation.pendingPage === navigation.page,
    navigationFailed: shellController?.navigationFailed ?? false,
    sidebarCollapsed: workspaceState.sidebarCollapsed(),
  });
  modal.restore();
  warningCenter?.observeConnection(session.bootstrap.connection_error);
  renderedPage = navigation.page;
  const currentPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
  workspaceState.restore(navigation.page, currentPageRoot);
  rulesFeature.syncHierarchy(currentPageRoot);
  if (navigation.page === "rules") fitCatalogueRegions(currentPageRoot);
  teamsFeature.drafts.restore(currentPageRoot, session.bootstrap);
  playersFeature.drafts.restore(currentPageRoot, session.bootstrap);
  shellController?.restoreFocus(shellFocus);
  if (navigation.page === "lineups") {
  queueMicrotask(() => {
    const matchCompetition = currentPageRoot.querySelector<HTMLSelectElement>("#new-match-competition");
    if (matchCompetition) {
      lineupsFeature.actions.updateCompetitionHierarchy("init");
      lineupsFeature.actions.autoSelectMatchSeason();
      lineupsFeature.actions.filterMatchTeamOptions();
      lineupsFeature.actions.updateFormationHierarchy("home", "init");
      lineupsFeature.actions.updateFormationHierarchy("away", "init");
    }
    enhanceSearchableSelects(currentPageRoot);
    refreshSearchableSelects(currentPageRoot);
  });
}
}

function appendWorkspaceTab(targetPage: "teams" | "players", tab: WorkspaceTabState): void {
  const current = workspaceState.module(targetPage);
  const existing = current.tabs.filter((item) => item.id !== tab.id);
  const tabs = [...existing, tab].slice(-6);
  workspaceState.patchModule(targetPage, { tabs, active_tab_id: tab.id });
}

function persistWorkspaceSelection(targetPage: "teams" | "players"): void {
  const ids = targetPage === "teams" ? [...teamsFeature.catalog.selectedIds] : [...playersFeature.catalog.selectedIds];
  workspaceState.patchModule(targetPage, { selected_object_ids: ids });
}

function removeWorkspaceObjects(
  targetPage: "teams" | "players",
  objectIds: Iterable<string>,
): string | null {
  const removed = new Set(objectIds);
  if (removed.size === 0) return workspaceState.module(targetPage).active_tab_id;
  const current = workspaceState.module(targetPage);
  const tabs = current.tabs.filter((item) => !removed.has(item.id));
  const activeId = current.active_tab_id && !removed.has(current.active_tab_id)
    ? current.active_tab_id
    : (tabs.at(-1)?.id ?? null);
  const selectedIds = current.selected_object_ids.filter((id) => !removed.has(id));
  workspaceState.patchModule(targetPage, {
    tabs,
    active_tab_id: activeId,
    selected_object_ids: selectedIds,
  });
  if (targetPage === "teams") {
    teamsFeature.catalog.remove(removed);
  } else {
    for (const id of removed) playersFeature.catalog.toggleSelection(id, false);
    if (playersFeature.catalog.detail && removed.has(playersFeature.catalog.detail.player.id)) playersFeature.catalog.adopt(null);
  }
  return activeId;
}

function isMissingWorkspaceObjectError(
  targetPage: "teams" | "players",
  error: unknown,
): boolean {
  const message = userFacingError(error);
  return targetPage === "teams"
    ? message.includes("球队不存在")
    : message.includes("球员不存在");
}

async function openAvailableWorkspaceTab(
  targetPage: "teams" | "players",
  preferredId: string | null,
): Promise<void> {
  const current = workspaceState.module(targetPage);
  const candidates = [
    preferredId,
    ...current.tabs.map((item) => item.id).reverse(),
  ].filter((id, index, values): id is string => Boolean(id) && values.indexOf(id) === index);
  for (const candidate of candidates) {
    try {
      if (targetPage === "teams") await teamsFeature.open(candidate);
      else await playersFeature.openDirectory(candidate);
      return;
    } catch (error) {
      if (!isMissingWorkspaceObjectError(targetPage, error)) throw error;
      removeWorkspaceObjects(targetPage, [candidate]);
    }
  }
  if (targetPage === "teams") {
    teamsFeature.catalog.clearDetail();
  } else {
    playersFeature.catalog.adopt(null);
  }
  render({ preserveForm: true });
}

async function activateWorkspaceTab(targetPage: "teams" | "players", objectId: string): Promise<void> {
  workspaceState.patchModule(targetPage, { active_tab_id: objectId });
  if (targetPage === "teams") await teamsFeature.open(objectId);
  else await playersFeature.openDirectory(objectId);
}

async function closeWorkspaceTab(targetPage: "teams" | "players", objectId: string): Promise<void> {
  const current = workspaceState.module(targetPage);
  const wasActive = current.active_tab_id === objectId;
  const activeId = removeWorkspaceObjects(targetPage, [objectId]);
  if (!wasActive) {
    render({ preserveForm: true });
    return;
  }
  await openAvailableWorkspaceTab(targetPage, activeId);
}

async function openSelectedWorkspaceObjects(targetPage: "teams" | "players"): Promise<void> {
  const ids = targetPage === "teams" ? [...teamsFeature.catalog.selectedIds] : [...playersFeature.catalog.selectedIds];
  const items = targetPage === "teams" ? (teamsFeature.catalog.list?.items ?? []) : (playersFeature.catalog.list?.items ?? []);
  for (const id of ids.slice(-6)) {
    const item = items.find((candidate) => candidate.id === id);
    appendWorkspaceTab(targetPage, { id, label: item?.canonical_name ?? id });
  }
  const active = ids.at(-1);
  if (!active) return;
  if (targetPage === "teams") await teamsFeature.open(active);
  else await playersFeature.openDirectory(active);
}

function setWorkspaceMode(targetPage: "teams" | "players", mode: WorkspaceLayoutMode): void {
  workspaceState.patchModule(targetPage, { layout_mode: mode });
  render({ preserveForm: true });
}

function toggleWorkspacePane(pane: "module-sidebar" | "inspector"): void {
  const current = workspaceState.module(navigation.page);
  const patch = pane === "module-sidebar"
    ? { module_sidebar_collapsed: !current.module_sidebar_collapsed }
    : { inspector_collapsed: !current.inspector_collapsed };
  workspaceState.patchModule(navigation.page, patch);
  render({ preserveForm: true });
}

function resetCurrentWorkspace(): void {
  if (navigation.pendingPage) return;
  const page = navigation.page;
  shellController?.confirmWorkspaceReset(pageTitle(page), () => {
    if (navigation.page !== page || browserLifecycleController.signal.aborted) return;
    resetWorkspaceContents();
  });
}

function resetWorkspaceContents(): void {
  modal.reset();
  workspaceState.clear(navigation.page);
  if (navigation.page === "teams") { teamsFeature.reset(); playersFeature.catalog.adopt(null); }
  if (navigation.page === "players") playersFeature.reset();
  if (navigation.page === "lineup_presets") { teamsFeature.reset(); }
  // Do not capture the discarded DOM back into the just-cleared workspace.
  renderedPage = null;
  render();
  toast("当前工作区已重置", "success");
}

function toast(
  message: string,
  kind: "normal" | "success" | "error" = "normal",
): void {
  warningCenter?.notify(message, kind === "error" ? "danger" : kind === "success" ? "success" : "info", "客户端");
}

function userFacingError(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  if (
    /column .* does not exist|relation .* does not exist|数据库迁移失败/i.test(
      message,
    )
  ) {
    return "数据库结构版本与客户端不一致。请重启客户端完成自动升级后重试，现有数据不会被删除。";
  }
  if (message.startsWith("PostgreSQL 连接或查询失败")) {
    return "数据库操作失败，请检查数据服务连接状态后重试。";
  }
  return message;
}

function technicalErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function isUserGuidanceMessage(message: string): boolean {
  return /^(请选择|请先|请填写|请输入|请至少|当前没有可|没有可用于|尚未选择)/.test(
    message.trim(),
  );
}

function recordClientIssue(
  error: unknown,
  operation: string,
  severity: IssueSeverity = "error",
): void {
  if (severity === "critical") warningCenter?.record({ key: `critical:${operation}`, source: "客户端", severity, title: operation, message: userFacingError(error), target: "logs" });
  if (issueWasLogged(error)) return;
  const technicalMessage = technicalErrorMessage(error);
  if (severity !== "critical" && isUserGuidanceMessage(technicalMessage))
    return;
  void api
    .recordIssue({
      severity,
      source: severity === "critical" ? "global" : "frontend",
      operation,
      user_message: userFacingError(error),
      technical_message: technicalMessage,
    })
    .catch(() => {
      // 问题日志不可用时保留原始界面错误，避免二次异常覆盖根因。
    });
}

async function runBusy<T>(operation: () => Promise<T>): Promise<T> {
  if (!warningCenter) throw new Error("应用告警模块尚未启动");
  return warningCenter.runTask(operation);
}

function value(id: string): string {
  const input = document.querySelector<
    HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement
  >(`#${id}`);
  if (!input) throw new Error(`缺少输入控件：${id}`);
  return input.value;
}

function nullableValue(id: string): string | null {
  const result = value(id).trim();
  return result.length > 0 ? result : null;
}

function closeModal(): void {
  modal.close();
}

function showHtmlModal(
  title: string,
  subtitle: string,
  body: string,
  footer = "",
  modalClass = "",
): void {
  modal.showHtml(title, subtitle, body, footer, modalClass);
}

function showModal(title: string, payload: unknown, footer = ""): void {
  modal.show(title, payload, footer);
}

function showConfirmation(
  title: string,
  description: string,
  facts: Array<[string, string]>,
  confirmLabel: string,
  action: () => Promise<void>,
  options: ModalConfirmationOptions = {},
): void {
  modal.confirm(title, description, facts, confirmLabel, action, options);
}

function routeSourceLabel(source: RouteDecision["source"]): string {
  const labels: Record<RouteDecision["source"], string> = {
    explicit_rule_package: "本场规则包覆盖",
    stage_binding: "赛事阶段绑定",
    season_binding: "赛季绑定",
    competition_binding: "具体赛事绑定",
    competition_kind_default: "赛事类型默认",
  };
  return labels[source];
}

function modelDisplayName(modelId: string): string {
  const labels: Record<string, string> = {
    p4: "P4 通用低比分概率模型",
    p4_league: "P4 联赛低比分概率模型",
    p4_group_stage: "P4 小组赛低比分概率模型",
    p4_knockout_90: "P4 单回合淘汰赛模型",
    p4_knockout_two_leg_90: "P4 两回合淘汰赛模型",
    p4_friendly: "P4 友谊赛低比分概率模型",
    p7: "P7 通用函数曲线模型",
    p7_league: "P7 联赛 90 分钟模型",
    p7_group_stage: "P7 小组赛 90 分钟模型",
    p7_knockout_90: "P7 单回合淘汰赛 90 分钟模型",
    p7_knockout_two_leg_90: "P7 两回合淘汰赛 90 分钟模型",
    p7_friendly: "P7 友谊赛 90 分钟模型",
  };
  return labels[modelId] ?? modelId;
}

interface ScorelineView {
  score: string;
  probability: number;
  cumulativeProbability: number | null;
  rank: number;
}

interface PredictionViewData {
  homeTeam: string;
  awayTeam: string;
  homeWin: number;
  draw: number;
  awayWin: number;
  btts: number | null;
  over25: number | null;
  homeLambda: number | null;
  awayLambda: number | null;
  homeCleanSheet: number | null;
  awayCleanSheet: number | null;
  scorelines: ScorelineView[];
  modelId: string;
  modelVersion: string;
  parameterVersion: string;
  rulePackage: string;
  rulePackageVersion: string;
  routeSource: string;
  snapshotType: string;
  runId: string;
  durationMs: number | null;
  createdAt: string | null;
  audit: Record<string, unknown>;
  inputAudit: Record<string, unknown>;
}

function objectValue(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function stringValueFrom(
  record: Record<string, unknown>,
  key: string,
  fallback = "",
): string {
  const candidate = record[key];
  return typeof candidate === "string" && candidate.trim().length > 0
    ? candidate
    : fallback;
}

function numberValueFrom(
  record: Record<string, unknown>,
  key: string,
): number | null {
  const candidate = record[key];
  return typeof candidate === "number" && Number.isFinite(candidate)
    ? candidate
    : null;
}

function predictionViewData(payload: unknown): PredictionViewData {
  const root = objectValue(payload);
  const rootOutput = objectValue(root.output);
  const wrappedOutput =
    Object.keys(objectValue(rootOutput.identity)).length > 0 ? rootOutput : {};
  const directOutput =
    Object.keys(objectValue(root.identity)).length > 0 &&
    Object.keys(objectValue(root.payload)).length > 0
      ? root
      : {};
  const modelPayload =
    Object.keys(wrappedOutput).length > 0
      ? objectValue(wrappedOutput.payload)
      : Object.keys(directOutput).length > 0
        ? objectValue(directOutput.payload)
        : rootOutput;
  const summary =
    Object.keys(wrappedOutput).length > 0
      ? objectValue(wrappedOutput.summary)
      : objectValue(root.summary);
  const identity =
    Object.keys(wrappedOutput).length > 0
      ? objectValue(wrappedOutput.identity)
      : Object.keys(directOutput).length > 0
        ? objectValue(directOutput.identity)
        : root;
  const input = objectValue(root.input);
  const inputHome = objectValue(input.team_a);
  const inputAway = objectValue(input.team_b);
  const route = objectValue(root.route);
  const rawScorelines = Array.isArray(modelPayload.scorelines)
    ? modelPayload.scorelines
    : [];
  const scorelines = rawScorelines
    .map((item, index): ScorelineView | null => {
      const row = objectValue(item);
      const probability = numberValueFrom(row, "probability");
      if (probability === null) return null;
      const goalsA = numberValueFrom(row, "goals_a");
      const goalsB = numberValueFrom(row, "goals_b");
      const score = stringValueFrom(
        row,
        "score",
        goalsA !== null && goalsB !== null ? `${goalsA}-${goalsB}` : "—",
      );
      const rank = numberValueFrom(row, "rank") ?? index + 1;
      return {
        score,
        probability,
        cumulativeProbability: numberValueFrom(row, "cumulative_probability"),
        rank,
      };
    })
    .filter((item): item is ScorelineView => item !== null)
    .sort(
      (left, right) =>
        left.rank - right.rank || right.probability - left.probability,
    );
  const routeSource = stringValueFrom(route, "source");
  const rulePackage = stringValueFrom(
    route,
    "package_display_name",
    stringValueFrom(root, "rule_package_name", "系统默认规则"),
  );
  const audit =
    Object.keys(wrappedOutput).length > 0
      ? objectValue(wrappedOutput.explanation)
      : objectValue(root.explanation);
  const rawInputAudit = objectValue(root.input_audit);
  const inputAudit: Record<string, unknown> = {
    audit_version: stringValueFrom(rawInputAudit, "audit_version", "未记录"),
    readiness_level: stringValueFrom(rawInputAudit, "readiness_level", "not_assessed"),
    readiness_score: numberValueFrom(rawInputAudit, "readiness_score"),
    input_manifest_sha256: stringValueFrom(
      rawInputAudit,
      "input_manifest_sha256",
      stringValueFrom(rawInputAudit, "manifest_sha256", ""),
    ),
    input_sha256: stringValueFrom(
      rawInputAudit,
      "input_sha256",
      stringValueFrom(root, "input_sha256", ""),
    ),
  };

  return {
    homeTeam: stringValueFrom(
      modelPayload,
      "team_a",
      stringValueFrom(inputHome, "name", "主队"),
    ),
    awayTeam: stringValueFrom(
      modelPayload,
      "team_b",
      stringValueFrom(inputAway, "name", "客队"),
    ),
    homeWin:
      numberValueFrom(summary, "home_win") ??
      numberValueFrom(modelPayload, "a_win") ??
      0,
    draw:
      numberValueFrom(summary, "draw") ??
      numberValueFrom(modelPayload, "draw") ??
      0,
    awayWin:
      numberValueFrom(summary, "away_win") ??
      numberValueFrom(modelPayload, "b_win") ??
      0,
    btts:
      numberValueFrom(summary, "btts") ?? numberValueFrom(modelPayload, "btts"),
    over25:
      numberValueFrom(summary, "over_2_5") ??
      numberValueFrom(modelPayload, "over_2_5"),
    homeLambda: numberValueFrom(modelPayload, "lambda_a"),
    awayLambda: numberValueFrom(modelPayload, "lambda_b"),
    homeCleanSheet: numberValueFrom(modelPayload, "clean_sheet_a"),
    awayCleanSheet: numberValueFrom(modelPayload, "clean_sheet_b"),
    scorelines,
    modelId: stringValueFrom(
      identity,
      "model_id",
      stringValueFrom(root, "model_key", "模型"),
    ),
    modelVersion: stringValueFrom(
      identity,
      "model_version",
      stringValueFrom(root, "model_version", "—"),
    ),
    parameterVersion: stringValueFrom(
      identity,
      "parameter_version",
      stringValueFrom(root, "parameter_version", "—"),
    ),
    rulePackage,
    rulePackageVersion: stringValueFrom(
      route,
      "package_version",
      stringValueFrom(root, "rule_package_version", "—"),
    ),
    routeSource:
      routeSource &&
      [
        "explicit_rule_package",
        "stage_binding",
        "season_binding",
        "competition_binding",
        "competition_kind_default",
      ].includes(routeSource)
        ? routeSourceLabel(routeSource as RouteDecision["source"])
        : "系统自动匹配",
    snapshotType: stringValueFrom(root, "snapshot_type", "—"),
    runId: stringValueFrom(root, "execution_mode") === "shadow"
      ? "影子推演（未写入数据库）"
      : stringValueFrom(
          root,
          "run_id",
          stringValueFrom(root, "id", "未写入数据库"),
        ),
    durationMs: numberValueFrom(root, "duration_ms"),
    createdAt: stringValueFrom(root, "created_at") || null,
    audit,
    inputAudit,
  };
}

function probabilityMeter(
  label: string,
  value: number,
  emphasized = false,
): string {
  const percent = Math.max(0, Math.min(100, value * 100));
  return `<div class="probability-meter ${emphasized ? "emphasized" : ""}"><div><span>${escapeHtml(label)}</span><strong>${formatPercent(value)}</strong></div><i><b style="width:${percent.toFixed(2)}%"></b></i></div>`;
}

function predictionTechnicalDetails(data: PredictionViewData): string {
  const readinessLevel = stringValueFrom(data.inputAudit, "readiness_level", "not_assessed");
  const readinessScore = numberValueFrom(data.inputAudit, "readiness_score");
  const manifestSha = stringValueFrom(data.inputAudit, "input_manifest_sha256", "未记录");
  const inputSha = stringValueFrom(data.inputAudit, "input_sha256", "未记录");
  const readinessLabel = readinessLevel === "formal_ready"
    ? "正式就绪"
    : readinessLevel === "ready_with_warnings"
      ? "带警告"
      : readinessLevel === "shadow_only"
        ? "仅影子"
        : readinessLevel === "blocked"
          ? "已阻断"
          : "未评估";
  return `<details class="technical-details prediction-technical"><summary>技术追踪与输入审计</summary><dl class="detail-facts compact">
    <div><dt>运行编号</dt><dd>${escapeHtml(data.runId)}</dd></div>
    <div><dt>模型版本</dt><dd>${escapeHtml(data.modelVersion)}</dd></div>
    <div><dt>参数版本</dt><dd>${escapeHtml(data.parameterVersion)}</dd></div>
    <div><dt>规则包版本</dt><dd>${escapeHtml(data.rulePackageVersion)}</dd></div>
    <div><dt>输入完整度</dt><dd>${escapeHtml(readinessLabel)}${readinessScore === null ? "" : ` · ${readinessScore}/100`}</dd></div>
    <div><dt>输入清单指纹</dt><dd title="${escapeHtml(manifestSha)}">${escapeHtml(manifestSha)}</dd></div>
    <div><dt>完整输入 SHA256</dt><dd title="${escapeHtml(inputSha)}">${escapeHtml(inputSha)}</dd></div>
    <div><dt>审计版本</dt><dd>${escapeHtml(stringValueFrom(data.inputAudit, "audit_version", "未记录"))}</dd></div>
  </dl></details>`;
}

function predictionDetailBody(payload: unknown): string {
  const data = predictionViewData(payload);
  const topScore = data.scorelines[0] ?? null;
  const scoreParts = topScore?.score.split("-") ?? ["—", "—"];
  const outcomeMax = Math.max(data.homeWin, data.draw, data.awayWin);
  const auditStatus = stringValueFrom(
    data.audit,
    "probability_sum_status",
    "PASS",
  );
  const coverageStatus = stringValueFrom(data.audit, "coverage_status", "PASS");
  return `<div class="prediction-detail">
    <section class="prediction-detail-hero">
      <div class="prediction-detail-team"><span>主队</span><strong>${escapeHtml(data.homeTeam)}</strong><small>${data.homeLambda === null ? "预计进球暂无" : `预计进球 ${data.homeLambda.toFixed(2)}`}</small></div>
      <div class="prediction-detail-score"><span>最可能比分</span><div><b>${escapeHtml(scoreParts[0] ?? "—")}</b><i>:</i><b>${escapeHtml(scoreParts[1] ?? "—")}</b></div><small>${topScore ? `单一比分概率 ${formatPercent(topScore.probability)}，并非确定赛果` : "暂无比分矩阵"}</small></div>
      <div class="prediction-detail-team away"><span>客队</span><strong>${escapeHtml(data.awayTeam)}</strong><small>${data.awayLambda === null ? "预计进球暂无" : `预计进球 ${data.awayLambda.toFixed(2)}`}</small></div>
    </section>
    <section class="prediction-detail-section"><div class="section-heading"><div><span>90 分钟结果</span><h3>胜平负概率</h3></div><small>概率越高，条形越长</small></div><div class="outcome-meter-grid">
      ${probabilityMeter("主胜", data.homeWin, data.homeWin === outcomeMax)}
      ${probabilityMeter("平局", data.draw, data.draw === outcomeMax)}
      ${probabilityMeter("客胜", data.awayWin, data.awayWin === outcomeMax)}
    </div></section>
    <section class="prediction-detail-section"><div class="section-heading"><div><span>常用判断</span><h3>进球相关概率</h3></div></div><div class="prediction-market-grid">
      <div><span>双方都进球</span><strong>${data.btts === null ? "—" : formatPercent(data.btts)}</strong></div>
      <div><span>总进球大于 2.5</span><strong>${data.over25 === null ? "—" : formatPercent(data.over25)}</strong></div>
      <div><span>${escapeHtml(data.homeTeam)} 零封</span><strong>${data.homeCleanSheet === null ? "—" : formatPercent(data.homeCleanSheet)}</strong></div>
      <div><span>${escapeHtml(data.awayTeam)} 零封</span><strong>${data.awayCleanSheet === null ? "—" : formatPercent(data.awayCleanSheet)}</strong></div>
    </div></section>
    <section class="prediction-detail-section scoreline-exhaustive-section"><div class="section-heading"><div><span>比分分布</span><h3>概率不低于 0.1% 的比分</h3></div><small>显示 ${data.scorelines.filter((item) => item.probability >= 0.001).length} 项；更低概率结果归入矩阵尾部</small></div>
      ${
        data.scorelines.length === 0
          ? `<div class="empty-state compact"><strong>暂无比分数据</strong></div>`
          : `<div class="scoreline-exhaustive-grid threshold-scroll">${data.scorelines
              .filter((item) => item.probability >= 0.001)
              .map(
                (item) =>
                  `<div class="scoreline-exhaustive-row"><span>${item.rank}</span><strong>${escapeHtml(item.score)}</strong><i><b style="width:${Math.min(100, item.probability / Math.max(data.scorelines[0]?.probability ?? item.probability, 0.000001) * 100).toFixed(2)}%"></b></i><em>${formatPercent(item.probability)}</em><small>${item.cumulativeProbability === null ? "" : `累计 ${formatPercent(item.cumulativeProbability)}`}</small></div>`,
              )
              .join("")}</div>`
      }
      <div class="scoreline-threshold-note"><span>默认阈值：0.1%</span><span>完整矩阵、模型血缘与技术信息在详细设置中查看</span></div>
    </section>
    <section class="prediction-detail-section model-basis-section"><div class="section-heading"><div><span>模型依据</span><h3>本次推演使用的链路</h3></div></div><div class="model-basis-flow">
      <div><span>匹配方式</span><strong>${escapeHtml(data.routeSource)}</strong></div><i>→</i>
      <div><span>规则</span><strong>${escapeHtml(data.rulePackage)}</strong></div><i>→</i>
      <div><span>模型</span><strong>${escapeHtml(modelDisplayName(data.modelId))}</strong></div><i>→</i>
      <div><span>数据时点</span><strong>${escapeHtml(data.snapshotType)}</strong></div>
    </div></section>
    <section class="prediction-quality-strip"><div><span>概率合计检查</span><strong>${auditStatus === "PASS" ? "通过" : "需要检查"}</strong></div><div><span>比分矩阵覆盖</span><strong>${coverageStatus === "PASS" ? "通过" : "需要检查"}</strong></div><div><span>计算耗时</span><strong>${data.durationMs === null ? "—" : `${data.durationMs} 毫秒`}</strong></div><div><span>完成时间</span><strong>${data.createdAt ? new Date(data.createdAt).toLocaleString() : "本次运行"}</strong></div></section>
    ${predictionTechnicalDetails(data)}
  </div>`;
}

function showPredictionDetail(title: string, payload: unknown): void {
  showHtmlModal(
    title,
    "比赛预测结果",
    predictionDetailBody(payload),
    "",
    "prediction-result-modal",
  );
}

function showRouteDetail(route: RouteDecision): void {
  showHtmlModal("赛事规则与模型判定", "清晰模型链路", routeDetailBody(route, { source: routeSourceLabel(route.source), model: modelDisplayName(route.model_id) }));
}

function showRouteDecision(route: RouteDecision): void {
  const target = document.querySelector<HTMLElement>("#route-preview");
  if (!target) {
    showRouteDetail(route);
    return;
  }
  target.innerHTML = `<div class="panel-heading"><div><span>模型已确定</span><h2>模型选择说明</h2></div></div>${routeCard(route, { source: routeSourceLabel(route.source), model: modelDisplayName(route.model_id) })}`;
}

type PredictionResultMode = "formal" | "shadow" | "temporary";

function showPredictionResult(
  execution: PredictionExecution | Record<string, unknown>,
  mode: PredictionResultMode,
): void {
  const presentedExecution = {
    ...(execution as Record<string, unknown>),
    execution_mode: mode,
  };
  lastPredictionResult = presentedExecution;
  const target = document.querySelector<HTMLElement>("#prediction-result");
  const modeLabel = mode === "formal" ? "正式推演" : mode === "shadow" ? "影子推演" : "临时演练";
  if (!target) {
    showPredictionDetail(`${modeLabel}结果`, presentedExecution);
    return;
  }
  const data = predictionViewData(execution);
  const topScore = data.scorelines[0] ?? null;
  const scoreParts = topScore?.score.split("-") ?? ["—", "—"];
  const outcomeMax = Math.max(data.homeWin, data.draw, data.awayWin);
  target.innerHTML = `
    <div class="panel-heading prediction-result-heading"><div><span>${modeLabel}完成</span><h2>${escapeHtml(data.homeTeam)} vs ${escapeHtml(data.awayTeam)}</h2>${mode === "shadow" ? '<small>本次结果仅供影子评估，不写入正式推演历史。</small>' : ""}</div><button class="secondary" data-action="show-last-result">查看完整结果</button></div>
    <div class="inline-prediction-hero">
      <div class="inline-score"><span>最可能比分</span><strong><b>${escapeHtml(scoreParts[0] ?? "—")}</b><i>:</i><b>${escapeHtml(scoreParts[1] ?? "—")}</b></strong><small>${topScore ? `${formatPercent(topScore.probability)} · 单一比分并非确定结果` : "暂无比分矩阵"}</small></div>
      <div class="inline-outcomes">
        ${probabilityMeter("主胜", data.homeWin, data.homeWin === outcomeMax)}
        ${probabilityMeter("平局", data.draw, data.draw === outcomeMax)}
        ${probabilityMeter("客胜", data.awayWin, data.awayWin === outcomeMax)}
      </div>
    </div>
    <div class="inline-market-grid"><div><span>双方都进球</span><strong>${data.btts === null ? "—" : formatPercent(data.btts)}</strong></div><div><span>大于 2.5 球</span><strong>${data.over25 === null ? "—" : formatPercent(data.over25)}</strong></div><div><span>主队预计进球</span><strong>${data.homeLambda === null ? "—" : data.homeLambda.toFixed(2)}</strong></div><div><span>客队预计进球</span><strong>${data.awayLambda === null ? "—" : data.awayLambda.toFixed(2)}</strong></div></div>
    <div class="prediction-result-footer"><div><span>使用规则</span><strong>${escapeHtml(data.rulePackage)}</strong><small>${escapeHtml(data.routeSource)} · ${escapeHtml(data.snapshotType)}</small></div><div><span>计算耗时</span><strong>${data.durationMs === null ? "—" : `${data.durationMs} 毫秒`}</strong></div></div>`;
}

function syncSimpleMatchInput(): Record<string, unknown> {
  const base = session.bootstrap?.data.default_match;
  const input =
    base && typeof base === "object"
      ? (JSON.parse(JSON.stringify(base)) as Record<string, unknown>)
      : {};
  delete input.match_id;
  delete input.snapshot;
  delete input.database_match_id;
  delete input.feature_snapshot_id;
  delete input.feature_quality_score;
  delete input.preparation_version;
  delete input.data_quality;
  const kickoffRaw = nullableValue("simple-kickoff");
  const kickoff = localDateTimeToIso(kickoffRaw);
  if (!kickoff) throw new Error("请选择开球时间");
  input.kickoff_time = kickoff;
  const teamA =
    typeof input.team_a === "object" && input.team_a !== null
      ? (input.team_a as Record<string, unknown>)
      : {};
  const teamB =
    typeof input.team_b === "object" && input.team_b !== null
      ? (input.team_b as Record<string, unknown>)
      : {};
  const homeName = nullableValue("simple-home-name");
  const awayName = nullableValue("simple-away-name");
  if (!homeName || !awayName) throw new Error("请输入主队和客队名称");
  teamA.name = homeName;
  teamB.name = awayName;
  input.team_a = teamA;
  input.team_b = teamB;
  return input;
}

function matchKickoff(): string {
  const input = syncSimpleMatchInput();
  const kickoff = input.kickoff_time;
  if (typeof kickoff !== "string" || kickoff.trim().length === 0) {
    throw new Error("请选择比赛开球时间");
  }
  return kickoff;
}

function routeSelection(): Omit<RoutePreviewCommand, "kickoff_time"> {
  return {
    competition_id: nullableValue("competition-id"),
    season_id: nullableValue("season-id"),
    stage_id: nullableValue("stage-id"),
    competition_kind: value("competition-kind") as CompetitionKind,
    explicit_rule_package_id: null,
    model_family: value("simulation-model-family") as PredictionModelFamily,
  };
}

async function previewRoute(): Promise<void> {
  const command: RoutePreviewCommand = {
    kickoff_time: matchKickoff(),
    ...routeSelection(),
  };
  const routeRoot = document.querySelector("#competition-id");
  const formIdentity = () => JSON.stringify([...app.querySelectorAll<HTMLInputElement | HTMLSelectElement>("#competition-id, #season-id, #stage-id, #competition-kind, #simulation-model-family, #simple-kickoff")].map(input => [input.id, input.value]));
  const identity = formIdentity(), page = navigation.page;
  const result = await rulesFeature.resolve(command, () => routeRoot?.isConnected === true && navigation.page === page && formIdentity() === identity);
  if (result) { showRouteDetail(result); toast("模型路由预览完成；正式运行仍会重新判定", "success"); }
}

async function executePrediction(): Promise<void> {
  const command: PredictionCommand = {
    match_input: syncSimpleMatchInput(),
    snapshot_type: value("snapshot-type"),
    ...routeSelection(),
  };
  const result = await runBusy(() => api.executePrediction(command));
  if (session.bootstrap) await session.refreshRecentRuns();
  render();
  showRouteDecision(result.route);
  showPredictionResult(result, "formal");
  toast("推演完成，规则选择和结果已保存到数据库", "success");
}

async function dryRun(): Promise<void> {
  const result = await runBusy(() => api.dryRunDefaultFixture());
  showPredictionResult(result, "temporary");
  toast("模型自检通过", "success");
}

function checked(id: string): boolean {
  const input = document.querySelector<HTMLInputElement>(`#${id}`);
  if (!input) throw new Error(`缺少复选框：${id}`);
  return input.checked;
}

function nullableNumber(id: string): number | null {
  const raw = value(id).trim();
  if (!raw) return null;
  const parsed = Number(raw);
  if (!Number.isFinite(parsed)) throw new Error(`${id} 必须是有效数字`);
  return parsed;
}

function localDateTimeToIso(
  raw: string | null,
  fallbackNow = false,
): string | null {
  if (!raw) return fallbackNow ? new Date().toISOString() : null;
  const parsed = new Date(raw);
  if (Number.isNaN(parsed.getTime())) throw new Error(`时间格式无效：${raw}`);
  return parsed.toISOString();
}

function localDateTimeInputValue(raw: string): string {
  const parsed = new Date(raw);
  if (Number.isNaN(parsed.getTime())) return "";
  const local = new Date(parsed.getTime() - parsed.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 16);
}

function applyLineups(result: LineupsLoadResult): void {
  playersFeature.catalog.setReferences(result.references);
  lineupsFeature.state.lineupRecords = result.records;
  coachList = result.coaches;
  const matches = result.references.managed_matches;
  if (lineupsFeature.state.selectedManagedMatchId && !matches.some((item) => item.id === lineupsFeature.state.selectedManagedMatchId)) {
    lineupsFeature.state.selectedManagedMatchId = null;
  }
  if (lineupsFeature.state.pairedLineupBuilder.match_id && !matches.some((item) => item.id === lineupsFeature.state.pairedLineupBuilder.match_id)) {
    lineupsFeature.state.pairedLineupBuilder = {
      ...lineupsFeature.state.pairedLineupBuilder,
      match_id: "",
      home: emptyPairedLineupSide(),
      away: emptyPairedLineupSide(),
    };
  }
  lineupsFeature.state.selectedManagedMatchId ??= matches[0]?.id ?? null;
}

async function loadLineups(): Promise<void> {
  if (!session.bootstrap?.data.database_configured) return;
  applyLineups(await fetchLineups());
}

async function loadReviewLineups(matchId: string): Promise<void> {
  reviewLineups = await fetchReviewLineups(matchId);
}

function applyReviewCenter(result: ReviewCenterLoadResult): void {
  reviewableMatches = result.matches;
  recentMatchReviews = result.reviews;
  selectedReviewMatchId = result.selectedMatchId;
  reviewLineups = result.lineups;
  matchReviewPackageWorkflow = result.workflow;
  matchReviewPackagePreview = result.preview;
  selectedReviewSettlement = result.settlement;
}

async function loadReviewCenter(): Promise<void> {
  if (!session.bootstrap?.data.database_configured) return;
  applyReviewCenter(await fetchReviewCenter(selectedReviewMatchId));
}

function applyAnalysisCenter(result: AnalysisCenterLoadResult): void {
  analyticsOverview = result.overview;
  analysisJobs = result.jobs;
  aiAnalysisSuggestions = result.suggestions;
  analysisAbilityCandidates = result.abilityCandidates;
  parameterTuningCandidates = result.tuningCandidates;
  postmatchOverview = result.postmatch;
}

async function loadAnalysisCenter(): Promise<void> {
  if (!session.bootstrap?.data.database_configured) return;
  applyAnalysisCenter(await fetchAnalysisCenter());
}

async function loadIssueLogs(): Promise<void> {
  issueLogs = await api.listIssueLogs(500);
}

async function loadP4MatchWorkspace(
  matchId: string,
  preferredTaskId: string | null = null,
): Promise<void> {
  selectedP4MatchId = matchId;
  const workspace = await api.readP4MatchWorkspace(matchId);
  p4MatchWorkspace = workspace;
  const taskId =
    preferredTaskId &&
    workspace.tasks.some((task) => task.id === preferredTaskId)
      ? preferredTaskId
      : (workspace.tasks[0]?.id ?? null);
  p4TaskWorkspace = taskId ? await api.readP4TaskWorkspace(taskId) : null;
}

async function refreshP4Workspace(): Promise<void> {
  const matchId =
    selectedP4MatchId ??
    playersFeature.catalog.references?.upcoming_matches[0]?.id ??
    playersFeature.catalog.references?.managed_matches[0]?.id ??
    null;
  if (!matchId) {
    p4MatchWorkspace = null;
    p4TaskWorkspace = null;
    return;
  }
  await loadP4MatchWorkspace(matchId, p4TaskWorkspace?.task.id ?? null);
}

async function loadApiWorkspace(
  selectedSessionId: string | null = apiWorkspaceDetail?.session.id ?? null,
  isCurrent: () => boolean = () => !browserLifecycleController.signal.aborted,
): Promise<void> {
  const [profiles, presets] = await Promise.all([
    api.listOpenAiProfiles(),
    api.listApiWorkspacePresets(),
  ]);
  if (!isCurrent()) return;
  openAiProfiles = profiles;
  apiWorkspacePresets = presets;
  selectedApiWorkspaceProfileId =
    selectedApiWorkspaceProfileId ||
    profiles.active_profile_id ||
    profiles.profiles.find(
      (item) => item.has_api_key && item.api_protocol === "responses",
    )?.id ||
    "";
  if (
    !selectedApiWorkspacePresetKey ||
    !presets.some((item) => item.key === selectedApiWorkspacePresetKey)
  ) {
    selectedApiWorkspacePresetKey =
      presets.find((item) => item.key === "plain_chat")?.key ??
      presets[0]?.key ??
      "plain_chat";
  }
  if (!session.bootstrap?.data.database_configured) {
    apiWorkspaceSessions = [];
    apiWorkspaceDetail = null;
    return;
  }
  const [sessions, references] = await Promise.all([
    api.listApiWorkspaceSessions(100),
    api.playerCatalogReferenceData(),
  ]);
  if (!isCurrent()) return;
  const sessionId =
    selectedSessionId && sessions.some((item) => item.id === selectedSessionId)
      ? selectedSessionId
      : null;
  const detail = sessionId
    ? await api.readApiWorkspaceSession(sessionId)
    : null;
  if (!isCurrent()) return;
  apiWorkspaceSessions = sessions;
  apiWorkspaceMatches = references.managed_matches;
  apiWorkspaceDetail = detail;
  if (apiWorkspaceDetail) {
    selectedApiWorkspaceProfileId = apiWorkspaceDetail.session.profile_id;
    selectedApiWorkspacePresetKey = apiWorkspaceDetail.session.preset_key;
    selectedApiWorkspaceMatchId = apiWorkspaceDetail.session.match_id;
    const metadata = apiWorkspaceDetail.session.metadata;
    selectedApiWorkspaceContextEntityType =
      metadata.context_entity_type === "team" ||
      metadata.context_entity_type === "player"
        ? metadata.context_entity_type
        : null;
    selectedApiWorkspaceContextEntityId =
      typeof metadata.context_entity_id === "string"
        ? metadata.context_entity_id
        : null;
    selectedApiWorkspaceContextEntityLabel =
      typeof metadata.context_entity_label === "string"
        ? metadata.context_entity_label
        : null;
    apiWorkspaceIncludeContext = Boolean(
      apiWorkspaceDetail.session.match_id ||
      selectedApiWorkspaceContextEntityId,
    );
  } else if (
    selectedApiWorkspaceMatchId &&
    !apiWorkspaceMatches.some((item) => item.id === selectedApiWorkspaceMatchId)
  ) {
    selectedApiWorkspaceMatchId = null;
  }
}

async function sendApiWorkspaceMessage(): Promise<void> {
  if (apiWorkspaceSending) throw new Error("当前 AI 问答消息仍在处理中");
  const textarea = document.querySelector<HTMLTextAreaElement>(
    "#api-workspace-message",
  );
  const message = textarea?.value.trim() ?? "";
  const profileId = value("api-workspace-profile").trim();
  const presetKey = value("api-workspace-preset").trim();
  const matchId = nullableValue("api-workspace-match");
  if (!message) throw new Error("请输入问题");
  if (!profileId) throw new Error("请选择可用的兼容 API 配置");

  const requestStartedAt = new Date().toISOString();
  const previousDetailId = apiWorkspaceDetail?.session.id ?? null;
  const requestOriginPage = navigation.page;
  const requestId = crypto.randomUUID();
  apiWorkspaceSending = true;
  apiWorkspaceActiveRequestId = requestId;
  apiWorkspacePendingMessage = {
    content: message,
    started_at: requestStartedAt,
    session_id: previousDetailId,
  };
  apiWorkspaceDraftMessage = "";
  render();
  browserLifecycleController.setTimeout(
    () =>
      document
        .querySelector("#api-workspace-conversation")
        ?.scrollTo({ top: 999999, behavior: motionScrollBehavior("smooth") }),
    0,
  );

  try {
    const detail = await api.sendApiWorkspaceMessage({
      session_id: previousDetailId,
      profile_id: profileId,
      preset_key: presetKey,
      title: null,
      match_id: matchId,
      context_entity_type: selectedApiWorkspaceContextEntityType,
      context_entity_id: selectedApiWorkspaceContextEntityId,
      context_entity_label: selectedApiWorkspaceContextEntityLabel,
      include_context: apiWorkspaceIncludeContext,
      request_id: requestId,
      message,
      attachments: [],
    });
    const stillViewingOrigin =
      navigation.page === requestOriginPage &&
      navigation.page === "api_workspace" &&
      (apiWorkspaceDetail?.session.id ?? null) === previousDetailId;
    apiWorkspaceSessions = await api.listApiWorkspaceSessions(100);
    if (stillViewingOrigin) {
      apiWorkspaceDetail = detail;
      selectedApiWorkspaceProfileId = detail.session.profile_id;
      selectedApiWorkspacePresetKey = detail.session.preset_key;
      selectedApiWorkspaceMatchId = detail.session.match_id;
      const metadata = detail.session.metadata;
      selectedApiWorkspaceContextEntityType =
        metadata.context_entity_type === "team" ||
        metadata.context_entity_type === "player"
          ? metadata.context_entity_type
          : null;
      selectedApiWorkspaceContextEntityId =
        typeof metadata.context_entity_id === "string"
          ? metadata.context_entity_id
          : null;
      selectedApiWorkspaceContextEntityLabel =
        typeof metadata.context_entity_label === "string"
          ? metadata.context_entity_label
          : null;
      toast("AI 回答已保存到当前会话", "success");
    } else {
      toast(`AI 回答已完成并保存到会话：${detail.session.title}`, "success");
    }
  } catch (error) {
    apiWorkspaceSessions = await api
      .listApiWorkspaceSessions(100)
      .catch(() => apiWorkspaceSessions);
    const messageText = userFacingError(error);
    if (
      messageText.includes("取消") ||
      messageText.toLocaleLowerCase().includes("cancel")
    ) {
      toast("当前 AI 请求已取消", "success");
      return;
    }
    throw error;
  } finally {
    apiWorkspaceSending = false;
    apiWorkspaceActiveRequestId = null;
    apiWorkspacePendingMessage = null;
    if (navigation.page === "api_workspace" && !browserLifecycleController.signal.aborted) {
      render();
      browserLifecycleController.setTimeout(
        () =>
          document
            .querySelector("#api-workspace-conversation")
            ?.scrollTo({ top: 999999 }),
        0,
      );
    }
  }
}

async function navigateTo(nextPage: Page): Promise<void> {
  await navigation.navigate(nextPage, async request => {
    try {
      await loadNavigationPage(request);
      if (navigation.isCurrent(request)) shellController?.clearNavigationFailure();
    } catch (error) {
      if (navigation.isCurrent(request)) shellController?.showNavigationFailure();
      throw error;
    }
  }, {
    started() {
      modal.reset();
      shellController?.clearNavigationFailure();
      render({ preserveForm: true });
    },
    settled() { render({ preserveForm: true }); },
  });
}

async function loadNavigationPage(request: NavigationRequest): Promise<void> {
  const nextPage = request.page;
  // Read-only Features consume Session/current static explanation; no page-owned loading state.
  if (nextPage === "dashboard" || nextPage === "architecture") return;
  if (nextPage === "logs") {
    const logs = await api.listIssueLogs(500);
    if (navigation.isCurrent(request)) issueLogs = logs;
    return;
  }
  if (nextPage === "openai") {
    const profiles = await api.listOpenAiProfiles();
    if (navigation.isCurrent(request)) {
      openAiProfiles = profiles;
      if (!creatingOpenAiProfile) {
        selectedOpenAiProfileId =
          selectedOpenAiProfileId ??
          profiles.active_profile_id ??
          profiles.profiles[0]?.id ??
          null;
      }
    }
    return;
  }
  if (nextPage === "api_workspace") {
    await loadApiWorkspace(undefined, request.isCurrent);
    return;
  }
  if (!session.bootstrap?.data.database_configured) return;
  if (nextPage === "release") {
    const runs = await api.listReleaseAcceptanceRuns(50);
    if (!navigation.isCurrent(request)) return;
    const selectedId = selectedReleaseAcceptanceRun?.id ?? runs[0]?.id ?? null;
    const selected = selectedId ? await api.readReleaseAcceptanceRun(selectedId) : null;
    request.commit(() => { releaseAcceptanceRuns = runs; selectedReleaseAcceptanceRun = selected; });
    return;
  }

  if (nextPage === "players") {
    await playersFeature.load(playersFeature.catalog.list === null, () => navigation.isCurrent(request));
  } else if (nextPage === "teams" || nextPage === "lineup_presets") {
    await teamsFeature.loadForRoute(nextPage, () => navigation.isCurrent(request));
  } else if (nextPage === "lineups") {
    const result = await fetchLineups();
    if (navigation.isCurrent(request)) applyLineups(result);
  } else if (nextPage === "prediction") {
    const references =
      playersFeature.catalog.references ?? (await api.playerCatalogReferenceData());
    const matchId =
      selectedP4MatchId &&
      references.managed_matches.some((item) => item.id === selectedP4MatchId)
        ? selectedP4MatchId
        : (references.upcoming_matches[0]?.id ??
          references.managed_matches[0]?.id ??
          null);
    const workspace = matchId
      ? await api.readP4MatchWorkspace(matchId)
      : null;
    const taskId =
      p4TaskWorkspace &&
      workspace?.tasks.some((task) => task.id === p4TaskWorkspace?.task.id)
        ? p4TaskWorkspace.task.id
        : (workspace?.tasks[0]?.id ?? null);
    const taskWorkspace = taskId
      ? await api.readP4TaskWorkspace(taskId)
      : null;
    if (navigation.isCurrent(request)) {
      playersFeature.catalog.setReferences(references);
      selectedP4MatchId = matchId;
      p4MatchWorkspace = workspace;
      p4TaskWorkspace = taskWorkspace;
    }
  } else if (nextPage === "review") {
    const result = await fetchReviewCenter(selectedReviewMatchId);
    if (navigation.isCurrent(request)) applyReviewCenter(result);
  } else if (nextPage === "analytics") {
    const result = await fetchAnalysisCenter();
    if (navigation.isCurrent(request)) applyAnalysisCenter(result);
  }
}

async function refreshBootstrapAndCatalog(): Promise<void> {
  await session.refresh();
  await playersFeature.load(true);
  if (teamsFeature.catalog.list !== null || navigation.page === "teams" || navigation.page === "lineup_presets") await teamsFeature.load(true);
}

async function previewPlayerFromTeam(playerId: string): Promise<void> {
  const source = await teamsFeature.readPlayerFromTeam(playerId); if (!source || !source.current()) return;
  playersFeature.catalog.adopt(source.player); workspaceState.patchModule("teams", { inspector_collapsed: false }); render({ preserveForm: true });
}

async function refreshLineupPresetPage(): Promise<void> {
  await runBusy(() => teamsFeature.load(false));
  if (teamsFeature.catalog.detail) workspaceState.patchModule("lineup_presets", { active_tab_id: teamsFeature.catalog.detail.team.id });
  render({ preserveForm: true });
}

async function bulkArchiveEntities(entityType: "team" | "player"): Promise<void> { if(entityType === "team") await teamsFeature.archive(); else await playersFeature.archive(); }

async function openPlayerFromTeam(playerId: string): Promise<void> {
  const source=await teamsFeature.readPlayerFromTeam(playerId); if(!source||!source.current())return;
  const now=new Date().toISOString();playersFeature.enterSource({source:"team_roster",team_id:source.teamId,team_name:source.teamName,player_id:playerId,origin_page:"teams",return_section:null,created_at:now,updated_at:now},source.player);

  workspaceState.patchModule("players", { active_section: "directory", inspector_collapsed: false });await navigateTo("players");
}

async function openPlayerProfileFromTeam(playerId: string): Promise<void> {
  const source=await teamsFeature.readPlayerFromTeam(playerId); if(!source||!source.current())return;
  const now=new Date().toISOString();playersFeature.enterSource({source:"team_roster",team_id:source.teamId,team_name:source.teamName,player_id:playerId,origin_page:"teams",return_section:null,created_at:now,updated_at:now},source.player);
  appendWorkspaceTab("players", {id:playerId,label:source.player.player.canonical_name});
  workspaceState.patchModule("players", { active_section: "profile", inspector_collapsed: true });await navigateTo("players");
}


async function returnToSourceTeamProfile(teamId: string): Promise<void> {
  const sourceTeamId = teamId || playersFeature.catalog.source?.team_id;
  if (!sourceTeamId) { workspaceState.patchModule("players", { active_section: "directory", inspector_collapsed: false }); render(); return; }
  await teamsFeature.open(sourceTeamId, "profile", true);
}

async function openPlayerFromLineup(playerId:string,teamId:string,teamName:string,returnSection:"builder"|"chain"="chain"):Promise<void>{
 if(!playerId||!teamId)throw new Error("阵容球员或球队身份缺失，请刷新后重试");const page=navigation.page;
 if(!await playersFeature.catalog.open(playerId,()=>page===navigation.page)||!playersFeature.catalog.detail)return;
 const now=new Date().toISOString();playersFeature.enterSource({source:"match_lineup",team_id:teamId,team_name:teamName||"来源球队",player_id:playerId,origin_page:"lineups",return_section:returnSection,created_at:now,updated_at:now},playersFeature.catalog.detail);
 workspaceState.patchModule("players",{active_section:"profile",inspector_collapsed:true});await navigateTo("players");
}

async function returnToLineupWorkspace(section: "builder" | "chain"): Promise<void> {
  playersFeature.catalog.setSource(null);
  workspaceState.patchModule("lineups", { active_section: section });
  await navigateTo("lineups");
}

async function openTeamApiWorkspace(teamId: string | null): Promise<void> {
  const current = teamsFeature.catalog.capture(), originPage = navigation.page;
  const team = teamId
    ? teamsFeature.catalog.detail?.team.id === teamId
      ? teamsFeature.catalog.detail
      : await api.readTeam(teamId)
    : null;
  if (!current() || navigation.page !== originPage) return;
  if (team && teamId && team.team.id !== teamId) throw new Error("球队来源身份不匹配");
  selectedApiWorkspacePresetKey = "team_profile_completion";
  selectedApiWorkspaceMatchId = null;
  selectedApiWorkspaceContextEntityType = team ? "team" : null;
  selectedApiWorkspaceContextEntityId = team?.team.id ?? null;
  selectedApiWorkspaceContextEntityLabel = team?.team.canonical_name ?? null;
  apiWorkspaceIncludeContext = Boolean(team);
  apiWorkspaceDraftMessage = team
    ? `请基于客户端附加的只读资料，概括“${team.team.canonical_name}”当前已知信息、明显缺口和下一步应通过球队月度 Excel 核验的字段。不要联网，不要生成文件，不要提出数据库写入操作。`
    : "请说明球队资料月度维护时应优先核验哪些字段，以及如何区分缺失、过期和冲突数据。";
  workspaceState.clear("api_workspace");
  await navigateTo("api_workspace");
}

async function openPlayerApiWorkspace(playerId: string): Promise<void> {
  const current=playersFeature.catalog.capture(),page=navigation.page;
  const detail =
    playersFeature.catalog.detail?.player.id === playerId
      ? playersFeature.catalog.detail
      : await api.readPlayer(playerId);
  if(!current()||page!==navigation.page)return;
  if(detail.player.id!==playerId)throw new Error("球员上下文响应身份不匹配");
  selectedApiWorkspacePresetKey = "player_profile_completion";
  selectedApiWorkspaceMatchId = null;
  selectedApiWorkspaceContextEntityType = "player";
  selectedApiWorkspaceContextEntityId = detail.player.id;
  selectedApiWorkspaceContextEntityLabel = detail.player.canonical_name;
  apiWorkspaceIncludeContext = true;
  apiWorkspaceDraftMessage = `请基于客户端附加的只读资料，概括球员“${detail.player.canonical_name}”当前已知信息、明显缺口和下一步应通过球员月度 Excel 核验的字段。不要联网，不要生成文件，不要提出数据库写入操作。`;
  workspaceState.clear("api_workspace");
  await navigateTo("api_workspace");
}

async function createLineupPlayerQuick(): Promise<void> {
  const playerName = value("quick-lineup-player-name").trim();
  const teamId = nullableValue("quick-lineup-player-team");
  const positionCode = nullableValue("quick-lineup-player-position");
  if (!playerName) throw new Error("请输入球员姓名");
  if (!teamId) throw new Error("请选择球员所属球队");
  const today = new Date().toISOString().slice(0, 10);
  await runBusy(async () => {
    const created = await api.createPlayer({
      canonical_name: playerName,
      date_of_birth: null,
      nationality_code: nullableValue("quick-lineup-player-nationality"),
      preferred_foot: "unknown",
      height_cm: null,
      status: "active",
      metadata: { entry_mode: "lineup_quick_create" },
    });
    await api.addPlayerTeamPeriod({
      player_id: created.id,
      team_id: teamId,
      season_id: null,
      squad_number: null,
      valid_from: today,
      valid_to: null,
      registration_status: "registered",
      source_document_id: null,
    });
    if (positionCode) {
      await api.assignPlayerPosition({
        player_id: created.id,
        position_code: positionCode,
        proficiency: 0.8,
        default_role_code: null,
        is_primary: true,
        valid_from: today,
        valid_to: null,
        source_document_id: null,
      });
    }
    await refreshBootstrapAndCatalog();
    lineupsFeature.state.lineupBuilderForm = { ...lineupsFeature.state.lineupBuilderForm, team_id: teamId };
    const list = await api.listPlayers({
      search: null,
      team_id: teamId,
      position_code: null,
      availability_status: null,
      player_status: "active",
      limit: 200,
      cursor_name: null,
      cursor_id: null,
    });
    lineupsFeature.state.lineupPlayerCandidates = list.items;
  });
  render();
  toast("球员已创建并加入球队，可继续编辑阵容", "success");
}

async function calculateStoredMatch(): Promise<void> {
  const matchId = lineupsFeature.actions.selectedExchangeMatchId();
  const result = await runBusy(() =>
    api.executePredictionFromMatch({
      match_id: matchId,
      snapshot_type: value("stored-match-snapshot"),
      explicit_rule_package_id: null,
      model_family: selectedPredictionModelFamily,
    }),
  );
  lastPredictionResult = result;
  if (session.bootstrap) await session.refreshRecentRuns();
  render();
  showPredictionDetail("比赛推演结果", result);
  toast("已使用数据库中的比赛、阵容和动态标签完成推演", "success");
}

function selectedPredictionMatchId(): string {
  const matchId = nullableValue("prediction-match-id");
  if (!matchId) throw new Error("请先选择比赛");
  return matchId;
}

function storedMatchRouteCommand(matchId: string): RoutePreviewCommand {
  const match = (
    playersFeature.catalog.references?.managed_matches ?? playersFeature.catalog.references?.upcoming_matches ?? []
  ).find((item) => item.id === matchId);
  if (!match) throw new Error("所选比赛不存在，请刷新比赛列表");
  const stage = match.stage_id
    ? session.bootstrap?.data.stages.find((item) => item.id === match.stage_id)
    : null;
  const competition = match.competition_id
    ? session.bootstrap?.data.competitions.find((item) => item.id === match.competition_id)
    : null;
  return {
    kickoff_time: match.kickoff_time,
    competition_id: match.competition_id,
    season_id: match.season_id,
    stage_id: match.stage_id,
    competition_kind:
      stage?.stage_kind ?? competition?.competition_kind ?? "custom",
    explicit_rule_package_id: nullableValue("explicit-rule-package-id"),
    model_family: selectedPredictionModelFamily,
  };
}

async function previewStoredRoute(): Promise<void> {
  const matchId = selectedPredictionMatchId();
  const route = await runBusy(() =>
    api.previewRoute(storedMatchRouteCommand(matchId)),
  );
  showRouteDecision(route);
  toast("模型判定完成", "success");
}

function selectedPredictionSnapshotType(): LineupSnapshotType {
  return value("prediction-stored-snapshot") as LineupSnapshotType;
}

function selectedStoredPredictionCommand(): import("./types").StoredMatchPredictionCommand {
  return {
    match_id: selectedPredictionMatchId(),
    snapshot_type: selectedPredictionSnapshotType(),
    explicit_rule_package_id: nullableValue("explicit-rule-package-id"),
    model_family: selectedPredictionModelFamily,
  };
}

async function readSelectedPredictionLineupChain(): Promise<MatchLineupChain> {
  const matchId = selectedPredictionMatchId();
  const snapshotType = selectedPredictionSnapshotType();
  selectedP4MatchId = matchId;
  selectedPredictionSnapshot = snapshotType;
  const chain = await api.readMatchLineupChain(matchId, snapshotType);
  lineupsFeature.state.selectedMatchLineupChain = chain;
  return chain;
}

function missingLineupTeam(chain: MatchLineupChain) {
  const teams = [chain.home, chain.away];
  return (
    teams.find((team) => team.versions.length === 0) ??
    teams.find((team) => !team.selected_lineup_id) ??
    teams.find((team) => team.blocking_issues.length > 0) ??
    chain.home
  );
}

async function openMissingPredictionLineup(
  chain: MatchLineupChain,
): Promise<void> {
  const target = missingLineupTeam(chain);
  lineupsFeature.state.pairedLineupBuilder = {
    match_id: chain.match_record.id,
    lineup_type: "expected",
    snapshot_type: chain.snapshot_type as LineupSnapshotType,
    captured_at: localDateTimeInputValue(chain.data_cutoff_time),
    source_urls: "",
    home: {
      ...emptyPairedLineupSide(),
      team_id: chain.home.team_id,
      team_name: chain.home.team_name,
      coach_id: lineupsFeature.actions.preferredCoachForTeam(chain.home.team_id),
    },
    away: {
      ...emptyPairedLineupSide(),
      team_id: chain.away.team_id,
      team_name: chain.away.team_name,
      coach_id: lineupsFeature.actions.preferredCoachForTeam(chain.away.team_id),
    },
  };
  lineupsFeature.state.selectedManagedMatchId = chain.match_record.id;
  lineupsFeature.state.selectedMatchLineupChain = chain;
  workflowNavigation.set({ page: "prediction", section: workspaceState.module("prediction").active_section ?? "formal" });
  workflowContinuation = {
    reason: `补齐${chain.match_record.home_team_name}与${chain.match_record.away_team_name}双方阵容`,
    matchId: chain.match_record.id,
    snapshotType: chain.snapshot_type as LineupSnapshotType,
  };
  workspaceState.patchModule("lineups", {
    active_section: "builder",
    controls: {},
  });
  await navigateTo("lineups");
  const [homeCount, awayCount] = await runBusy(lineupsFeature.actions.loadBothPairedLineupSides);
  render({ preserveForm: true });
  toast(
    `已进入双方阵容编排：${chain.home.team_name} ${homeCount} 人，${chain.away.team_name} ${awayCount} 人`,
    "success",
  );
  if (target.blocking_issues.length > 0) {
    toast(`${target.team_name}当前阻断：${target.blocking_issues.join("；")}`, "normal");
  }
}

async function checkPredictionLineupChain(): Promise<void> {
  const command = selectedStoredPredictionCommand();
  const outcome = await runBusy(async () => {
    const readiness = await api.inspectMatchPredictionReadiness(command);
    let chain: MatchLineupChain | null = null;
    try {
      chain = await api.readMatchLineupChain(command.match_id, command.snapshot_type);
    } catch {
      chain = null;
    }
    return { readiness, chain };
  });
  selectedPredictionReadiness = outcome.readiness;
  lineupsFeature.state.selectedMatchLineupChain = outcome.chain;
  render({ preserveForm: true });
  toast(
    outcome.readiness.can_run_formal
      ? `赛前完整度 ${outcome.readiness.score}/100，已允许正式推演`
      : `${outcome.readiness.level === "shadow_only" ? "仅允许影子推演" : "正式推演被阻断"}：${outcome.readiness.score}/100`,
    outcome.readiness.can_run_formal ? "success" : "normal",
  );
}

async function preparePredictionLineups(): Promise<void> {
  const chain =
    lineupsFeature.state.selectedMatchLineupChain?.match_record.id === selectedPredictionMatchId() &&
    lineupsFeature.state.selectedMatchLineupChain.snapshot_type === selectedPredictionSnapshotType()
      ? lineupsFeature.state.selectedMatchLineupChain
      : await runBusy(readSelectedPredictionLineupChain);
  if (chain.ready_for_model) {
    render({ preserveForm: true });
    toast("双方阵容已经就绪，可以直接正式推演", "success");
    return;
  }
  await openMissingPredictionLineup(chain);
}

async function continueLineupPrediction(): Promise<void> {
  const chain = lineupsFeature.state.selectedMatchLineupChain;
  if (!chain?.ready_for_model) throw new Error("双方阵容尚未通过模型门禁");
  workflowContinuation = null;
  workflowNavigation.clear();
  selectedP4MatchId = chain.match_record.id;
  selectedPredictionSnapshot = chain.snapshot_type as LineupSnapshotType;
  workspaceState.patchModule("prediction", { active_section: "formal" });
  await navigateTo("prediction");
  render({ preserveForm: true });
  toast("双方阵容已就绪，可以开始正式推演", "success");
}

function showPredictionReadinessModal(readiness: MatchPredictionReadiness): void {
  const shadowAction = readiness.can_run_shadow && !readiness.can_run_formal
    ? '<button type="button" class="primary" data-action="run-shadow-prediction-match">运行影子推演</button>'
    : "";
  const footer = `<button type="button" class="secondary" data-action="close-workspace-detail">关闭</button>${shadowAction}`;
  showModal("赛前数据完整度门禁", readiness, footer);
}

async function runShadowPredictionMatch(): Promise<void> {
  const explicitRulePackageId = nullableValue("explicit-rule-package-id");
  const command = {
    ...selectedStoredPredictionCommand(),
    explicit_rule_package_id: explicitRulePackageId,
  };
  closeModal();
  selectedP4MatchId = command.match_id;
  selectedPredictionSnapshot = command.snapshot_type as LineupSnapshotType;
  const result = await runBusy(() => api.executeShadowPredictionFromMatch(command));
  lastPredictionResult = result;
  render();
  showRouteDecision(result.route);
  showPredictionResult(result, "shadow");
  toast(
    `影子推演完成，输入指纹 ${result.input_audit?.input_manifest_sha256.slice(0, 12) ?? "已生成"}；结果未写入正式历史`,
    "success",
  );
}

async function calculatePredictionMatch(): Promise<void> {
  const explicitRulePackageId = nullableValue("explicit-rule-package-id");
  const command = {
    ...selectedStoredPredictionCommand(),
    explicit_rule_package_id: explicitRulePackageId,
  };
  selectedP4MatchId = command.match_id;
  selectedPredictionSnapshot = command.snapshot_type as LineupSnapshotType;
  const outcome = await runBusy(async () => {
    const readiness = await api.inspectMatchPredictionReadiness(command);
    let chain: MatchLineupChain | null = null;
    try {
      chain = await api.readMatchLineupChain(command.match_id, command.snapshot_type);
    } catch {
      chain = null;
    }
    if (!readiness.can_run_formal) return { readiness, chain, result: null };
    const result = await api.executePredictionFromMatch(command);
    return { readiness, chain, result };
  });
  selectedPredictionReadiness = outcome.readiness;
  lineupsFeature.state.selectedMatchLineupChain = outcome.chain;
  if (!outcome.result) {
    render({ preserveForm: true });
    if (outcome.chain && !outcome.chain.ready_for_model) {
      await openMissingPredictionLineup(outcome.chain);
      return;
    }
    showPredictionReadinessModal(outcome.readiness);
    toast(
      outcome.readiness.level === "shadow_only"
        ? "当前数据只允许影子推演，可在弹窗或完整度面板中直接运行"
        : "赛前数据完整度门禁未通过",
      "normal",
    );
    return;
  }
  lastPredictionResult = outcome.result;
  if (session.bootstrap) await session.refreshRecentRuns();
  render();
  showRouteDecision(outcome.result.route);
  showPredictionResult(outcome.result, "formal");
  toast(
    `正式推演完成，输入指纹 ${outcome.result.input_audit?.input_manifest_sha256.slice(0, 12) ?? "已保存"}`,
    "success",
  );
}

async function planP4Horizons(): Promise<void> {
  const matchId = selectedP4MatchId;
  if (!matchId) throw new Error("请先在单场研究工作台选择比赛");
  const rulePackageId = nullableValue("p4-plan-rule-package");
  if (!rulePackageId) throw new Error("请选择已启用的 P4 正式规则包");
  await runBusy(() =>
    api.planP4Horizons({
      match_id: matchId,
      explicit_rule_package_id: rulePackageId,
      requested_fact_keys: [],
    }),
  );
  await loadP4MatchWorkspace(matchId, p4TaskWorkspace?.task.id ?? null);
  render();
  toast("三个计划窗口已建立并完成幂等校验", "success");
}

async function openP4Task(taskId: string): Promise<void> {
  if (!taskId) throw new Error("缺少冻结任务 ID");
  p4TaskWorkspace = await runBusy(() => api.readP4TaskWorkspace(taskId));
  render({ preserveForm: true });
}

async function refreshP4Workbench(): Promise<void> {
  await runBusy(refreshP4Workspace);
  render({ preserveForm: true });
  toast("单场研究工作台已刷新", "success");
}

async function resolveP4Conflict(
  conflictId: string,
  acceptUnknown: boolean,
): Promise<void> {
  const taskId = p4TaskWorkspace?.task.id;
  if (!taskId) throw new Error("请先选择一个计划窗口任务");
  if (!conflictId) throw new Error("缺少冲突 ID");
  const selected = document.querySelector<HTMLInputElement>(
    `input[name="p4-conflict-${conflictId}"]:checked`,
  );
  if (!acceptUnknown && !selected?.value)
    throw new Error("请选择要采用的证据来源");
  const note = nullableValue(`p4-conflict-note-${conflictId}`);
  const command: ResolveP4ConflictCommand = {
    task_id: taskId,
    conflict_id: conflictId,
    decision_kind: acceptUnknown ? "accept_unknown" : "select_evidence",
    selected_evidence_ids: acceptUnknown ? [] : [selected!.value],
    note,
  };
  p4TaskWorkspace = await runBusy(() => api.resolveP4Conflict(command));
  if (selectedP4MatchId)
    p4MatchWorkspace = await api.readP4MatchWorkspace(selectedP4MatchId);
  render();
  toast(
    acceptUnknown ? "已追加“接受未知”决策" : "已追加证据选择决策",
    "success",
  );
}

function requestHideRunHistory(runId: string, runLabel: string): void {
  if (!runId) throw new Error("缺少推演记录编号");
  showConfirmation(
    "从历史列表删除",
    "该操作只隐藏历史列表中的记录，不删除模型运行、概率矩阵、快照、复盘或收敛血缘。",
    [
      ["推演", runLabel],
      ["底层数据", "完整保留，可继续用于审计、复盘与收敛"],
    ],
    "确认从列表删除",
    async () => {
      await runBusy(() => api.hideModelRunHistory(runId, "用户从历史列表删除"));
      if (session.bootstrap) await session.refreshRecentRuns();
      render();
      toast("推演记录已从历史列表移除", "success");
    },
    { tone: "neutral" },
  );
}

function numberFromReviewRow(row: HTMLElement, field: string): number {
  const input = row.querySelector<HTMLInputElement>(`[data-field="${field}"]`);
  if (!input || input.value.trim() === "") return 0;
  const parsed = Number(input.value);
  if (!Number.isFinite(parsed)) throw new Error(`${field} 必须是有效数字`);
  return parsed;
}

async function loadSelectedReviewMatch(): Promise<void> {
  const matchId = nullableValue("review-match-id");
  if (!matchId) {
    toast("当前没有可载入的复盘比赛", "normal");
    return;
  }
  selectedReviewMatchId = matchId;
  selectedMatchReview = null;
  matchReviewPackagePreview = null;
  lineupsFeature.state.lineupBuilderForm = {
    match_id: matchId,
    team_id: "",
    lineup_type: "actual",
    snapshot_type: "T-1h",
    formation_id: "4737da75-7c7b-52f5-acf5-ea9bfa809c48",
    formation: "4-2-3-1",
    coach_id: "",
    source_urls: "",
    captured_at: "",
    quality_score: 0.9,
  };
  lineupsFeature.state.lineupBuilderPlayers = [];
  lineupsFeature.state.lineupPlayerCandidates = [];
  await runBusy(async () => {
    const result = await fetchReviewCenter(matchId);
    applyReviewCenter(result);
    const existing = result.matches.find(
      (item) => item.match_record.id === matchId,
    )?.latest_review;
    if (existing) selectedMatchReview = await api.readMatchReview(existing.id);
  });
  render();
}



async function refreshMatchReviewPackageWorkflow(): Promise<void> {
  matchReviewPackageWorkflow = selectedReviewMatchId
    ? await api.readMatchReviewPackageWorkflow(selectedReviewMatchId)
    : null;
  matchReviewPackagePreview = matchReviewPackageWorkflow?.preview ?? null;
}

async function exportMatchReviewPackage(): Promise<void> {
  if (!selectedReviewMatchId) throw new Error("请先选择并载入需要复盘的比赛");
  const selected = reviewableMatches.find((item) => item.match_record.id === selectedReviewMatchId);
  if (!selected) throw new Error("当前比赛已不在可复盘列表中，请刷新后重试");
  const safeName = `${selected.match_record.home_team_name}_vs_${selected.match_record.away_team_name}`.replace(/[\/:*?"<>|]/g, "-");
  const outputPath = await api.chooseExcelExportFile(`赛后复盘资料包_${safeName}.xlsx`);
  if (!outputPath) return;
  const summary = await runBusy(() => api.exportMatchReviewPackage(outputPath, selectedReviewMatchId!));
  matchReviewPackagePreview = null;
  await refreshMatchReviewPackageWorkflow();
  render();
  toast(`赛后复盘资料包已导出：${summary.player_count} 名球员`, "success");
  showModal("赛后复盘资料包已导出", summary);
}

async function previewMatchReviewPackage(): Promise<void> {
  if (!selectedReviewMatchId) throw new Error("请先选择并载入需要复盘的比赛");
  if (!matchReviewWorkflowAllows(matchReviewPackageWorkflow, "preview_import")) {
    throw new Error("请先导出本轮赛后复盘资料包，再选择填写后的文件预检");
  }
  const inputPath = await api.chooseExcelImportFile();
  if (!inputPath) return;
  matchReviewPackagePreview = await runBusy(() => api.previewMatchReviewPackage(inputPath, selectedReviewMatchId));
  await refreshMatchReviewPackageWorkflow();
  render();
  const blocking = matchReviewPackagePreview.errors.length;
  toast(
    blocking ? `赛后复盘资料包有 ${blocking} 条阻断错误` : "赛后复盘资料包预检通过",
    blocking ? "error" : "success",
  );
}

function confirmMatchReviewPackage(): void {
  const preview = matchReviewPackagePreview;
  const workflow = matchReviewPackageWorkflow;
  if (!preview?.ready || preview.errors.length || !matchReviewWorkflowAllows(workflow, "confirm_import")) {
    throw new Error("本轮资料包预检尚未通过，不能人工确认");
  }
  showConfirmation(
    "人工确认赛后复盘资料包",
    "后端会重新读取文件、核对本轮 package_id 与 SHA256。确认只冻结本次导入，不会立即写入赛后事实。",
    [
      ["比赛", `${preview.home_team_name} vs ${preview.away_team_name}`],
      ["资料包", preview.package_id],
      ["结构化事件", `${preview.events.length} 条`],
      ["阻断错误", `${preview.errors.length} 条`],
    ],
    "确认本轮资料包",
    async () => {
      matchReviewPackageWorkflow = await runBusy(() => api.confirmMatchReviewPackage({
        package_id: preview.package_id,
        confirmed_by: nullableValue("review-package-confirmed-by"),
        confirmation_note: nullableValue("review-package-confirmation-note"),
      }));
      render();
      toast("资料包已人工确认；下一步写入真实赛后事实", "success");
    },
  );
}

function commitMatchReviewPackageFacts(): void {
  const workflow = matchReviewPackageWorkflow;
  if (!workflow || !matchReviewWorkflowAllows(workflow, "commit_facts")) {
    throw new Error("资料包尚未人工确认，不能写入赛后事实");
  }
  showConfirmation(
    "写入真实赛后事实",
    "将写入实际阵容、正式赛果、换人、结构化比赛事件和球员观察。赛前预计/确认阵容与模型快照不会被覆盖。",
    [
      ["比赛", workflow.match_key],
      ["资料包", workflow.package_id],
      ["文件校验", workflow.import_sha256?.slice(0, 16) ?? "未记录"],
    ],
    "写入真实赛后事实",
    async () => {
      const result = await runBusy(() => api.commitMatchReviewPackageFacts(workflow.package_id));
      matchReviewPackageWorkflow = result.workflow;
      if (selectedReviewMatchId) await loadReviewLineups(selectedReviewMatchId);
      render();
      showModal("真实赛后事实已写入", result);
      toast("真实赛后事实已写入；下一步生成正式复盘", "success");
    },
  );
}

async function generateMatchReviewFromPackage(): Promise<void> {
  const workflow = matchReviewPackageWorkflow;
  if (!workflow || !matchReviewWorkflowAllows(workflow, "generate_review")) {
    throw new Error("真实赛后事实尚未写入，不能生成正式复盘");
  }
  const result = await runBusy(() => api.generateMatchReviewFromPackage(workflow.package_id));
  selectedMatchReview = result.review;
  matchReviewPackageWorkflow = result.workflow;
  selectedReviewMatchId = result.review.summary.match_id;
  await loadReviewCenter();
  render();
  toast("正式复盘已生成；下一步检查正式结算门禁", "success");
}

function collectPlayerReviewObservations(): PlayerMatchObservationDraft[] {
  const rows = Array.from(
    document.querySelectorAll<HTMLElement>("[data-review-player]"),
  );
  if (rows.length === 0) throw new Error("没有可用于复盘的阵容球员");
  return rows.map((row) => {
    const playerId = row.dataset.playerId;
    const teamId = row.dataset.teamId;
    if (!playerId || !teamId)
      throw new Error("球员复盘信息不完整：缺少球员或球队");
    const minutes = numberFromReviewRow(row, "minutes");
    const ratingInput = row.querySelector<HTMLInputElement>(
      '[data-field="rating"]',
    );
    const rating = ratingInput?.value.trim() ? Number(ratingInput.value) : null;
    if (minutes > 0 && (rating === null || !Number.isFinite(rating))) {
      throw new Error(`请为出场球员填写 0–10 评分`);
    }
    if (rating !== null && (rating < 0 || rating > 10)) {
      throw new Error("球员评分必须位于 0–10");
    }
    return {
      player_id: playerId,
      team_id: teamId,
      position_code: row.dataset.positionCode || null,
      role_code: row.dataset.roleCode || null,
      started: row.dataset.started === "true",
      minutes_played: minutes,
      performance_score: null,
      input_confidence: minutes > 0 ? 0.9 : 0.6,
      metrics: {
        goals: numberFromReviewRow(row, "goals"),
        assists: numberFromReviewRow(row, "assists"),
        expected_goals: numberFromReviewRow(row, "expected_goals"),
        expected_assists: numberFromReviewRow(row, "expected_assists"),
        shots: numberFromReviewRow(row, "shots"),
        shots_on_target: numberFromReviewRow(row, "shots_on_target"),
        key_passes: numberFromReviewRow(row, "key_passes"),
        progressive_actions: numberFromReviewRow(row, "progressive_actions"),
        tackles: numberFromReviewRow(row, "tackles"),
        interceptions: numberFromReviewRow(row, "interceptions"),
        clearances: numberFromReviewRow(row, "clearances"),
        blocks: numberFromReviewRow(row, "blocks"),
        duels_won: numberFromReviewRow(row, "duels_won"),
        duels_total: numberFromReviewRow(row, "duels_total"),
        fouls: numberFromReviewRow(row, "fouls"),
        yellow_cards: numberFromReviewRow(row, "yellow_cards"),
        red_cards: numberFromReviewRow(row, "red_cards"),
        errors_leading_to_shot: numberFromReviewRow(
          row,
          "errors_leading_to_shot",
        ),
        provider_rating: rating,
        extra: {},
      },
      source_document_id: null,
    };
  });
}

async function generateMatchReview(): Promise<void> {
  if (!selectedReviewMatchId) throw new Error("请先选择比赛并载入阵容");
  const selected = reviewableMatches.find(
    (item) => item.match_record.id === selectedReviewMatchId,
  );
  if (!selected) throw new Error("所选比赛已经不在待复盘列表中");
  if (reviewLineups.length < 2)
    throw new Error("主客队阵容不完整，请先保存两队阵容");
  const playerObservations = collectPlayerReviewObservations();
  const finalizedAt = localDateTimeToIso(
    nullableValue("review-finalized-at"),
    true,
  );
  if (!finalizedAt) throw new Error("赛果确认时间无效");
  const substitutions = playerObservations
    .filter((player) => !player.started && player.minutes_played > 0)
    .map((player) => ({
      team_id: player.team_id,
      player_out_id: null,
      player_in_id: player.player_id,
      minute: Math.max(0, 90 - Math.min(90, player.minutes_played)),
      period: "normal_time",
      reason: "由实际出场分钟推断",
      source_document_id: null,
      metadata: { inferred: true },
    }));
  const draft: MatchReviewDraft = {
    match_id: selectedReviewMatchId,
    review_version: nullableValue("review-version"),
    data_coverage: Number(value("review-data-coverage")),
    source_run_id: null,
    result: {
      match_id: selectedReviewMatchId,
      home_goals_90: Number(value("review-home-goals")),
      away_goals_90: Number(value("review-away-goals")),
      home_goals_extra_time: null,
      away_goals_extra_time: null,
      home_penalties: null,
      away_penalties: null,
      finalized_at: finalizedAt,
      source_document_id: null,
      metadata: { entry_mode: "desktop_review" },
    },
    substitutions,
    events: [],
    player_observations: playerObservations,
    notes: nullableValue("review-notes"),
  };
  selectedMatchReview = await runBusy(() => api.generateMatchReview(draft));
  await loadReviewCenter();
  render();
  toast("赛后复盘已生成，能力变化进入待审核候选", "success");
}

async function openMatchReview(reviewId: string): Promise<void> {
  if (!reviewId) throw new Error("未找到需要处理的复盘记录");
  selectedMatchReview = await runBusy(() => api.readMatchReview(reviewId));
  selectedReviewMatchId = selectedMatchReview.summary.match_id;
  matchReviewPackagePreview = null;
  applyReviewCenter(await fetchReviewCenter(selectedReviewMatchId));
  render();
}

async function decideAbilityCandidate(
  candidateId: string,
  decision: "accept" | "reject",
): Promise<void> {
  if (!candidateId) throw new Error("未找到需要处理的能力建议");
  await runBusy(() =>
    api.decideAbilityCandidate({
      candidate_id: candidateId,
      decision,
      decided_by: "local_user",
      decision_note:
        decision === "accept" ? "桌面端人工接受" : "桌面端人工拒绝",
    }),
  );
  if (selectedMatchReview)
    selectedMatchReview = await api.readMatchReview(
      selectedMatchReview.summary.id,
    );
  if (navigation.page === "analytics") await loadAnalysisCenter();
  await session.refresh();
  render();
  toast(
    decision === "accept" ? "候选已接受并写入能力历史" : "候选已拒绝",
    "success",
  );
}

function analysisHistoryReady(): boolean {
  return postmatchOverview.settlement_count > 0;
}

function fullAnalysisReady(): boolean {
  return analysisHistoryReady()
    && Boolean(analyticsOverview?.generated_at)
    && (analyticsOverview?.sample_size ?? 0) > 0;
}

function analysisQualityReady(): boolean {
  return fullAnalysisReady()
    && Boolean(analyticsOverview?.data_quality.scan_id)
    && (analyticsOverview?.data_quality.critical ?? 0) === 0;
}

function analysisReviewGateReady(): boolean {
  const pendingSuggestions = aiAnalysisSuggestions.filter((item) => item.status === "pending").length;
  return analysisQualityReady() && pendingSuggestions + analysisAbilityCandidates.length === 0;
}

async function enqueueAnalysisTask(
  jobType: EnqueueJobDraft["job_type"],
): Promise<void> {
  const draft: EnqueueJobDraft = {
    job_type: jobType,
    payload: {},
    idempotency_key: `${jobType}-${new Date().toISOString().slice(0, 16)}`,
    priority: jobType === "full_analysis_refresh" ? 10 : 0,
    max_attempts: 3,
  };
  await runBusy(() => api.enqueueAnalysisJob(draft));
  await loadAnalysisCenter();
  render();
  toast("后台任务已加入队列", "success");
}

async function inspectPostmatchReadiness(reviewId: string): Promise<void> {
  if (!reviewId) throw new Error("复盘记录不存在");
  const readiness = await runBusy(() => api.postmatchSettlementReadiness(reviewId));
  showModal("接入点 H 结算门禁", readiness);
}

function settlePostmatchReview(reviewId: string): void {
  if (!reviewId) throw new Error("复盘记录不存在");
  const review = recentMatchReviews.find((item) => item.id === reviewId);
  showConfirmation(
    "创建正式赛后结算",
    "结算将冻结赛果、成功推演、赛事 Profile、模型与参数版本，并为快照证据建立人工评分队列。结算账本不可覆盖。",
    [
      ["比赛", review ? `${review.home_team_name} vs ${review.away_team_name}` : "当前复盘"],
      ["复盘版本", review?.review_version ?? selectedMatchReview?.summary.review_version ?? "未记录"],
      ["自动改参", "禁止"],
    ],
    "确认正式结算",
    async () => {
      const settlement = await runBusy(() => api.settlePostmatchReview({
        match_review_id: reviewId,
        settled_by: null,
        settlement_note: "用户在赛后复盘页确认正式结算",
      }));
      postmatchOverview = await api.postmatchOverview(100);
      await loadReviewCenter();
      render();
      showModal("正式赛后结算", settlement);
      toast("正式结算已创建，证据已进入人工评分队列", "success");
    },
  );
}

async function refreshPostmatchMonitoring(): Promise<void> {
  const competitionId = value("postmatch-competition-id");
  if (!competitionId) throw new Error("请选择需要监控的具体赛事");
  const request: PostmatchMonitoringRequest = {
    competition_id: competitionId,
    horizon: value("postmatch-horizon"),
    baseline_size: Number(value("postmatch-baseline-size")),
    current_size: Number(value("postmatch-current-size")),
  };
  postmatchOverview = await runBusy(() => api.refreshPostmatchMonitoring(request));
  render();
  toast("接入点 H 正式分区监控已刷新", "success");
}

function prepareEvidenceDecision(itemId: string): void {
  const item = postmatchOverview.evidence_queue.find((value) => value.id === itemId);
  if (!item) throw new Error("证据评分项不存在");
  showHtmlModal(
    "人工判定证据",
    `${item.provider_name ?? "未绑定供应商"} · ${item.field_key}`,
    `<div class="form-grid"><label class="field"><span>赛后核验结论</span><select id="evidence-verdict"><option value="correct">正确</option><option value="partial">部分正确</option><option value="incorrect">错误</option><option value="not_verifiable">无法验证</option></select></label><label class="field"><span>判定说明</span><textarea id="evidence-decision-note" rows="4" placeholder="写明依据、对照结果和无法验证原因，至少 8 个字符"></textarea></label></div>`,
    `<button class="secondary" data-action="close-workspace-detail">取消</button><button class="primary" data-action="submit-evidence-decision" data-evidence-item-id="${escapeHtml(itemId)}">保存不可变判定</button>`,
  );
}

async function submitEvidenceDecision(itemId: string): Promise<void> {
  const draft: EvidenceScoringDecisionDraft = {
    item_id: itemId,
    verdict: value("evidence-verdict") as EvidenceScoringDecisionDraft["verdict"],
    decided_by: null,
    decision_note: value("evidence-decision-note").trim(),
  };
  const item = await runBusy(() => api.decideEvidenceScoringItem(draft));
  postmatchOverview = await api.postmatchOverview(100);
  closeModal();
  render();
  showModal("证据判定记录", item);
  toast("证据判定已写入不可变账本", "success");
}

async function refreshAnalysisPage(): Promise<void> {
  await runBusy(loadAnalysisCenter);
  render();
}

async function generateParameterTuning(): Promise<void> {
  const draft: ParameterTuningDraft = {
    competition_id: nullableValue("tuning-competition-id"),
    snapshot_type: value(
      "tuning-snapshot-type",
    ) as ParameterTuningDraft["snapshot_type"],
    target_module: value(
      "tuning-module",
    ) as ParameterTuningDraft["target_module"],
    minimum_sample_size: Number(value("tuning-min-sample")),
    max_relative_change: Number(value("tuning-max-change")),
  };
  const candidate = await runBusy(() =>
    api.generateParameterTuningCandidate(draft),
  );
  parameterTuningCandidates = await api.listParameterTuningCandidates(100);
  render();
  showModal("参数候选诊断", candidate);
  toast("已生成单模块参数候选，正式模型未被修改", "success");
}

async function decideParameterTuning(
  candidateId: string,
  decision: ParameterTuningDecisionDraft["decision"],
): Promise<void> {
  if (!candidateId) throw new Error("参数候选不存在");
  const draft: ParameterTuningDecisionDraft = {
    candidate_id: candidateId,
    decision,
    decision_note:
      decision === "accept_for_backtest" ? "进入回测队列" : "人工拒绝",
  };
  await runBusy(() => api.decideParameterTuningCandidate(draft));
  parameterTuningCandidates = await api.listParameterTuningCandidates(100);
  render();
  toast(
    decision === "accept_for_backtest"
      ? "候选已进入回测队列，不会自动上线"
      : "候选已拒绝",
    "success",
  );
}

async function checkParameterLifecycleReadiness(): Promise<void> {
  const request: ParameterLifecycleReadinessRequest = {
    competition_id: nullableValue("tuning-competition-id"),
    snapshot_type: value(
      "tuning-snapshot-type",
    ) as ParameterLifecycleReadinessRequest["snapshot_type"],
    minimum_sample_size: Number(value("tuning-min-sample")),
  };
  const readiness = await runBusy(() =>
    api.parameterLifecycleReadiness(request),
  );
  showModal("阶段 I 门禁检查", readiness);
  toast(
    readiness.ready_for_shadow_validation
      ? "阶段 I 影子验证门禁已满足"
      : "阶段 I 仍被前置条件阻断",
    readiness.ready_for_shadow_validation ? "success" : "normal",
  );
}

async function runParameterShadowValidation(candidateId: string): Promise<void> {
  if (!candidateId) throw new Error("参数候选不存在");
  const record = await runBusy(() =>
    api.runParameterShadowValidation({ candidate_id: candidateId }),
  );
  parameterTuningCandidates = await api.listParameterTuningCandidates(100);
  render();
  showModal("阶段 I 影子验证", record);
  toast(
    record.status === "passed"
      ? "影子验证通过，仍需人工晋升"
      : record.status === "blocked"
        ? "影子验证被接入点 H 门禁阻断"
        : "影子验证未通过，正式模型保持不变",
    record.status === "passed" ? "success" : "normal",
  );
}

function promoteParameterCandidate(candidateId: string): void {
  if (!candidateId) throw new Error("参数候选不存在");
  const candidate = parameterTuningCandidates.find(
    (item) => item.id === candidateId,
  );
  showConfirmation(
    "人工晋升参数候选",
    "该操作会把当前赛事活动绑定切换到候选模型与参数版本。系统不会自动执行此操作。",
    [
      ["赛事", candidate?.competition_name ?? "未识别"],
      ["分区", candidate?.partition_key ?? "未记录"],
      ["候选版本", candidate?.candidate_parameter_version ?? "未记录"],
    ],
    "确认人工晋升",
    async () => {
      const request: ParameterPromotionRequest = {
        candidate_id: candidateId,
        decided_by: null,
        decision_note: "用户在分析中心确认全部影子门禁通过并人工晋升",
      };
      const decision = await runBusy(() =>
        api.promoteParameterCandidate(request),
      );
      parameterTuningCandidates = await api.listParameterTuningCandidates(100);
      render();
      showModal("人工晋升记录", decision);
      toast("候选已人工晋升，可按绑定快照回滚", "success");
    },
  );
}

function rollbackParameterCandidate(candidateId: string): void {
  if (!candidateId) throw new Error("参数候选不存在");
  const candidate = parameterTuningCandidates.find(
    (item) => item.id === candidateId,
  );
  showConfirmation(
    "回滚参数候选",
    "回滚只恢复本次晋升记录中的原绑定；若绑定已被其他版本修改，系统会拒绝覆盖。",
    [
      ["赛事", candidate?.competition_name ?? "未识别"],
      ["当前候选", candidate?.candidate_parameter_version ?? "未记录"],
      ["回滚目标", candidate?.parameter_version ?? "未记录"],
    ],
    "确认回滚",
    async () => {
      const request: ParameterRollbackRequest = {
        candidate_id: candidateId,
        decided_by: null,
        decision_note: "用户在分析中心确认按晋升绑定快照回滚",
      };
      const decision = await runBusy(() =>
        api.rollbackParameterCandidate(request),
      );
      parameterTuningCandidates = await api.listParameterTuningCandidates(100);
      render();
      showModal("回滚记录", decision);
      toast("候选晋升已按原绑定快照回滚", "success");
    },
  );
}

async function showParameterLifecycleHistory(candidateId: string): Promise<void> {
  if (!candidateId) throw new Error("参数候选不存在");
  const [validations, decisions] = await runBusy(() =>
    Promise.all([
      api.listParameterShadowValidations(candidateId),
      api.listParameterPromotionDecisions(candidateId),
    ]),
  );
  showModal("阶段 I 生命周期记录", {
    shadow_validations: validations,
    promotion_and_rollback_decisions: decisions,
  });
}

async function exportAiAnalysisPackage(): Promise<void> {
  const outputPath = await api.chooseAiAnalysisExportFile(
    `足球模型智能分析资料_${new Date().toISOString().slice(0, 10)}.zip`,
  );
  if (!outputPath) return;
  const summary = await runBusy(() => api.exportAiAnalysisPackage(outputPath));
  lastAiAnalysisPackageId = summary.package_id;
  localPreferences.setItem(ANALYSIS_PACKAGE_ID_KEY, summary.package_id);
  render();
  toast(
    `智能分析资料已导出，包含 ${summary.sample_size} 个评估样本`,
    "success",
  );
}

async function exportAiAnalysisResponseTemplate(): Promise<void> {
  const outputPath = await api.chooseAiAnalysisResponseTemplateFile(
    `足球模型建议填写模板_${new Date().toISOString().slice(0, 10)}.zip`,
  );
  if (!outputPath) return;
  await runBusy(() =>
    api.exportAiAnalysisResponseTemplate(outputPath, lastAiAnalysisPackageId),
  );
  toast("建议填写模板已生成", "success");
}

async function decideDataQualityFinding(
  findingId: string,
  decision: "resolve" | "ignore",
): Promise<void> {
  if (!findingId) throw new Error("未找到需要处理的数据质量问题");
  await runBusy(() =>
    api.decideDataQualityFinding({
      finding_id: findingId,
      decision,
      resolution_note:
        decision === "resolve" ? "桌面端确认已处理" : "桌面端人工忽略",
    }),
  );
  await loadAnalysisCenter();
  render();
  toast(
    decision === "resolve" ? "已标记为处理完成" : "已忽略该提示",
    "success",
  );
}

async function previewAiAnalysisResponse(): Promise<void> {
  const inputPath = await api.chooseAiAnalysisResponseFile();
  if (!inputPath) return;
  aiAnalysisResponsePath = inputPath;
  aiAnalysisResponsePreview = await runBusy(() =>
    api.previewAiAnalysisResponse(inputPath),
  );
  render();
  const blocking = aiAnalysisResponsePreview.blocking_errors.length;
  toast(
    blocking > 0
      ? `建议文件存在 ${blocking} 个必须先处理的问题`
      : "建议文件检查通过",
    blocking > 0 ? "error" : "success",
  );
}

async function importAiAnalysisResponse(): Promise<void> {
  if (!aiAnalysisResponsePath || !aiAnalysisResponsePreview)
    throw new Error("没有已检查的智能分析建议文件");
  if (aiAnalysisResponsePreview.blocking_errors.length > 0)
    throw new Error("智能分析建议文件仍有必须先处理的问题");
  aiAnalysisSuggestions = await runBusy(() =>
    api.importAiAnalysisResponse(aiAnalysisResponsePath!),
  );
  aiAnalysisResponsePreview = null;
  aiAnalysisResponsePath = null;
  await loadAnalysisCenter();
  render();
  toast("智能分析建议已进入审核区，没有自动修改正式数据", "success");
}

async function decideAiSuggestion(
  suggestionId: string,
  decision: "accept" | "reject",
): Promise<void> {
  const suggestion = aiAnalysisSuggestions.find(
    (item) => item.id === suggestionId,
  );
  if (!suggestion) throw new Error("智能分析建议不存在");
  await runBusy(() =>
    api.decideAiAnalysisSuggestion({
      suggestion_id: suggestionId,
      decision,
      decided_by: "local_user",
      decision_note:
        decision === "accept" ? "桌面端人工接受" : "桌面端人工拒绝",
    }),
  );
  await loadAnalysisCenter();
  await session.refresh();
  render();
  toast(
    decision === "accept" ? "建议已接受并按受控流程处理" : "建议已拒绝",
    "success",
  );
}

function openAiProfileDraftFromForm(): OpenAiProfileDraft {
  const profileId = nullableValue("openai-profile-id");
  const apiKey = nullableValue("openai-api-key");
  return {
    id: profileId,
    name: value("openai-profile-name").trim(),
    api_key: apiKey,
    api_base_url: value("openai-api-base-url").trim(),
    api_protocol: value(
      "openai-api-protocol",
    ) as OpenAiProfileDraft["api_protocol"],
    api_endpoint: value("openai-api-endpoint").trim(),
    token_limit_field: value(
      "openai-token-limit-field",
    ) as OpenAiProfileDraft["token_limit_field"],
    api_workspace_web_search_mode: value(
      "openai-api-workspace-web-search-mode",
    ) as OpenAiProfileDraft["api_workspace_web_search_mode"],
    api_example_template: nullableValue("openai-api-example"),
    research_model: value("openai-research-model").trim(),
    extraction_model: value("openai-extraction-model").trim(),
    fallback_model: nullableValue("openai-fallback-model"),
    reasoning_effort: value(
      "openai-reasoning-effort",
    ) as OpenAiProfileDraft["reasoning_effort"],
    timeout_seconds: Number(value("openai-timeout-seconds")),
    max_retries: Number(value("openai-max-retries")),
    max_concurrency: Number(value("openai-max-concurrency")),
    max_output_tokens: Number(value("openai-max-output-tokens")),
    max_tool_calls: Number(value("openai-max-tool-calls")),
    search_context_size: value(
      "openai-search-context-size",
    ) as OpenAiProfileDraft["search_context_size"],
  };
}

function setOpenAiApiExampleStatus(
  state: "idle" | "parsing" | "success" | "error",
  title: string,
  message: string,
): void {
  const container = document.querySelector<HTMLDivElement>(
    "#openai-api-example-status",
  );
  if (!container) return;
  container.className = `api-example-status ${state}`;
  const strong = container.querySelector<HTMLElement>("strong");
  const span = container.querySelector<HTMLElement>("span");
  if (strong) strong.textContent = title;
  if (span) span.textContent = message;
}

function setInputValue(id: string, next: string): void {
  const input = document.querySelector<HTMLInputElement | HTMLSelectElement>(
    `#${id}`,
  );
  if (input) input.value = next;
}

function applyOpenAiApiExample(result: OpenAiApiExampleParseResult): void {
  const candidate = result.selected;
  setInputValue("openai-api-protocol", candidate.protocol);
  setInputValue("openai-api-endpoint", candidate.endpoint_url);
  setInputValue("openai-api-base-url", candidate.api_base_url);
  setInputValue("openai-token-limit-field", candidate.token_limit_field);
  if (candidate.model_id) {
    setInputValue("openai-research-model", candidate.model_id);
    setInputValue("openai-extraction-model", candidate.model_id);
  }
  if (candidate.max_output_tokens !== null) {
    setInputValue(
      "openai-max-output-tokens",
      String(candidate.max_output_tokens),
    );
  }
  if (candidate.api_key) setInputValue("openai-api-key", candidate.api_key);
  const protocol =
    candidate.protocol === "responses" ? "Responses" : "Chat Completions";
  const warning = candidate.warnings[0] ?? "已识别请求端点、协议和模型";
  setOpenAiApiExampleStatus(
    "success",
    `已识别 ${protocol} · ${result.candidates.length} 个候选`,
    `${candidate.endpoint_url}；${warning}`,
  );
}

async function parseOpenAiApiExampleNow(): Promise<boolean> {
  if (browserLifecycleController.signal.aborted) return false;
  const request = openAiExampleRequests.begin();
  const textarea = document.querySelector<HTMLTextAreaElement>(
    "#openai-api-example",
  );
  if (!textarea) return true;
  const example = textarea.value.trim();
  if (!example) {
    openAiApiExampleLastParsed = "";
    setOpenAiApiExampleStatus(
      "idle",
      "等待输入",
      "粘贴或编辑后会自动替换下方协议、端点和模型。",
    );
    return true;
  }
  const protocol = value("openai-api-protocol") as OpenAiApiProtocol;
  setOpenAiApiExampleStatus(
    "parsing",
    "正在解析",
    "正在识别 curl、请求体和协议……",
  );
  try {
    const result = await api.parseOpenAiApiExample(example, protocol);
    if (!request.isCurrent() || textarea !== document.querySelector("#openai-api-example") || textarea.value.trim() !== example) return false;
    applyOpenAiApiExample(result);
    openAiApiExampleLastParsed = example;
    return true;
  } catch (error: unknown) {
    if (!request.isCurrent() || textarea !== document.querySelector("#openai-api-example") || textarea.value.trim() !== example) return false;
    setOpenAiApiExampleStatus("error", "无法解析", userFacingError(error));
    return false;
  }
}

function scheduleOpenAiApiExampleParse(): void {
  openAiExampleRequests.invalidate();
  openAiApiExampleTimer.schedule(() => {
    void parseOpenAiApiExampleNow();
  }, 320);
}

async function reloadOpenAiProfiles(
  preferredProfileId: string | null = null,
): Promise<void> {
  openAiProfiles = await api.listOpenAiProfiles();
  selectedOpenAiProfileId =
    preferredProfileId ??
    selectedOpenAiProfileId ??
    openAiProfiles.active_profile_id ??
    openAiProfiles.profiles[0]?.id ??
    null;
  if (
    selectedOpenAiProfileId &&
    !openAiProfiles.profiles.some(
      (profile) => profile.id === selectedOpenAiProfileId,
    )
  ) {
    selectedOpenAiProfileId =
      openAiProfiles.active_profile_id ??
      openAiProfiles.profiles[0]?.id ??
      null;
  }
}

async function saveOpenAiProfile(showSuccess = true): Promise<string> {
  const example =
    document
      .querySelector<HTMLTextAreaElement>("#openai-api-example")
      ?.value.trim() ?? "";
  if (example && example !== openAiApiExampleLastParsed) {
    openAiApiExampleTimer.cancel();
    const parsed = await parseOpenAiApiExampleNow();
    if (!parsed) throw new Error("API Example尚未通过解析，无法保存配置");
  }
  const saved = await runBusy(() =>
    api.saveOpenAiProfile(openAiProfileDraftFromForm()),
  );
  creatingOpenAiProfile = false;
  selectedOpenAiProfileId = saved.id;
  await reloadOpenAiProfiles(saved.id);
  render();
  if (showSuccess) toast("兼容 API 配置已安全保存", "success");
  return saved.id;
}

async function testOpenAiProfile(): Promise<void> {
  const profileId = await saveOpenAiProfile(false);
  const result = await runBusy(() => api.testOpenAiProfile(profileId));
  await reloadOpenAiProfiles(profileId);
  render();
  const protocol =
    result.protocol === "responses" ? "Responses" : "Chat Completions";
  toast(
    `连接成功：${protocol} · ${result.model_id} · ${result.latency_ms} 毫秒`,
    "success",
  );
}

async function activateOpenAiProfile(): Promise<void> {
  const profileId = await saveOpenAiProfile(false);
  openAiProfiles = await runBusy(() => api.setActiveOpenAiProfile(profileId));
  selectedOpenAiProfileId = profileId;
  render();
  toast("已切换为当前兼容 API 配置", "success");
}

function requestDeleteOpenAiProfile(): void {
  const profile = openAiProfiles?.profiles.find(
    (item) => item.id === selectedOpenAiProfileId,
  );
  if (!profile) throw new Error("没有可删除的兼容 API 配置");
  showConfirmation(
    "删除兼容 API 配置",
    "该操作会同时删除该配置在 Windows 凭据管理器中的 API Key，且无法恢复。其他配置不受影响。",
    [
      ["配置名称", profile.name],
      ["研究模型", profile.research_model],
      ["当前使用", profile.is_active ? "是" : "否"],
    ],
    "确认删除",
    async () => {
      openAiProfiles = await runBusy(() => api.deleteOpenAiProfile(profile.id));
      creatingOpenAiProfile = false;
      selectedOpenAiProfileId =
        openAiProfiles.active_profile_id ??
        openAiProfiles.profiles[0]?.id ??
        null;
      render();
      toast("兼容 API 配置和对应密钥已删除", "success");
    },
  );
}

function requestClearOpenAiKey(): void {
  const profile = openAiProfiles?.profiles.find(
    (item) => item.id === selectedOpenAiProfileId,
  );
  if (!profile) throw new Error("没有可处理的兼容 API 配置");
  showConfirmation(
    "移除 API Key",
    "仅删除 Windows 凭据管理器中的密钥，配置名称、API 地址和模型参数会保留。",
    [
      ["配置名称", profile.name],
      ["密钥状态", profile.has_api_key ? "已保存" : "未保存"],
    ],
    "移除密钥",
    async () => {
      openAiProfiles = await runBusy(() =>
        api.clearOpenAiProfileKey(profile.id),
      );
      render();
      toast("API Key 已从 Windows 凭据管理器移除", "success");
    },
  );
}

async function refreshReleaseAcceptance(): Promise<void> {
  releaseAcceptanceRuns = await runBusy(() => api.listReleaseAcceptanceRuns(50));
  if (selectedReleaseAcceptanceRun) {
    selectedReleaseAcceptanceRun = await runBusy(() => api.readReleaseAcceptanceRun(selectedReleaseAcceptanceRun!.id));
  } else if (releaseAcceptanceRuns[0]) {
    selectedReleaseAcceptanceRun = await runBusy(() => api.readReleaseAcceptanceRun(releaseAcceptanceRuns[0].id));
  }
  render();
}

function optionalNumber(id: string): number | null {
  const raw = value(id).trim();
  if (!raw) return null;
  const parsed = Number(raw);
  if (!Number.isFinite(parsed) || parsed < 0) throw new Error(`${id.includes("daily") ? "单日" : "周期"}成本预算必须为非负数`);
  return parsed;
}

async function runReleaseAcceptance(): Promise<void> {
  const request: ReleaseAcceptanceRequest = {
    performance_window_days: Math.max(1, Math.min(365, Number(value("release-performance-window") || "30"))),
    cost_window_days: Math.max(1, Math.min(365, Number(value("release-cost-window") || "30"))),
    daily_cost_budget_usd: optionalNumber("release-daily-budget"),
    monthly_cost_budget_usd: optionalNumber("release-monthly-budget"),
    requested_by: nullableValue("release-requested-by"),
  };
  selectedReleaseAcceptanceRun = await runBusy(() => api.runReleaseAcceptance(request));
  releaseAcceptanceRuns = await api.listReleaseAcceptanceRuns(50);
  workspaceState.patchModule("release", { active_section: "overview" });
  render();
  toast(selectedReleaseAcceptanceRun.overall_status === "blocked" ? "验收完成：存在发布阻断项" : "发布验收已完成并写入不可变账本", selectedReleaseAcceptanceRun.overall_status === "blocked" ? "error" : "success");
}

async function openReleaseAcceptanceRun(runId: string): Promise<void> {
  selectedReleaseAcceptanceRun = await runBusy(() => api.readReleaseAcceptanceRun(runId));
  render();
}

async function handleAction(
  action: string,
  button: HTMLElement,
): Promise<void> {
  const shellAction = shellController?.handleAction(action);
  if (shellAction === true || (shellAction && await shellAction)) return;
  switch (action) {
    case "toggle-workspace-pane":
      toggleWorkspacePane(button.dataset.pane === "inspector" ? "inspector" : "module-sidebar");
      break;
    case "complete-workflow":
      await startWorkflowCompletion(button);
      break;
    case "workflow-return":
      await returnToWorkflow();
      break;
    case "workflow-cancel":
      workflowContinuation = null;
      workflowNavigation.clear();
      render({ preserveForm: true });
      break;
    case "select-workspace-section": {
      const sectionId = button.dataset.sectionId;
      if (!sectionId) break;
      if (sectionId === "directory" && navigation.page === "teams") {
        const id = teamsFeature.catalog.detail?.team.id ?? null;
        teamsFeature.directory(); if (id) workspaceState.restoreFocus("teams", app.querySelector(".page-container") ?? app, `team-directory-${id}`); break;
      }
      workspaceState.patchModule(navigation.page, { active_section: sectionId });
      render({ preserveForm: true });
      if (sectionId === "directory" && navigation.page === "players" && playersFeature.catalog.detail) workspaceState.restoreFocus("players", app.querySelector(".page-container") ?? app, `player-directory-${playersFeature.catalog.detail.player.id}`);
      break;
    }
    case "refresh-service-state":
      await runBusy(() => session.refresh());
      render({ preserveForm: true });
      break;
    case "show-database-technical":
      if (session.bootstrap) showHtmlModal("技术连接信息", "凭据已脱敏，不显示密码。", databaseTechnical(session.bootstrap));
      break;
    case "show-database-scope":
      showModal("数据服务使用边界", "已连接不等于所有业务已就绪。连接与保存可能执行迁移与内置资料注册；清除连接不删除业务数据。危险重置须核对当前数据库身份并输入名称确认，无法撤销。统计、模型提供器与业务许可分别校验。");
      break;
    case "show-architecture-scope":
      showModal("架构说明范围", "本页说明职责、历史和路由原则，不代表实时健康、模型可执行性或发布验收结果。连接状态、输入条件、提供器与许可由对应业务流程校验。");
      break;
    case "show-model-runtime-boundary":
      showModal("模型接口与运行能力", "统一接口与规则登记不等于已安装模型运行时。当前公共仓库不捆绑外部模型运行时，实际执行前仍须验证提供器、模型许可和本次输入条件。");
      break;
    case "jump-workspace-anchor": {
      const anchorId = button.dataset.anchorId;
      if (!anchorId) break;
      const target = document.getElementById(anchorId);
      if (!target) break;
      if (activateSectionPanel(app, anchorId)) {
        workspaceState.patchModule(navigation.page, { active_section: anchorId });
        break;
      }
      if (target instanceof HTMLDetailsElement) target.open = true;
      const nav = button.closest<HTMLElement>(".workspace-anchor-nav, .workspace-task-anchor-nav");
      nav?.querySelectorAll<HTMLElement>("button[data-anchor-id]").forEach((item) => item.classList.toggle("active", item === button));
      target.scrollIntoView({ behavior: motionScrollBehavior("smooth"), block: "start" });
      break;
    }
    case "select-review-workflow-step": {
      const step = Number(button.dataset.reviewStep);
      if (!Number.isInteger(step) || step < 1 || step > 9) break;
      workspaceState.patchModule("review", { active_section: `step-${step}` });
      render({ preserveForm: true });
      break;
    }
    case "run-release-acceptance":
      await runReleaseAcceptance();
      break;
    case "refresh-release-acceptance":
      await refreshReleaseAcceptance();
      break;
    case "open-release-acceptance-run":
      await openReleaseAcceptanceRun(button.dataset.runId ?? "");
      break;
    case "show-release-check-evidence": {
      const item = selectedReleaseAcceptanceRun?.checks.find((check) => check.id === button.dataset.checkId);
      if (item) showModal(`验收证据 · ${item.title}`, item.evidence);
      break;
    }
    case "show-release-acceptance-json":
      if (selectedReleaseAcceptanceRun) showModal("接入点 J 完整验收报告", selectedReleaseAcceptanceRun);
      break;
    case "open-selected-teams":
      await openSelectedWorkspaceObjects("teams");
      break;
    case "open-selected-players":
      await openSelectedWorkspaceObjects("players");
      break;
    case "activate-teams-tab":
      await activateWorkspaceTab("teams", button.dataset.objectId ?? "");
      break;
    case "activate-players-tab":
      await activateWorkspaceTab("players", button.dataset.objectId ?? "");
      break;
    case "close-teams-tab":
      await closeWorkspaceTab("teams", button.dataset.objectId ?? "");
      break;
    case "close-players-tab":
      await closeWorkspaceTab("players", button.dataset.objectId ?? "");
      break;
    case "set-teams-workspace-mode":
      setWorkspaceMode("teams", (button.dataset.mode ?? "detail") as WorkspaceLayoutMode);
      break;
    case "set-players-workspace-mode":
      setWorkspaceMode("players", (button.dataset.mode ?? "detail") as WorkspaceLayoutMode);
      break;
    case "new-api-workspace-session":
      workspaceState.patchModule("api_workspace", { active_section: "chat" });
      apiWorkspaceDetail = null;
      selectedApiWorkspacePresetKey = "plain_chat";
      selectedApiWorkspaceMatchId = null;
      selectedApiWorkspaceContextEntityType = null;
      selectedApiWorkspaceContextEntityId = null;
      selectedApiWorkspaceContextEntityLabel = null;
      apiWorkspaceIncludeContext = false;
      apiWorkspaceDraftMessage = "";
      render();
      break;
    case "refresh-api-workspace":
      await loadApiWorkspace();
      render();
      toast("AI 问答已刷新", "success");
      break;
    case "select-api-workspace-session": {
      const sessionId = button.dataset.sessionId;
      if (!sessionId) break;
      apiWorkspaceDetail = await api.readApiWorkspaceSession(sessionId);
      selectedApiWorkspaceProfileId = apiWorkspaceDetail.session.profile_id;
      selectedApiWorkspacePresetKey = apiWorkspaceDetail.session.preset_key;
      selectedApiWorkspaceMatchId = apiWorkspaceDetail.session.match_id;
      const metadata = apiWorkspaceDetail.session.metadata;
      selectedApiWorkspaceContextEntityType =
        metadata.context_entity_type === "team" ||
        metadata.context_entity_type === "player"
          ? metadata.context_entity_type
          : null;
      selectedApiWorkspaceContextEntityId =
        typeof metadata.context_entity_id === "string"
          ? metadata.context_entity_id
          : null;
      selectedApiWorkspaceContextEntityLabel =
        typeof metadata.context_entity_label === "string"
          ? metadata.context_entity_label
          : null;
      apiWorkspaceIncludeContext = Boolean(
        selectedApiWorkspaceMatchId || selectedApiWorkspaceContextEntityId,
      );
      workspaceState.patchModule("api_workspace", { active_section: "chat" });
      render();
      break;
    }
    case "archive-api-workspace-session": {
      const sessionId = button.dataset.sessionId;
      if (!sessionId) break;
      const sessionTitle = button.dataset.sessionTitle ?? "当前会话";
      showConfirmation(
        "删除 AI 问答会话",
        "会话将从常用列表归档。历史消息、旧提案和历史生成文件不会被物理删除，仍保留审计数据。",
        [["会话", sessionTitle]],
        "确认删除",
        async () => {
          await api.archiveApiWorkspaceSession(sessionId);
          if (apiWorkspaceDetail?.session.id === sessionId)
            apiWorkspaceDetail = null;
          apiWorkspaceSessions = await api.listApiWorkspaceSessions(100);
          render();
          toast("AI 问答会话已归档", "success");
        },
      );
      break;
    }
    case "use-api-workspace-prompt": {
      const textarea = document.querySelector<HTMLTextAreaElement>(
        "#api-workspace-message",
      );
      if (textarea) {
        textarea.value = button.dataset.prompt ?? "";
        apiWorkspaceDraftMessage = textarea.value;
        textarea.focus();
      }
      break;
    }
    case "clear-api-workspace-draft":
      apiWorkspaceDraftMessage = "";
      render();
      break;
    case "copy-api-workspace-message": {
      const messageId = button.dataset.messageId;
      const content = apiWorkspaceDetail?.messages.find(
        (item) => item.id === messageId,
      )?.content;
      if (!content) break;
      await clipboard.writeText(content);
      toast("AI 回答已复制", "success");
      break;
    }
    case "cancel-api-workspace-request": {
      const requestId = button.dataset.requestId ?? apiWorkspaceActiveRequestId;
      if (!requestId) break;
      const cancelled = await api.cancelApiWorkspaceRequest(requestId);
      toast(
        cancelled ? "已请求取消当前 AI 问答" : "请求已结束，无需取消",
        "success",
      );
      break;
    }
    case "send-api-workspace-message":
      await sendApiWorkspaceMessage();
      break;
    case "new-openai-profile":
      creatingOpenAiProfile = true;
      selectedOpenAiProfileId = null;
      render();
      break;
    case "select-openai-profile":
      creatingOpenAiProfile = false;
      selectedOpenAiProfileId = button.dataset.profileId ?? null;
      render();
      break;
    case "toggle-openai-key-visibility": {
      const input = document.querySelector<HTMLInputElement>("#openai-api-key");
      if (!input) break;
      const showing = input.type === "text";
      input.type = showing ? "password" : "text";
      button.textContent = showing ? "显示" : "隐藏";
      break;
    }
    case "save-openai-profile":
      await saveOpenAiProfile();
      break;
    case "test-openai-profile":
      await testOpenAiProfile();
      break;
    case "activate-openai-profile":
      await activateOpenAiProfile();
      break;
    case "request-delete-openai-profile":
      requestDeleteOpenAiProfile();
      break;
    case "request-clear-openai-key":
      requestClearOpenAiKey();
      break;
    case "refresh-issue-logs":
      await runBusy(loadIssueLogs);
      render();
      break;
    case "export-issue-logs": {
      const outputPath = await api.chooseIssueLogExportFile(
        "足球赛事模型平台_问题日志报告.txt",
      );
      if (!outputPath) break;
      await runBusy(() => api.exportIssueLogs(outputPath));
      toast(`问题日志已导出：${outputPath}`, "success");
      break;
    }
    case "request-clear-issue-logs":
      showConfirmation(
        "清空问题日志",
        "清空后无法在客户端恢复。该操作只删除问题日志，不影响比赛、球员、模型和数据库业务数据。",
        [
          ["独立问题", String(issueLogs.length)],
          [
            "总发生次数",
            String(
              issueLogs.reduce((sum, item) => sum + item.occurrence_count, 0),
            ),
          ],
        ],
        "确认清空",
        async () => {
          await runBusy(() => api.clearIssueLogs());
          issueLogs = [];
          render();
          toast("问题日志已清空", "success");
        },
      );
      break;
    case "connect-database":
      await databaseFeature.connect(app);
      break;
    case "disconnect-database":
      await databaseFeature.disconnect();
      break;
    case "open-command-menu":
      openCommandMenu();
      break;
    case "request-reset-database":
      databaseFeature.requestReset(document);
      break;
    case "execute-database-reset":
      await modal.runPendingAction();
      break;
    case "dry-run":
      await dryRun();
      break;
    case "preview-route":
      await previewRoute();
      break;
    case "execute-prediction":
      await executePrediction();
      break;
    case "preview-stored-route":
      await previewStoredRoute();
      break;
    case "calculate-prediction-match":
      await calculatePredictionMatch();
      break;
    case "run-shadow-prediction-match":
      await runShadowPredictionMatch();
      break;
    case "check-prediction-lineup-chain":
      await checkPredictionLineupChain();
      break;
    case "prepare-prediction-lineups":
      await preparePredictionLineups();
      break;
    case "continue-lineup-prediction":
      await continueLineupPrediction();
      break;
    case "plan-p4-horizons":
      await planP4Horizons();
      break;
    case "refresh-p4-workbench":
      await refreshP4Workbench();
      break;
    case "open-p4-task":
      await openP4Task(button.dataset.taskId ?? "");
      break;
    case "resolve-p4-conflict-select":
      await resolveP4Conflict(button.dataset.conflictId ?? "", false);
      break;
    case "resolve-p4-conflict-unknown":
      await resolveP4Conflict(button.dataset.conflictId ?? "", true);
      break;
    case "show-p4-snapshot":
      if (p4TaskWorkspace?.snapshot)
        showModal("P4 冻结快照", p4TaskWorkspace.snapshot);
      break;
    case "create-competition":
      await competitionFeature.createCompetition(app);
      break;
    case "clear-rules-catalogue-filters":
      clearRulesCatalogueFilters(filterRulesCompetitionCatalogue);
      break;
    case "show-competition-path":
      competitionFeature.showPath(button.dataset.competitionId ?? "");
      break;
    case "request-delete-competition":
      competitionFeature.requestDelete(button.dataset.competitionId ?? "");
      break;
    case "create-season":
      await competitionFeature.createSeason(app);
      break;
    case "create-stage":
      await competitionFeature.createStage(app);
      break;
    case "create-round":
      await competitionFeature.createRound(app);
      break;
    case "register-rule-package":
      await rulesFeature.registerPackage();
      break;
    case "clear-rule-package":
      rulesFeature.clearPackage();
      break;
    case "show-pending-rule-package":
      if (rulesFeature.packages.pending) showModal("规则包只读详情", rulesFeature.packages.pending);
      break;
    case "create-binding":
      await rulesFeature.createBinding(app);
      break;
    case "refresh-player-catalog":
      await runBusy(() => playersFeature.load(true));
      render();
      break;
    case "refresh-team-catalog":
      await runBusy(() => teamsFeature.load(true));
      render();
      break;
    case "show-lineup-preset-rules":
      showHtmlModal("预设套用规则", "本场副本与球队预设独立", "<p>先选择比赛与球队，预览最新名单、阵型与角色。确认仅替换目标一侧草稿；比赛或草稿变化后需重新预览。双方阵容仍需各11名首发并通过独立保存与模型准入检查。</p>");
      break;
    case "refresh-lineup-preset-page":
      await refreshLineupPresetPage();
      break;
    case "select-lineup-preset-team":
      await teamsFeature.selectPresetTeam(button.dataset.teamId ?? "");
      break;
    case "search-teams":
      await teamsFeature.search();
      break;
    case "search-teams-from-detail":
      await teamsFeature.search(true);
      break;
    case "clear-team-filters":
      await teamsFeature.clearFilters();
      break;
    case "clear-team-filters-from-detail":
      await teamsFeature.clearFilters(true);
      break;
    case "next-team-page":
      await teamsFeature.next();
      break;
    case "previous-team-page":
      await teamsFeature.previous();
      break;
    case "open-team":
      await teamsFeature.open(button.dataset.teamId ?? "");
      break;
    case "return-team-directory": {
      const id = teamsFeature.catalog.detail?.team.id ?? null;
      teamsFeature.directory(); if (id) workspaceState.restoreFocus("teams", app.querySelector(".page-container") ?? app, `team-directory-${id}`);
      break;
    }
    case "update-team":
      await teamsFeature.editor.update(button.dataset.teamId ?? "");
      break;
    case "add-team-name":
      await teamsFeature.editor.addName(button.dataset.teamId ?? "");
      break;
    case "save-team-profile":
      await teamsFeature.editor.profile(button.dataset.teamId ?? "");
      break;
    case "save-formation-usage":
      await teamsFeature.editor.formation(button.dataset.teamId ?? "");
      break;
    case "export-team-package-template":
      await teamsFeature.exportFile("package-template");
      break;
    case "preview-team-package-import":
      await teamsFeature.previewPackage();
      break;
    case "resolve-team-package-conflict":
      await teamsFeature.resolvePackage(
        button.dataset.packageScope === "player" ? "player" : "team",
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-team-package-conflict":
      await teamsFeature.resolvePackage(
        button.dataset.packageScope === "player" ? "player" : "team",
        button.dataset.rowId ?? "",
        null,
        true,
      );
      break;
    case "commit-team-package-import":
      await teamsFeature.commitPackage();
      break;
    case "show-team-package-preview-json":
      if (teamsFeature.imports.package)
        showModal(
          "球队完整资料包预检结果",
          teamsFeature.imports.package,
          `<button type="button" class="secondary" data-action="close-workspace-detail">关闭</button><button type="button" class="primary" data-action="export-team-package-preview-json">导出完整预检 JSON</button>`,
        );
      break;
    case "export-team-package-preview-json":
      await teamsFeature.exportPackagePreview();
      break;
    case "export-team-template":
      await teamsFeature.exportFile("monthly-template");
      break;
    case "export-team-data":
      await teamsFeature.exportFile("monthly-data");
      break;
    case "preview-team-import":
      await teamsFeature.previewMonthly();
      break;
    case "resolve-team-import-conflict":
      await teamsFeature.resolveMonthly(
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-team-import-conflict":
      await teamsFeature.resolveMonthly(button.dataset.rowId ?? "", null, true);
      break;
    case "commit-team-import":
      await teamsFeature.commitMonthly();
      break;
    case "show-team-import-preview-json":
      if (teamsFeature.imports.monthly)
        showModal("球队月度工作簿预检结果", teamsFeature.imports.monthly);
      break;
    case "create-coach":
      await teamsFeature.editor.createCoach();
      break;
    case "add-team-coach-period":
      await teamsFeature.editor.coachPeriod(button.dataset.teamId ?? "");
      break;
    case "bulk-archive-teams":
      await bulkArchiveEntities("team");
      break;
    case "bulk-archive-players":
      await bulkArchiveEntities("player");
      break;
    case "bulk-delete-teams":
      await teamsFeature.requestDelete();
      break;
    case "request-force-delete-team":
      await teamsFeature.requestForceDeleteTeam(button.dataset.teamId ?? "");
      break;
    case "confirm-force-delete-team":
      await teamsFeature.confirmForceDeleteTeam();
      break;
    case "bulk-delete-players":
      playersFeature.requestBulkDelete();
      break;
    case "preview-player-from-team":
      await previewPlayerFromTeam(button.dataset.playerId ?? "");
      break;
    case "open-player-from-team":
      await openPlayerFromTeam(button.dataset.playerId ?? "");
      break;
    case "open-player-profile-from-team":
      await openPlayerProfileFromTeam(button.dataset.playerId ?? "");
      break;
    case "return-to-source-team-profile":
      await returnToSourceTeamProfile(button.dataset.teamId ?? "");
      break;
    case "open-player-from-lineup":
      await openPlayerFromLineup(
        button.dataset.playerId ?? "",
        button.dataset.teamId ?? "",
        button.dataset.teamName ?? "",
        button.dataset.returnSection === "builder" ? "builder" : "chain",
      );
      break;
    case "return-to-lineup-workspace":
      await returnToLineupWorkspace(button.dataset.returnSection === "builder" ? "builder" : "chain");
      break;
    case "open-team-api-workspace":
      await openTeamApiWorkspace(button.dataset.teamId ?? null);
      break;
    case "open-player-api-workspace":
      await openPlayerApiWorkspace(button.dataset.playerId ?? "");
      break;
    case "search-team-options":
      await playersFeature.searchTeamOptions();
      break;
    case "search-players":
      await playersFeature.search();
      break;
    case "clear-player-filters":
      await playersFeature.clearFilters();
      break;
    case "next-player-page":
      await playersFeature.next();
      break;
    case "previous-player-page":
      await playersFeature.previous();
      break;
    case "open-player":
      await playersFeature.openDirectory(button.dataset.playerId ?? "");
      break;
    case "open-player-profile":
      await playersFeature.openProfile(button.dataset.playerId ?? "");
      break;
    case "open-team-profile":
      await teamsFeature.openProfile(button.dataset.teamId ?? "");
      break;
    case "show-player-json":
      if (playersFeature.catalog.detail) showModal("球员完整档案", playersFeature.catalog.detail);
      break;
    case "create-team":
      await teamsFeature.create();
      break;
    case "create-provider":
      await playersFeature.editor.createProvider();
      break;
    case "create-player":
      await playersFeature.editor.createPlayer();
      break;
    case "create-lineup-player":
      await createLineupPlayerQuick();
      break;
    case "update-player":
      await playersFeature.editor.updatePlayer(button.dataset.playerId ?? "");
      break;
    case "request-delete-player":
      await playersFeature.requestDelete(
        button.dataset.playerId ?? "",
        button.dataset.playerName ?? "未命名球员",
      );
      break;
    case "export-player-template":
      await playersFeature.exportTemplate();
      break;
    case "export-player-data":
      await playersFeature.exportData();
      break;
    case "preview-player-import":
      await playersFeature.previewImport();
      break;
    case "resolve-import-conflict":
      await playersFeature.resolveImport(
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-import-conflict":
      await playersFeature.resolveImport(button.dataset.rowId ?? "", null, true);
      break;
    case "commit-player-import":
      await playersFeature.commitImport();
      break;
    case "show-import-preview-json":
      if (playersFeature.imports.preview) showModal("表格导入检查结果", playersFeature.imports.preview);
      break;
    case "add-player-name":
      await playersFeature.editor.addPlayerName(button.dataset.playerId ?? "");
      break;
    case "assign-player-position":
      await playersFeature.editor.assignPlayerPosition(button.dataset.playerId ?? "");
      break;
    case "add-player-team-period":
      await playersFeature.editor.addPlayerTeamPeriod(button.dataset.playerId ?? "");
      break;
    case "add-player-availability":
      await playersFeature.editor.addPlayerAvailability(button.dataset.playerId ?? "");
      break;
    case "add-player-ability":
      await playersFeature.editor.addPlayerAbility(button.dataset.playerId ?? "");
      break;
    case "add-player-external-id":
      await playersFeature.editor.addPlayerExternalId(button.dataset.playerId ?? "");
      break;
    case "add-player-dynamic-tag":
      await playersFeature.editor.addPlayerDynamicTag(button.dataset.playerId ?? "");
      break;
    case "calculate-player-contribution":
      await playersFeature.editor.calculatePlayerContribution(button.dataset.playerId ?? "");
      break;
    case "export-match-template":
      await lineupsFeature.actions.exportMatchTemplate();
      break;
    case "export-match-data":
      await lineupsFeature.actions.exportMatchData();
      break;
    case "export-ai-match-package":
      await lineupsFeature.actions.exportAiMatchPackage();
      break;
    case "calculate-stored-match":
      await calculateStoredMatch();
      break;
    case "preview-match-import":
      await lineupsFeature.actions.previewMatchImport(false);
      break;
    case "preview-ai-match-import":
      await lineupsFeature.actions.previewMatchImport(true);
      break;
    case "resolve-match-import-conflict":
      await lineupsFeature.actions.resolveMatchImportConflict(
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-match-import-conflict":
      await lineupsFeature.actions.resolveMatchImportConflict(button.dataset.rowId ?? "", null, true);
      break;
    case "commit-match-import":
      await lineupsFeature.actions.commitMatchImport();
      break;
    case "show-match-import-json":
      if (lineupsFeature.imports.preview) {
        await lineupsFeature.imports.readback();
        if(lineupsFeature.imports.preview)showModal("比赛与阵容导入预检", lineupsFeature.imports.preview);
      }
      break;
    case "select-managed-match": {
      const matchId = button.dataset.matchId ?? "";
      lineupsFeature.state.selectedManagedMatchId = matchId || null;
      render({ preserveForm: true });
      queueMicrotask(() => {
        lineupsFeature.actions.autoSelectMatchSeason();
        lineupsFeature.actions.filterMatchTeamOptions();
      });
      break;
    }
    case "new-managed-match":
      lineupsFeature.state.selectedManagedMatchId = null;
      render({ preserveForm: true });
      break;
    case "open-match-lineups": {
      const matchId = button.dataset.matchId ?? "";
      lineupsFeature.actions.resetPairedBuilderForMatch(matchId, false);
      workspaceState.patchModule("lineups", { active_section: "builder" });
      await runBusy(lineupsFeature.actions.loadBothPairedLineupSides);
      render({ preserveForm: true });
      break;
    }
    case "open-match-prediction": {
      const matchId = button.dataset.matchId ?? "";
      selectedP4MatchId = matchId;
      selectedPredictionSnapshot = lineupsFeature.state.pairedLineupBuilder.match_id === matchId
        ? lineupsFeature.state.pairedLineupBuilder.snapshot_type
        : "T-N";
      workspaceState.patchModule("prediction", { active_section: "formal" });
      await navigateTo("prediction");
      break;
    }
    case "create-match":
      await lineupsFeature.actions.createMatch();
      break;
    case "request-delete-match":
      lineupsFeature.actions.requestDeleteMatch(
        button.dataset.matchId ?? "",
        button.dataset.matchLabel ?? "未命名比赛",
      );
      break;
    case "reload-paired-lineup-side": {
      const side = button.dataset.lineupSide as LineupSide;
      const count = await runBusy(() => lineupsFeature.actions.loadPairedLineupSide(side));
      render();
      toast(`${lineupsFeature.actions.pairedSide(side).team_name}已加载 ${count} 名球员`, count > 0 ? "success" : "normal");
      break;
    }
    case "clear-paired-lineup-side":
      lineupsFeature.actions.clearPairedLineupSide(button.dataset.lineupSide as LineupSide);
      break;
    case "open-team-lineup-preset-manager":
      await lineupsFeature.actions.openTeamLineupPresetManager(
        button.dataset.teamId ?? teamsFeature.catalog.detail?.team.id ?? "",
        button.dataset.teamName ?? teamsFeature.catalog.detail?.team.canonical_name ?? "球队",
      );
      break;
    case "open-lineup-preset-manager":
      await lineupsFeature.actions.openLineupPresetManagerForSide(button.dataset.lineupSide as LineupSide);
      break;
    case "request-delete-team-lineup-preset":
      lineupsFeature.actions.requestDeleteTeamLineupPreset(
        button.dataset.presetId ?? "",
        button.dataset.presetName ?? "阵容预设",
        button.dataset.teamId ?? "",
        button.dataset.teamName ?? "球队",
        button.dataset.presetStatus ?? "active",
        Number(button.dataset.memberCount ?? 0),
      );
      break;
    case "open-team-lineup-preset-editor":
      lineupsFeature.actions.openTeamLineupPresetEditor(button.dataset.presetId || null);
      break;
    case "auto-assign-preset-formation":
      lineupsFeature.actions.assignPresetFormationSlots();
      break;
    case "save-team-lineup-preset":
      await lineupsFeature.actions.saveTeamLineupPreset(button.dataset.presetId || null);
      break;
    case "duplicate-team-lineup-preset":
      lineupsFeature.actions.openDuplicateLineupPreset(
        button.dataset.presetId ?? "",
        button.dataset.presetName ?? "阵容预设",
      );
      break;
    case "confirm-duplicate-lineup-preset":
      await lineupsFeature.actions.duplicateLineupPreset(button.dataset.presetId ?? "");
      break;
    case "archive-team-lineup-preset":
      lineupsFeature.actions.requestArchiveLineupPreset(
        button.dataset.presetId ?? "",
        button.dataset.presetName ?? "阵容预设",
      );
      break;
    case "save-current-lineup-as-preset":
      lineupsFeature.actions.openSaveCurrentLineupPreset(button.dataset.lineupSide as LineupSide);
      break;
    case "confirm-save-current-lineup-preset":
      await lineupsFeature.actions.saveCurrentLineupAsPreset(button.dataset.lineupSide as LineupSide);
      break;
    case "preview-apply-lineup-preset":
      await lineupsFeature.actions.previewApplyLineupPreset(button.dataset.lineupSide as LineupSide);
      break;
    case "confirm-apply-lineup-preset":
      await lineupsFeature.actions.applyLineupPreset(
        button.dataset.lineupSide as LineupSide,
        button.dataset.presetId ?? "",
      );
      break;
    case "create-lineup-pair":
      await lineupsFeature.actions.createPairedLineups();
      break;
    case "inspect-paired-lineup-chain":
      await lineupsFeature.actions.inspectPairedLineupChain();
      break;
    case "load-lineup-players":
      await lineupsFeature.actions.loadLineupPlayers();
      break;
    case "add-selected-lineup-player":
      lineupsFeature.actions.addSelectedPairedLineupPlayer(button.dataset.lineupSide as LineupSide);
      break;
    case "open-lineup-player-settings":
      lineupsFeature.actions.openPairedLineupPlayerSettings(
        button.dataset.lineupSide as LineupSide,
        button.dataset.playerId ?? "",
      );
      break;
    case "save-lineup-player-settings":
      lineupsFeature.actions.savePairedLineupPlayerSettings(
        button.dataset.lineupSide as LineupSide,
        button.dataset.playerId ?? "",
      );
      break;
    case "add-lineup-player": {
      const side = button.dataset.lineupSide as LineupSide | undefined;
      if (side) {
        lineupsFeature.actions.addPairedLineupPlayer(
          side,
          button.dataset.playerId ?? "",
          button.dataset.role === "starter",
        );
      } else {
        lineupsFeature.actions.addLineupPlayer(
          button.dataset.playerId ?? "",
          button.dataset.role === "starter",
        );
      }
      break;
    }
    case "remove-lineup-player": {
      const side = button.dataset.lineupSide as LineupSide | undefined;
      if (side) lineupsFeature.actions.removePairedLineupPlayer(side, button.dataset.playerId ?? "");
      else lineupsFeature.actions.removeLineupPlayer(button.dataset.playerId ?? "");
      break;
    }
    case "clear-lineup-builder":
      lineupsFeature.actions.captureLineupFormFromDom();
      lineupsFeature.state.lineupBuilderPlayers = [];
      render();
      break;
    case "inspect-lineup-chain":
      await lineupsFeature.actions.inspectLineupChain();
      break;
    case "create-lineup":
      await lineupsFeature.actions.createLineup();
      break;
    case "refresh-lineups":
      await runBusy(loadLineups);
      render();
      break;
    case "refresh-review":
      await runBusy(loadReviewCenter);
      render();
      break;
    case "load-review-match":
      await loadSelectedReviewMatch();
      break;
    case "export-match-review-package":
      await exportMatchReviewPackage();
      break;
    case "preview-match-review-package":
      await previewMatchReviewPackage();
      break;
    case "confirm-match-review-package":
      confirmMatchReviewPackage();
      break;
    case "commit-match-review-package-facts":
      commitMatchReviewPackageFacts();
      break;
    case "generate-match-review-from-package":
      await generateMatchReviewFromPackage();
      break;
    case "show-match-review-package-json":
      if (matchReviewPackagePreview) showModal("赛后复盘资料包预检", matchReviewPackagePreview);
      break;
    case "generate-match-review":
      await generateMatchReview();
      break;
    case "open-match-review":
      await openMatchReview(button.dataset.reviewId ?? "");
      break;
    case "decide-ability-candidate":
      await decideAbilityCandidate(
        button.dataset.candidateId ?? "",
        button.dataset.decision === "accept" ? "accept" : "reject",
      );
      break;
    case "show-review-json":
      if (selectedMatchReview) showModal("复盘完整详情", selectedMatchReview);
      break;
    case "inspect-postmatch-readiness":
      await inspectPostmatchReadiness(button.dataset.reviewId ?? "");
      break;
    case "settle-postmatch-review":
      settlePostmatchReview(button.dataset.reviewId ?? "");
      break;
    case "show-candidate-json": {
      const candidate =
        selectedMatchReview?.ability_candidates.find(
          (item) => item.id === button.dataset.candidateId,
        ) ??
        analysisAbilityCandidates.find(
          (item) => item.id === button.dataset.candidateId,
        );
      if (candidate) showModal("能力更新证据", candidate);
      break;
    }
    case "refresh-analysis-page":
      await refreshAnalysisPage();
      break;
    case "refresh-postmatch-monitoring":
      await refreshPostmatchMonitoring();
      break;
    case "prepare-evidence-decision":
      prepareEvidenceDecision(button.dataset.evidenceItemId ?? "");
      break;
    case "submit-evidence-decision":
      await submitEvidenceDecision(button.dataset.evidenceItemId ?? "");
      break;
    case "generate-parameter-tuning":
      if (!analysisReviewGateReady()) throw new Error("请先通过数据质量门禁并处理全部待审核建议");
      await generateParameterTuning();
      break;
    case "check-parameter-lifecycle-readiness":
      await checkParameterLifecycleReadiness();
      break;
    case "run-parameter-shadow-validation":
      await runParameterShadowValidation(button.dataset.candidateId ?? "");
      break;
    case "promote-parameter-candidate":
      promoteParameterCandidate(button.dataset.candidateId ?? "");
      break;
    case "rollback-parameter-candidate":
      rollbackParameterCandidate(button.dataset.candidateId ?? "");
      break;
    case "show-parameter-lifecycle-history":
      await showParameterLifecycleHistory(button.dataset.candidateId ?? "");
      break;
    case "decide-parameter-tuning":
      await decideParameterTuning(
        button.dataset.candidateId ?? "",
        button.dataset.decision === "accept_for_backtest"
          ? "accept_for_backtest"
          : "reject",
      );
      break;
    case "show-parameter-tuning": {
      const candidate = parameterTuningCandidates.find(
        (item) => item.id === button.dataset.candidateId,
      );
      if (candidate) showModal("参数候选诊断", candidate);
      break;
    }
    case "run-full-analysis":
      if (!analysisHistoryReady()) throw new Error("请先在赛后复盘中完成至少一场正式结算");
      await enqueueAnalysisTask("full_analysis_refresh");
      break;
    case "run-quality-scan":
      if (!fullAnalysisReady()) throw new Error("请先完成包含有效样本的完整分析");
      await enqueueAnalysisTask("data_quality_scan");
      break;
    case "refresh-analysis-jobs":
      await refreshAnalysisPage();
      break;
    case "cancel-analysis-job":
      await runBusy(() => api.cancelBackgroundJob(button.dataset.jobId ?? ""));
      await loadAnalysisCenter();
      render();
      toast("已请求取消任务", "success");
      break;
    case "retry-analysis-job":
      await runBusy(() => api.retryBackgroundJob(button.dataset.jobId ?? ""));
      await loadAnalysisCenter();
      render();
      toast("任务已重新排队", "success");
      break;
    case "show-job-json": {
      const job = analysisJobs.find((item) => item.id === button.dataset.jobId);
      if (job) showModal("后台任务详情", job);
      break;
    }
    case "export-ai-analysis-package":
      if (!analysisQualityReady()) throw new Error("请先完成数据质量扫描并清除全部严重问题");
      await exportAiAnalysisPackage();
      break;
    case "export-ai-response-template":
      if (!lastAiAnalysisPackageId) throw new Error("请先导出本轮智能分析资料包");
      await exportAiAnalysisResponseTemplate();
      break;
    case "preview-ai-analysis-response":
      if (!lastAiAnalysisPackageId) throw new Error("请先导出分析资料并生成对应建议模板");
      await previewAiAnalysisResponse();
      break;
    case "import-ai-analysis-response":
      await importAiAnalysisResponse();
      break;
    case "show-ai-response-json":
      if (aiAnalysisResponsePreview)
        showModal("分析建议检查结果", aiAnalysisResponsePreview);
      break;
    case "decide-ai-suggestion":
      await decideAiSuggestion(
        button.dataset.suggestionId ?? "",
        button.dataset.decision === "accept" ? "accept" : "reject",
      );
      break;
    case "show-ai-suggestion-json": {
      const suggestion = aiAnalysisSuggestions.find(
        (item) => item.id === button.dataset.suggestionId,
      );
      if (suggestion) showModal("智能分析建议依据", suggestion);
      break;
    }
    case "decide-quality-finding":
      await decideDataQualityFinding(
        button.dataset.findingId ?? "",
        button.dataset.decision === "ignore" ? "ignore" : "resolve",
      );
      break;
    case "show-quality-json": {
      const finding = analyticsOverview?.data_quality.findings.find(
        (item) => item.id === button.dataset.findingId,
      );
      if (finding) showModal("数据质量详情", finding);
      break;
    }
    case "open-lineup":
      showModal(
        "阵容详情",
        await runBusy(() => api.readLineup(button.dataset.lineupId ?? "")),
      );
      break;
    case "refresh-runs":
      if (!session.bootstrap) return;
      await runBusy(() => session.refreshRecentRuns());
      render();
      break;
    case "request-remove-lineup-history":
      lineupsFeature.actions.requestRemoveLineupHistory(
        button.dataset.lineupId ?? "",
        button.dataset.lineupLabel ?? "未命名阵容",
      );
      break;
    case "request-hide-run-history":
      requestHideRunHistory(
        button.dataset.runId ?? "",
        button.dataset.runLabel ?? "未命名推演",
      );
      break;
    case "open-run":
      showPredictionDetail(
        "历史推演结果",
        await runBusy(() => api.readRun(button.dataset.runId ?? "")),
      );
      break;
    case "show-route-json": {
      if (rulesFeature.route) { showRouteDetail(rulesFeature.route); break; }
      if (lastPredictionResult === null) break;
      const root = objectValue(lastPredictionResult);
      const route =
        Object.keys(objectValue(root.route)).length > 0
          ? objectValue(root.route)
          : root;
      if (
        typeof route.source === "string" &&
        typeof route.package_display_name === "string"
      ) {
        showRouteDetail(route as unknown as RouteDecision);
      }
      break;
    }
    case "show-last-result":
      if (lastPredictionResult !== null)
        showPredictionDetail("推演完整结果", lastPredictionResult);
      break;
    case "confirm-workspace-action":
      await modal.runPendingAction();
      break;
    case "close-workspace-detail":
      teamsFeature.deletion.cancelForce();
      closeModal();
      break;
  }
}

function closeAppContextMenu(): void { contextMenu.close(); }
function showAppContextMenu(event: MouseEvent,target: HTMLElement): void { contextMenu.open(event,target); }

document.addEventListener("contextmenu", (event) => {
  const target = (event.target as HTMLElement).closest<HTMLElement>("[data-context-kind]");
  if (!target) return;
  showAppContextMenu(event, target);
}, { signal: browserLifecycleController.signal });

document.addEventListener("keydown", (event) => {
  if (event.key !== "Enter" || !(event.target instanceof HTMLInputElement)) return;
  const action =
    event.target.id === "team-search"
      ? "search-teams"
      : event.target.id === "player-search"
        ? "search-players"
        : null;
  if (!action) return;
  event.preventDefault();
  document.querySelector<HTMLButtonElement>(`[data-action="${action}"]`)?.click();
}, { signal: browserLifecycleController.signal });


document.addEventListener("click", (event) => {
  const target = event.target as HTMLElement;
  if (shellController?.hasConfirmation) return;
  if (target.closest("[data-shell-controls]")) return;
  if (target.closest("button:disabled, [inert], [aria-disabled='true']")) return;
  const summary = target.closest<HTMLElement>("[data-validation-summary]");
  const formRegion = summary?.closest<HTMLElement>(".inline-setup-panel");
  if (summary && formRegion && focusValidationTarget(event, summary, formRegion)) return;
  if (!target.closest("#app-context-menu")) closeAppContextMenu();
  if (selectRulesDirectoryLevel(target)) return;
  const pageButton = target.closest<HTMLElement>("[data-page]");
  if (pageButton?.dataset.page) {
    const targetPage = pageButton.dataset.page;
    if (isRoute(targetPage)) activatePage(targetPage);
    return;
  }
  const actionButton = target.closest<HTMLElement>("[data-action]");
  if (!actionButton?.dataset.action) return;
  dispatchAction(actionButton.dataset.action, actionButton);
}, { signal: browserLifecycleController.signal });

function activatePage(targetPage: Page): void {
  const resetPlayerSource = targetPage === "players" && playersFeature.directEntry();
  if (targetPage === navigation.page && !resetPlayerSource) return;
  const navigationTraceId = recordUiOperation("navigation", `page:${targetPage}`, { fromPage: navigation.page, toPage: targetPage });
  void navigateTo(targetPage).catch((error: unknown) => {
    recordUiOperationFailure(`page:${targetPage}`, error, navigationTraceId, { fromPage: navigation.page, toPage: targetPage });
    recordClientIssue(error, `打开页面：${pageTitle(targetPage)}`);
    toast(userFacingError(error), "error");
  });
}

function dispatchAction(action: string, actionButton: HTMLElement): void {
  const actionTraceId = recordUiOperation("ui_action", `action:${action}`, {
    page: navigation.page,
    dataset: { ...actionButton.dataset },
  });
  void handleAction(action, actionButton).catch((error: unknown) => {
    recordUiOperationFailure(`action:${action}`, error, actionTraceId, { page: navigation.page });
    recordClientIssue(error, `${pageTitle(navigation.page)} / ${action}`);
    toast(userFacingError(error), "error");
  });
}

function filterSelectOptions(
  selectId: string,
  dataKey: string,
  expected: string | null,
): void {
  const select = document.querySelector<HTMLSelectElement>(`#${selectId}`);
  if (!select) return;
  for (const option of Array.from(select.options)) {
    if (!option.value) {
      option.hidden = false;
      continue;
    }
    option.hidden = expected !== null && option.dataset[dataKey] !== expected;
  }
  if (select.selectedOptions[0]?.hidden) select.value = "";
}

document.addEventListener("input", (event) => {
  const target = event.target as HTMLInputElement | HTMLTextAreaElement;
  if (["competition-id", "season-id", "stage-id", "competition-kind", "simulation-model-family", "simple-kickoff"].includes(target.id) || target.id.startsWith("binding-")) rulesFeature.invalidatePreview();
  if (target.id === "openai-api-example") {
    scheduleOpenAiApiExampleParse();
    return;
  }
  if (
    target.id === "api-workspace-message" &&
    target instanceof HTMLTextAreaElement
  ) {
    apiWorkspaceDraftMessage = target.value;
    return;
  }
  if (
    target.id === "api-workspace-session-search" &&
    target instanceof HTMLInputElement
  ) {
    apiWorkspaceSessionSearch = target.value;
    render({ preserveForm: true });
    return;
  }
  if (target.id === "managed-match-search" && target instanceof HTMLInputElement) {
    lineupsFeature.actions.filterManagedMatchList(target.value);
    return;
  }
  if (target.id === "rules-competition-search" && target instanceof HTMLInputElement) {
    filterRulesCompetitionCatalogue();
  }
}, { signal: browserLifecycleController.signal });

document.addEventListener("change", (event) => {
  const target = event.target as HTMLInputElement | HTMLSelectElement;
  if (["competition-id", "season-id", "stage-id", "competition-kind", "simulation-model-family", "simple-kickoff"].includes(target.id) || target.id.startsWith("binding-")) rulesFeature.invalidatePreview();
  if (
    target instanceof HTMLInputElement &&
    target.classList.contains("player-select-checkbox")
  ) {
    const playerId = target.dataset.playerId;
    if (playerId)
      target.checked
        ? playersFeature.catalog.toggleSelection(playerId, true)
        : playersFeature.catalog.toggleSelection(playerId, false);
    persistWorkspaceSelection("players");
    render({ preserveForm: true });
    return;
  }
  if (target instanceof HTMLInputElement && target.id === "player-select-all") {
    for (const item of playersFeature.catalog.list?.items ?? [])
      target.checked
        ? playersFeature.catalog.toggleSelection(item.id, true)
        : playersFeature.catalog.toggleSelection(item.id, false);
    persistWorkspaceSelection("players");
    render({ preserveForm: true });
    return;
  }
  if (
    target instanceof HTMLInputElement &&
    target.classList.contains("team-select-checkbox")
  ) {
    const teamId = target.dataset.teamId;
    if (teamId)
      teamsFeature.catalog.toggleSelection(teamId, target.checked);
    persistWorkspaceSelection("teams");
    render({ preserveForm: true });
    return;
  }
  if (target instanceof HTMLInputElement && target.id === "team-select-all") {
    teamsFeature.catalog.selectPage(target.checked);
    persistWorkspaceSelection("teams");
    render({ preserveForm: true });
    return;
  }
  if (target.id === "openai-api-protocol") {
    void parseOpenAiApiExampleNow();
    return;
  }
  if (
    target.id === "api-workspace-preset" &&
    target instanceof HTMLSelectElement
  ) {
    selectedApiWorkspacePresetKey = target.value;
    const preset = apiWorkspacePresets.find(
      (item) => item.key === target.value,
    );
    if (preset?.requires_match && !selectedApiWorkspaceMatchId) {
      selectedApiWorkspaceMatchId = apiWorkspaceMatches[0]?.id ?? null;
    }
    render({ preserveForm: true });
    return;
  }
  if (
    target.id === "api-workspace-profile" &&
    target instanceof HTMLSelectElement
  ) {
    selectedApiWorkspaceProfileId = target.value;
    return;
  }
  if (
    target.id === "api-workspace-match" &&
    target instanceof HTMLSelectElement
  ) {
    selectedApiWorkspaceMatchId = target.value || null;
    if (!selectedApiWorkspaceMatchId && !selectedApiWorkspaceContextEntityId)
      apiWorkspaceIncludeContext = false;
    render({ preserveForm: true });
    return;
  }
  if (
    target.id === "api-workspace-include-context" &&
    target instanceof HTMLInputElement
  ) {
    apiWorkspaceIncludeContext = target.checked;
    return;
  }
  if (target instanceof HTMLInputElement && target.classList.contains("preset-member-enabled")) {
    const row = target.closest<HTMLElement>("[data-preset-player-id]");
    if (row) lineupsFeature.actions.syncPresetMemberRowState(row);
    lineupsFeature.actions.updatePresetEditorSummary();
    return;
  }
  if (target instanceof HTMLSelectElement && target.id === "lineup-preset-formation") {
    lineupsFeature.actions.refreshPresetPositionOptions(true);
    return;
  }
  if (target instanceof HTMLSelectElement && target.classList.contains("preset-member-role")) {
    const row = target.closest<HTMLElement>("[data-preset-player-id]");
    if (row) lineupsFeature.actions.syncPresetMemberRowState(row);
    lineupsFeature.actions.updatePresetEditorSummary();
    return;
  }
  if (target instanceof HTMLSelectElement && target.classList.contains("preset-member-position")) {
    const row = target.closest<HTMLElement>("[data-preset-player-id]");
    if (row) lineupsFeature.actions.refreshPresetTacticalRoleSelect(row);
    lineupsFeature.actions.updatePresetEditorSummary();
    return;
  }
  if (target instanceof HTMLSelectElement && target.classList.contains("preset-member-tactical-role")) {
    lineupsFeature.actions.updatePresetEditorSummary();
    return;
  }
  if (target.id === "rule-package-file" && target instanceof HTMLInputElement) {
    const file = target.files?.[0];
    if (file) void rulesFeature.readPackage(file).catch((error: unknown) => {
      recordClientIssue(error, "赛事设置 / 读取规则包文件");
      toast(userFacingError(error), "error");
    });
    return;
  }
  if (target.id === "new-match-kickoff" && target instanceof HTMLInputElement) {
    lineupsFeature.actions.autoSelectMatchSeason();
    lineupsFeature.actions.filterMatchTeamOptions();
    return;
  }
  if (!(target instanceof HTMLSelectElement)) return;
  if (target.id === "spreadsheet-import-mode") { playersFeature.changeImportMode(); return; }
  if (target.id === "team-package-import-mode" || target.id === "team-spreadsheet-import-mode") {
    teamsFeature.changeImportMode(target.id === "team-package-import-mode" ? "package" : "monthly");
    return;
  }
  if (target.matches("[data-rules-region-select]")) {
    selectCatalogueRegion(target);
    filterRulesCompetitionCatalogue();
    return;
  }
  if (target.id === "rules-competition-kind") {
    filterRulesCompetitionCatalogue();
    return;
  }
  if (target.id === "prediction-match-id") {
    const matchId = target.value;
    if (!matchId) return;
    selectedP4MatchId = matchId;
    lineupsFeature.state.selectedMatchLineupChain = null;
    selectedPredictionReadiness = null;
    void runBusy(() => loadP4MatchWorkspace(matchId))
      .then(() => {
        render({ preserveForm: true });
      })
      .catch((error: unknown) => {
        recordClientIssue(error, "赛事推演 / 加载单场研究工作台");
        toast(userFacingError(error), "error");
      });
    return;
  }
  if (target.id === "prediction-stored-snapshot") {
    selectedPredictionSnapshot = target.value as LineupSnapshotType;
    lineupsFeature.state.selectedMatchLineupChain = null;
    selectedPredictionReadiness = null;
    render({ preserveForm: true });
    return;
  }
  if (target.id === "prediction-model-family") {
    selectedPredictionModelFamily = target.value as PredictionModelFamily;
    lineupsFeature.state.selectedMatchLineupChain = null;
    selectedPredictionReadiness = null;
    render({ preserveForm: true });
    return;
  }
  if (target.id === "explicit-rule-package-id") {
    selectedPredictionReadiness = null;
    render({ preserveForm: true });
    return;
  }
  if (target.id === "p4-workbench-match-id") {
    const matchId = target.value;
    if (!matchId) return;
    void runBusy(() => loadP4MatchWorkspace(matchId))
      .then(() => {
        render({ preserveForm: true });
      })
      .catch((error: unknown) => {
        recordClientIssue(error, "赛事推演 / 切换单场研究历史");
        toast(userFacingError(error), "error");
      });
    return;
  }
  if (target.id === "competition-id") {
    const selected = target.selectedOptions[0];
    const kind = selected?.dataset.kind;
    const kindSelect =
      document.querySelector<HTMLSelectElement>("#competition-kind");
    if (kind && kindSelect) kindSelect.value = kind;
    rulesFeature.syncHierarchy(app, target.id);
    refreshSearchableSelects(app);
  }
  if (target.id === "season-id") {
    rulesFeature.syncHierarchy(app, target.id);
    refreshSearchableSelects(app);
  }
  if (target.id === "stage-id") {
    const kind = target.selectedOptions[0]?.dataset.kind;
    const kindSelect =
      document.querySelector<HTMLSelectElement>("#competition-kind");
    if (kind && kindSelect) kindSelect.value = kind;
  }
  if (target.id.startsWith("binding-")) {
    rulesFeature.invalidatePreview();
    rulesFeature.syncHierarchy(app, target.id);
    refreshSearchableSelects(app);
  }
  if (target.id === "new-match-competition-scope") {
    lineupsFeature.actions.updateCompetitionHierarchy("scope");
    return;
  }
  if (target.id === "new-match-competition-region") {
    lineupsFeature.actions.updateCompetitionHierarchy("region");
    return;
  }
  if (target.id === "new-match-competition") {
    lineupsFeature.actions.updateCompetitionHierarchy("competition");
    filterSelectOptions(
      "new-match-season",
      "competitionId",
      target.value || null,
    );
    filterSelectOptions(
      "new-match-stage",
      "competitionId",
      target.value || null,
    );
    const scope = document.querySelector<HTMLSelectElement>("#new-match-team-scope");
    if (scope?.value === "auto") scope.value = "auto";
    lineupsFeature.actions.autoSelectMatchSeason();
    lineupsFeature.actions.filterMatchTeamOptions();
    return;
  }
  if (target.id === "new-match-season") {
    filterSelectOptions("new-match-stage", "seasonId", target.value || null);
    lineupsFeature.actions.filterMatchTeamOptions();
    return;
  }
  if (target.id === "new-match-stage") {
    filterSelectOptions("new-match-round", "stageId", target.value || null);
    return;
  }
  if (target.id === "new-match-team-scope") {
    lineupsFeature.actions.filterMatchTeamOptions();
    return;
  }
  if (target.id === "paired-home-formation-level1" || target.id === "paired-away-formation-level1") {
    lineupsFeature.actions.updateFormationHierarchy(target.id.includes("home") ? "home" : "away", "level1");
    return;
  }
  if (target.id === "paired-home-formation-level2" || target.id === "paired-away-formation-level2") {
    lineupsFeature.actions.updateFormationHierarchy(target.id.includes("home") ? "home" : "away", "level2");
    return;
  }
  if (target.id === "paired-home-formation-id" || target.id === "paired-away-formation-id") {
    lineupsFeature.actions.updateFormationHierarchy(target.id.includes("home") ? "home" : "away", "formation");
    return;
  }
  if (target.id === "paired-lineup-match") {
    if (!target.value) return;
    lineupsFeature.actions.resetPairedBuilderForMatch(target.value, true);
    void runBusy(lineupsFeature.actions.loadBothPairedLineupSides)
      .then(([homeCount, awayCount]) => {
        render();
        toast(`双方名单已加载：主队 ${homeCount} 人，客队 ${awayCount} 人`, "success");
      })
      .catch((error: unknown) => {
        recordClientIssue(error, "阵容编排 / 加载双方名单");
        toast(userFacingError(error), "error");
      });
    return;
  }
  if (["paired-lineup-type", "paired-lineup-snapshot", "paired-home-coach", "paired-away-coach", "paired-home-quality", "paired-away-quality"].includes(target.id)) {
    lineupsFeature.actions.capturePairedLineupFromDom();
    lineupsFeature.state.selectedMatchLineupChain = null;
    return;
  }
  if (target.id === "new-lineup-match") {
    const selected = target.selectedOptions[0];
    lineupsFeature.actions.captureLineupFormFromDom({ allowBlankIdentity: true });
    lineupsFeature.state.lineupBuilderForm = {
      ...lineupsFeature.state.lineupBuilderForm,
      match_id: target.value,
      team_id: selected?.dataset.homeTeam ?? "",
    };
    lineupsFeature.state.lineupBuilderPlayers = [];
    lineupsFeature.state.lineupPlayerCandidates = [];
    lineupsFeature.state.lineupPlayerLoadSequence.invalidate();
    const teamId = lineupsFeature.state.lineupBuilderForm.team_id;
    render();
    if (teamId) void lineupsFeature.actions.autoLoadLineupPlayers(teamId);
    return;
  }
  if (["new-lineup-team", "new-lineup-type", "new-lineup-snapshot", "new-lineup-formation-id", "new-lineup-coach"].includes(target.id)) {
    const previousTeam = lineupsFeature.state.lineupBuilderForm.team_id;
    lineupsFeature.actions.captureLineupFormFromDom({
      allowBlankIdentity: target.id === "new-lineup-team",
    });
    if (
      target.id === "new-lineup-team" &&
      previousTeam !== lineupsFeature.state.lineupBuilderForm.team_id
    ) {
      lineupsFeature.state.lineupBuilderPlayers = [];
      lineupsFeature.state.lineupPlayerCandidates = [];
      lineupsFeature.state.lineupPlayerLoadSequence.invalidate();
      const teamId = lineupsFeature.state.lineupBuilderForm.team_id;
      render();
      if (teamId) void lineupsFeature.actions.autoLoadLineupPlayers(teamId);
      return;
    }
  }
}, { signal: browserLifecycleController.signal });

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") closeModal();
}, { signal: browserLifecycleController.signal });

window.addEventListener("error", (event) => {
  recordClientIssue(
    event.error ?? event.message,
    "客户端未捕获异常",
    "critical",
  );
}, { signal: browserLifecycleController.signal });

window.addEventListener("unhandledrejection", (event) => {
  recordClientIssue(event.reason, "客户端未处理的异步异常", "critical");
}, { signal: browserLifecycleController.signal });

async function initializeApplication(): Promise<void> {
  await workspaceState.initialize();
  if (browserLifecycleController.signal.aborted) return;
  teamsFeature.catalog.replaceSelection(workspaceState.module("teams").selected_object_ids);
  playersFeature.catalog.replaceSelection(new Set(workspaceState.module("players").selected_object_ids));
  await session.start();
  if (browserLifecycleController.signal.aborted) return;
  render();
  if (sessionPreferences.getItem(DATABASE_RESET_COMPLETE_KEY) === "1") {
    sessionPreferences.removeItem(DATABASE_RESET_COMPLETE_KEY);
    toast("数据库已彻底清空并重建为空白状态", "success");
  }
}

window.addEventListener("beforeunload", () => {
  if (renderedPage) {
    const currentPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
    workspaceState.capture(renderedPage, currentPageRoot, true);
  }
  void workspaceState.flush().catch(error => recordClientIssue(error, "工作区 / 退出保存"));
}, { signal: browserLifecycleController.signal });

export interface BrowserApplicationModule {
  readonly name: "browser-application";
  start(): Promise<void>;
  destroy(): Promise<void>;
  stop(): void;
}

let browserApplicationCreated = false;

export function createBrowserApplicationModule(
  root: HTMLDivElement,
): BrowserApplicationModule {
  if (browserApplicationCreated) {
    throw new Error("浏览器应用模块只能由组合根创建一次");
  }
  browserApplicationCreated = true;
  app = root;
  warningCenter = new WarningCenterController(root, { navigate: target => activatePage(target) });
  shellController = new ApplicationShellController(root, { navigate: activatePage, action: dispatchAction }, {
    sidebarCollapsed: () => workspaceState.sidebarCollapsed(),
    setSidebarCollapsed: collapsed => workspaceState.setSidebarCollapsed(collapsed),
    toggleTheme: () => session.toggleTheme(),
    renderPreservingWorkspace: () => render({ preserveForm: true }),
    historyBack: () => modal.back(),
    historyForward: () => modal.forward(),
    resetWorkspace: resetCurrentWorkspace,
    retryPageRead: () => navigateTo(navigation.page),
    openLogs: () => navigation.page === "logs" ? navigateTo("logs") : activatePage("logs"),
  });
  const lifecycle = new MountHandle({
    closedMessage: "浏览器应用生命周期已结束",
    stop: stopResources,
    async start() {
      warningCenter?.start();
      shellController?.start();
      globalShortcuts.start();
      workspaceState.start(root, () => renderedPage, () => app.querySelector<HTMLElement>(".page-container"));
      bindSearchableSelectDiagnostics(browserLifecycleController.signal);
      render();
      try {
        await initializeApplication();
      } catch (error: unknown) {
        recordClientIssue(error, "平台启动", "critical");
        throw new Error(userFacingError(error), { cause: error });
      }
    },
    async destroy() {
      try { await workspaceState.destroy(); }
      finally { try { await session.destroy(); } finally { stopTauriTransport(); } }
    },
  });

  function stopResources(): void {
    if (renderedPage) {
      const currentPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
      workspaceState.capture(renderedPage, currentPageRoot, true);
    }
    warningCenter?.destroy();
    shellController?.destroy();
    browserLifecycleController.destroy();
    databaseFeature.stop();
    rulesFeature.stop();
    teamsFeature.stop();
  playersFeature.stop();
    session.stop();
    openAiExampleRequests.destroy();
    navigation.destroy();
    modal.reset();
    contextMenu.destroy();
    globalShortcuts.destroy();
    commandMenu.destroy();
    workflowNavigation.clear();
    workspaceState.stop();
    lineupsFeature.stop();
    openAiApiExampleTimer.destroy();
  }

  return { name: "browser-application", start: () => lifecycle.start(), destroy: () => lifecycle.destroy(), stop: () => lifecycle.stop() };
}

window.addEventListener("resize", () => fitCatalogueRegions(app), { signal: browserLifecycleController.signal });
