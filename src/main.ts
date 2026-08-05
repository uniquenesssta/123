import "./styles/app.css";
import "./styles/components.css";
import "./styles/layout.css";
import "./styles/entityCenter.css";
import "./styles/taskWorkspace.css";
import "./styles/coreWorkspaces.css";
import "./styles/moduleWorkspaces.css";
import "./styles/visualSystem.css";
import "./styles/workspacePanels.css";
import {
  api,
  issueWasLogged,
  recordUiOperation,
  recordUiOperationFailure,
} from "./api/client";
import { ModalController } from "./app/modal";
import { NavigationCoordinator } from "./app/navigation";
import { pageTitle, renderShell } from "./app/shell";
import { WorkspaceStateStore, type WorkspaceLayoutMode, type WorkspaceTabState } from "./app/viewState";
import { matchReviewWorkflowAllows } from "./app/matchReviewWorkflow";
import {
  fetchAnalysisCenter,
  fetchLineups,
  fetchPlayerCatalog,
  fetchTeamCatalog,
  fetchReviewCenter,
  fetchReviewLineups,
  type AnalysisCenterLoadResult,
  type LineupsLoadResult,
  type PlayerCatalogLoadResult,
  type TeamCatalogLoadResult,
  type ReviewCenterLoadResult,
} from "./controllers/pageLoaders";
import { competitionKindLabel } from "./components/competition";
import { escapeHtml, formatPercent } from "./components/format";
import { displayPlayerName, positionLabel } from "./components/footballText";
import { enhanceSearchableSelects, refreshSearchableSelects } from "./components/searchableSelect";
import { bindSearchableSelectDiagnostics } from "./diagnostics/searchableSelectDiagnostics";
import { architecturePage } from "./pages/architecture";
import { analyticsPage } from "./pages/analytics";
import { dashboardPage } from "./pages/dashboard";
import { databasePage } from "./pages/database";
import { predictionPage } from "./pages/prediction";
import { rulesPage } from "./pages/rules";
import { runsPage } from "./pages/runs";
import { playerTableRows, playersPage } from "./pages/players";
import { teamsPage } from "./pages/teams";
import { lineupPresetsPage } from "./pages/lineupPresets";
import { workbooksPage } from "./pages/workbooks";
import { lineupsPage } from "./pages/lineups";
import { logsPage } from "./pages/logs";
import { openAiPage } from "./pages/openai";
import { apiWorkspacePage } from "./pages/apiWorkspace";
import { reviewPage } from "./pages/review";
import { releasePage } from "./pages/release";
import type {
  BootstrapResponse,
  CompetitionBindingDraft,
  CompetitionDraft,
  CompetitionKind,
  DatabaseOptions,
  DataProviderDraft,
  ExternalEntityIdDraft,
  LineupDraft,
  LineupPairDraft,
  LineupPlayerDraft,
  LineupRecord,
  MatchDraft,
  MatchLineupChain,
  MatchPredictionReadiness,
  MatchRecord,
  MatchReviewDetail,
  MatchReviewDraft,
  MatchReviewSummary,
  MatchReviewPackagePreview,
  MatchReviewPackageWorkflowRecord,
  PostmatchSettlementRecord,
  PlayerMatchObservationDraft,
  ReviewableMatch,
  Page,
  PlayerAbilityObservationDraft,
  PlayerAvailabilityDraft,
  PlayerCatalogReferenceData,
  PlayerDetail,
  PlayerDraft,
  PlayerDynamicTagDraft,
  PlayerMatchContributionRequest,
  PlayerListPage,
  PlayerListQuery,
  PlayerNavigationContext,
  PlayerNameDraft,
  PlayerPositionDraft,
  PlayerTeamPeriodDraft,
  PredictionCommand,
  PredictionExecution,
  PredictionModelFamily,
  RouteDecision,
  RoutePreviewCommand,
  RoundDraft,
  RulePackageDraft,
  SeasonDraft,
  StageDraft,
  TeamDraft,
  TeamDetail,
  TeamListPage,
  TeamListQuery,
  TeamMatchLineupHistoryItem,
  TeamLineupPresetApplicationPreview,
  TeamLineupPresetDraft,
  TeamLineupPresetMemberDraft,
  TeamLineupPresetRecord,
  TeamProfileDraft,
  BulkDeleteResult,
  TeamForceDeletePreview,
  TeamForceDeleteResult,
  EntityDeletionCheck,
  BulkArchiveResult,
  CoachDraft,
  CoachListItem,
  TeamCoachPeriodDraft,
  FormationRecord,
  FormationUsageDistributionDraft,
  Theme,
  SpreadsheetImportMode,
  SpreadsheetImportPreview,
  TeamPackageImportPreview,
  AnalyticsOverview,
  BackgroundJob,
  AiAnalysisResponsePreview,
  AiAnalysisSuggestionRecord,
  AbilityUpdateCandidateRecord,
  EnqueueJobDraft,
  LineupBuilderPlayer,
  LineupBuilderFormState,
  LineupSnapshotType,
  PairedLineupBuilderState,
  PlayerListItem,
  ParameterTuningCandidateRecord,
  ParameterTuningDraft,
  ParameterTuningDecisionDraft,
  ParameterLifecycleReadinessRequest,
  PostmatchOverview,
  PostmatchMonitoringRequest,
  EvidenceScoringDecisionDraft,
  ParameterPromotionRequest,
  ParameterRollbackRequest,
  IssueLogEntry,
  IssueSeverity,
  OpenAiApiExampleParseResult,
  OpenAiApiProtocol,
  OpenAiProfileDraft,
  OpenAiProfilesState,
  P4MatchWorkspace,
  P4TaskWorkspace,
  ApiWorkspacePreset,
  ApiWorkspaceSessionDetail,
  ApiWorkspaceSessionRecord,
  ResolveP4ConflictCommand,
  ReleaseAcceptanceRequest,
  ReleaseAcceptanceRun,
  ReleaseAcceptanceRunSummary,
} from "./types";

const browserLifecycleController = new AbortController();

const THEME_KEY = "football-model-platform.theme";
const DATABASE_RESET_COMPLETE_KEY = "football-model-platform.database-reset-complete";
const ANALYSIS_PACKAGE_ID_KEY = "football-model-platform.last-analysis-package-id";
const TEAM_QUERY_KEY = "football-model-platform.team-query";
const PLAYER_QUERY_KEY = "football-model-platform.player-query";
const PLAYER_NAV_CONTEXT_KEY = "football-model-platform.player-navigation-context";
function readStoredObject<T>(key: string): Partial<T> | null {
  try {
    const raw = window.localStorage.getItem(key);
    if (!raw) return null;
    const value = JSON.parse(raw);
    return value && typeof value === "object" && !Array.isArray(value) ? value as Partial<T> : null;
  } catch {
    window.localStorage.removeItem(key);
    return null;
  }
}

function initialTeamQuery(): TeamListQuery {
  const stored = readStoredObject<TeamListQuery>(TEAM_QUERY_KEY);
  return {
    search: typeof stored?.search === "string" ? stored.search : null,
    country_code: typeof stored?.country_code === "string" ? stored.country_code : null,
    team_type: typeof stored?.team_type === "string" ? stored.team_type as TeamListQuery["team_type"] : null,
    active_only: typeof stored?.active_only === "boolean" ? stored.active_only : true,
    limit: typeof stored?.limit === "number" && stored.limit > 0 ? stored.limit : 50,
    cursor_name: null,
    cursor_id: null,
  };
}

function initialPlayerQuery(): PlayerListQuery {
  const stored = readStoredObject<PlayerListQuery>(PLAYER_QUERY_KEY);
  return {
    search: typeof stored?.search === "string" ? stored.search : null,
    team_id: typeof stored?.team_id === "string" ? stored.team_id : null,
    position_code: typeof stored?.position_code === "string" ? stored.position_code : null,
    availability_status: typeof stored?.availability_status === "string" ? stored.availability_status as PlayerListQuery["availability_status"] : null,
    player_status: typeof stored?.player_status === "string" ? stored.player_status as PlayerListQuery["player_status"] : "active",
    limit: typeof stored?.limit === "number" && stored.limit > 0 ? stored.limit : 50,
    cursor_name: null,
    cursor_id: null,
  };
}

function readPlayerNavigationContext(): PlayerNavigationContext | null {
  const stored = readStoredObject<PlayerNavigationContext>(PLAYER_NAV_CONTEXT_KEY);
  const source = stored?.source === "match_lineup" ? "match_lineup" : stored?.source === "team_roster" ? "team_roster" : null;
  if (!source || typeof stored?.team_id !== "string" || typeof stored.team_name !== "string") return null;
  const originPage = stored.origin_page === "lineups" ? "lineups" : "teams";
  const now = new Date().toISOString();
  return {
    source,
    team_id: stored.team_id,
    team_name: stored.team_name,
    player_id: typeof stored.player_id === "string" ? stored.player_id : null,
    origin_page: originPage,
    return_section: stored.return_section === "builder" || stored.return_section === "chain" ? stored.return_section : null,
    created_at: typeof stored.created_at === "string" ? stored.created_at : now,
    updated_at: typeof stored.updated_at === "string" ? stored.updated_at : now,
  };
}

function persistTeamQuery(): void {
  const { cursor_name: _cursorName, cursor_id: _cursorId, ...stable } = teamQuery;
  window.localStorage.setItem(TEAM_QUERY_KEY, JSON.stringify(stable));
}

function persistPlayerQuery(): void {
  const { cursor_name: _cursorName, cursor_id: _cursorId, ...stable } = playerQuery;
  window.localStorage.setItem(PLAYER_QUERY_KEY, JSON.stringify(stable));
}

function setPlayerNavigationContext(context: PlayerNavigationContext | null): void {
  playerNavigationContext = context;
  if (context) window.localStorage.setItem(PLAYER_NAV_CONTEXT_KEY, JSON.stringify(context));
  else window.localStorage.removeItem(PLAYER_NAV_CONTEXT_KEY);
}

function prepareDirectPlayerDirectoryEntry(): void {
  const sourceTeamId = playerNavigationContext?.team_id ?? null;
  if (!sourceTeamId) return;

  playerCursorHistory = [];
  if (playerQuery.team_id === sourceTeamId) {
    playerQuery = {
      ...playerQuery,
      team_id: null,
      cursor_name: null,
      cursor_id: null,
    };
    persistPlayerQuery();
  }
  setPlayerNavigationContext(null);
  selectedPlayer = null;
  workspaceState.patchModule("players", {
    active_section: "directory",
    inspector_collapsed: true,
  });
}

let app: HTMLDivElement;

let state: BootstrapResponse | null = null;
let page: Page = "dashboard";
let busy = false;
let lastPredictionResult: unknown = null;
let currentTheme: Theme = readInitialTheme();
let playerReferences: PlayerCatalogReferenceData | null = null;
let playerListPage: PlayerListPage | null = null;
let selectedPlayer: PlayerDetail | null = null;
let selectedPlayerIds = new Set<string>();
let teamListPage: TeamListPage | null = null;
let selectedTeam: TeamDetail | null = null;
let selectedTeamLineupHistory: TeamMatchLineupHistoryItem[] = [];
let selectedTeamLineupPresets: TeamLineupPresetRecord[] = [];
let pairedLineupPresets: Record<"home" | "away", TeamLineupPresetRecord[]> = {
  home: [],
  away: [],
};
let selectedTeamIds = new Set<string>();
let teamCursorHistory: Array<{ cursor_name: string | null; cursor_id: string | null }> = [];
let playerCursorHistory: Array<{ cursor_name: string | null; cursor_id: string | null }> = [];
let playerPageLoading = false;
let coachList: CoachListItem[] = [];
let formationCatalog: FormationRecord[] = [];
let teamQuery: TeamListQuery = initialTeamQuery();
let playerQuery: PlayerListQuery = initialPlayerQuery();
let playerNavigationContext: PlayerNavigationContext | null = readPlayerNavigationContext();
if (playerNavigationContext) playerQuery.team_id = playerNavigationContext.team_id;
let lineupRecords: LineupRecord[] = [];
let selectedMatchLineupChain: MatchLineupChain | null = null;
let selectedPredictionReadiness: MatchPredictionReadiness | null = null;
let spreadsheetPreview: SpreadsheetImportPreview | null = null;
let teamSpreadsheetPreview: SpreadsheetImportPreview | null = null;
let teamPackagePreview: TeamPackageImportPreview | null = null;
let pendingTeamForceDelete: TeamForceDeletePreview | null = null;
let matchSpreadsheetPreview: SpreadsheetImportPreview | null = null;
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
let lastAiAnalysisPackageId: string | null = window.localStorage.getItem(ANALYSIS_PACKAGE_ID_KEY);
let pendingRulePackage: RulePackageDraft | null = null;
let lineupPlayerCandidates: PlayerListItem[] = [];
let lineupBuilderPlayers: LineupBuilderPlayer[] = [];
let lineupPlayerLoadSequence = 0;
let lineupBuilderForm: LineupBuilderFormState = {
  match_id: "",
  team_id: "",
  lineup_type: "expected",
  snapshot_type: "T-N",
  formation_id: "4737da75-7c7b-52f5-acf5-ea9bfa809c48",
  formation: "4-2-3-1",
  coach_id: "",
  source_urls: "",
  captured_at: "",
  quality_score: 0.8,
};
type LineupSide = "home" | "away";

const DEFAULT_FORMATION_ID = "4737da75-7c7b-52f5-acf5-ea9bfa809c48";
const DEFAULT_FORMATION_CODE = "4-2-3-1";

function emptyPairedLineupSide(): PairedLineupBuilderState[LineupSide] {
  return {
    team_id: "",
    team_name: "",
    formation_id: DEFAULT_FORMATION_ID,
    formation: DEFAULT_FORMATION_CODE,
    coach_id: "",
    quality_score: 0.8,
    players: [],
    candidates: [],
  };
}

let pairedLineupBuilder: PairedLineupBuilderState = {
  match_id: "",
  lineup_type: "expected",
  snapshot_type: "T-N",
  captured_at: "",
  source_urls: "",
  home: emptyPairedLineupSide(),
  away: emptyPairedLineupSide(),
};
let pairedLineupLoadSequence: Record<LineupSide, number> = { home: 0, away: 0 };
let selectedManagedMatchId: string | null = null;

interface WorkflowContinuation {
  readonly returnPage: Page;
  readonly returnSection: string | null;
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
let openAiApiExampleTimer: number | null = null;
let openAiApiExampleSequence = 0;
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
applyTheme(currentTheme);

function readInitialTheme(): Theme {
  const stored = window.localStorage.getItem(THEME_KEY);
  if (stored === "light" || stored === "dark") return stored;
  return "light";
}

function applyTheme(theme: Theme): void {
  currentTheme = theme;
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme = theme;
}

function toggleTheme(): void {
  const next: Theme = currentTheme === "dark" ? "light" : "dark";
  applyTheme(next);
  window.localStorage.setItem(THEME_KEY, next);
  render({ preserveForm: true });
}

const navigation = new NavigationCoordinator<Page>();
const workspaceState = new WorkspaceStateStore<Page>({
  read: () => api.readWorkspaceState(),
  save: (document) => api.saveWorkspaceState(document),
  clear: () => api.clearWorkspaceState(),
});
const modal = new ModalController();
let navigationPendingPage: Page | null = null;
let renderedPage: Page | null = null;

interface RenderOptions {
  readonly preserveForm?: boolean;
}

function renderWorkflowContinuationBanner(): string {
  if (!workflowContinuation || page === workflowContinuation.returnPage) return "";
  return `<section class="workflow-continuation-banner" role="status"><div><span>正在补齐前置资料</span><strong>${escapeHtml(workflowContinuation.reason)}</strong><small>完成当前资料后可返回原工作流，比赛和时间窗口不会丢失。</small></div><div class="button-row"><button class="secondary" data-action="workflow-return">返回原任务</button><button class="ghost" data-action="workflow-cancel">取消返回</button></div></section>`;
}

function workflowSection(pageName: Page): string | null {
  return workspaceState.module(pageName).active_section ?? null;
}

async function startWorkflowCompletion(button: HTMLElement): Promise<void> {
  const targetPage = button.dataset.targetPage as Page | undefined;
  if (!targetPage) throw new Error("缺少补录目标页面");
  workflowContinuation = {
    returnPage: page,
    returnSection: workflowSection(page),
    reason: button.dataset.returnReason ?? "补齐缺失资料后返回",
    matchId: pairedLineupBuilder.match_id || selectedManagedMatchId,
    snapshotType: pairedLineupBuilder.snapshot_type,
  };
  const targetSection = button.dataset.targetSection ?? null;
  if (targetSection) workspaceState.patchModule(targetPage, { active_section: targetSection });
  await navigateTo(targetPage);
}

async function returnToWorkflow(): Promise<void> {
  const continuation = workflowContinuation;
  if (!continuation) return;
  workflowContinuation = null;
  if (continuation.returnSection) {
    workspaceState.patchModule(continuation.returnPage, {
      active_section: continuation.returnSection,
    });
  }
  if (continuation.matchId) {
    selectedManagedMatchId = continuation.matchId;
    selectedP4MatchId = continuation.matchId;
    pairedLineupBuilder = {
      ...pairedLineupBuilder,
      match_id: continuation.matchId,
      snapshot_type: continuation.snapshotType ?? pairedLineupBuilder.snapshot_type,
    };
  }
  await navigateTo(continuation.returnPage);
  if (continuation.returnPage === "lineups" && continuation.matchId) {
    resetPairedBuilderForMatch(continuation.matchId, false);
    await loadBothPairedLineupSides();
    render({ preserveForm: true });
  }
}

function render(options: RenderOptions = {}): void {
  if (options.preserveForm === true && renderedPage === "lineups") {
    capturePairedLineupFromDom();
  }
  if (options.preserveForm === true && renderedPage === "review") {
    captureLineupFormFromDom();
  }
  if (!state) {
    app.innerHTML = `<div class="boot-screen"><div class="spinner"></div><strong>正在启动平台</strong></div>`;
    renderedPage = null;
    return;
  }
  if (renderedPage) {
    const previousPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
    workspaceState.capture(renderedPage, previousPageRoot, options.preserveForm === true);
  }

  let content = "";
  switch (page) {
    case "dashboard":
      content = dashboardPage(state);
      break;
    case "database":
      content = databasePage(state);
      break;
    case "rules": {
      const workspace = workspaceState.module("rules");
      content = rulesPage(state, pendingRulePackage, workspace.active_section ?? "catalog");
      break;
    }
    case "players": {
      const workspace = workspaceState.module("players");
      content = playersPage(
        state, playerReferences, playerListPage, selectedPlayer, playerQuery, spreadsheetPreview, selectedPlayerIds,
        workspace.tabs, workspace.active_tab_id, workspace.layout_mode, workspace.module_sidebar_collapsed, workspace.inspector_collapsed,
        workspace.active_section ?? "directory", playerCursorHistory.length + 1, playerNavigationContext,
      );
      break;
    }
    case "teams": {
      const workspace = workspaceState.module("teams");
      content = teamsPage(
        state, teamListPage, selectedTeam, selectedPlayer, teamQuery, selectedTeamIds, coachList, formationCatalog,
        teamPackagePreview, selectedTeamLineupHistory, selectedTeamLineupPresets, workspace.tabs, workspace.active_tab_id,
        workspace.layout_mode, workspace.module_sidebar_collapsed, workspace.inspector_collapsed,
        workspace.active_section ?? "directory", teamCursorHistory.length + 1,
      );
      break;
    }
    case "lineup_presets": {
      content = lineupPresetsPage(
        state, teamListPage, selectedTeam, selectedTeamLineupPresets, teamQuery, teamCursorHistory.length + 1,
      );
      break;
    }
    case "workbooks": {
      const workspace = workspaceState.module("workbooks");
      content = workbooksPage(state, teamSpreadsheetPreview, spreadsheetPreview, matchSpreadsheetPreview, workspace.module_sidebar_collapsed, workspace.inspector_collapsed, workspace.active_section ?? "team");
      break;
    }
    case "lineups": {
      const workspace = workspaceState.module("lineups");
      content = lineupsPage(
        state,
        playerReferences,
        lineupRecords,
        matchSpreadsheetPreview,
        pairedLineupBuilder,
        pairedLineupPresets,
        coachList,
        selectedMatchLineupChain,
        selectedManagedMatchId,
        workspace.module_sidebar_collapsed,
        workspace.inspector_collapsed,
        workspace.active_section ?? "matches",
      );
      break;
    }
    case "prediction": {
      const workspace = workspaceState.module("prediction");
      content = predictionPage(
        state,
        playerReferences,
        selectedP4MatchId,
        p4MatchWorkspace,
        p4TaskWorkspace,
        selectedMatchLineupChain,
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
        state,
        reviewableMatches,
        selectedReviewMatchId,
        reviewLineups,
        selectedMatchReview,
        matchReviewPackagePreview,
        matchReviewPackageWorkflow,
        selectedReviewSettlement,
        recentMatchReviews,
        playerReferences,
        lineupPlayerCandidates,
        lineupBuilderPlayers,
        lineupBuilderForm,
        Number.isInteger(storedStep) && storedStep >= 1 && storedStep <= 9 ? storedStep : null,
      );
      break;
    }
    case "analytics":
      content = analyticsPage(
        state,
        analyticsOverview,
        analysisJobs,
        aiAnalysisResponsePreview,
        aiAnalysisSuggestions,
        analysisAbilityCandidates,
        state.data.competitions,
        parameterTuningCandidates,
        postmatchOverview,
        lastAiAnalysisPackageId,
      );
      break;
    case "runs":
      content = runsPage(state);
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
        state,
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
        state,
        releaseAcceptanceRuns,
        selectedReleaseAcceptanceRun,
        workspace.module_sidebar_collapsed,
        workspace.inspector_collapsed,
        workspace.active_section ?? "overview",
      );
      break;
    }
    case "architecture":
      content = architecturePage();
      break;
  }
  content = renderWorkflowContinuationBanner() + content;
  app.innerHTML = renderShell({
    state,
    page,
    theme: currentTheme,
    content,
    busy,
    navigationPending: navigationPendingPage === page,
    sidebarCollapsed: workspaceState.sidebarCollapsed(),
  });
  modal.restore();
  renderedPage = page;
  const currentPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
  workspaceState.restore(page, currentPageRoot);
  if (page === "lineups") {
    queueMicrotask(() => {
      updateCompetitionHierarchy("init");
      autoSelectMatchSeason();
      filterMatchTeamOptions();
      updateFormationHierarchy("home", "init");
      updateFormationHierarchy("away", "init");
      enhanceSearchableSelects(currentPageRoot);
      refreshSearchableSelects(currentPageRoot);
    });
  }
}

async function refresh(): Promise<void> {
  state = await api.bootstrap();
  render();
}

function appendWorkspaceTab(targetPage: "teams" | "players", tab: WorkspaceTabState): void {
  const current = workspaceState.module(targetPage);
  const existing = current.tabs.filter((item) => item.id !== tab.id);
  const tabs = [...existing, tab].slice(-6);
  workspaceState.patchModule(targetPage, { tabs, active_tab_id: tab.id });
}

function persistWorkspaceSelection(targetPage: "teams" | "players"): void {
  const ids = targetPage === "teams" ? [...selectedTeamIds] : [...selectedPlayerIds];
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
    for (const id of removed) selectedTeamIds.delete(id);
    if (selectedTeam && removed.has(selectedTeam.team.id)) {
      selectedTeam = null;
      selectedTeamLineupHistory = [];
      selectedTeamLineupPresets = [];
    }
  } else {
    for (const id of removed) selectedPlayerIds.delete(id);
    if (selectedPlayer && removed.has(selectedPlayer.player.id)) selectedPlayer = null;
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
      if (targetPage === "teams") await openTeam(candidate);
      else await openPlayer(candidate);
      return;
    } catch (error) {
      if (!isMissingWorkspaceObjectError(targetPage, error)) throw error;
      removeWorkspaceObjects(targetPage, [candidate]);
    }
  }
  if (targetPage === "teams") {
    selectedTeam = null;
    selectedTeamLineupHistory = [];
    selectedTeamLineupPresets = [];
  } else {
    selectedPlayer = null;
  }
  render({ preserveForm: true });
}

async function activateWorkspaceTab(targetPage: "teams" | "players", objectId: string): Promise<void> {
  workspaceState.patchModule(targetPage, { active_tab_id: objectId });
  if (targetPage === "teams") await openTeam(objectId);
  else await openPlayer(objectId);
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
  const ids = targetPage === "teams" ? [...selectedTeamIds] : [...selectedPlayerIds];
  const items = targetPage === "teams" ? (teamListPage?.items ?? []) : (playerListPage?.items ?? []);
  for (const id of ids.slice(-6)) {
    const item = items.find((candidate) => candidate.id === id);
    appendWorkspaceTab(targetPage, { id, label: item?.canonical_name ?? id });
  }
  const active = ids.at(-1);
  if (!active) return;
  if (targetPage === "teams") await openTeam(active);
  else await openPlayer(active);
}

function setWorkspaceMode(targetPage: "teams" | "players", mode: WorkspaceLayoutMode): void {
  workspaceState.patchModule(targetPage, { layout_mode: mode });
  render({ preserveForm: true });
}

function toggleWorkspacePane(pane: "module-sidebar" | "inspector"): void {
  const current = workspaceState.module(page);
  const patch = pane === "module-sidebar"
    ? { module_sidebar_collapsed: !current.module_sidebar_collapsed }
    : { inspector_collapsed: !current.inspector_collapsed };
  workspaceState.patchModule(page, patch);
  render({ preserveForm: true });
}

function resetCurrentWorkspace(): void {
  workspaceState.clear(page);
  if (page === "teams") { selectedTeamIds.clear(); selectedTeam = null; selectedPlayer = null; selectedTeamLineupHistory = []; selectedTeamLineupPresets = []; teamCursorHistory = []; }
  if (page === "players") { selectedPlayerIds.clear(); selectedPlayer = null; playerCursorHistory = []; }
  if (page === "lineup_presets") { selectedTeam = null; selectedTeamLineupPresets = []; selectedTeamLineupHistory = []; teamCursorHistory = []; }
  render();
  toast("当前工作区已重置", "success");
}

function toast(
  message: string,
  kind: "normal" | "success" | "error" = "normal",
): void {
  const node = document.querySelector<HTMLDivElement>("#toast");
  if (!node) return;
  node.textContent = message;
  node.className = `toast visible ${kind}`;
  window.setTimeout(() => node.classList.remove("visible"), 3200);
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
  if (busy) throw new Error("已有任务正在执行");
  busy = true;
  document.querySelector("#busy")?.classList.add("visible");
  try {
    return await operation();
  } finally {
    busy = false;
    document.querySelector("#busy")?.classList.remove("visible");
  }
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
): void {
  modal.confirm(title, description, facts, confirmLabel, action);
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

function routeDetailBody(route: RouteDecision): string {
  return `<div class="route-detail">
    <div class="route-detail-summary"><span>系统判定</span><h3>${escapeHtml(route.package_display_name)}</h3><p>${escapeHtml(routeSourceLabel(route.source))}，用于${escapeHtml(competitionKindLabel(route.competition_profile.competition_kind))}。</p></div>
    <div class="model-basis-flow route-basis-flow">
      <div><span>1 · 识别赛事</span><strong>${escapeHtml(competitionKindLabel(route.competition_profile.competition_kind))}</strong></div><i>→</i>
      <div><span>2 · 匹配层级</span><strong>${escapeHtml(routeSourceLabel(route.source))}</strong></div><i>→</i>
      <div><span>3 · 选择规则</span><strong>${escapeHtml(route.package_display_name)}</strong></div><i>→</i>
      <div><span>4 · 运行模型</span><strong>${escapeHtml(modelDisplayName(route.model_id))}</strong></div>
    </div>
    <dl class="detail-facts route-readable-facts"><div><dt>模型版本</dt><dd>${escapeHtml(route.model_version)}</dd></div><div><dt>参数版本</dt><dd>${escapeHtml(route.parameter_version)}</dd></div><div><dt>规则包版本</dt><dd>${escapeHtml(route.package_version)}</dd></div><div><dt>最终判定</dt><dd>可直接开始推演</dd></div></dl>
    <details class="technical-details"><summary>技术追踪信息</summary><dl class="detail-facts compact"><div><dt>规则包编号</dt><dd>${escapeHtml(route.rule_package_id)}</dd></div><div><dt>绑定编号</dt><dd>${escapeHtml(route.binding_id ?? "默认匹配")}</dd></div><div><dt>模型版本编号</dt><dd>${escapeHtml(route.model_version_id)}</dd></div><div><dt>参数集编号</dt><dd>${escapeHtml(route.parameter_set_id)}</dd></div></dl></details>
  </div>`;
}

function showRouteDetail(route: RouteDecision): void {
  showHtmlModal("赛事规则与模型判定", "清晰模型链路", routeDetailBody(route));
}

function routeCard(route: RouteDecision): string {
  return `
    <div class="route-card">
      <div class="route-card-main"><span>${escapeHtml(routeSourceLabel(route.source))}</span><strong>${escapeHtml(route.package_display_name)}</strong><small>规则包版本 ${escapeHtml(route.package_version)}</small></div>
      <div class="route-chain"><div><span>赛事类型</span><b>${escapeHtml(competitionKindLabel(route.competition_profile.competition_kind))}</b></div><div><span>匹配方式</span><b>${escapeHtml(routeSourceLabel(route.source))}</b></div><div><span>模型</span><b>${escapeHtml(modelDisplayName(route.model_id))}</b></div><div><span>参数版本</span><b>${escapeHtml(route.parameter_version)}</b></div></div>
      <button class="secondary" data-action="show-route-json">查看完整判定</button>
    </div>`;
}

function showRouteDecision(route: RouteDecision): void {
  lastPredictionResult = route;
  const target = document.querySelector<HTMLElement>("#route-preview");
  if (!target) {
    showRouteDetail(route);
    return;
  }
  target.innerHTML = `<div class="panel-heading"><div><span>模型已确定</span><h2>模型选择说明</h2></div></div>${routeCard(route)}`;
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

async function connectDatabase(): Promise<void> {
  const options: DatabaseOptions = {
    connection_url: value("database-url").trim(),
    max_connections: Number(value("max-connections")),
    connect_timeout_seconds: Number(value("connect-timeout")),
  };
  if (!options.connection_url) throw new Error("请输入数据库连接地址");
  await runBusy(() => api.configureDatabase(options));
  await refresh();
  toast("数据服务已连接，本页功能已恢复", "success");
}

function requestDatabaseReset(): void {
  const databaseName = state?.data.database_health?.database_name;
  if (!databaseName) throw new Error("当前未连接数据库，无法执行彻底清空");
  const body = `
    <div class="confirm-visual">
      <div class="confirm-icon">!</div>
      <p>此操作会删除当前数据库中的全部应用数据，并从迁移文件重新建立空白结构。球队、球员、比赛、阵容、推演、P4 快照、导入记录、AI 会话、任务、审核和审计数据都无法恢复。</p>
      <div class="visual-grid">
        <div class="visual-field"><span>当前数据库</span><strong>${escapeHtml(databaseName)}</strong></div>
        <div class="visual-field"><span>清空后保留</span><strong>数据库本身、结构和本机连接配置</strong></div>
      </div>
      <div class="database-reset-confirmation">
        <label for="database-reset-confirmation">输入数据库名称 <code>${escapeHtml(databaseName)}</code> 以继续</label>
        <input id="database-reset-confirmation" type="text" autocomplete="off" spellcheck="false" data-database-name="${escapeHtml(databaseName)}" />
        <small>名称必须完全一致。清空期间客户端会暂时断开，完成后自动重新连接并重新启动界面。</small>
      </div>
    </div>`;
  const footer = `<button type="button" class="secondary" data-action="close-workspace-detail">取消</button><button type="button" class="primary danger-action" data-action="execute-database-reset" disabled>确认永久清空</button>`;
  showHtmlModal("彻底清空数据库", "不可撤销的危险操作", body, footer, "database-reset-modal");
  queueMicrotask(() => document.querySelector<HTMLInputElement>("#database-reset-confirmation")?.focus());
}

async function executeDatabaseReset(): Promise<void> {
  const input = document.querySelector<HTMLInputElement>("#database-reset-confirmation");
  const expected = input?.dataset.databaseName ?? "";
  const confirmation = input?.value.trim() ?? "";
  if (!expected || confirmation !== expected) {
    throw new Error("数据库名称不匹配，已拒绝清空");
  }
  closeModal();
  await runBusy(() => api.resetDatabase(confirmation));
  await api.clearWorkspaceState();
  window.localStorage.removeItem(TEAM_QUERY_KEY);
  window.localStorage.removeItem(PLAYER_QUERY_KEY);
  window.localStorage.removeItem(PLAYER_NAV_CONTEXT_KEY);
  window.sessionStorage.setItem(DATABASE_RESET_COMPLETE_KEY, "1");
  window.location.reload();
}

function syncSimpleMatchInput(): Record<string, unknown> {
  const base = state?.data.default_match;
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
  const result = await runBusy(() => api.previewRoute(command));
  showRouteDecision(result);
  toast("模型路由解析完成", "success");
}

async function executePrediction(): Promise<void> {
  const command: PredictionCommand = {
    match_input: syncSimpleMatchInput(),
    snapshot_type: value("snapshot-type"),
    ...routeSelection(),
  };
  const result = await runBusy(() => api.executePrediction(command));
  if (state) state.data.recent_runs = await api.listRecentRuns(100);
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

async function createCompetition(): Promise<void> {
  const draft: CompetitionDraft = {
    code: "",
    name: value("competition-name").trim(),
    country_code: nullableValue("competition-country"),
    timezone: value("competition-timezone").trim(),
    competition_kind: value("new-competition-kind") as CompetitionKind,
    metadata: {},
  };
  await runBusy(() => api.createCompetition(draft));
  await refresh();
  page = "rules";
  render();
  toast("赛事已创建", "success");
}

function showCompetitionPath(competitionId: string): void {
  if (!state) return;
  const competition = state.data.competitions.find(
    (item) => item.id === competitionId,
  );
  if (!competition) throw new Error("赛事不存在");
  const exactBindings = state.data.competition_bindings
    .filter((item) => item.is_active && item.competition_id === competitionId)
    .sort((left, right) => right.priority - left.priority);
  const defaultBindings = state.data.competition_bindings
    .filter(
      (item) =>
        item.is_active &&
        !item.competition_id &&
        item.competition_kind === competition.competition_kind,
    )
    .sort((left, right) => right.priority - left.priority);
  const selectedBinding = exactBindings[0] ?? defaultBindings[0] ?? null;
  const selectedPackage = selectedBinding
    ? (state.data.rule_packages.find(
        (item) => item.id === selectedBinding.rule_package_id,
      ) ?? null)
    : (state.data.rule_packages
        .filter(
          (item) =>
            item.competition_kind === competition.competition_kind &&
            item.status === "active",
        )
        .sort((left, right) => right.priority - left.priority)[0] ?? null);
  const sourceLabel =
    exactBindings.length > 0
      ? "具体赛事绑定"
      : defaultBindings.length > 0
        ? "赛事类型默认绑定"
        : selectedPackage
          ? "规则包类型默认"
          : "未匹配";
  const selectedModel =
    selectedPackage?.model_id ?? selectedBinding?.model_id ?? "未匹配";
  const body = `<div class="route-detail">
    <div class="route-detail-summary"><span>赛事规则路径</span><h3>${escapeHtml(competition.name)}</h3><p>系统按比赛所属赛事、赛季和阶段自动匹配，当前优先使用“${escapeHtml(sourceLabel)}”。</p></div>
    <div class="model-basis-flow route-basis-flow">
      <div><span>1 · 识别赛事</span><strong>${escapeHtml(competitionKindLabel(competition.competition_kind))}</strong><small>${escapeHtml(String(competition.metadata?.region ?? competition.country_code ?? "未分类"))}</small></div><i>→</i>
      <div><span>2 · 匹配范围</span><strong>${escapeHtml(sourceLabel)}</strong><small>具体比赛上下文会自动优先</small></div><i>→</i>
      <div><span>3 · 使用规则</span><strong>${escapeHtml(selectedPackage?.display_name ?? selectedBinding?.rule_package_name ?? "尚未配置")}</strong><small>${escapeHtml(selectedPackage?.version ?? "需要配置规则")}</small></div><i>→</i>
      <div><span>4 · 运行模型</span><strong>${escapeHtml(modelDisplayName(selectedModel))}</strong><small>${escapeHtml(selectedPackage ? `${selectedPackage.model_version} · 参数 ${selectedPackage.parameter_version}` : "暂无可执行路径")}</small></div>
    </div>
    <section class="prediction-detail-section"><div class="section-heading"><div><span>备用规则</span><h3>同类规则匹配顺序</h3></div><small>系统自动处理，不需要手工选择优先级</small></div>${[...exactBindings, ...defaultBindings].length ? `<div class="route-option-list">${[...exactBindings, ...defaultBindings].map((binding, index) => `<article><span>${index + 1}</span><div><strong>${escapeHtml(binding.rule_package_name)}</strong><small>${binding.competition_id ? "只用于当前赛事" : `用于全部${escapeHtml(competitionKindLabel(competition.competition_kind))}`}</small></div><b>${binding.id === selectedBinding?.id ? "当前采用" : "候选"}</b></article>`).join("")}</div>` : `<div class="empty-state compact"><strong>当前没有赛事绑定</strong><span>系统会尝试使用同类型已激活规则包。</span></div>`}</section>
    <details class="technical-details"><summary>技术追踪信息</summary><dl class="detail-facts compact"><div><dt>赛事编号</dt><dd>${escapeHtml(competition.id)}</dd></div><div><dt>规则包编号</dt><dd>${escapeHtml(selectedPackage?.id ?? selectedBinding?.rule_package_id ?? "未匹配")}</dd></div><div><dt>模型标识</dt><dd>${escapeHtml(selectedModel)}</dd></div></dl></details>
  </div>`;
  showHtmlModal("赛事规则与模型路径", "可视化路由", body);
}

function requestDeleteCompetition(
  competitionId: string,
  competitionName: string,
): void {
  showConfirmation(
    "删除赛事",
    "赛事将从目录和自动路由中隐藏，已有比赛、推演和复盘历史会保留，避免破坏历史数据。",
    [
      ["赛事", competitionName],
      ["处理方式", "停用赛事并停用赛事级绑定"],
    ],
    "确认删除",
    async () => {
      await runBusy(() => api.deleteCompetition(competitionId));
      await refresh();
      page = "rules";
      render();
      toast("赛事已删除，历史数据仍然保留", "success");
    },
  );
}

async function createSeason(): Promise<void> {
  const competitionId = nullableValue("season-competition-id");
  if (!competitionId) throw new Error("请选择赛季所属赛事");
  const draft: SeasonDraft = {
    competition_id: competitionId,
    name: value("season-name").trim(),
    starts_on: nullableValue("season-starts-on"),
    ends_on: nullableValue("season-ends-on"),
    status: value("season-status") as SeasonDraft["status"],
    metadata: {},
  };
  await runBusy(() => api.createSeason(draft));
  await refresh();
  page = "rules";
  render();
  toast("赛季已创建", "success");
}

async function createStage(): Promise<void> {
  const seasonId = nullableValue("stage-season-id");
  if (!seasonId) throw new Error("请选择阶段所属赛季");
  const draft: StageDraft = {
    season_id: seasonId,
    code: "",
    name: value("stage-name").trim(),
    stage_kind: value("stage-kind") as CompetitionKind,
    sequence_no: Number(value("stage-sequence")),
    rules: {},
  };
  await runBusy(() => api.createStage(draft));
  await refresh();
  page = "rules";
  render();
  toast("赛事阶段已创建", "success");
}

async function createRound(): Promise<void> {
  const stageId = nullableValue("round-stage-id");
  if (!stageId) throw new Error("请选择轮次所属阶段");
  const draft: RoundDraft = {
    stage_id: stageId,
    code: "",
    name: value("round-name").trim(),
    sequence_no: Number(value("round-sequence")),
    starts_at: null,
    ends_at: null,
  };
  await runBusy(() => api.createRound(draft));
  await refresh();
  page = "rules";
  render();
  toast("比赛轮次已创建", "success");
}

async function registerRulePackage(): Promise<void> {
  if (!pendingRulePackage) throw new Error("请先选择规则包文件");
  await runBusy(() => api.registerRulePackage(pendingRulePackage!));
  pendingRulePackage = null;
  await refresh();
  page = "rules";
  render();
  toast("规则包已校验并注册", "success");
}

async function createBinding(): Promise<void> {
  const competitionId = nullableValue("binding-competition-id");
  const packageId = nullableValue("binding-rule-package-id");
  if (!competitionId || !packageId) throw new Error("请选择赛事和规则包");
  const draft: CompetitionBindingDraft = {
    binding_name: nullableValue("binding-name"),
    competition_id: competitionId,
    season_id: nullableValue("binding-season-id"),
    stage_id: nullableValue("binding-stage-id"),
    competition_kind: null,
    rule_package_id: packageId,
    priority: Number(value("binding-priority")),
    valid_from: null,
    valid_to: null,
  };
  await runBusy(() => api.createCompetitionBinding(draft));
  await refresh();
  page = "rules";
  render();
  toast("赛事专属路由已建立", "success");
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

function isChineseName(languageCode: string | null, name: string): boolean {
  const language = languageCode?.toLowerCase() ?? "";
  return ["zh-cn", "zh-hans", "zh"].includes(language) || /[一-龥]/u.test(name);
}

function currentPlayerLocalizedName(detail: PlayerDetail | null): string {
  if (!detail) return "";
  const records = detail.names.filter((item) => {
    const name = typeof item.name === "string" ? item.name : "";
    const language = typeof item.language_code === "string" ? item.language_code : null;
    return isChineseName(language, name);
  });
  records.sort((left, right) => {
    const languageRank = (record: Record<string, unknown>) => {
      const language = typeof record.language_code === "string" ? record.language_code.toLowerCase() : "";
      return ["zh-cn", "zh-hans", "zh"].includes(language) ? 1 : 0;
    };
    const byLanguage = languageRank(right) - languageRank(left);
    if (byLanguage !== 0) return byLanguage;
    const leftFrom = typeof left.valid_from === "string" ? left.valid_from : "";
    const rightFrom = typeof right.valid_from === "string" ? right.valid_from : "";
    const leftId = typeof left.id === "string" ? left.id : "";
    const rightId = typeof right.id === "string" ? right.id : "";
    return rightFrom.localeCompare(leftFrom) || rightId.localeCompare(leftId);
  });
  return typeof records[0]?.name === "string" ? records[0].name.trim() : "";
}

function currentTeamLocalizedName(detail: TeamDetail | null): string {
  if (!detail) return "";
  const records = detail.names.filter((item) => isChineseName(item.language_code, item.name));
  records.sort((left, right) => {
    const languageRank = (language: string | null) => ["zh-cn", "zh-hans", "zh"].includes(language?.toLowerCase() ?? "") ? 1 : 0;
    const byLanguage = languageRank(right.language_code) - languageRank(left.language_code);
    if (byLanguage !== 0) return byLanguage;
    return (right.valid_from ?? "").localeCompare(left.valid_from ?? "") || right.id.localeCompare(left.id);
  });
  return records[0]?.name.trim() ?? "";
}

function todayDate(): string {
  return new Date().toISOString().slice(0, 10);
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

function applyPlayerCatalog(result: PlayerCatalogLoadResult): void {
  playerReferences = result.references;
  playerListPage = result.list;
  playerQuery = result.query;
  selectedPlayer = result.selected;
}

async function loadPlayerCatalog(resetCursor: boolean): Promise<void> {
  if (!state?.data.database_configured) return;
  applyPlayerCatalog(
    await fetchPlayerCatalog(
      playerQuery,
      selectedPlayer,
      resetCursor,
      resetCursor ? null : playerReferences,
    ),
  );
}

function setPlayerPageLoading(active: boolean): void {
  playerPageLoading = active;
  const main = document.querySelector<HTMLElement>(
    '.app-shell[data-current-page="players"] .player-browser .entity-main',
  );
  if (!main) return;
  main.setAttribute("aria-busy", String(active));
  main.querySelectorAll<HTMLButtonElement>(
    '[data-action="previous-player-page"], [data-action="next-player-page"]',
  ).forEach((button) => {
    if (active) {
      button.dataset.paginationDisabled = String(button.disabled);
      button.disabled = true;
      return;
    }
    button.disabled = button.dataset.paginationDisabled === "true";
    delete button.dataset.paginationDisabled;
  });
}

async function loadPlayerPage(
  nextQuery: PlayerListQuery,
): Promise<PlayerCatalogLoadResult | null> {
  if (!state?.data.database_configured || playerPageLoading) return null;
  setPlayerPageLoading(true);
  try {
    return await fetchPlayerCatalog(
      nextQuery,
      selectedPlayer,
      false,
      playerReferences,
    );
  } finally {
    setPlayerPageLoading(false);
  }
}

function refreshPlayerPageDom(): boolean {
  if (page !== "players" || !playerListPage) return false;
  const tbody = document.querySelector<HTMLTableSectionElement>(
    ".entity-page-players .player-directory-table tbody",
  );
  const resultCount = document.querySelector<HTMLElement>(
    ".entity-page-players .player-list-toolbar .entity-list-actions > span",
  );
  const footer = document.querySelector<HTMLElement>(
    ".entity-page-players .entity-main-footer",
  );
  const pageLabel = footer?.querySelector<HTMLElement>("div > b") ?? null;
  const previousButton = footer?.querySelector<HTMLButtonElement>(
    '[data-action="previous-player-page"]',
  ) ?? null;
  const nextButton = footer?.querySelector<HTMLButtonElement>(
    '[data-action="next-player-page"]',
  ) ?? null;
  if (!tbody || !resultCount || !footer || !pageLabel || !previousButton || !nextButton) {
    return false;
  }

  tbody.innerHTML = playerTableRows(
    playerListPage,
    selectedPlayer?.player.id ?? null,
    selectedPlayerIds,
  );
  resultCount.textContent = `${playerListPage.items.length} 条当前结果`;
  pageLabel.textContent = `第 ${playerCursorHistory.length + 1} 页`;
  previousButton.disabled = playerCursorHistory.length === 0;
  nextButton.disabled = !playerListPage.has_more;
  const selectAll = document.querySelector<HTMLInputElement>("#player-select-all");
  if (selectAll) selectAll.checked = false;

  const ribbonItems = document.querySelectorAll<HTMLElement>(
    ".entity-page-players .task-context-ribbon .task-context-item",
  );
  const resultRibbon = ribbonItems.item(1);
  const ribbonValue = resultRibbon?.querySelector<HTMLElement>("strong") ?? null;
  const ribbonNote = resultRibbon?.querySelector<HTMLElement>("small") ?? null;
  if (ribbonValue) ribbonValue.textContent = `${playerListPage.items.length} 名球员`;
  if (ribbonNote) {
    const activeFilterCount = [
      playerQuery.team_id,
      playerQuery.position_code,
      playerQuery.availability_status,
      playerQuery.player_status,
    ].filter(Boolean).length;
    ribbonNote.textContent = `第 ${playerCursorHistory.length + 1} 页 · ${activeFilterCount} 项筛选`;
  }

  const sectionBadge = document.querySelector<HTMLElement>(
    '.entity-page-players [data-action="select-workspace-section"][data-section-id="directory"] .workspace-section-badge',
  );
  if (sectionBadge) sectionBadge.textContent = String(playerListPage.items.length);

  document.querySelector<HTMLElement>(
    ".entity-page-players .player-table-wrap",
  )?.scrollTo({ top: 0, behavior: "auto" });
  return true;
}

function applyTeamCatalog(result: TeamCatalogLoadResult): void {
  teamListPage = result.list;
  teamQuery = result.query;
  selectedTeam = result.selected;
}

async function loadTeamCatalog(resetCursor: boolean): Promise<void> {
  if (!state?.data.database_configured) return;
  const [catalog, coaches, formations] = await Promise.all([
    fetchTeamCatalog(teamQuery, selectedTeam, resetCursor),
    api.listCoaches({ search: null, active_only: false, limit: 500 }),
    api.listFormations(false),
  ]);
  applyTeamCatalog(catalog);
  if (selectedTeam) {
    [selectedTeamLineupHistory, selectedTeamLineupPresets] = await Promise.all([
      api.listTeamMatchLineups(selectedTeam.team.id, 100),
      api.listTeamLineupPresets(selectedTeam.team.id, true),
    ]);
  } else {
    selectedTeamLineupHistory = [];
    selectedTeamLineupPresets = [];
  }
  coachList = coaches;
  formationCatalog = formations;
}

function applyLineups(result: LineupsLoadResult): void {
  playerReferences = result.references;
  lineupRecords = result.records;
  coachList = result.coaches;
  const matches = result.references.managed_matches;
  if (selectedManagedMatchId && !matches.some((item) => item.id === selectedManagedMatchId)) {
    selectedManagedMatchId = null;
  }
  if (pairedLineupBuilder.match_id && !matches.some((item) => item.id === pairedLineupBuilder.match_id)) {
    pairedLineupBuilder = {
      ...pairedLineupBuilder,
      match_id: "",
      home: emptyPairedLineupSide(),
      away: emptyPairedLineupSide(),
    };
  }
  selectedManagedMatchId ??= matches[0]?.id ?? null;
}

async function loadLineups(): Promise<void> {
  if (!state?.data.database_configured) return;
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
  if (!state?.data.database_configured) return;
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
  if (!state?.data.database_configured) return;
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
    playerReferences?.upcoming_matches[0]?.id ??
    playerReferences?.managed_matches[0]?.id ??
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
): Promise<void> {
  const [profiles, presets] = await Promise.all([
    api.listOpenAiProfiles(),
    api.listApiWorkspacePresets(),
  ]);
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
  if (!state?.data.database_configured) {
    apiWorkspaceSessions = [];
    apiWorkspaceDetail = null;
    return;
  }
  const [sessions, references] = await Promise.all([
    api.listApiWorkspaceSessions(100),
    api.playerCatalogReferenceData(),
  ]);
  apiWorkspaceSessions = sessions;
  apiWorkspaceMatches = references.managed_matches;
  const sessionId =
    selectedSessionId && sessions.some((item) => item.id === selectedSessionId)
      ? selectedSessionId
      : null;
  apiWorkspaceDetail = sessionId
    ? await api.readApiWorkspaceSession(sessionId)
    : null;
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
  const requestOriginPage = page;
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
  window.setTimeout(
    () =>
      document
        .querySelector("#api-workspace-conversation")
        ?.scrollTo({ top: 999999, behavior: "smooth" }),
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
      page === requestOriginPage &&
      page === "api_workspace" &&
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
    if (page === "api_workspace") {
      render();
      window.setTimeout(
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
  modal.reset();
  const request = navigation.begin(nextPage);
  navigationPendingPage = nextPage;
  page = nextPage;
  render({ preserveForm: true });

  try {
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
      await loadApiWorkspace();
      return;
    }
    if (!state?.data.database_configured) return;
    if (nextPage === "release") {
      releaseAcceptanceRuns = await api.listReleaseAcceptanceRuns(50);
      const selectedId = selectedReleaseAcceptanceRun?.id ?? releaseAcceptanceRuns[0]?.id ?? null;
      selectedReleaseAcceptanceRun = selectedId ? await api.readReleaseAcceptanceRun(selectedId) : null;
      return;
    }

    if (nextPage === "players") {
      const result = await fetchPlayerCatalog(
        playerQuery,
        selectedPlayer,
        playerListPage === null,
      );
      if (navigation.isCurrent(request)) applyPlayerCatalog(result);
    } else if (nextPage === "teams" || nextPage === "lineup_presets") {
      const preferredPresetTeamId = nextPage === "lineup_presets"
        ? selectedTeam?.team.id ?? workspaceState.module("lineup_presets").active_tab_id
        : null;
      const [catalog, coaches, formations] = await Promise.all([
        fetchTeamCatalog(teamQuery, selectedTeam, teamListPage === null),
        api.listCoaches({ search: null, active_only: false, limit: 500 }),
        api.listFormations(true),
      ]);
      const restoredPresetTeam = preferredPresetTeamId
        && catalog.list.items.some((item) => item.id === preferredPresetTeamId)
        && catalog.selected?.team.id !== preferredPresetTeamId
          ? await api.readTeam(preferredPresetTeamId)
          : catalog.selected;
      const result = { ...catalog, selected: restoredPresetTeam };
      const [lineupHistory, presets] = result.selected
        ? await Promise.all([
            api.listTeamMatchLineups(result.selected.team.id, 100),
            api.listTeamLineupPresets(result.selected.team.id, true),
          ])
        : [[], [] as TeamLineupPresetRecord[]];
      if (navigation.isCurrent(request)) {
        applyTeamCatalog(result);
        coachList = coaches;
        formationCatalog = formations;
        selectedTeamLineupHistory = lineupHistory;
        selectedTeamLineupPresets = presets;
        if (nextPage === "lineup_presets" && result.selected) {
          workspaceState.patchModule("lineup_presets", { active_tab_id: result.selected.team.id });
        }
      }
    } else if (nextPage === "lineups") {
      const result = await fetchLineups();
      if (navigation.isCurrent(request)) applyLineups(result);
    } else if (nextPage === "prediction") {
      const references =
        playerReferences ?? (await api.playerCatalogReferenceData());
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
        playerReferences = references;
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
  } catch (error) {
    if (!navigation.isCurrent(request)) return;
    throw error;
  } finally {
    if (navigation.complete(request)) {
      navigationPendingPage = null;
      render({ preserveForm: true });
    }
  }
}

async function refreshBootstrapAndCatalog(): Promise<void> {
  state = await api.bootstrap();
  await loadPlayerCatalog(true);
  if (teamListPage !== null || page === "teams" || page === "lineup_presets") await loadTeamCatalog(true);
}

async function searchTeamOptions(): Promise<void> {
  const search = nullableValue("team-option-search");
  const teams = await runBusy(() => api.listTeamOptions(search, 200));
  if (!playerReferences)
    playerReferences = await api.playerCatalogReferenceData();
  playerReferences = { ...playerReferences, teams };
  render();
  toast(`已加载 ${teams.length} 支球队选项`, "success");
}

async function searchPlayers(): Promise<void> {
  playerCursorHistory = [];
  const selectedTeamId = nullableValue("player-filter-team");
  playerQuery = {
    ...playerQuery,
    search: nullableValue("player-search"),
    team_id: selectedTeamId,
    position_code: nullableValue("player-filter-position"),
    availability_status: nullableValue(
      "player-filter-availability",
    ) as PlayerListQuery["availability_status"],
    player_status: nullableValue(
      "player-filter-status",
    ) as PlayerListQuery["player_status"],
    cursor_name: null,
    cursor_id: null,
  };
  if (playerNavigationContext && playerNavigationContext.team_id !== selectedTeamId) {
    setPlayerNavigationContext(null);
  }
  persistPlayerQuery();
  await runBusy(() => loadPlayerCatalog(false));
  if (selectedPlayer && !playerListPage?.items.some((item) => item.id === selectedPlayer?.player.id)) {
    selectedPlayer = null;
    workspaceState.patchModule("players", { inspector_collapsed: true });
  }
  render();
}

async function clearPlayerFilters(): Promise<void> {
  playerCursorHistory = [];
  playerQuery = {
    search: null,
    team_id: null,
    position_code: null,
    availability_status: null,
    player_status: null,
    limit: playerQuery.limit,
    cursor_name: null,
    cursor_id: null,
  };
  setPlayerNavigationContext(null);
  persistPlayerQuery();
  await runBusy(() => loadPlayerCatalog(false));
  if (selectedPlayer && !playerListPage?.items.some((item) => item.id === selectedPlayer?.player.id)) {
    selectedPlayer = null;
    workspaceState.patchModule("players", { inspector_collapsed: true });
  }
  render();
}

async function nextPlayerPage(): Promise<void> {
  if (
    playerPageLoading
    || !playerListPage?.has_more
    || !playerListPage.next_cursor_name
    || !playerListPage.next_cursor_id
  ) return;
  const previousCursor = {
    cursor_name: playerQuery.cursor_name,
    cursor_id: playerQuery.cursor_id,
  };
  const nextQuery: PlayerListQuery = {
    ...playerQuery,
    cursor_name: playerListPage.next_cursor_name,
    cursor_id: playerListPage.next_cursor_id,
  };
  const result = await loadPlayerPage(nextQuery);
  if (!result) return;
  playerCursorHistory.push(previousCursor);
  applyPlayerCatalog({ ...result, selected: selectedPlayer });
  if (!refreshPlayerPageDom()) render({ preserveForm: true });
}

async function previousPlayerPage(): Promise<void> {
  if (playerPageLoading) return;
  const previous = playerCursorHistory.at(-1);
  if (!previous) return;
  const previousQuery: PlayerListQuery = { ...playerQuery, ...previous };
  const result = await loadPlayerPage(previousQuery);
  if (!result) return;
  playerCursorHistory.pop();
  applyPlayerCatalog({ ...result, selected: selectedPlayer });
  if (!refreshPlayerPageDom()) render({ preserveForm: true });
}

async function openPlayer(playerId: string): Promise<void> {
  selectedPlayer = await runBusy(() => api.readPlayer(playerId));
  appendWorkspaceTab("players", { id: playerId, label: selectedPlayer.player.canonical_name });
  workspaceState.patchModule("players", { active_section: "directory", inspector_collapsed: false });
  render();
}

async function openPlayerProfile(playerId: string): Promise<void> {
  selectedPlayer = await runBusy(() => api.readPlayer(playerId));
  appendWorkspaceTab("players", { id: playerId, label: selectedPlayer.player.canonical_name });
  workspaceState.patchModule("players", { active_section: "profile", inspector_collapsed: true });
  render();
}

async function previewPlayerFromTeam(playerId: string): Promise<void> {
  selectedPlayer = await runBusy(() => api.readPlayer(playerId));
  workspaceState.patchModule("teams", { inspector_collapsed: false });
  render();
}

async function searchTeams(returnToDirectory = false): Promise<void> {
  teamCursorHistory = [];
  teamQuery = {
    ...teamQuery,
    search: nullableValue("team-search"),
    country_code: nullableValue("team-filter-country"),
    team_type: nullableValue("team-filter-type") as TeamListQuery["team_type"],
    active_only: checked("team-filter-active"),
    cursor_name: null,
    cursor_id: null,
  };
  persistTeamQuery();
  await runBusy(() => loadTeamCatalog(false));
  if (
    returnToDirectory
    || (selectedTeam && !teamListPage?.items.some((item) => item.id === selectedTeam?.team.id))
  ) {
    selectedTeam = null;
    selectedTeamLineupHistory = [];
    selectedTeamLineupPresets = [];
    selectedPlayer = null;
    workspaceState.patchModule("teams", {
      active_tab_id: null,
      active_section: "directory",
    });
  }
  render({ preserveForm: true });
}

async function clearTeamFilters(returnToDirectory = false): Promise<void> {
  teamCursorHistory = [];
  teamQuery = {
    ...teamQuery,
    search: null,
    country_code: null,
    team_type: null,
    active_only: true,
    cursor_name: null,
    cursor_id: null,
  };
  persistTeamQuery();
  await runBusy(() => loadTeamCatalog(false));
  if (returnToDirectory) {
    selectedTeam = null;
    selectedTeamLineupHistory = [];
    selectedTeamLineupPresets = [];
    selectedPlayer = null;
    workspaceState.patchModule("teams", {
      active_tab_id: null,
      active_section: "directory",
    });
  }
  render({ preserveForm: true });
}

async function nextTeamPage(): Promise<void> {
  if (!teamListPage?.has_more || !teamListPage.next_cursor_name || !teamListPage.next_cursor_id) return;
  teamCursorHistory.push({ cursor_name: teamQuery.cursor_name, cursor_id: teamQuery.cursor_id });
  teamQuery = { ...teamQuery, cursor_name: teamListPage.next_cursor_name, cursor_id: teamListPage.next_cursor_id };
  await runBusy(() => loadTeamCatalog(false));
  render();
}

async function previousTeamPage(): Promise<void> {
  const previous = teamCursorHistory.pop();
  if (!previous) return;
  teamQuery = { ...teamQuery, ...previous };
  await runBusy(() => loadTeamCatalog(false));
  render();
}

async function openTeam(teamId: string): Promise<void> {
  const [detail, lineupHistory, presets] = await runBusy(() =>
    Promise.all([
      api.readTeam(teamId),
      api.listTeamMatchLineups(teamId, 100),
      api.listTeamLineupPresets(teamId, true),
    ]),
  );
  selectedTeam = detail;
  selectedPlayer = null;
  selectedTeamLineupHistory = lineupHistory;
  selectedTeamLineupPresets = presets;
  appendWorkspaceTab("teams", { id: teamId, label: detail.team.canonical_name });
  workspaceState.patchModule("teams", { active_section: "directory" });
  render();
}

async function selectLineupPresetTeam(teamId: string): Promise<void> {
  const [detail, presets] = await runBusy(() =>
    Promise.all([
      api.readTeam(teamId),
      api.listTeamLineupPresets(teamId, true),
    ]),
  );
  selectedTeam = detail;
  selectedPlayer = null;
  selectedTeamLineupPresets = presets;
  selectedTeamLineupHistory = [];
  workspaceState.patchModule("lineup_presets", { active_tab_id: teamId });
  render({ preserveForm: true });
}

async function refreshLineupPresetPage(): Promise<void> {
  await runBusy(() => loadTeamCatalog(false));
  if (selectedTeam) workspaceState.patchModule("lineup_presets", { active_tab_id: selectedTeam.team.id });
  render({ preserveForm: true });
}

async function openTeamProfile(teamId: string): Promise<void> {
  const [detail, lineupHistory, presets] = await runBusy(() =>
    Promise.all([
      api.readTeam(teamId),
      api.listTeamMatchLineups(teamId, 100),
      api.listTeamLineupPresets(teamId, true),
    ]),
  );
  selectedTeam = detail;
  selectedPlayer = null;
  selectedTeamLineupHistory = lineupHistory;
  selectedTeamLineupPresets = presets;
  appendWorkspaceTab("teams", { id: teamId, label: detail.team.canonical_name });
  workspaceState.patchModule("teams", { active_section: "profile", inspector_collapsed: true });
  render();
}

async function reloadSelectedTeam(): Promise<void> {
  if (!selectedTeam) return;
  const teamId = selectedTeam.team.id;
  [selectedTeam, selectedTeamLineupHistory, selectedTeamLineupPresets, teamListPage] = await Promise.all([
    api.readTeam(teamId),
    api.listTeamMatchLineups(teamId, 100),
    api.listTeamLineupPresets(teamId, true),
    api.listTeams(teamQuery),
  ]);
}

async function exportTeamPackageTemplate(): Promise<void> {
  const outputPath = await api.chooseExcelExportFile("球队完整资料包.xlsx");
  if (!outputPath) return;
  const result = await runBusy(() => api.exportTeamPackageTemplate(outputPath));
  toast(`标准资料包已导出：${result.output_path}`, "success");
}

async function previewTeamPackageImport(): Promise<void> {
  const inputPath = await api.chooseExcelImportFile();
  if (!inputPath) return;
  const mode = value("team-package-import-mode") as SpreadsheetImportMode;
  teamPackagePreview = await runBusy(() =>
    api.previewTeamPackageImport(inputPath, mode),
  );
  render();
  const teamCounts = teamPackagePreview.team_preview?.counts;
  const playerCounts = teamPackagePreview.player_preview?.counts;
  const blocking =
    (teamCounts?.conflict ?? 0) +
    (teamCounts?.error ?? 0) +
    (playerCounts?.conflict ?? 0) +
    (playerCounts?.error ?? 0) +
    teamPackagePreview.coverage.blockers.length;
  toast(
    blocking > 0
      ? `统一预检完成：仍有 ${blocking} 项需要处理`
      : `统一预检通过，P4 就绪度 ${teamPackagePreview.coverage.readiness_score}/100`,
    blocking > 0 ? "error" : "success",
  );
}

function teamPackagePreviewExportName(sourceFileName: string): string {
  const base = sourceFileName.replace(/\.xlsx$/i, "").trim() || "球队完整资料包";
  const safe = base.replace(/[<>:\"/\\|?*\u0000-\u001f]+/g, "_");
  return `${safe}-完整预检.json`;
}

async function exportTeamPackagePreviewJson(): Promise<void> {
  if (!teamPackagePreview) throw new Error("没有可导出的球队完整资料包预检结果");
  const outputPath = await api.chooseJsonExportFile(
    teamPackagePreviewExportName(teamPackagePreview.source_file_name),
  );
  if (!outputPath) return;
  const result = await runBusy(() =>
    api.exportTeamPackagePreviewJson(outputPath, teamPackagePreview!),
  );
  toast(
    `完整预检 JSON 已导出：${result.output_path}（${result.exported_row_count} 条记录）`,
    "success",
  );
}

async function resolveTeamPackageConflict(
  scope: "team" | "player",
  rowId: string,
  selectedEntityId: string | null,
  skip: boolean,
): Promise<void> {
  if (!teamPackagePreview) throw new Error("没有可处理的球队完整资料包预检结果");
  if (scope === "team") {
    const batchId = teamPackagePreview.team_preview?.batch_id;
    if (!batchId) throw new Error("资料包中没有球队链预检批次");
    teamPackagePreview.team_preview = await runBusy(() =>
      api.resolveTeamMonthlyImportConflict(batchId, {
        row_id: rowId,
        selected_entity_id: selectedEntityId,
        skip,
      }),
    );
  } else {
    const batchId = teamPackagePreview.player_preview?.batch_id;
    if (!batchId) throw new Error("资料包中没有球员链预检批次");
    teamPackagePreview.player_preview = await runBusy(() =>
      api.resolvePlayerCatalogImportConflict(batchId, {
        row_id: rowId,
        selected_entity_id: selectedEntityId,
        skip,
      }),
    );
  }
  render();
  toast(skip ? "该行已标记跳过" : "冲突关联已确认", "success");
}

async function commitTeamPackageImport(): Promise<void> {
  if (!teamPackagePreview) throw new Error("没有可提交的球队完整资料包预检结果");
  const result = await runBusy(() =>
    api.commitTeamPackageImport({
      team_batch_id: teamPackagePreview?.team_preview?.batch_id ?? null,
      player_batch_id: teamPackagePreview?.player_preview?.batch_id ?? null,
    }),
  );
  teamPackagePreview = null;
  await refreshBootstrapAndCatalog();
  render();
  toast(
    `完整资料包导入完成：新增 ${result.inserted_count}，更新 ${result.updated_count}，结束旧记录 ${result.ended_previous_count}`,
    result.error_count > 0 ? "error" : "success",
  );
}

async function exportTeamTemplate(): Promise<void> {
  const outputPath = await api.chooseExcelExportFile("球队月度更新模板.xlsx");
  if (!outputPath) return;
  const result = await runBusy(() => api.exportTeamMonthlyTemplate(outputPath));
  toast(`模板已导出：${result.output_path}`, "success");
}

async function exportTeamData(): Promise<void> {
  const outputPath = await api.chooseExcelExportFile("球队月度更新.xlsx");
  if (!outputPath) return;
  const result = await runBusy(() => api.exportTeamMonthlyData(outputPath));
  toast(`已导出 ${result.team_count} 支球队，关联 ${result.related_row_count} 行`, "success");
}

async function previewTeamImport(): Promise<void> {
  const inputPath = await api.chooseExcelImportFile();
  if (!inputPath) return;
  const mode = value("team-spreadsheet-import-mode") as SpreadsheetImportMode;
  teamSpreadsheetPreview = await runBusy(() =>
    api.previewTeamMonthlyImport(inputPath, mode),
  );
  render();
  const blocking =
    teamSpreadsheetPreview.counts.conflict + teamSpreadsheetPreview.counts.error;
  toast(
    blocking > 0
      ? `预检完成：发现 ${blocking} 条需要处理`
      : "球队月度工作簿预检通过",
    blocking > 0 ? "error" : "success",
  );
}

async function resolveTeamImportConflict(
  rowId: string,
  selectedEntityId: string | null,
  skip: boolean,
): Promise<void> {
  if (!teamSpreadsheetPreview) throw new Error("没有可处理的球队工作簿预检结果");
  teamSpreadsheetPreview = await runBusy(() =>
    api.resolveTeamMonthlyImportConflict(teamSpreadsheetPreview!.batch_id, {
      row_id: rowId,
      selected_entity_id: selectedEntityId,
      skip,
    }),
  );
  render();
  toast(skip ? "该行已标记跳过" : "冲突关联已确认", "success");
}

async function commitTeamImport(): Promise<void> {
  if (!teamSpreadsheetPreview) throw new Error("没有可提交的球队工作簿预检结果");
  const result = await runBusy(() =>
    api.commitTeamMonthlyImport(teamSpreadsheetPreview!.batch_id),
  );
  teamSpreadsheetPreview = await api.readTeamMonthlyImportPreview(result.batch_id);
  await refreshBootstrapAndCatalog();
  render();
  toast(
    `导入完成：新增 ${result.inserted_count}，更新 ${result.updated_count}，结束旧记录 ${result.ended_previous_count}`,
    "success",
  );
}

async function updateTeam(teamId: string): Promise<void> {
  const localizedName = value("team-localized-name").trim();
  const previousLocalizedName = currentTeamLocalizedName(selectedTeam);
  await runBusy(async () => {
    await api.updateTeam(teamId, {
      canonical_name: value("team-canonical-name").trim(),
      country_code: nullableValue("team-country-code"),
      metadata: { entry_mode: "manual_team_center" },
    });
    if (localizedName && localizedName !== previousLocalizedName) {
      await api.addTeamName({
        team_id: teamId,
        name: localizedName,
        language_code: "zh-CN",
        valid_from: todayDate(),
        valid_to: null,
      });
    }
    await reloadSelectedTeam();
    playerReferences = await api.playerCatalogReferenceData();
  });
  workspaceState.patchModule("teams", { active_section: "profile" });
  render();
  toast("球队基础身份已更新并重新载入", "success");
}

async function addTeamName(teamId: string): Promise<void> {
  const name = value("team-alias-name").trim();
  const languageCode = nullableValue("team-alias-language");
  if (!name) throw new Error("请输入需要保存的球队名称");
  if (!languageCode) throw new Error("请选择名称语言");
  await runBusy(async () => {
    await api.addTeamName({
      team_id: teamId,
      name,
      language_code: languageCode,
      valid_from: todayDate(),
      valid_to: null,
    });
    await reloadSelectedTeam();
  });
  workspaceState.patchModule("teams", { active_section: "profile" });
  render();
  toast("球队名称已保存并持续显示", "success");
}

async function saveTeamProfile(teamId: string): Promise<void> {
  const draft: TeamProfileDraft = {
    short_name: nullableValue("team-short-name"),
    team_type: value("team-profile-type") as TeamProfileDraft["team_type"],
    founded_year: nullableNumber("team-founded-year"),
    city: nullableValue("team-city"),
    stadium: nullableValue("team-stadium"),
    head_coach: selectedTeam?.profile?.head_coach ?? null,
    default_formation: nullableValue("team-default-formation"),
    tactical_style: value(
      "team-tactical-style",
    ) as TeamProfileDraft["tactical_style"],
    attack_rating: nullableNumber("team-attack-rating"),
    midfield_rating: nullableNumber("team-midfield-rating"),
    defence_rating: nullableNumber("team-defence-rating"),
    goalkeeper_rating: nullableNumber("team-goalkeeper-rating"),
    reputation: nullableNumber("team-reputation"),
    data_confidence: Number(value("team-profile-confidence")),
    notes: nullableValue("team-profile-notes"),
    metadata: {
      source: "manual_team_center",
      organization_model: "identity_roster_tactics_ratings_history",
    },
  };
  await runBusy(async () => {
    await api.upsertTeamProfile(teamId, draft);
    await reloadSelectedTeam();
  });
  render();
  toast("球队档案与战术资料已保存", "success");
}

async function saveFormationUsage(teamId: string): Promise<void> {
  const scopeType = value("formation-scope-type") as FormationUsageDistributionDraft["scope_type"];
  const coachId = nullableValue("formation-coach-id");
  if (scopeType === "team_coach" && !coachId) throw new Error("球队 + 教练作用域必须选择教练");
  const entries = Array.from(document.querySelectorAll<HTMLInputElement>(".formation-usage-count"))
    .map((input) => ({ formation_id: input.dataset.formationId ?? "", usage_count: Number(input.value || 0) }))
    .filter((entry) => entry.formation_id && entry.usage_count > 0);
  const draft: FormationUsageDistributionDraft = {
    scope_type: scopeType,
    team_id: teamId,
    coach_id: scopeType === "team_coach" ? coachId : null,
    competition_id: null,
    window_preset: value("formation-window-preset") as FormationUsageDistributionDraft["window_preset"],
    window_start: nullableValue("formation-window-start"),
    window_end: nullableValue("formation-window-end"),
    observed_matches: Number(value("formation-observed-matches")),
    confidence: Number(value("formation-confidence")),
    alpha: Number(value("formation-alpha")),
    source_document_id: null,
    metadata: { source: "manual_team_center" },
    entries,
  };
  await runBusy(async () => {
    await api.saveFormationUsageDistribution(draft);
    await reloadSelectedTeam();
  });
  render();
  toast("阵型观察已保存并归一化", "success");
}

function bulkArchiveSummary(
  result: BulkArchiveResult,
  entityLabel: string,
): string {
  const failed = result.failed.length
    ? `<ul>${result.failed.map((item) => `<li><strong>${escapeHtml(item.label)}</strong>：${escapeHtml(item.reason)}</li>`).join("")}</ul>`
    : "<p>没有归档失败的记录。</p>";
  return `<div class="route-detail"><div class="route-detail-summary"><span>批量归档结果</span><h3>新归档 ${result.archived_ids.length}，已归档 ${result.already_archived_ids.length} / ${result.requested_count} ${entityLabel}</h3><p>归档只改变活动状态，不删除履历、比赛、阵容、观察或审计记录。</p></div><section class="prediction-detail-section"><h3>失败记录</h3>${failed}</section></div>`;
}

async function bulkArchiveEntities(
  entityType: "team" | "player",
): Promise<void> {
  const ids = Array.from(
    entityType === "team" ? selectedTeamIds : selectedPlayerIds,
  );
  if (ids.length === 0) throw new Error("请先选择要归档的对象");
  const result = await runBusy(() =>
    api.bulkArchiveEntities(entityType, ids),
  );
  const removedIds = [...result.archived_ids, ...result.already_archived_ids];
  const activeId = removeWorkspaceObjects(entityType === "team" ? "teams" : "players", removedIds);
  state = await api.bootstrap();
  if (entityType === "team") await loadTeamCatalog(true);
  else await loadPlayerCatalog(true);
  playerReferences = await api.playerCatalogReferenceData();
  persistWorkspaceSelection(entityType === "team" ? "teams" : "players");
  if (activeId) await openAvailableWorkspaceTab(entityType === "team" ? "teams" : "players", activeId);
  else render();
  showHtmlModal(
    entityType === "team" ? "球队批量归档结果" : "球员批量归档结果",
    "保留历史关系",
    bulkArchiveSummary(result, entityType === "team" ? "支球队" : "名球员"),
  );
}

function bulkDeleteSummary(
  result: BulkDeleteResult,
  entityLabel: string,
): string {
  const blocked = result.blocked.length
    ? `<ul>${result.blocked.map((item) => `<li><strong>${escapeHtml(item.label)}</strong>：${escapeHtml(item.reason)}</li>`).join("")}</ul>`
    : "<p>没有被拦截的记录。</p>";
  return `<div class="route-detail"><div class="route-detail-summary"><span>批量删除结果</span><h3>已删除 ${result.deleted_ids.length} / ${result.requested_count} ${entityLabel}</h3><p>未删除的记录会保留原数据，不会出现半删除关联。</p></div><section class="prediction-detail-section"><h3>拦截记录</h3>${blocked}</section></div>`;
}

function requestBulkDeletePlayers(): void {
  const ids = Array.from(selectedPlayerIds);
  if (ids.length === 0) throw new Error("请先选择球员");
  showConfirmation(
    "批量删除球员",
    "将逐个执行安全删除；存在不可删除关联的球员会被保留并显示原因。",
    [
      ["已选择", `${ids.length} 名球员`],
      ["处理方式", "逐条事务、保留失败项"],
    ],
    "确认批量删除",
    async () => {
      const result = await runBusy(() => api.bulkDeletePlayers(ids));
      const activeId = removeWorkspaceObjects("players", result.deleted_ids);
      await refreshBootstrapAndCatalog();
      persistWorkspaceSelection("players");
      if (activeId) await openAvailableWorkspaceTab("players", activeId);
      else render();
      showHtmlModal(
        "球员批量删除结果",
        "安全删除",
        bulkDeleteSummary(result, "名球员"),
      );
    },
  );
}

const TEAM_REFERENCE_LABELS: Record<string, string> = {
  matches: "比赛",
  lineups: "正式阵容",
  team_lineup_presets: "球队阵容预设",
  player_team_periods: "球员效力履历",
  team_coach_periods: "教练任期",
  team_season_memberships: "赛季注册",
  player_availability: "球员可用性",
  formation_usage: "阵型使用观察",
  team_tactical_observations: "球队战术观察",
  team_ability_observations: "球队能力观察",
  substitutions: "换人记录",
  dynamic_tag_opponents: "对手动态标签",
  team_match_reviews: "球队赛后复盘",
  player_match_reviews: "球员赛后复盘",
  player_match_observations: "球员赛后观察",
};

function teamDeletionCheckSummary(checks: EntityDeletionCheck[]): string {
  if (checks.length === 0) return "<p>没有需要说明的受保护球队。</p>";
  return `<ul>${checks.map((check) => {
    const references = check.references
      .filter((item) => item.count > 0)
      .map((item) => `${TEAM_REFERENCE_LABELS[item.relation] ?? item.relation} ${item.count}`)
      .join("、");
    const detail = references || check.reason;
    return `<li><strong>${escapeHtml(check.label)}</strong>：${escapeHtml(detail)}</li>`;
  }).join("")}</ul>`;
}

async function requestBulkDeleteTeams(): Promise<void> {
  const ids = Array.from(selectedTeamIds);
  if (ids.length === 0) throw new Error("请先选择球队");

  const checks = await runBusy(() =>
    Promise.all(ids.map((id) => api.checkEntityDeletion("team", id))),
  );
  const missingIds = checks
    .filter((check) => !check.exists)
    .map((check) => check.entity_id);
  const deletableIds = checks
    .filter((check) => check.exists && check.can_permanently_delete)
    .map((check) => check.entity_id);
  const protectedChecks = checks.filter(
    (check) => check.exists && !check.can_permanently_delete,
  );

  let activeId = removeWorkspaceObjects("teams", missingIds);
  if (missingIds.length > 0) {
    state = await api.bootstrap();
    await loadTeamCatalog(true);
    playerReferences = await api.playerCatalogReferenceData();
    persistWorkspaceSelection("teams");
    if (activeId) await openAvailableWorkspaceTab("teams", activeId);
    else render();
  }

  if (deletableIds.length === 0) {
    const missingNotice = missingIds.length
      ? `<p>另有 ${missingIds.length} 个对象已不存在，已从标签页和选择中清除。</p>`
      : "";
    showHtmlModal(
      "所选球队不能永久删除",
      "请使用归档",
      `<div class="route-detail"><div class="route-detail-summary"><span>永久删除预检</span><h3>没有可永久删除的球队</h3><p>完整资料包导入后形成的球员效力、教练任期、阵型与评分观察都属于历史或业务引用。为保证 P4 输入可追溯，这些球队只能归档，不能连同关系记录一起删除。</p>${missingNotice}</div><section class="prediction-detail-section"><h3>引用明细</h3>${teamDeletionCheckSummary(protectedChecks)}</section></div>`,
    );
    return;
  }

  showConfirmation(
    "永久删除无引用球队",
    "只会删除没有任何业务或历史引用的空球队。存在球员履历、教练任期、阵型、评分观察、比赛或复盘的球队会被保留；不再级联删除这些资料。",
    [
      ["原选择", `${ids.length} 支球队`],
      ["可永久删除", `${deletableIds.length} 支球队`],
      ["受保护", `${protectedChecks.length} 支球队，请使用归档`],
    ],
    "确认永久删除无引用球队",
    async () => {
      const result = await runBusy(() => api.bulkDeleteTeams(deletableIds));
      activeId = removeWorkspaceObjects("teams", result.deleted_ids);
      state = await api.bootstrap();
      await loadTeamCatalog(true);
      playerReferences = await api.playerCatalogReferenceData();
      persistWorkspaceSelection("teams");
      if (activeId) await openAvailableWorkspaceTab("teams", activeId);
      else render();
      const combinedResult: BulkDeleteResult = {
        requested_count: ids.length,
        deleted_ids: result.deleted_ids,
        blocked: [
          ...protectedChecks.map((check) => ({
            id: check.entity_id,
            label: check.label,
            reason: check.reason,
          })),
          ...result.blocked,
        ],
      };
      showHtmlModal(
        "球队永久删除结果",
        combinedResult.blocked.length ? "受引用球队已保留" : "删除完成",
        bulkDeleteSummary(combinedResult, "支球队"),
      );
    },
  );
}


const TEAM_FORCE_DELETE_LABELS: Record<string, string> = {
  teams: "球队主体",
  matches: "相关比赛",
  players: "关联球员主体",
  coaches: "关联教练主体",
  feature_snapshots: "P4 输入快照",
  model_runs: "模型运行",
  research_runs: "研究运行",
  p4_freeze_tasks: "P4 冻结任务",
  postmatch_settlements: "赛后结算",
  match_reviews: "比赛复盘",
  ability_update_candidates: "能力更新候选",
  import_batches: "导入批次",
  team_lineup_presets: "球队阵容预设",
  team_lineup_preset_members: "阵容预设成员",
  player_team_periods: "球员效力关系",
  player_ability_observations: "球员能力观察",
  player_dynamic_tags: "球员动态评分/标签",
  formation_usage_observations: "阵型使用记录",
  team_tactical_observations: "球队战术观察",
  team_ability_observations: "球队能力观察",
};

function forceDeleteReferenceRows(preview: TeamForceDeletePreview): string {
  if (preview.references.length === 0) return "<p>未发现额外关联记录。</p>";
  return `<div class="force-delete-impact-grid">${preview.references
    .map(
      (item) =>
        `<div><span>${escapeHtml(TEAM_FORCE_DELETE_LABELS[item.relation] ?? item.relation)}</span><strong>${item.count}</strong></div>`,
    )
    .join("")}</div>`;
}

async function requestForceDeleteTeam(teamId: string): Promise<void> {
  if (!teamId) throw new Error("缺少球队 ID");
  const preview = await runBusy(() => api.previewForceDeleteTeam(teamId));
  pendingTeamForceDelete = preview;
  showHtmlModal(
    `强制删除：${preview.label}`,
    "最高危险等级",
    `<div class="force-delete-warning">
      <strong>这不是归档，也不是普通删除。</strong>
      <p>${escapeHtml(preview.warning)}</p>
      <p>删除成功后，可使用相同球队代码和新版 Excel 重新导入，不会继续读取本次清除的球队、球员、教练、评分、阵型、比赛或 P4 记录。</p>
    </div>
    <section class="prediction-detail-section"><h3>预计清除范围 · ${preview.total_rows} 条</h3>${forceDeleteReferenceRows(preview)}</section>
    <label class="field force-delete-confirmation"><span>请输入完整球队名称确认</span><input id="force-delete-team-confirmation" autocomplete="off" placeholder="${escapeHtml(preview.confirmation_text)}"><small>必须完全输入：${escapeHtml(preview.confirmation_text)}</small></label>`,
    `<button type="button" class="secondary" data-action="close-workspace-detail">取消</button><button type="button" class="primary danger-action" data-action="confirm-force-delete-team">确认强制删除全部资料</button>`,
  );
  document.querySelector<HTMLInputElement>("#force-delete-team-confirmation")?.focus();
}

function forceDeleteResultSummary(result: TeamForceDeleteResult): string {
  const counts = Object.entries(result.deleted_counts)
    .filter(([, count]) => count > 0)
    .map(
      ([key, count]) =>
        `<div><span>${escapeHtml(TEAM_FORCE_DELETE_LABELS[key] ?? key)}</span><strong>${count}</strong></div>`,
    )
    .join("");
  return `<div class="route-detail"><div class="route-detail-summary"><span>强制清除完成</span><h3>${escapeHtml(result.label)} 已从当前数据库中移除</h3><p>相关球队、球员、教练、比赛、评分、动态状态、导入批次及直接关联的 P4 历史已在同一事务内清除。</p></div><section class="prediction-detail-section"><h3>清除结果</h3><div class="force-delete-impact-grid">${counts || "<p>没有额外记录。</p>"}</div></section></div>`;
}

async function confirmForceDeleteTeam(): Promise<void> {
  const preview = pendingTeamForceDelete;
  if (!preview) throw new Error("强制删除预检已失效，请重新打开");
  const confirmation = value("force-delete-team-confirmation").trim();
  if (confirmation !== preview.confirmation_text) {
    throw new Error(`确认文字不匹配，请完整输入：${preview.confirmation_text}`);
  }

  const result = await runBusy(() =>
    api.forceDeleteTeam({
      team_id: preview.team_id,
      confirmation_text: confirmation,
    }),
  );
  pendingTeamForceDelete = null;
  closeModal();

  const nextTeamId = removeWorkspaceObjects("teams", [result.team_id]);
  const nextPlayerId = removeWorkspaceObjects("players", result.deleted_player_ids);
  teamPackagePreview = null;
  teamSpreadsheetPreview = null;
  spreadsheetPreview = null;
  matchSpreadsheetPreview = null;
  state = await api.bootstrap();
  await Promise.all([loadTeamCatalog(true), loadPlayerCatalog(true)]);
  playerReferences = await api.playerCatalogReferenceData();
  persistWorkspaceSelection("teams");
  persistWorkspaceSelection("players");

  if (page === "teams" && nextTeamId) await openAvailableWorkspaceTab("teams", nextTeamId);
  else if (page === "players" && nextPlayerId) await openAvailableWorkspaceTab("players", nextPlayerId);
  else render();

  showHtmlModal("球队及全部资料已强制删除", "事务已提交", forceDeleteResultSummary(result));
}

async function openPlayerFromTeam(playerId: string): Promise<void> {
  if (!selectedTeam) throw new Error("请先选择球队，再从阵容进入球员档案");
  const teamId = selectedTeam.team.id;
  const teamName = selectedTeam.team.canonical_name;
  playerCursorHistory = [];
  playerQuery = {
    search: null,
    team_id: teamId,
    position_code: null,
    availability_status: null,
    player_status: "active",
    limit: playerQuery.limit,
    cursor_name: null,
    cursor_id: null,
  };
  const now = new Date().toISOString();
  setPlayerNavigationContext({
    source: "team_roster",
    team_id: teamId,
    team_name: teamName,
    player_id: playerId,
    origin_page: "teams",
    return_section: null,
    created_at: now,
    updated_at: now,
  });
  playerQuery.player_status = null;
  persistPlayerQuery();
  selectedPlayer = await runBusy(() => api.readPlayer(playerId));
  workspaceState.patchModule("players", { active_section: "directory", inspector_collapsed: false });
  await navigateTo("players");
}

async function openPlayerProfileFromTeam(playerId: string): Promise<void> {
  if (!selectedTeam) throw new Error("请先选择球队，再打开球员完整档案");
  const teamId = selectedTeam.team.id;
  const teamName = selectedTeam.team.canonical_name;
  playerCursorHistory = [];
  playerQuery = {
    search: null,
    team_id: teamId,
    position_code: null,
    availability_status: null,
    player_status: null,
    limit: playerQuery.limit,
    cursor_name: null,
    cursor_id: null,
  };
  const now = new Date().toISOString();
  setPlayerNavigationContext({
    source: "team_roster",
    team_id: teamId,
    team_name: teamName,
    player_id: playerId,
    origin_page: "teams",
    return_section: null,
    created_at: now,
    updated_at: now,
  });
  persistPlayerQuery();
  selectedPlayer = await runBusy(() => api.readPlayer(playerId));
  appendWorkspaceTab("players", { id: playerId, label: selectedPlayer.player.canonical_name });
  workspaceState.patchModule("players", { active_section: "profile", inspector_collapsed: true });
  await navigateTo("players");
}


async function returnToSourceTeamProfile(teamId: string): Promise<void> {
  const sourceTeamId = teamId || playerNavigationContext?.team_id;
  if (!sourceTeamId) {
    workspaceState.patchModule("players", { active_section: "directory", inspector_collapsed: false });
    render();
    return;
  }
  const [detail, lineupHistory, presets] = await runBusy(() =>
    Promise.all([
      api.readTeam(sourceTeamId),
      api.listTeamMatchLineups(sourceTeamId, 100),
      api.listTeamLineupPresets(sourceTeamId, true),
    ]),
  );
  selectedTeam = detail;
  selectedPlayer = null;
  selectedTeamLineupHistory = lineupHistory;
  selectedTeamLineupPresets = presets;
  appendWorkspaceTab("teams", { id: sourceTeamId, label: detail.team.canonical_name });
  workspaceState.patchModule("teams", { active_section: "profile", inspector_collapsed: true });
  await navigateTo("teams");
}

async function openPlayerFromLineup(
  playerId: string,
  teamId: string,
  teamName: string,
  returnSection: "builder" | "chain" = "chain",
): Promise<void> {
  if (!playerId) throw new Error("阵容球员身份缺失，请刷新后重试");
  if (!teamId) throw new Error("阵容球队身份缺失，请重新检查阵容链路");
  playerCursorHistory = [];
  playerQuery = {
    search: null,
    team_id: teamId,
    position_code: null,
    availability_status: null,
    player_status: null,
    limit: playerQuery.limit,
    cursor_name: null,
    cursor_id: null,
  };
  const now = new Date().toISOString();
  setPlayerNavigationContext({
    source: "match_lineup",
    team_id: teamId,
    team_name: teamName || "来源球队",
    player_id: playerId,
    origin_page: "lineups",
    return_section: returnSection,
    created_at: now,
    updated_at: now,
  });
  persistPlayerQuery();
  selectedPlayer = await runBusy(() => api.readPlayer(playerId));
  workspaceState.patchModule("players", { active_section: "profile", inspector_collapsed: true });
  await navigateTo("players");
}

async function returnToLineupWorkspace(section: "builder" | "chain"): Promise<void> {
  setPlayerNavigationContext(null);
  workspaceState.patchModule("lineups", { active_section: section });
  await navigateTo("lineups");
}

async function openTeamApiWorkspace(teamId: string | null): Promise<void> {
  const team = teamId
    ? selectedTeam?.team.id === teamId
      ? selectedTeam
      : await api.readTeam(teamId)
    : null;
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
  const detail =
    selectedPlayer?.player.id === playerId
      ? selectedPlayer
      : await api.readPlayer(playerId);
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

async function reloadSelectedPlayer(): Promise<void> {
  if (!selectedPlayer) return;
  selectedPlayer = await api.readPlayer(selectedPlayer.player.id);
  playerListPage = await api.listPlayers(playerQuery);
}

async function createTeam(): Promise<void> {
  const draft: TeamDraft = {
    canonical_name: value("new-team-name").trim(),
    country_code: nullableValue("new-team-country"),
    metadata: {},
  };
  await runBusy(async () => {
    await api.createTeam(draft);
    await refreshBootstrapAndCatalog();
  });
  render();
  toast("球队已创建", "success");
}

async function createCoach(): Promise<void> {
  const canonicalName = value("new-coach-name").trim();
  if (!canonicalName) throw new Error("请输入教练正式姓名");
  const draft: CoachDraft = {
    canonical_name: canonicalName,
    nationality_code: nullableValue("new-coach-nationality"),
    status: "active",
    metadata: { entry_mode: "manual_team_center" },
  };
  const created = await runBusy(() => api.createCoach(draft));
  coachList = await api.listCoaches({
    search: null,
    active_only: false,
    limit: 500,
  });
  render();
  toast(`教练“${created.canonical_name}”已创建`, "success");
}

async function addTeamCoachPeriod(teamId: string): Promise<void> {
  const coachId = nullableValue("team-coach-id");
  const validFrom = nullableValue("team-coach-valid-from");
  if (!coachId) throw new Error("请选择教练");
  if (!validFrom) throw new Error("请选择任期开始日期");
  const role = value("team-coach-role") as TeamCoachPeriodDraft["role"];
  const draft: TeamCoachPeriodDraft = {
    team_id: teamId,
    coach_id: coachId,
    role,
    valid_from: validFrom,
    valid_to: nullableValue("team-coach-valid-to"),
    is_interim: checked("team-coach-interim") ||
      role === "interim_head_coach" || role === "caretaker",
    confidence: Number(value("team-coach-confidence")),
    source_document_id: null,
    end_previous: checked("team-coach-end-previous"),
    metadata: { entry_mode: "manual_team_center" },
  };
  await runBusy(async () => {
    await api.addTeamCoachPeriod(draft);
    await reloadSelectedTeam();
    coachList = await api.listCoaches({
      search: null,
      active_only: false,
      limit: 500,
    });
  });
  render();
  toast("教练任期已保存，主教练投影已刷新", "success");
}

async function createProvider(): Promise<void> {
  const providerName = value("new-provider-name").trim();
  if (!providerName) throw new Error("请输入数据源名称");
  const draft: DataProviderDraft = {
    code: `provider-${Date.now()}`,
    name: providerName,
    provider_type: "football_data",
    base_url: nullableValue("new-provider-url"),
    metadata: { entry_mode: "desktop" },
  };
  await runBusy(async () => {
    await api.createDataProvider(draft);
    playerReferences = await api.playerCatalogReferenceData();
  });
  render();
  toast("数据源已保存", "success");
}

async function createPlayer(): Promise<void> {
  const height = nullableNumber("new-player-height");
  const draft: PlayerDraft = {
    canonical_name: value("new-player-name").trim(),
    date_of_birth: nullableValue("new-player-birth"),
    nationality_code: nullableValue("new-player-nationality"),
    preferred_foot: value("new-player-foot") as PlayerDraft["preferred_foot"],
    height_cm: height,
    status: "active",
    metadata: {},
  };
  await runBusy(async () => {
    const created = await api.createPlayer(draft);
    await refreshBootstrapAndCatalog();
    selectedPlayer = await api.readPlayer(created.id);
  });
  render();
  toast("球员已创建并加入目录", "success");
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
    lineupBuilderForm = { ...lineupBuilderForm, team_id: teamId };
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
    lineupPlayerCandidates = list.items;
  });
  render();
  toast("球员已创建并加入球队，可继续编辑阵容", "success");
}

async function requestDeletePlayer(
  playerId: string,
  playerName: string,
): Promise<void> {
  const check = await runBusy(() =>
    api.checkEntityDeletion("player", playerId),
  );
  if (!check.can_permanently_delete) {
    const references = check.references
      .filter((item) => item.count > 0)
      .map((item) => `<li>${escapeHtml(item.relation)}：${item.count}</li>`)
      .join("");
    showHtmlModal(
      "球员不能永久删除",
      "请改用归档",
      `<div class="route-detail"><div class="route-detail-summary"><span>历史保护</span><h3>${escapeHtml(playerName)}</h3><p>${escapeHtml(check.reason)}</p></div><section class="prediction-detail-section"><h3>引用统计</h3>${references ? `<ul>${references}</ul>` : "<p>存在受保护引用。</p>"}</section></div>`,
    );
    return;
  }
  showConfirmation(
    "永久删除空球员",
    "该球员尚未进入正式履历、阵容、观察或复盘链路；删除后仅保留审计记录。",
    [
      ["球员", playerName],
      ["引用检查", "未发现历史引用"],
    ],
    "永久删除球员",
    async () => {
      await runBusy(() => api.deletePlayer(playerId));
      selectedPlayer = null;
      await refreshBootstrapAndCatalog();
      render();
      toast("空球员对象已永久删除", "success");
    },
  );
}

async function updatePlayer(playerId: string): Promise<void> {
  const height = nullableNumber("edit-player-height");
  const localizedName = value("edit-player-localized-name").trim();
  const previousLocalizedName = currentPlayerLocalizedName(selectedPlayer);
  const draft: PlayerDraft = {
    canonical_name: value("edit-player-name").trim(),
    date_of_birth: nullableValue("edit-player-birth"),
    nationality_code: nullableValue("edit-player-nationality"),
    preferred_foot: value("edit-player-foot") as PlayerDraft["preferred_foot"],
    height_cm: height,
    status: value("edit-player-status") as PlayerDraft["status"],
    metadata: { entry_mode: "manual_edit" },
  };
  await runBusy(async () => {
    await api.updatePlayer(playerId, draft);
    if (localizedName && localizedName !== previousLocalizedName) {
      await api.addPlayerName({
        player_id: playerId,
        name: localizedName,
        language_code: "zh-CN",
        is_primary: false,
        valid_from: todayDate(),
        valid_to: null,
      });
    }
    await reloadSelectedPlayer();
    playerListPage = await api.listPlayers({
      ...playerQuery,
      cursor_name: null,
      cursor_id: null,
    });
  });
  workspaceState.patchModule("players", { active_section: "profile" });
  render();
  toast("球员基础资料已更新并重新载入", "success");
}

async function exportPlayerTemplate(): Promise<void> {
  const outputPath = await api.chooseExcelExportFile("球员月度更新模板.xlsx");
  if (!outputPath) return;
  const result = await runBusy(() =>
    api.exportPlayerCatalogTemplate(outputPath),
  );
  toast(`模板已导出：${result.output_path}`, "success");
}

async function exportPlayerData(): Promise<void> {
  const outputPath = await api.chooseExcelExportFile("球员月度更新.xlsx");
  if (!outputPath) return;
  const result = await runBusy(() => api.exportPlayerCatalogData(outputPath));
  toast(
    `已导出 ${result.player_count} 名球员和 ${result.team_count} 支球队`,
    "success",
  );
}

async function previewPlayerImport(): Promise<void> {
  const inputPath = await api.chooseExcelImportFile();
  if (!inputPath) return;
  const mode = value("spreadsheet-import-mode") as SpreadsheetImportMode;
  spreadsheetPreview = await runBusy(() =>
    api.previewPlayerCatalogImport(inputPath, mode),
  );
  render();
  const blocking =
    spreadsheetPreview.counts.conflict + spreadsheetPreview.counts.error;
  toast(
    blocking > 0
      ? `预检完成：发现 ${blocking} 条需要修正`
      : "预检通过，可以确认导入",
    blocking > 0 ? "error" : "success",
  );
}

async function resolvePlayerImportConflict(
  rowId: string,
  selectedEntityId: string | null,
  skip: boolean,
): Promise<void> {
  if (!spreadsheetPreview) throw new Error("没有可处理的表格检查结果");
  spreadsheetPreview = await runBusy(() =>
    api.resolvePlayerCatalogImportConflict(spreadsheetPreview!.batch_id, {
      row_id: rowId,
      selected_entity_id: selectedEntityId,
      skip,
    }),
  );
  render();
  toast(skip ? "该行已标记跳过" : "冲突关联已确认", "success");
}

async function commitPlayerImport(): Promise<void> {
  if (!spreadsheetPreview) throw new Error("没有可提交的表格检查结果");
  const result = await runBusy(() =>
    api.commitPlayerCatalogImport(spreadsheetPreview!.batch_id),
  );
  spreadsheetPreview = await api.readPlayerCatalogImportPreview(
    result.batch_id,
  );
  await refreshBootstrapAndCatalog();
  render();
  toast(
    `导入完成：新增 ${result.inserted_count}，更新 ${result.updated_count}，结束旧记录 ${result.ended_previous_count}`,
    "success",
  );
}

async function addPlayerName(playerId: string): Promise<void> {
  const name = value("player-alias-name").trim();
  const languageCode = nullableValue("player-alias-language");
  if (!name) throw new Error("请输入需要保存的球员名称");
  if (!languageCode) throw new Error("请选择名称语言");
  const draft: PlayerNameDraft = {
    player_id: playerId,
    name,
    language_code: languageCode,
    is_primary: checked("player-alias-primary"),
    valid_from: todayDate(),
    valid_to: null,
  };
  await runBusy(async () => {
    await api.addPlayerName(draft);
    await reloadSelectedPlayer();
  });
  workspaceState.patchModule("players", { active_section: "profile" });
  render();
  toast("球员名称已保存并持续显示", "success");
}

async function assignPlayerPosition(playerId: string): Promise<void> {
  const draft: PlayerPositionDraft = {
    player_id: playerId,
    position_code: value("player-position-code"),
    proficiency: Number(value("player-position-proficiency")),
    default_role_code: nullableValue("player-position-default-role"),
    is_primary: checked("player-position-primary"),
    valid_from: null,
    valid_to: null,
    source_document_id: null,
  };
  await runBusy(async () => {
    await api.assignPlayerPosition(draft);
    await reloadSelectedPlayer();
  });
  render();
  toast("球员位置已记录", "success");
}

async function addPlayerTeamPeriod(playerId: string): Promise<void> {
  const teamId = nullableValue("player-team-id");
  const validFrom = nullableValue("player-team-valid-from");
  if (!teamId || !validFrom) throw new Error("请选择球队并填写效力开始日期");
  const squadNumber = nullableNumber("player-squad-number");
  const draft: PlayerTeamPeriodDraft = {
    player_id: playerId,
    team_id: teamId,
    season_id: null,
    squad_number: squadNumber,
    valid_from: validFrom,
    valid_to: nullableValue("player-team-valid-to"),
    registration_status: value("player-registration-status"),
    source_document_id: null,
  };
  await runBusy(async () => {
    await api.addPlayerTeamPeriod(draft);
    await reloadSelectedPlayer();
  });
  render();
  toast("球员效力时间线已更新", "success");
}

async function addPlayerAvailability(playerId: string): Promise<void> {
  const validFrom = localDateTimeToIso(
    nullableValue("player-availability-from"),
    true,
  );
  if (!validFrom) throw new Error("可用性生效时间无效");
  const draft: PlayerAvailabilityDraft = {
    player_id: playerId,
    team_id: nullableValue("player-team-id"),
    competition_id: null,
    status: value(
      "player-availability-status",
    ) as PlayerAvailabilityDraft["status"],
    reason: nullableValue("player-availability-reason"),
    confidence: Number(value("player-availability-confidence")),
    valid_from: validFrom,
    valid_to: localDateTimeToIso(nullableValue("player-availability-to")),
    source_document_id: null,
    metadata: {},
  };
  await runBusy(async () => {
    await api.addPlayerAvailability(draft);
    await reloadSelectedPlayer();
  });
  render();
  toast("球员可用性已记录", "success");
}

async function addPlayerAbility(playerId: string): Promise<void> {
  const observedAt = localDateTimeToIso(
    nullableValue("player-ability-observed-at"),
    true,
  );
  if (!observedAt) throw new Error("能力观察时间无效");
  const draft: PlayerAbilityObservationDraft = {
    player_id: playerId,
    dimension_code: value("player-ability-dimension"),
    context_type: "general",
    context_id: null,
    value: Number(value("player-ability-value")),
    confidence: Number(value("player-ability-confidence")),
    sample_size: Number(value("player-ability-sample-size")),
    observed_at: observedAt,
    effective_from: observedAt,
    effective_to: null,
    calculation_version: "manual.v1",
    source_document_id: null,
    metadata: { entry_mode: "manual" },
  };
  await runBusy(async () => {
    await api.addPlayerAbilityObservation(draft);
    await reloadSelectedPlayer();
  });
  render();
  toast("能力观察已写入，当前能力投影已更新", "success");
}

async function addPlayerExternalId(playerId: string): Promise<void> {
  const providerId = nullableValue("player-provider-id");
  if (!providerId) throw new Error("请先创建并选择数据源");
  const draft: ExternalEntityIdDraft = {
    provider_id: providerId,
    entity_type: "player",
    entity_id: playerId,
    external_id: value("player-external-id").trim(),
    metadata: {},
  };
  await runBusy(async () => {
    await api.addExternalEntityId(draft);
    await reloadSelectedPlayer();
  });
  render();
  toast("外部球员 ID 已绑定", "success");
}

async function addPlayerDynamicTag(playerId: string): Promise<void> {
  const now = new Date();
  const validFrom =
    localDateTimeToIso(nullableValue("player-dynamic-tag-from")) ??
    now.toISOString();
  const validTo =
    localDateTimeToIso(nullableValue("player-dynamic-tag-to")) ??
    new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000).toISOString();
  const draft: PlayerDynamicTagDraft = {
    player_id: playerId,
    tag_code: value("player-dynamic-tag-code"),
    value: Number(value("player-dynamic-tag-value")),
    label: nullableValue("player-dynamic-tag-label"),
    confidence: Number(value("player-dynamic-tag-confidence")),
    observed_at: now.toISOString(),
    valid_from: validFrom,
    valid_to: validTo,
    competition_id: nullableValue("player-dynamic-tag-competition"),
    position_code: nullableValue("player-dynamic-tag-position"),
    opponent_team_id: null,
    sample_size: 1,
    source_type: "manual",
    calculation_version: "manual.v1",
    source_document_id: null,
    metadata: {},
  };
  await runBusy(async () => {
    await api.addPlayerDynamicTag(draft);
    await reloadSelectedPlayer();
  });
  render();
  toast("动态标签已保存，并将在失效时间后自动退出计算", "success");
}

async function calculatePlayerContribution(playerId: string): Promise<void> {
  const request: PlayerMatchContributionRequest = {
    player_id: playerId,
    match_id: null,
    competition_id: nullableValue("player-dynamic-tag-competition"),
    position_code: nullableValue("player-dynamic-tag-position"),
    role_code: null,
    role_origin: null,
    role_source_position_code: null,
    opponent_team_id: null,
    as_of: new Date().toISOString(),
    expected_minutes: null,
  };
  const result = await runBusy(() =>
    api.calculatePlayerMatchContribution(request),
  );
  showModal("球员当前有效贡献", result);
}

function selectedExchangeMatchId(): string {
  const matchId = nullableValue("exchange-match-id");
  if (!matchId) throw new Error("请先选择比赛");
  return matchId;
}

async function exportMatchTemplate(): Promise<void> {
  const outputPath = await api.chooseExcelExportFile("比赛与阵容输入模板.xlsx");
  if (!outputPath) return;
  const result = await runBusy(() => api.exportMatchLineupTemplate(outputPath));
  toast(`比赛模板已导出：${result.output_path}`, "success");
}

async function exportMatchData(): Promise<void> {
  const matchId = selectedExchangeMatchId();
  const outputPath = await api.chooseExcelExportFile("比赛与阵容导出.xlsx");
  if (!outputPath) return;
  const result = await runBusy(() =>
    api.exportMatchLineupData(outputPath, matchId),
  );
  toast(
    `已导出 ${result.lineup_count} 份阵容和 ${result.player_count} 名球员`,
    "success",
  );
}

async function exportAiMatchPackage(): Promise<void> {
  const matchId = selectedExchangeMatchId();
  const outputPath = await api.chooseZipExportFile("比赛推演分析包.zip");
  if (!outputPath) return;
  const result = await runBusy(() =>
    api.exportAiMatchPackage(outputPath, matchId),
  );
  toast(
    `智能分析资料已导出，包含 ${result.player_count} 名球员信息`,
    "success",
  );
}

async function calculateStoredMatch(): Promise<void> {
  const matchId = selectedExchangeMatchId();
  const result = await runBusy(() =>
    api.executePredictionFromMatch({
      match_id: matchId,
      snapshot_type: value("stored-match-snapshot"),
      explicit_rule_package_id: null,
      model_family: selectedPredictionModelFamily,
    }),
  );
  lastPredictionResult = result;
  if (state) state.data.recent_runs = await api.listRecentRuns(100);
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
    playerReferences?.managed_matches ?? playerReferences?.upcoming_matches ?? []
  ).find((item) => item.id === matchId);
  if (!match) throw new Error("所选比赛不存在，请刷新比赛列表");
  const stage = match.stage_id
    ? state?.data.stages.find((item) => item.id === match.stage_id)
    : null;
  const competition = match.competition_id
    ? state?.data.competitions.find((item) => item.id === match.competition_id)
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
  selectedMatchLineupChain = chain;
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
  pairedLineupBuilder = {
    match_id: chain.match_record.id,
    lineup_type: "expected",
    snapshot_type: chain.snapshot_type as LineupSnapshotType,
    captured_at: localDateTimeInputValue(chain.data_cutoff_time),
    source_urls: "",
    home: {
      ...emptyPairedLineupSide(),
      team_id: chain.home.team_id,
      team_name: chain.home.team_name,
      coach_id: preferredCoachForTeam(chain.home.team_id),
    },
    away: {
      ...emptyPairedLineupSide(),
      team_id: chain.away.team_id,
      team_name: chain.away.team_name,
      coach_id: preferredCoachForTeam(chain.away.team_id),
    },
  };
  selectedManagedMatchId = chain.match_record.id;
  selectedMatchLineupChain = chain;
  workflowContinuation = {
    returnPage: "prediction",
    returnSection: workspaceState.module("prediction").active_section ?? "formal",
    reason: `补齐${chain.match_record.home_team_name}与${chain.match_record.away_team_name}双方阵容`,
    matchId: chain.match_record.id,
    snapshotType: chain.snapshot_type as LineupSnapshotType,
  };
  workspaceState.patchModule("lineups", {
    active_section: "builder",
    controls: {},
  });
  await navigateTo("lineups");
  const [homeCount, awayCount] = await runBusy(loadBothPairedLineupSides);
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
  selectedMatchLineupChain = outcome.chain;
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
    selectedMatchLineupChain?.match_record.id === selectedPredictionMatchId() &&
    selectedMatchLineupChain.snapshot_type === selectedPredictionSnapshotType()
      ? selectedMatchLineupChain
      : await runBusy(readSelectedPredictionLineupChain);
  if (chain.ready_for_model) {
    render({ preserveForm: true });
    toast("双方阵容已经就绪，可以直接正式推演", "success");
    return;
  }
  await openMissingPredictionLineup(chain);
}

async function continueLineupPrediction(): Promise<void> {
  const chain = selectedMatchLineupChain;
  if (!chain?.ready_for_model) throw new Error("双方阵容尚未通过模型门禁");
  workflowContinuation = null;
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
  selectedMatchLineupChain = outcome.chain;
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
  if (state) state.data.recent_runs = await api.listRecentRuns(100);
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

async function previewMatchImport(aiPackage: boolean): Promise<void> {
  const inputPath = aiPackage
    ? await api.chooseZipImportFile()
    : await api.chooseExcelImportFile();
  if (!inputPath) return;
  const mode = value("match-import-mode") as SpreadsheetImportMode;
  matchSpreadsheetPreview = await runBusy(() =>
    aiPackage
      ? api.previewAiMatchPackage(inputPath, mode)
      : api.previewMatchLineupImport(inputPath, mode),
  );
  render();
  const blocking =
    matchSpreadsheetPreview.counts.conflict +
    matchSpreadsheetPreview.counts.error;
  toast(
    blocking > 0 ? `预检完成：${blocking} 条需要处理` : "比赛与阵容预检通过",
    blocking > 0 ? "error" : "success",
  );
}

async function resolveMatchImportConflict(
  rowId: string,
  entityId: string | null,
  skip: boolean,
): Promise<void> {
  if (!matchSpreadsheetPreview) throw new Error("没有比赛导入预检批次");
  matchSpreadsheetPreview = await runBusy(() =>
    api.resolveMatchLineupImportConflict(matchSpreadsheetPreview!.batch_id, {
      row_id: rowId,
      selected_entity_id: entityId,
      skip,
    }),
  );
  render();
}

async function commitMatchImport(): Promise<void> {
  if (!matchSpreadsheetPreview) throw new Error("没有比赛导入预检批次");
  const result = await runBusy(() =>
    api.commitMatchLineupImport(matchSpreadsheetPreview!.batch_id),
  );
  matchSpreadsheetPreview = await api.readMatchLineupImportPreview(
    result.batch_id,
  );
  await refreshBootstrapAndCatalog();
  lineupRecords = await api.listLineups(null, 100);
  render();
  toast(
    `比赛与阵容导入完成：新增 ${result.inserted_count}，更新 ${result.updated_count}`,
    "success",
  );
}

function pairedSide(side: LineupSide): PairedLineupBuilderState[LineupSide] {
  return pairedLineupBuilder[side];
}

function selectedManagedMatch(matchId = pairedLineupBuilder.match_id): MatchRecord | null {
  return (
    playerReferences?.managed_matches.find((match) => match.id === matchId) ??
    playerReferences?.upcoming_matches.find((match) => match.id === matchId) ??
    null
  );
}

function preferredCoachForTeam(teamId: string): string {
  const candidates = coachList.filter((coach) => coach.current_team_id === teamId);
  return (
    candidates.find((coach) => coach.current_role === "head_coach") ??
    candidates.find((coach) => coach.current_role === "interim_head_coach") ??
    candidates.find((coach) => coach.current_role === "caretaker") ??
    candidates[0]
  )?.id ?? "";
}

function resetPairedBuilderForMatch(matchId: string, preserveCommon = true): void {
  const match = selectedManagedMatch(matchId);
  if (!match) throw new Error("比赛不存在或尚未加载");
  const now = Date.now();
  const kickoff = new Date(match.kickoff_time).getTime();
  const defaultCaptured = localDateTimeInputValue(
    new Date(Math.min(now, kickoff - 1_000)).toISOString(),
  );
  pairedLineupLoadSequence.home += 1;
  pairedLineupLoadSequence.away += 1;
  pairedLineupBuilder = {
    match_id: match.id,
    lineup_type: preserveCommon ? pairedLineupBuilder.lineup_type : "expected",
    snapshot_type: preserveCommon ? pairedLineupBuilder.snapshot_type : "T-N",
    captured_at: preserveCommon && pairedLineupBuilder.captured_at
      ? pairedLineupBuilder.captured_at
      : defaultCaptured,
    source_urls: preserveCommon ? pairedLineupBuilder.source_urls : "",
    home: {
      ...emptyPairedLineupSide(),
      team_id: match.home_team_id,
      team_name: match.home_team_name,
      coach_id: preferredCoachForTeam(match.home_team_id),
    },
    away: {
      ...emptyPairedLineupSide(),
      team_id: match.away_team_id,
      team_name: match.away_team_name,
      coach_id: preferredCoachForTeam(match.away_team_id),
    },
  };
  pairedLineupPresets = { home: [], away: [] };
  selectedManagedMatchId = match.id;
  selectedMatchLineupChain = null;
}

function capturePairedSideFromDom(side: LineupSide): void {
  const current = pairedSide(side);
  const formationSelect = document.querySelector<HTMLSelectElement>(
    `#paired-${side}-formation-id`,
  );
  const formationInput = document.querySelector<HTMLInputElement>(
    `#paired-${side}-formation`,
  );
  const coach = document.querySelector<HTMLSelectElement>(`#paired-${side}-coach`);
  const quality = document.querySelector<HTMLInputElement>(`#paired-${side}-quality`);
  const rows = Array.from(
    document.querySelectorAll<HTMLElement>(
      `[data-lineup-builder-row][data-lineup-side="${side}"]`,
    ),
  );
  const players = rows.length === 0
    ? current.players
    : rows.map((row) => {
        const playerId = row.dataset.playerId ?? "";
        const existing = current.players.find((item) => item.player_id === playerId);
        if (!existing) throw new Error(`${current.team_name || "球队"}阵容状态丢失，请重新加载名单`);
        const read = (field: string): string =>
          row.querySelector<HTMLInputElement | HTMLSelectElement>(
            `[data-lineup-field="${field}"]`,
          )?.value ?? "";
        const minutes = read("expected_minutes").trim();
        const shirt = read("shirt_number").trim();
        const benchOrder = read("bench_order").trim();
        const startingProbability = read("starting_probability").trim();
        const membershipOverride = row.querySelector<HTMLInputElement>(
          '[data-lineup-field="membership_override"]',
        )?.checked ?? false;
        const positionCode = read("position_code") || null;
        const currentRole = read("role_code") || null;
        const candidate = current.candidates.find((item) => item.id === playerId);
        const previousDefaultRole = candidate
          ? defaultRoleForPlayer(candidate, existing.position_code)
          : null;
        const nextDefaultRole = candidate
          ? defaultRoleForPlayer(candidate, positionCode)
          : null;
        const roleCode = !currentRole || currentRole === previousDefaultRole
          ? nextDefaultRole
          : currentRole;
        return {
          ...existing,
          is_starter: read("is_starter") === "true",
          position_code: positionCode,
          role_code: roleCode,
          expected_minutes: minutes ? Number(minutes) : null,
          shirt_number: shirt ? Number(shirt) : null,
          bench_order: benchOrder ? Number(benchOrder) : null,
          starting_probability: startingProbability ? Number(startingProbability) : null,
          membership_override: membershipOverride,
        };
      });
  const selectedFormation = formationSelect?.selectedOptions[0];
  pairedLineupBuilder = {
    ...pairedLineupBuilder,
    [side]: {
      ...current,
      formation_id: formationSelect?.value ?? current.formation_id,
      formation: selectedFormation?.dataset.code ?? formationInput?.value ?? current.formation,
      coach_id: coach?.value ?? current.coach_id,
      quality_score: Number(quality?.value || current.quality_score),
      players,
    },
  };
}

function capturePairedLineupFromDom(): void {
  const match = document.querySelector<HTMLSelectElement>("#paired-lineup-match");
  if (!match) return;
  const lineupType = document.querySelector<HTMLSelectElement>("#paired-lineup-type");
  const snapshot = document.querySelector<HTMLSelectElement>("#paired-lineup-snapshot");
  const captured = document.querySelector<HTMLInputElement>("#paired-lineup-captured-at");
  const sourceUrls = document.querySelector<HTMLInputElement>("#paired-lineup-source-urls");
  pairedLineupBuilder = {
    ...pairedLineupBuilder,
    match_id: match.value || pairedLineupBuilder.match_id,
    lineup_type: (lineupType?.value ?? pairedLineupBuilder.lineup_type) as PairedLineupBuilderState["lineup_type"],
    snapshot_type: (snapshot?.value ?? pairedLineupBuilder.snapshot_type) as LineupSnapshotType,
    captured_at: captured?.value ?? pairedLineupBuilder.captured_at,
    source_urls: sourceUrls?.value ?? pairedLineupBuilder.source_urls,
  };
  capturePairedSideFromDom("home");
  capturePairedSideFromDom("away");
}

async function loadPairedLineupSide(side: LineupSide): Promise<number> {
  const current = pairedSide(side);
  if (!current.team_id) throw new Error(`${side === "home" ? "主队" : "客队"}尚未确定`);
  const requestSequence = ++pairedLineupLoadSequence[side];
  const teamId = current.team_id;
  const [result, presets] = await Promise.all([
    api.listPlayers({
      search: null,
      team_id: teamId,
      position_code: null,
      availability_status: null,
      player_status: "active",
      limit: 200,
      cursor_name: null,
      cursor_id: null,
    }),
    api.listTeamLineupPresets(teamId, false),
  ]);
  if (
    requestSequence !== pairedLineupLoadSequence[side] ||
    pairedSide(side).team_id !== teamId
  ) {
    return 0;
  }
  const selected = pairedSide(side).players.filter((item) =>
    result.items.some((candidate) => candidate.id === item.player_id),
  );
  pairedLineupPresets = { ...pairedLineupPresets, [side]: presets };
  pairedLineupBuilder = {
    ...pairedLineupBuilder,
    [side]: {
      ...pairedSide(side),
      coach_id: pairedSide(side).coach_id || preferredCoachForTeam(teamId),
      candidates: result.items,
      players: selected,
    },
  };
  return result.items.length;
}

async function loadBothPairedLineupSides(): Promise<[number, number]> {
  if (!pairedLineupBuilder.match_id) throw new Error("请先选择比赛");
  return Promise.all([
    loadPairedLineupSide("home"),
    loadPairedLineupSide("away"),
  ]);
}

function defaultRoleForPlayer(
  player: PlayerListItem,
  positionCode: string | null,
): string | null {
  const code = positionCode || player.primary_position_code;
  if (code && player.position_role_map?.[code]) return player.position_role_map[code];
  return player.primary_role_code;
}

function addPairedLineupPlayer(
  side: LineupSide,
  playerId: string,
  starter: boolean,
  positionCode: string | null = null,
): void {
  capturePairedLineupFromDom();
  const current = pairedSide(side);
  if (current.players.some((item) => item.player_id === playerId)) return;
  const player = current.candidates.find((item) => item.id === playerId);
  if (!player) throw new Error(`${current.team_name}名单中不存在该球员`);
  if (starter && current.players.filter((item) => item.is_starter).length >= 11) {
    throw new Error(`${current.team_name}首发人数不能超过 11 人`);
  }
  const playerName = displayPlayerName(player);
  const next: LineupBuilderPlayer = {
    player_id: player.id,
    player_name: playerName.primary,
    player_secondary_name: playerName.secondary,
    position_code: positionCode || player.primary_position_code,
    role_code: defaultRoleForPlayer(player, positionCode),
    is_starter: starter,
    expected_minutes: starter ? 90 : 20,
    shirt_number: null,
    bench_order: starter ? null : current.players.filter((item) => !item.is_starter).length + 1,
    starting_probability: starter ? 1 : 0,
    membership_override: false,
    availability_status: player.availability_status,
  };
  pairedLineupBuilder = {
    ...pairedLineupBuilder,
    [side]: { ...current, players: [...current.players, next] },
  };
  render();
}

function addSelectedPairedLineupPlayer(side: LineupSide): void {
  const playerSelect = document.querySelector<HTMLSelectElement>(`#paired-${side}-candidate`);
  const roleSelect = document.querySelector<HTMLSelectElement>(`#paired-${side}-candidate-role`);
  const positionSelect = document.querySelector<HTMLSelectElement>(`#paired-${side}-candidate-position`);
  const playerId = playerSelect?.value ?? "";
  if (!playerId) throw new Error("请先选择球员");
  addPairedLineupPlayer(
    side,
    playerId,
    (roleSelect?.value ?? "starter") === "starter",
    positionSelect?.value || null,
  );
}

function presetMemberDraftFromBuilder(
  player: LineupBuilderPlayer,
  index: number,
): TeamLineupPresetMemberDraft {
  return {
    player_id: player.player_id,
    position_code: player.position_code,
    role_code: player.role_code,
    is_starter: player.is_starter,
    shirt_number: player.shirt_number,
    expected_minutes: player.expected_minutes,
    sequence_no: index + 1,
    bench_order: player.is_starter ? null : player.bench_order,
    is_captain: false,
    metadata: {},
  };
}

function presetContextLabel(value: string): string {
  return ({
    general: "通用",
    league: "联赛常规",
    cup: "杯赛轮换",
    attacking: "进攻方案",
    defensive: "防守方案",
    rotation: "轮换方案",
    temporary: "临时方案",
  } as Record<string, string>)[value] ?? value;
}

const TACTICAL_ROLE_OPTIONS: Readonly<Record<string, readonly string[]>> = {
  GK: ["门线型门将", "出击型门将", "清道夫门将"],
  CB: ["盯人中卫", "出球中卫", "拖后中卫"],
  FB: ["防守型边后卫", "进攻型边后卫", "内收边后卫"],
  WB: ["翼卫", "进攻型翼卫", "防守型翼卫"],
  DM: ["单后腰", "防守型后腰", "组织型后腰", "抢球型中场"],
  CM: ["全能中场", "组织核心", "控球中场", "抢球型中场"],
  AM: ["前场组织核心", "组织核心", "影锋", "边路组织者"],
  W: ["边锋", "内切边锋", "边路组织者"],
  SS: ["影锋", "前场组织核心", "第二前锋"],
  ST: ["抢点中锋", "支点中锋", "全能前锋", "伪九号", "反击前锋"],
};

function formationSlotCodes(formationId: string | null): string[] {
  if (!formationId) return [];
  const formation = formationCatalog.find((item) => item.id === formationId);
  if (!formation || !Array.isArray(formation.slot_definition)) return [];
  return [...new Set(formation.slot_definition.filter((item): item is string => typeof item === "string" && item.trim().length > 0).map((item) => item.trim().toUpperCase()))];
}

function positionGroupKey(positionCode: string | null): keyof typeof TACTICAL_ROLE_OPTIONS {
  const code = (positionCode ?? "").trim().toUpperCase();
  if (code === "GK") return "GK";
  if (["CB", "LCB", "RCB", "SW"].includes(code)) return "CB";
  if (["LB", "RB"].includes(code)) return "FB";
  if (["LWB", "RWB"].includes(code)) return "WB";
  if (["DM", "CDM", "LDM", "RDM"].includes(code)) return "DM";
  if (["CM", "LCM", "RCM", "LM", "RM"].includes(code)) return "CM";
  if (["AM", "CAM", "LAM", "RAM"].includes(code)) return "AM";
  if (["LW", "RW", "LF", "RF"].includes(code)) return "W";
  if (code === "SS") return "SS";
  return "ST";
}

function tacticalRoleOptions(
  positionCode: string | null,
  inheritedRole: string | null,
  explicitRole: string | null,
): string {
  const roles = new Set<string>();
  if (positionCode?.trim()) {
    for (const role of TACTICAL_ROLE_OPTIONS[positionGroupKey(positionCode)] ?? []) roles.add(role);
  }
  if (inheritedRole?.trim()) roles.add(inheritedRole.trim());
  if (explicitRole?.trim()) roles.add(explicitRole.trim());
  const inheritLabel = inheritedRole?.trim()
    ? `自动继承：${inheritedRole.trim()}`
    : "自动继承球员资料角色";
  return `<option value="" ${explicitRole ? "" : "selected"}>${escapeHtml(inheritLabel)}</option>${[...roles].map((role) => `<option value="${escapeHtml(role)}" ${role === explicitRole ? "selected" : ""}>${escapeHtml(role)}</option>`).join("")}`;
}

function positionOptionGroups(
  formationId: string | null,
  selected: string | null,
): string {
  const formationSlots = formationSlotCodes(formationId);
  const regularPositions = playerReferences?.positions ?? [];
  const known = new Set<string>();
  const renderOption = (code: string, label: string): string => {
    const normalized = code.trim().toUpperCase();
    if (!normalized || known.has(normalized)) return "";
    known.add(normalized);
    return `<option value="${escapeHtml(normalized)}" ${normalized === selected?.trim().toUpperCase() ? "selected" : ""}>${escapeHtml(label)}（${escapeHtml(normalized)}）</option>`;
  };
  const slotOptions = formationSlots.map((code) => renderOption(code, positionLabel(code))).join("");
  const regularOptions = regularPositions.map((item) => renderOption(item.code, item.name || positionLabel(item.code))).join("");
  const selectedCode = selected?.trim().toUpperCase() ?? "";
  const currentOption = selectedCode && !known.has(selectedCode)
    ? `<optgroup label="当前保存值">${renderOption(selectedCode, positionLabel(selectedCode))}</optgroup>`
    : "";
  return `<option value="">请选择战术位置</option>${slotOptions ? `<optgroup label="当前阵型槽位">${slotOptions}</optgroup>` : ""}${regularOptions ? `<optgroup label="常规位置">${regularOptions}</optgroup>` : ""}${currentOption}`;
}

function teamLineupPresetMemberRows(
  preset: TeamLineupPresetRecord | null,
): string {
  if (!selectedTeam) return "";
  const existing = new Map(preset?.members.map((member) => [member.player_id, member]) ?? []);
  return selectedTeam.squad.map((player, index) => {
    const member = existing.get(player.player_id);
    const enabled = Boolean(member) || (!preset && index < 18);
    const starter = member?.is_starter ?? (!preset && index < 11);
    const selectedPosition = member?.position_code ?? player.position_code ?? null;
    const inheritedRole = member?.role_origin === "player_position_default"
      ? member.role_code
      : player.role_code;
    const explicitRole = member?.role_origin === "lineup_override"
      ? member.role_code
      : null;
    return `<article class="preset-member-row ${enabled ? "" : "excluded"} ${starter ? "starter" : "substitute"}" data-preset-player-id="${escapeHtml(player.player_id)}" data-player-default-position="${escapeHtml(player.position_code ?? "")}" data-player-inherited-role="${escapeHtml(inheritedRole ?? "")}">
      <label class="preset-member-toggle"><input class="preset-member-enabled" type="checkbox" ${enabled ? "checked" : ""}><span><strong>${escapeHtml(player.player_name)}</strong><small>${escapeHtml(positionLabel(player.position_code))} · #${player.squad_number ?? "—"}</small></span></label>
      <select class="preset-member-role" data-native-select aria-label="出场身份" ${enabled ? "" : "disabled"}><option value="starter" ${starter ? "selected" : ""}>首发</option><option value="substitute" ${starter ? "" : "selected"}>替补</option></select>
      <select class="preset-member-position" data-native-select aria-label="战术位置" ${enabled ? "" : "disabled"}>${positionOptionGroups(preset?.formation_id ?? null, selectedPosition)}</select>
      <select class="preset-member-tactical-role" data-native-select aria-label="战术角色" ${enabled ? "" : "disabled"}>${tacticalRoleOptions(selectedPosition, inheritedRole, explicitRole)}</select>
      <label class="preset-member-captain"><input type="radio" name="preset-captain" value="${escapeHtml(player.player_id)}" ${member?.is_captain ? "checked" : ""} ${enabled ? "" : "disabled"}>队长</label>
    </article>`;
  }).join("");
}
function lineupPresetManagerBody(
  teamId: string,
  teamName: string,
  presets: TeamLineupPresetRecord[],
): string {
  const active = presets.filter((preset) => preset.status === "active");
  const archived = presets.filter((preset) => preset.status === "archived");
  const canEdit = selectedTeam?.team.id === teamId;
  const rows = presets.length
    ? presets.map((preset) => `<article class="lineup-preset-manager-row ${preset.status}">
        <div><span>${preset.status === "active" ? (preset.is_default ? "默认 · 使用中" : "使用中") : "已归档"}</span><strong>${escapeHtml(preset.name)}</strong><small>${escapeHtml(preset.formation_code ?? "阵型未设置")} · 首发 ${preset.starter_count} · 共 ${preset.member_count} 人 · v${preset.version}</small></div>
        <div class="lineup-preset-manager-actions">${canEdit && preset.status === "active" ? `<button class="secondary tiny" data-action="open-team-lineup-preset-editor" data-preset-id="${escapeHtml(preset.id)}">编辑</button>` : ""}<button class="ghost tiny danger" data-action="request-delete-team-lineup-preset" data-preset-id="${escapeHtml(preset.id)}" data-preset-name="${escapeHtml(preset.name)}" data-team-id="${escapeHtml(teamId)}" data-team-name="${escapeHtml(teamName)}" data-preset-status="${escapeHtml(preset.status)}" data-member-count="${preset.member_count}">永久删除</button></div>
      </article>`).join("")
    : `<div class="empty-state compact"><strong>暂无阵容预设</strong><p>当前球队没有活动或已归档预设。</p></div>`;
  return `<div class="lineup-preset-manager">
    <div class="lineup-preset-manager-summary"><div><span>活动预设</span><strong>${active.length}</strong></div><div><span>已归档</span><strong>${archived.length}</strong></div><div><span>合计</span><strong>${presets.length}</strong></div></div>
    <div class="blocking-note warning">永久删除会同时删除该预设的首发与替补成员，无法恢复；不会删除球队、球员或比赛阵容。</div>
    <div class="lineup-preset-manager-list">${rows}</div>
  </div>`;
}

async function openTeamLineupPresetManager(teamId: string, teamName: string): Promise<void> {
  if (!teamId) throw new Error("缺少球队 ID，无法管理阵容预设");
  const presets = await runBusy(() => api.listTeamLineupPresets(teamId, true));
  if (selectedTeam?.team.id === teamId) selectedTeamLineupPresets = presets;
  showHtmlModal(
    "预设阵容管理",
    teamName || "球队阵容预设",
    lineupPresetManagerBody(teamId, teamName, presets),
    `<button class="secondary" data-action="close-workspace-detail">关闭</button>${selectedTeam?.team.id === teamId ? '<button class="primary" data-action="open-team-lineup-preset-editor">新建阵容预设</button>' : ""}`,
    "lineup-preset-manager-modal",
  );
}

async function openLineupPresetManagerForSide(side: LineupSide): Promise<void> {
  const current = pairedSide(side);
  if (!current.team_id) throw new Error(`${side === "home" ? "主队" : "客队"}尚未确定`);
  await openTeamLineupPresetManager(current.team_id, current.team_name);
}

async function refreshTeamLineupPresetCaches(teamId: string): Promise<void> {
  const [allPresets, activePresets] = await Promise.all([
    api.listTeamLineupPresets(teamId, true),
    api.listTeamLineupPresets(teamId, false),
  ]);
  if (selectedTeam?.team.id === teamId) selectedTeamLineupPresets = allPresets;
  if (pairedSide("home").team_id === teamId) {
    pairedLineupPresets = { ...pairedLineupPresets, home: activePresets };
  }
  if (pairedSide("away").team_id === teamId) {
    pairedLineupPresets = { ...pairedLineupPresets, away: activePresets };
  }
}

function requestDeleteTeamLineupPreset(
  presetId: string,
  presetName: string,
  teamId: string,
  teamName: string,
  status: string,
  memberCount: number,
): void {
  showConfirmation(
    "永久删除阵容预设",
    "该操作无法恢复。只删除此预设及其成员，不影响球队、球员、比赛阵容和历史快照。",
    [
      ["球队", teamName],
      ["预设", presetName],
      ["状态", status === "archived" ? "已归档" : "活动"],
      ["预设成员", `${memberCount} 人`],
    ],
    "确认永久删除",
    async () => {
      await runBusy(() => api.deleteTeamLineupPreset(presetId));
      await refreshTeamLineupPresetCaches(teamId);
      render({ preserveForm: true });
      toast("阵容预设已永久删除", "success");
      await openTeamLineupPresetManager(teamId, teamName);
    },
  );
}

function openTeamLineupPresetEditor(presetId: string | null = null): void {
  if (!selectedTeam) throw new Error("请先选择球队");
  const preset = presetId
    ? selectedTeamLineupPresets.find((item) => item.id === presetId) ?? null
    : null;
  if (presetId && !preset) throw new Error("阵容预设不存在或尚未加载");
  const teamCoaches = coachList.filter((coach) => coach.current_team_id === selectedTeam!.team.id);
  const body = `<div class="lineup-preset-editor">
    <div class="form-grid three-column clean-form">
      <label class="field"><span>预设名称</span><input id="lineup-preset-name" value="${escapeHtml(preset?.name ?? "")}" placeholder="例如：联赛主力阵容"></label>
      <label class="field"><span>阵型</span><select id="lineup-preset-formation"><option value="">未设置</option>${formationCatalog.map((formation) => `<option value="${escapeHtml(formation.id)}" ${formation.id === preset?.formation_id ? "selected" : ""}>${escapeHtml(formation.code)} · ${escapeHtml(formation.name)}</option>`).join("")}</select></label>
      <label class="field"><span>教练</span><select id="lineup-preset-coach"><option value="">不绑定</option>${teamCoaches.map((coach) => `<option value="${escapeHtml(coach.id)}" ${coach.id === preset?.coach_id ? "selected" : ""}>${escapeHtml(coach.canonical_name)} · ${escapeHtml(coach.current_role ?? "教练")}</option>`).join("")}</select></label>
      <label class="field"><span>适用场景</span><select id="lineup-preset-context">${["general","league","cup","attacking","defensive","rotation","temporary"].map((context) => `<option value="${context}" ${context === (preset?.usage_context ?? "general") ? "selected" : ""}>${presetContextLabel(context)}</option>`).join("")}</select></label>
      <label class="field"><span>使用概率（0–1）</span><input id="lineup-preset-probability" type="number" min="0" max="1" step="0.01" value="${preset?.usage_probability ?? ""}"></label>
      <label class="check-row"><input id="lineup-preset-default" type="checkbox" ${preset?.is_default ? "checked" : ""}><span><strong>设为默认方案</strong><small>同一球队只能有一个默认预设</small></span></label>
      <label class="field span-3"><span>备注</span><textarea id="lineup-preset-notes" rows="2">${escapeHtml(preset?.notes ?? "")}</textarea></label>
    </div>
    <div class="preset-member-editor-heading"><div><strong>出场人员</strong><span>首发必须覆盖当前阵型的 11 个战术位置；战术角色可直接选择或自动继承球员资料。</span></div><div class="preset-member-editor-actions"><span id="lineup-preset-member-summary">正在检查阵容</span><button class="secondary compact" data-action="auto-assign-preset-formation">按阵型自动分配</button></div></div>
    <div class="preset-member-table-head" aria-hidden="true"><span>球员</span><span>身份</span><span>战术位置</span><span>战术角色</span><span>队长</span></div>
    <div id="lineup-preset-validation-note" class="preset-editor-validation"></div>
    <div class="preset-member-list">${teamLineupPresetMemberRows(preset)}</div>
  </div>`;
  const footer = `<button class="secondary" data-action="close-workspace-detail">取消</button><button class="primary" data-action="save-team-lineup-preset" data-preset-id="${escapeHtml(preset?.id ?? "")}">保存阵容预设</button>`;
  showHtmlModal(
    preset ? `编辑：${preset.name}` : "新建阵容预设",
    selectedTeam.team.canonical_name,
    body,
    footer,
    "lineup-preset-modal",
  );
  queueMicrotask(() => {
    const workspaceRoot = document.querySelector(".workspace-panel-root") ?? document;
    enhanceSearchableSelects(workspaceRoot);
    const memberList = document.querySelector<HTMLElement>(".preset-member-list");
    if (memberList) memberList.scrollTop = 0;
    syncPresetEditorRows();
    updatePresetEditorSummary();
  });
}

function presetPositionCompatibility(playerPosition: string, slot: string): number {
  const player = playerPosition.trim().toUpperCase();
  const target = slot.trim().toUpperCase();
  if (!player) return 0;
  if (player === target) return 100;
  const aliases: Record<string, readonly string[]> = {
    GK: ["GK"],
    CB: ["CB", "LCB", "RCB", "SW"],
    LB: ["LB", "LWB"],
    RB: ["RB", "RWB"],
    LWB: ["LWB", "LB", "LM"],
    RWB: ["RWB", "RB", "RM"],
    DM: ["DM", "CDM", "LDM", "RDM", "CM"],
    CDM: ["DM", "CDM", "LDM", "RDM"],
    CM: ["CM", "LCM", "RCM", "DM", "AM"],
    AM: ["AM", "CAM", "LAM", "RAM", "CM", "SS"],
    CAM: ["CAM", "AM", "LAM", "RAM", "SS"],
    LM: ["LM", "LW", "LWB", "LCM"],
    RM: ["RM", "RW", "RWB", "RCM"],
    LW: ["LW", "LM", "LAM", "LF"],
    RW: ["RW", "RM", "RAM", "RF"],
    SS: ["SS", "AM", "CAM", "CF", "ST"],
    CF: ["CF", "ST", "SS", "LST", "RST"],
    ST: ["ST", "CF", "LST", "RST", "SS"],
  };
  const playerAliases = aliases[player] ?? [player];
  const targetBase = target.replace(/^[LR](?=(CB|DM|CM|AM|ST)$)/, "");
  if (playerAliases.includes(target)) return 90;
  if (playerAliases.includes(targetBase)) return 82;
  if (positionGroupKey(player) === positionGroupKey(target)) return 60;
  return 10;
}

function setNativeSelectOptions(
  select: HTMLSelectElement,
  optionsHtml: string,
  preferredValue: string | null,
): void {
  select.innerHTML = optionsHtml;
  if (preferredValue && Array.from(select.options).some((option) => option.value === preferredValue)) {
    select.value = preferredValue;
  }
}

function refreshPresetTacticalRoleSelect(row: HTMLElement): void {
  const positionSelect = row.querySelector<HTMLSelectElement>(".preset-member-position");
  const roleSelect = row.querySelector<HTMLSelectElement>(".preset-member-tactical-role");
  if (!positionSelect || !roleSelect) return;
  const current = roleSelect.value || null;
  const inherited = row.dataset.playerInheritedRole || null;
  setNativeSelectOptions(
    roleSelect,
    tacticalRoleOptions(positionSelect.value || null, inherited, current),
    current,
  );
}

function syncPresetMemberRowState(row: HTMLElement): void {
  const enabled = row.querySelector<HTMLInputElement>(".preset-member-enabled")?.checked ?? false;
  const role = row.querySelector<HTMLSelectElement>(".preset-member-role")?.value ?? "substitute";
  row.classList.toggle("excluded", !enabled);
  row.classList.toggle("starter", enabled && role === "starter");
  row.classList.toggle("substitute", enabled && role !== "starter");
  row.querySelectorAll<HTMLInputElement | HTMLSelectElement>("select, input[type='radio']").forEach((control) => {
    control.disabled = !enabled;
  });
  const captain = row.querySelector<HTMLInputElement>(".preset-member-captain input");
  if (!enabled && captain) captain.checked = false;
}

function syncPresetEditorRows(): void {
  document.querySelectorAll<HTMLElement>("[data-preset-player-id]").forEach((row) => {
    syncPresetMemberRowState(row);
    refreshPresetTacticalRoleSelect(row);
  });
}

function presetEditorFormationId(): string | null {
  return document.querySelector<HTMLSelectElement>("#lineup-preset-formation")?.value || null;
}

function refreshPresetPositionOptions(autoAssign = false): void {
  const formationId = presetEditorFormationId();
  const rows = Array.from(document.querySelectorAll<HTMLElement>("[data-preset-player-id]"));
  for (const row of rows) {
    const select = row.querySelector<HTMLSelectElement>(".preset-member-position");
    if (!select) continue;
    const current = select.value || row.dataset.playerDefaultPosition || null;
    setNativeSelectOptions(select, positionOptionGroups(formationId, current), current);
  }
  if (autoAssign) assignPresetFormationSlots(false);
  rows.forEach(refreshPresetTacticalRoleSelect);
  updatePresetEditorSummary();
}

function assignPresetFormationSlots(showFeedback = true): void {
  const formationId = presetEditorFormationId();
  const slots = formationSlotCodes(formationId);
  if (slots.length !== 11) {
    if (showFeedback) toast("请先选择一个包含 11 个战术位置的标准阵型", "error");
    return;
  }
  const starterRows = Array.from(document.querySelectorAll<HTMLElement>("[data-preset-player-id]"))
    .filter((row) => row.querySelector<HTMLInputElement>(".preset-member-enabled")?.checked)
    .filter((row) => row.querySelector<HTMLSelectElement>(".preset-member-role")?.value === "starter");
  if (starterRows.length !== 11) {
    if (showFeedback) toast(`自动分配需要恰好 11 名首发，当前为 ${starterRows.length} 名`, "error");
    return;
  }
  const availableRows = new Set(starterRows);
  for (const slot of slots) {
    const best = [...availableRows].sort((left, right) => {
      const leftScore = presetPositionCompatibility(left.dataset.playerDefaultPosition ?? "", slot);
      const rightScore = presetPositionCompatibility(right.dataset.playerDefaultPosition ?? "", slot);
      return rightScore - leftScore;
    })[0];
    if (!best) continue;
    availableRows.delete(best);
    const select = best.querySelector<HTMLSelectElement>(".preset-member-position");
    if (select) {
      setNativeSelectOptions(select, positionOptionGroups(formationId, slot), slot);
      select.value = slot;
      refreshPresetTacticalRoleSelect(best);
    }
  }
  updatePresetEditorSummary();
  if (showFeedback) toast("已按阵型和球员登记位置完成首发槽位分配", "success");
}

function updatePresetEditorSummary(): void {
  const rows = Array.from(document.querySelectorAll<HTMLElement>("[data-preset-player-id]"));
  const enabled = rows.filter((row) => row.querySelector<HTMLInputElement>(".preset-member-enabled")?.checked);
  const starters = enabled.filter((row) => row.querySelector<HTMLSelectElement>(".preset-member-role")?.value === "starter");
  const substitutes = enabled.length - starters.length;
  const starterPositions = starters.map((row) => row.querySelector<HTMLSelectElement>(".preset-member-position")?.value ?? "").filter(Boolean);
  const duplicatePositions = starterPositions.filter((value, index, values) => values.indexOf(value) !== index);
  const formationSlots = formationSlotCodes(presetEditorFormationId());
  const missingSlots = formationSlots.filter((slot) => !starterPositions.includes(slot));
  const missingPositionCount = starters.length - starterPositions.length;
  const ready = starters.length === 11
    && missingPositionCount === 0
    && duplicatePositions.length === 0
    && (formationSlots.length === 0 || missingSlots.length === 0);
  const summary = document.querySelector<HTMLElement>("#lineup-preset-member-summary");
  if (summary) {
    summary.textContent = `已选 ${enabled.length} 人 · 首发 ${starters.length}/11 · 替补 ${substitutes}`;
    summary.classList.toggle("ready", ready);
    summary.classList.toggle("warning", !ready);
  }
  const note = document.querySelector<HTMLElement>("#lineup-preset-validation-note");
  if (!note) return;
  const issues: string[] = [];
  if (starters.length !== 11) issues.push(`首发应为 11 人，当前 ${starters.length} 人`);
  if (missingPositionCount > 0) issues.push(`${missingPositionCount} 名首发尚未选择战术位置`);
  if (duplicatePositions.length > 0) issues.push(`首发战术位置重复：${[...new Set(duplicatePositions)].join("、")}`);
  if (formationSlots.length > 0 && missingSlots.length > 0) issues.push(`阵型槽位未覆盖：${missingSlots.join("、")}`);
  note.classList.toggle("ready", issues.length === 0);
  note.classList.toggle("warning", issues.length > 0);
  note.textContent = issues.length ? issues.join("；") : "阵容结构有效，可以保存。";
}

function validateTeamLineupPresetDraft(draft: TeamLineupPresetDraft): void {
  if (!draft.name) throw new Error("请输入预设名称");
  if (draft.members.length < 11) throw new Error("阵容预设至少需要 11 名球员");
  const starters = draft.members.filter((member) => member.is_starter);
  if (starters.length !== 11) throw new Error(`首发必须恰好为 11 人，当前为 ${starters.length} 人`);
  const missingPositions = starters.filter((member) => !member.position_code);
  if (missingPositions.length > 0) throw new Error(`仍有 ${missingPositions.length} 名首发未选择战术位置`);
  const positions = starters.map((member) => member.position_code as string);
  const duplicates = positions.filter((value, index) => positions.indexOf(value) !== index);
  if (duplicates.length > 0) throw new Error(`首发战术位置不能重复：${[...new Set(duplicates)].join("、")}`);
  const slots = formationSlotCodes(draft.formation_id);
  if (slots.length > 0) {
    const missing = slots.filter((slot) => !positions.includes(slot));
    const unexpected = positions.filter((position) => !slots.includes(position));
    if (missing.length || unexpected.length) {
      throw new Error(`首发位置必须完整匹配当前阵型。缺少：${missing.join("、") || "无"}；不属于阵型：${unexpected.join("、") || "无"}`);
    }
  }
  if (draft.usage_probability !== null && (draft.usage_probability < 0 || draft.usage_probability > 1)) {
    throw new Error("使用概率必须位于 0 到 1 之间");
  }
}

function collectTeamLineupPresetDraft(presetId: string | null): TeamLineupPresetDraft {
  if (!selectedTeam) throw new Error("请先选择球队");
  const rows = Array.from(document.querySelectorAll<HTMLElement>("[data-preset-player-id]"));
  const members = rows.flatMap((row, index): TeamLineupPresetMemberDraft[] => {
    const enabled = row.querySelector<HTMLInputElement>(".preset-member-enabled")?.checked ?? false;
    if (!enabled) return [];
    const playerId = row.dataset.presetPlayerId ?? "";
    const role = row.querySelector<HTMLSelectElement>(".preset-member-role")?.value ?? "substitute";
    const position = row.querySelector<HTMLSelectElement>(".preset-member-position")?.value.trim() || null;
    const tacticalRole = row.querySelector<HTMLSelectElement>(".preset-member-tactical-role")?.value.trim() || null;
    const captain = row.querySelector<HTMLInputElement>(".preset-member-captain input")?.checked ?? false;
    return [{
      player_id: playerId,
      position_code: position,
      role_code: tacticalRole,
      is_starter: role === "starter",
      shirt_number: selectedTeam!.squad.find((item) => item.player_id === playerId)?.squad_number ?? null,
      expected_minutes: role === "starter" ? 90 : 20,
      sequence_no: index + 1,
      bench_order: role === "starter" ? null : index + 1,
      is_captain: captain,
      metadata: {},
    }];
  });
  return {
    id: presetId,
    team_id: selectedTeam.team.id,
    name: value("lineup-preset-name").trim(),
    formation_id: nullableValue("lineup-preset-formation"),
    coach_id: nullableValue("lineup-preset-coach"),
    usage_context: value("lineup-preset-context"),
    usage_probability: nullableNumber("lineup-preset-probability"),
    is_default: checked("lineup-preset-default"),
    source_lineup_id: null,
    notes: nullableValue("lineup-preset-notes"),
    members,
  };
}

async function saveTeamLineupPreset(presetId: string | null): Promise<void> {
  const draft = collectTeamLineupPresetDraft(presetId);
  validateTeamLineupPresetDraft(draft);
  await runBusy(() => api.saveTeamLineupPreset(draft));
  closeModal();
  await reloadSelectedTeam();
  render();
  toast(presetId ? "阵容预设已更新" : "阵容预设已保存", "success");
}

function openDuplicateLineupPreset(presetId: string, sourceName: string): void {
  const body = `<label class="field"><span>新预设名称</span><input id="duplicate-lineup-preset-name" value="${escapeHtml(`${sourceName} - 副本`)}"></label>`;
  const footer = `<button class="secondary" data-action="close-workspace-detail">取消</button><button class="primary" data-action="confirm-duplicate-lineup-preset" data-preset-id="${escapeHtml(presetId)}">创建副本</button>`;
  showHtmlModal("复制阵容预设", sourceName, body, footer, "duplicate-lineup-preset-modal");
}

async function duplicateLineupPreset(presetId: string): Promise<void> {
  const name = value("duplicate-lineup-preset-name").trim();
  if (!name) throw new Error("请输入新预设名称");
  await runBusy(() => api.duplicateTeamLineupPreset(presetId, name));
  closeModal();
  await reloadSelectedTeam();
  render();
  toast("阵容预设副本已创建", "success");
}

function requestArchiveLineupPreset(presetId: string, name: string): void {
  showConfirmation(
    "归档阵容预设",
    "归档后不会出现在比赛快速套用列表中，历史数据仍保留。",
    [["预设", name]],
    "确认归档",
    async () => {
      await runBusy(() => api.archiveTeamLineupPreset(presetId));
      await reloadSelectedTeam();
      render();
      toast("阵容预设已归档", "success");
    },
  );
}

function openSaveCurrentLineupPreset(side: LineupSide): void {
  capturePairedLineupFromDom();
  const current = pairedSide(side);
  const starterCount = current.players.filter((player) => player.is_starter).length;
  if (starterCount !== 11) throw new Error(`当前阵容必须恰好有 11 名首发，当前为 ${starterCount} 名`);
  const body = `<div class="form-grid two-column clean-form">
    <label class="field"><span>预设名称</span><input id="quick-lineup-preset-name" value="${escapeHtml(`${current.team_name} 常用阵容`)}"></label>
    <label class="field"><span>适用场景</span><select id="quick-lineup-preset-context">${["general","league","cup","attacking","defensive","rotation","temporary"].map((context) => `<option value="${context}">${presetContextLabel(context)}</option>`).join("")}</select></label>
    <label class="field"><span>使用概率（0–1）</span><input id="quick-lineup-preset-probability" type="number" min="0" max="1" step="0.01"></label>
    <label class="check-row"><input id="quick-lineup-preset-default" type="checkbox"><span><strong>设为默认方案</strong><small>替换该球队现有默认预设</small></span></label>
    <label class="field span-2"><span>备注</span><textarea id="quick-lineup-preset-notes" rows="2">从 ${selectedManagedMatch()?.home_team_name ?? "比赛"} vs ${selectedManagedMatch()?.away_team_name ?? ""} 的本次阵容保存</textarea></label>
  </div>`;
  const footer = `<button class="secondary" data-action="close-workspace-detail">取消</button><button class="primary" data-action="confirm-save-current-lineup-preset" data-lineup-side="${side}">保存预设</button>`;
  showHtmlModal("保存当前阵容为预设", current.team_name, body, footer, "quick-lineup-preset-modal");
  queueMicrotask(() => enhanceSearchableSelects(document.querySelector("#modal-root") ?? document));
}

async function saveCurrentLineupAsPreset(side: LineupSide): Promise<void> {
  capturePairedLineupFromDom();
  const current = pairedSide(side);
  const draft: TeamLineupPresetDraft = {
    id: null,
    team_id: current.team_id,
    name: value("quick-lineup-preset-name").trim(),
    formation_id: current.formation_id || null,
    coach_id: current.coach_id || null,
    usage_context: value("quick-lineup-preset-context"),
    usage_probability: nullableNumber("quick-lineup-preset-probability"),
    is_default: checked("quick-lineup-preset-default"),
    source_lineup_id: null,
    notes: nullableValue("quick-lineup-preset-notes"),
    members: current.players.map(presetMemberDraftFromBuilder),
  };
  await runBusy(() => api.saveTeamLineupPreset(draft));
  pairedLineupPresets = {
    ...pairedLineupPresets,
    [side]: await api.listTeamLineupPresets(current.team_id, false),
  };
  closeModal();
  render({ preserveForm: true });
  toast("当前阵容已保存为球队预设", "success");
}

async function previewApplyLineupPreset(side: LineupSide): Promise<void> {
  capturePairedLineupFromDom();
  const presetId = nullableValue(`paired-${side}-preset`);
  if (!presetId) throw new Error("请先选择阵容预设");
  const preview = await runBusy(() => api.previewTeamLineupPresetApplication(presetId));
  const current = pairedSide(side);
  const overwriteWarning = current.players.length
    ? `<div class="blocking-note warning">当前已编辑 ${current.players.length} 名球员，套用后将替换这一侧的当前草稿。</div>`
    : "";
  const body = `<div class="lineup-preset-preview">
    <div class="lineup-preset-preview-summary"><div><span>预设</span><strong>${escapeHtml(preview.preset.name)}</strong></div><div><span>阵型</span><strong>${escapeHtml(preview.preset.formation_code ?? "未设置")}</strong></div><div><span>人员</span><strong>首发 ${preview.preset.starter_count} · 共 ${preview.preset.member_count} 人</strong></div></div>
    ${overwriteWarning}
    <div class="lineup-preset-issues">
      ${preview.blockers.map((item) => `<div class="lineup-preset-issue blocker">${escapeHtml(item)}</div>`).join("")}
      ${preview.warnings.map((item) => `<div class="lineup-preset-issue warning">${escapeHtml(item)}</div>`).join("")}
    </div>
    ${preview.blockers.length === 0 && preview.warnings.length === 0 ? `<div class="success-note">当前预设成员均仍属于该球队，没有明确伤停阻断。</div>` : ""}
    <div class="lineup-preset-preview-members">${preview.preset.members.map((member, index) => `<article class="lineup-preset-preview-member"><span>${index + 1}</span><div><strong>${escapeHtml(member.player_name)}${member.alternate_name ? `（${escapeHtml(member.alternate_name)}）` : ""}</strong><small>${member.is_starter ? "首发" : "替补"} · ${escapeHtml(member.position_code ?? "位置待补")} · ${escapeHtml(member.role_code ?? "角色待补")} · ${member.role_origin === "player_position_default" ? `资料继承${member.role_source_position_code ? `（${escapeHtml(member.role_source_position_code)}）` : ""}` : member.role_origin === "lineup_override" ? "预设覆盖" : "角色缺失"}</small></div><b>${escapeHtml(member.availability_status ?? "unknown")}</b></article>`).join("")}</div>
  </div>`;
  const footer = `<button class="secondary" data-action="close-workspace-detail">取消</button><button class="primary" data-action="confirm-apply-lineup-preset" data-lineup-side="${side}" data-preset-id="${escapeHtml(presetId)}" ${preview.can_apply ? "" : "disabled"}>确认套用</button>`;
  showHtmlModal("阵容预设套用预检", current.team_name, body, footer, "lineup-preset-preview-modal");
}

async function applyLineupPreset(side: LineupSide, presetId: string): Promise<void> {
  const preview: TeamLineupPresetApplicationPreview = await runBusy(() =>
    api.previewTeamLineupPresetApplication(presetId),
  );
  if (!preview.can_apply) throw new Error(preview.blockers.join("；") || "阵容预设当前不可用");
  const current = pairedSide(side);
  if (preview.preset.team_id !== current.team_id) throw new Error("阵容预设与当前球队不一致");
  const players: LineupBuilderPlayer[] = preview.preset.members.map((member) => ({
    player_id: member.player_id,
    player_name: member.player_name,
    player_secondary_name: member.alternate_name,
    position_code: member.position_code,
    role_code: member.role_code,
    is_starter: member.is_starter,
    expected_minutes: member.expected_minutes ?? (member.is_starter ? 90 : 20),
    shirt_number: member.shirt_number,
    bench_order: member.is_starter ? null : member.bench_order,
    starting_probability: member.is_starter ? 1 : 0,
    membership_override: false,
    availability_status: member.availability_status,
  }));
  pairedLineupBuilder = {
    ...pairedLineupBuilder,
    [side]: {
      ...current,
      formation_id: preview.preset.formation_id ?? current.formation_id,
      formation: preview.preset.formation_code ?? current.formation,
      coach_id: preview.preset.coach_id ?? current.coach_id,
      players,
    },
  };
  closeModal();
  render();
  toast(`已套用“${preview.preset.name}”，本场阵容可继续独立调整`, "success");
}

function openPairedLineupPlayerSettings(side: LineupSide, playerId: string): void {
  capturePairedLineupFromDom();
  const player = pairedSide(side).players.find((item) => item.player_id === playerId);
  if (!player) throw new Error("当前阵容中不存在该球员");
  const expectedMinutes = player.expected_minutes ?? (player.is_starter ? 90 : 20);
  const startingProbability = player.starting_probability ?? (player.is_starter ? 1 : 0);
  const candidate = pairedSide(side).candidates.find((item) => item.id === playerId);
  const inheritedRole = candidate
    ? defaultRoleForPlayer(candidate, player.position_code)
    : null;
  const roleSourceNote = inheritedRole && player.role_code === inheritedRole
    ? `当前继承球员位置档案：${inheritedRole}`
    : inheritedRole
      ? `位置档案默认：${inheritedRole}；填写不同内容将作为本场覆盖`
      : "当前位置档案没有默认角色，可在球员中心或此处补充";
  const body = `<div class="form-grid compact lineup-player-settings-form">
    <label class="field"><span>角色与职责</span><input id="lineup-player-role-code" value="${escapeHtml(player.role_code ?? inheritedRole ?? "")}" placeholder="例如：组织核心、单后腰"><small>${escapeHtml(roleSourceNote)}</small></label>
    <label class="field"><span>预计分钟</span><input id="lineup-player-expected-minutes" type="number" min="0" max="130" value="${expectedMinutes}"></label>
    <label class="field"><span>首发概率（0–1）</span><input id="lineup-player-starting-probability" type="number" min="0" max="1" step="0.01" value="${startingProbability}"></label>
    <label class="field"><span>替补顺序</span><input id="lineup-player-bench-order" type="number" min="1" value="${player.bench_order ?? ""}" ${player.is_starter ? "disabled" : ""}></label>
    <label class="field"><span>球衣号码</span><input id="lineup-player-shirt-number" type="number" min="1" max="99" value="${player.shirt_number ?? ""}"></label>
    <label class="check-row"><input id="lineup-player-membership-override" type="checkbox" ${player.membership_override ? "checked" : ""}><span><strong>允许履历例外</strong><small>仅在球员关系资料尚未补齐、但来源已核验时使用。</small></span></label>
  </div>`;
  const footer = `<button type="button" class="secondary" data-action="close-workspace-detail">取消</button><button type="button" class="primary" data-action="save-lineup-player-settings" data-lineup-side="${side}" data-player-id="${escapeHtml(playerId)}">保存球员设置</button>`;
  showHtmlModal(player.player_name, side === "home" ? "主队球员高级设置" : "客队球员高级设置", body, footer, "lineup-player-settings-modal");
}

function savePairedLineupPlayerSettings(side: LineupSide, playerId: string): void {
  const roleCode = nullableValue("lineup-player-role-code");
  const expectedMinutes = nullableNumber("lineup-player-expected-minutes");
  const startingProbability = nullableNumber("lineup-player-starting-probability");
  const benchOrder = nullableNumber("lineup-player-bench-order");
  const shirtNumber = nullableNumber("lineup-player-shirt-number");
  if (expectedMinutes !== null && (expectedMinutes < 0 || expectedMinutes > 130)) {
    throw new Error("预计分钟必须在 0 到 130 之间");
  }
  if (startingProbability !== null && (startingProbability < 0 || startingProbability > 1)) {
    throw new Error("首发概率必须在 0 到 1 之间");
  }
  if (benchOrder !== null && benchOrder < 1) throw new Error("替补顺序必须大于 0");
  if (shirtNumber !== null && (shirtNumber < 1 || shirtNumber > 99)) {
    throw new Error("球衣号码必须在 1 到 99 之间");
  }
  const membershipOverride = document.querySelector<HTMLInputElement>("#lineup-player-membership-override")?.checked ?? false;
  const current = pairedSide(side);
  const exists = current.players.some((item) => item.player_id === playerId);
  if (!exists) throw new Error("当前阵容中不存在该球员");
  pairedLineupBuilder = {
    ...pairedLineupBuilder,
    [side]: {
      ...current,
      players: current.players.map((item) => item.player_id === playerId
        ? {
            ...item,
            role_code: roleCode,
            expected_minutes: expectedMinutes,
            starting_probability: startingProbability,
            bench_order: item.is_starter ? null : benchOrder,
            shirt_number: shirtNumber,
            membership_override: membershipOverride,
          }
        : item),
    },
  };
  closeModal();
  render({ preserveForm: true });
  toast("球员高级设置已保存", "success");
}

function removePairedLineupPlayer(side: LineupSide, playerId: string): void {
  capturePairedLineupFromDom();
  const current = pairedSide(side);
  pairedLineupBuilder = {
    ...pairedLineupBuilder,
    [side]: {
      ...current,
      players: current.players.filter((item) => item.player_id !== playerId),
    },
  };
  render();
}

function clearPairedLineupSide(side: LineupSide): void {
  capturePairedLineupFromDom();
  pairedLineupBuilder = {
    ...pairedLineupBuilder,
    [side]: { ...pairedSide(side), players: [] },
  };
  render();
}

function lineupPlayersDraft(players: LineupBuilderPlayer[]): LineupPlayerDraft[] {
  return players.map((item, index) => ({
    player_id: item.player_id,
    position_code: item.position_code,
    role_code: item.role_code,
    is_starter: item.is_starter,
    shirt_number: item.shirt_number,
    expected_minutes: item.expected_minutes,
    actual_minutes: null,
    sequence_no: index + 1,
    bench_order: item.is_starter ? null : item.bench_order,
    availability_status: item.availability_status,
    starting_probability: item.starting_probability,
    membership_override: item.membership_override,
    source_urls: [],
    metadata: {},
  }));
}

function buildPairedLineupDraft(
  side: LineupSide,
  capturedAt: string,
): LineupDraft {
  const current = pairedSide(side);
  const starterCount = current.players.filter((item) => item.is_starter).length;
  if (starterCount !== 11) {
    throw new Error(`${current.team_name}必须恰好 11 名首发，当前为 ${starterCount} 名`);
  }
  if (current.players.length < 11) {
    throw new Error(`${current.team_name}阵容人数不足 11 人`);
  }
  return {
    match_id: pairedLineupBuilder.match_id,
    team_id: current.team_id,
    lineup_type: pairedLineupBuilder.lineup_type,
    snapshot_type: pairedLineupBuilder.snapshot_type,
    formation: current.formation.trim() || null,
    formation_id: current.formation_id || null,
    coach_id: current.coach_id || null,
    captured_at: capturedAt,
    source_document_id: null,
    source_urls: pairedLineupBuilder.source_urls
      .split(/[;\n,]/)
      .map((item) => item.trim())
      .filter(Boolean),
    quality_score: current.quality_score,
    metadata: { entry_mode: "paired_visual_builder", team_side: side },
    players: lineupPlayersDraft(current.players),
  };
}

async function createPairedLineups(): Promise<void> {
  capturePairedLineupFromDom();
  const match = selectedManagedMatch();
  if (!match) throw new Error("请选择比赛");
  const capturedAt = localDateTimeToIso(
    pairedLineupBuilder.captured_at || null,
    pairedLineupBuilder.snapshot_type === "T-N",
  );
  if (!capturedAt) throw new Error("请填写双方阵容的记录时间");
  const chain = await api.readMatchLineupChain(
    match.id,
    pairedLineupBuilder.snapshot_type,
  );
  const capturedTime = new Date(capturedAt).getTime();
  const cutoffTime = new Date(chain.data_cutoff_time).getTime();
  const windowStart = chain.data_window_start_time
    ? new Date(chain.data_window_start_time).getTime()
    : null;
  if (capturedTime > cutoffTime) {
    throw new Error(`记录时间不能晚于窗口截止 ${new Date(chain.data_cutoff_time).toLocaleString()}`);
  }
  if (windowStart !== null && capturedTime < windowStart) {
    throw new Error(`${pairedLineupBuilder.snapshot_type} 只能使用 ${new Date(chain.data_window_start_time ?? "").toLocaleString()} 之后的数据`);
  }
  if (
    pairedLineupBuilder.lineup_type !== "actual" &&
    capturedTime >= new Date(match.kickoff_time).getTime()
  ) {
    throw new Error("预计或确认阵容的记录时间必须早于开球时间");
  }
  const pair: LineupPairDraft = {
    home: buildPairedLineupDraft("home", capturedAt),
    away: buildPairedLineupDraft("away", capturedAt),
  };
  await runBusy(async () => {
    const created = await api.createLineupPair(pair);
    lineupRecords = await api.listLineups(null, 100);
    selectedMatchLineupChain = await api.readMatchLineupChain(
      match.id,
      pairedLineupBuilder.snapshot_type,
    );
    lastPredictionResult = created;
  });
  workspaceState.patchModule("lineups", { active_section: "chain" });
  render({ preserveForm: true });
  toast(`${match.home_team_name}与${match.away_team_name}阵容已同时提交`, "success");
}

async function inspectPairedLineupChain(): Promise<void> {
  capturePairedLineupFromDom();
  if (!pairedLineupBuilder.match_id) throw new Error("请选择比赛");
  selectedMatchLineupChain = await runBusy(() =>
    api.readMatchLineupChain(
      pairedLineupBuilder.match_id,
      pairedLineupBuilder.snapshot_type,
    ),
  );
  workspaceState.patchModule("lineups", { active_section: "chain" });
  render({ preserveForm: true });
}

function inferCompetitionTeamScope(competitionId: string | null): "national" | "club" | "all" {
  const competition = state?.data.competitions.find((item) => item.id === competitionId);
  if (!competition) return "all";
  const explicitScope = typeof competition.metadata?.scope === "string" ? competition.metadata.scope.toLowerCase() : "";
  if (explicitScope === "national" || explicitScope === "club") return explicitScope;
  const text = `${competition.name} ${competition.code}`.toLowerCase();
  if (/world cup|世界杯|euro|欧洲杯|nations|国家联赛|international|qualif|预选|copa america|美洲杯|asian cup|亚洲杯|afcon|非洲杯|gold cup|金杯|national/.test(text)) return "national";
  if (/league|联赛|cup|杯|champions|libertadores|俱乐部|superliga|premier|bundesliga|serie|liga|ligue/.test(text)) return "club";
  return "all";
}

function autoSelectMatchSeason(): void {
  const competitionId = nullableValue("new-match-competition");
  const season = document.querySelector<HTMLSelectElement>("#new-match-season");
  const kickoffRaw = nullableValue("new-match-kickoff");
  if (!competitionId || !season) return;
  const kickoffDate = kickoffRaw ? kickoffRaw.slice(0, 10) : null;
  const candidates = Array.from(season.options).filter((option) =>
    option.value && option.dataset.competitionId === competitionId,
  );
  const matched = kickoffDate
    ? candidates.find((option) => {
        const starts = option.dataset.startsOn || null;
        const ends = option.dataset.endsOn || null;
        return (!starts || starts <= kickoffDate) && (!ends || ends >= kickoffDate);
      })
    : null;
  if (!season.value || season.selectedOptions[0]?.hidden) {
    season.value = matched?.value ?? candidates[0]?.value ?? "";
  }
}

function updateCompetitionHierarchy(source: "scope" | "region" | "competition" | "init" = "init"): void {
  const scope = document.querySelector<HTMLSelectElement>("#new-match-competition-scope");
  const region = document.querySelector<HTMLSelectElement>("#new-match-competition-region");
  const competition = document.querySelector<HTMLSelectElement>("#new-match-competition");
  if (!scope || !region || !competition) return;

  const competitionOptions = Array.from(competition.options).filter((option) => option.value);
  const selectedScope = scope.value;
  if (source === "scope") {
    region.value = "";
    competition.value = "";
  } else if (source === "region") {
    competition.value = "";
  }

  const allowedRegions = new Set(
    competitionOptions
      .filter((option) => !selectedScope || option.dataset.scope === selectedScope)
      .map((option) => option.dataset.region ?? ""),
  );
  for (const option of Array.from(region.options)) {
    if (!option.value) continue;
    const allowed = allowedRegions.has(option.value);
    option.hidden = !allowed;
    option.disabled = !allowed;
  }
  if (region.selectedOptions[0]?.disabled) region.value = "";

  region.disabled = !selectedScope;
  const regionPlaceholder = region.options[0];
  if (regionPlaceholder) regionPlaceholder.textContent = selectedScope ? "选择地区或足联" : "先选择参赛体系";

  const selectedRegion = region.value;
  competition.disabled = !selectedScope || !selectedRegion;
  const competitionPlaceholder = competition.options[0];
  if (competitionPlaceholder) competitionPlaceholder.textContent = selectedRegion ? "选择具体赛事" : "先选择地区或足联";
  for (const option of competitionOptions) {
    const allowed =
      (!selectedScope || option.dataset.scope === selectedScope) &&
      (!selectedRegion || option.dataset.region === selectedRegion);
    option.hidden = !allowed;
    option.disabled = !allowed;
  }
  if (competition.selectedOptions[0]?.disabled) competition.value = "";

  if (source !== "competition") {
    filterSelectOptions("new-match-season", "competitionId", competition.value || null);
    filterSelectOptions("new-match-stage", "competitionId", competition.value || null);
    autoSelectMatchSeason();
    filterMatchTeamOptions();
  }
  refreshSearchableSelects(app);
}

function updateFormationHierarchy(
  side: LineupSide,
  source: "level1" | "level2" | "formation" | "init" = "init",
): void {
  const level1 = document.querySelector<HTMLSelectElement>(`#paired-${side}-formation-level1`);
  const level2 = document.querySelector<HTMLSelectElement>(`#paired-${side}-formation-level2`);
  const formation = document.querySelector<HTMLSelectElement>(`#paired-${side}-formation-id`);
  const hiddenCode = document.querySelector<HTMLInputElement>(`#paired-${side}-formation`);
  if (!level1 || !level2 || !formation) return;

  if (source === "level1") {
    level2.value = "";
    formation.value = "";
  } else if (source === "level2") {
    formation.value = "";
  }

  for (const option of Array.from(level2.options)) {
    if (!option.value) continue;
    const allowed = !level1.value || option.dataset.level1 === level1.value;
    option.hidden = !allowed;
    option.disabled = !allowed;
  }
  if (level2.selectedOptions[0]?.disabled) level2.value = "";

  for (const option of Array.from(formation.options)) {
    if (!option.value) continue;
    const allowed =
      (!level1.value || option.dataset.level1 === level1.value) &&
      (!level2.value || option.dataset.level2 === level2.value);
    option.hidden = !allowed;
    option.disabled = !allowed;
  }
  if (formation.selectedOptions[0]?.disabled) formation.value = "";
  const selected = formation.selectedOptions[0];
  if (hiddenCode && selected?.value) hiddenCode.value = selected.dataset.code ?? "";
  if (source !== "init") {
    capturePairedLineupFromDom();
    selectedMatchLineupChain = null;
  }
}

function filterMatchTeamOptions(): void {
  const competitionId = nullableValue("new-match-competition");
  const seasonId = nullableValue("new-match-season");
  const scopeSelect = document.querySelector<HTMLSelectElement>("#new-match-team-scope");
  const requested = scopeSelect?.value ?? "auto";
  const scope = requested === "auto" ? inferCompetitionTeamScope(competitionId) : requested;
  const registered = new Set(
    (playerReferences?.season_team_memberships ?? [])
      .filter((item) => item.season_id === seasonId && ["registered", "guest"].includes(item.registration_status))
      .map((item) => item.team_id),
  );
  const restrictMembership = Boolean(seasonId && registered.size > 0);
  for (const selectId of ["new-match-home-team", "new-match-away-team"]) {
    const select = document.querySelector<HTMLSelectElement>(`#${selectId}`);
    if (!select) continue;
    for (const option of Array.from(select.options)) {
      if (!option.value) {
        option.hidden = false;
        continue;
      }
      const typeMatches = scope === "all" || option.dataset.teamType === scope;
      const memberMatches = !restrictMembership || registered.has(option.value);
      option.hidden = !option.selected && !(typeMatches && memberMatches);
    }
    if (select.selectedOptions[0]?.hidden) select.value = "";
  }
  const note = document.querySelector<HTMLElement>("#match-team-filter-note");
  if (note) {
    note.textContent = restrictMembership
      ? `当前赛季已注册 ${registered.size} 支球队，主客队仅从注册名单选择。`
      : scope === "national"
        ? "当前赛事按国家队体系筛选。"
        : scope === "club"
          ? "当前赛事按俱乐部体系筛选。"
          : "当前赛事未限定球队体系。";
  }
}

function filterManagedMatchList(search: string): void {
  const normalized = search.trim().toLowerCase();
  for (const item of Array.from(document.querySelectorAll<HTMLElement>(".match-list-item"))) {
    item.hidden = normalized.length > 0 && !(item.textContent ?? "").toLowerCase().includes(normalized);
  }
}

async function createMatch(): Promise<void> {
  autoSelectMatchSeason();
  const competitionId = nullableValue("new-match-competition");
  const seasonId = nullableValue("new-match-season");
  const homeTeamId = nullableValue("new-match-home-team");
  const awayTeamId = nullableValue("new-match-away-team");
  const kickoff = localDateTimeToIso(nullableValue("new-match-kickoff"));
  if (!competitionId) throw new Error("请先选择赛事；缺少赛事时可跳转赛事设置补充后返回");
  if (!homeTeamId || !awayTeamId || !kickoff) {
    throw new Error("请选择主客队并填写开球时间");
  }
  if (homeTeamId === awayTeamId) throw new Error("主队和客队不能相同");
  const existingExternalKey = nullableValue("managed-match-external-key");
  const draft: MatchDraft = {
    external_key: existingExternalKey ?? `manual:${competitionId}:${homeTeamId}:${awayTeamId}:${kickoff}`,
    competition_id: competitionId,
    season_id: seasonId,
    stage_id: nullableValue("new-match-stage"),
    round_id: nullableValue("new-match-round"),
    home_team_id: homeTeamId,
    away_team_id: awayTeamId,
    kickoff_time: kickoff,
    status: value("new-match-status") as MatchDraft["status"],
    venue: nullableValue("new-match-venue"),
    metadata: { entry_mode: "match_center_hierarchy" },
  };
  const created = await runBusy(async () => {
    const saved = await api.createMatch(draft);
    state = await api.bootstrap();
    playerReferences = await api.playerCatalogReferenceData();
    lineupRecords = await api.listLineups(null, 100);
    return saved;
  });
  selectedManagedMatchId = created.id;
  render({ preserveForm: true });
  toast(existingExternalKey ? "比赛修改已保存" : "比赛已创建", "success");
}

function requestRemoveLineupHistory(lineupId: string, lineupLabel: string): void {
  if (!lineupId) throw new Error("缺少阵容版本编号");
  showConfirmation(
    "删除阵容历史",
    "未被正式推演、快照或球员贡献引用的版本会永久删除；已被引用的版本会归档并从历史列表隐藏，模型血缘继续保留。",
    [
      ["阵容", lineupLabel],
      ["活动版本", "删除后自动恢复上一有效版本（如存在）"],
    ],
    "确认处理阵容版本",
    async () => {
      const result = await runBusy(() =>
        api.removeLineupHistory(lineupId, "用户从阵容历史删除"),
      );
      lineupRecords = await api.listLineups(null, 100);
      if (pairedLineupBuilder.match_id) {
        selectedMatchLineupChain = await api.readMatchLineupChain(
          pairedLineupBuilder.match_id,
          pairedLineupBuilder.snapshot_type,
        );
      }
      render();
      toast(
        result.removal_mode === "deleted"
          ? "未引用阵容版本已永久删除"
          : "已引用阵容版本已归档并隐藏",
        "success",
      );
    },
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
      if (state) state.data.recent_runs = await api.listRecentRuns(100);
      render();
      toast("推演记录已从历史列表移除", "success");
    },
  );
}

function requestDeleteMatch(matchId: string, matchLabel: string): void {
  showConfirmation(
    "删除比赛",
    "未进入P4研究、冻结或正式赛后结算的比赛可以永久删除；阵容、赛果和普通复盘会一并删除，模型运行与快照保留但解除比赛关联。已有不可变P4血缘时系统会拒绝删除。",
    [
      ["比赛", matchLabel],
      ["保护", "P4研究、冻结或正式赛后结算存在时禁止永久删除"],
      ["保留", "普通模型运行和特征快照保留为历史记录"],
    ],
    "永久删除比赛",
    async () => {
      await runBusy(() => api.deleteMatch(matchId));
      lineupBuilderForm = { ...lineupBuilderForm, match_id: "", team_id: "" };
      lineupBuilderPlayers = [];
      if (pairedLineupBuilder.match_id === matchId) {
        pairedLineupBuilder = {
          ...pairedLineupBuilder,
          match_id: "",
          home: emptyPairedLineupSide(),
          away: emptyPairedLineupSide(),
        };
      }
      if (selectedManagedMatchId === matchId) selectedManagedMatchId = null;
      await refreshBootstrapAndCatalog();
      lineupRecords = await api.listLineups(null, 100);
      render();
      toast("比赛及关联阵容已删除", "success");
    },
  );
}

interface CaptureLineupFormOptions {
  readonly allowBlankIdentity?: boolean;
}

function captureLineupFormFromDom(
  options: CaptureLineupFormOptions = {},
): void {
  const match = document.querySelector<HTMLSelectElement>("#new-lineup-match");
  const team = document.querySelector<HTMLSelectElement>("#new-lineup-team");
  const type = document.querySelector<HTMLSelectElement>("#new-lineup-type");
  const snapshot = document.querySelector<HTMLSelectElement>("#new-lineup-snapshot");
  const coach = document.querySelector<HTMLSelectElement | HTMLInputElement>("#new-lineup-coach");
  const sourceUrls = document.querySelector<HTMLInputElement>("#new-lineup-source-urls");
  const formation = document.querySelector<HTMLInputElement>("#new-lineup-formation");
  const formationSelect = document.querySelector<HTMLSelectElement>("#new-lineup-formation-id");
  const captured = document.querySelector<HTMLInputElement>(
    "#new-lineup-captured-at",
  );
  const quality = document.querySelector<HTMLInputElement>(
    "#new-lineup-quality",
  );
  if (!match || !team || !type || !snapshot || !formation || !captured || !quality) return;
  const selectedFormation = formationSelect?.selectedOptions[0];
  const formationCode = selectedFormation?.dataset.code ?? formation.value;
  const preserveIdentity = options.allowBlankIdentity !== true;
  const matchId =
    match.value || (preserveIdentity ? lineupBuilderForm.match_id : "");
  const teamId =
    team.value || (preserveIdentity ? lineupBuilderForm.team_id : "");
  lineupBuilderForm = {
    match_id: matchId,
    team_id: teamId,
    lineup_type: type.value as LineupBuilderFormState["lineup_type"],
    snapshot_type: snapshot.value as LineupBuilderFormState["snapshot_type"],
    formation_id: formationSelect?.value ?? "",
    formation: formationCode,
    coach_id: coach?.value ?? "",
    source_urls: sourceUrls?.value ?? "",
    captured_at: captured.value,
    quality_score: Number(quality.value || 0.8),
  };
}

function captureLineupBuilderFromDom(): void {
  const rows = Array.from(
    document.querySelectorAll<HTMLElement>("[data-lineup-builder-row]:not([data-lineup-side])"),
  );
  if (rows.length === 0) return;
  lineupBuilderPlayers = rows.map((row) => {
    const playerId = row.dataset.playerId ?? "";
    const existing = lineupBuilderPlayers.find(
      (item) => item.player_id === playerId,
    );
    if (!existing) throw new Error("阵容球员状态丢失，请重新加载球队名单");
    const read = (field: string): string =>
      row.querySelector<HTMLInputElement | HTMLSelectElement>(
        `[data-lineup-field="${field}"]`,
      )?.value ?? "";
    const minutes = read("expected_minutes").trim();
    const shirt = read("shirt_number").trim();
    const benchOrder = read("bench_order").trim();
    const startingProbability = read("starting_probability").trim();
    const membershipOverride = row.querySelector<HTMLInputElement>(
      '[data-lineup-field="membership_override"]',
    )?.checked ?? false;
    const positionCode = read("position_code") || null;
    const currentRole = read("role_code") || null;
    const candidate = lineupPlayerCandidates.find((item) => item.id === playerId);
    const previousDefaultRole = candidate
      ? defaultRoleForPlayer(candidate, existing.position_code)
      : null;
    const nextDefaultRole = candidate
      ? defaultRoleForPlayer(candidate, positionCode)
      : null;
    const roleCode = !currentRole || currentRole === previousDefaultRole
      ? nextDefaultRole
      : currentRole;
    return {
      ...existing,
      is_starter: read("is_starter") === "true",
      position_code: positionCode,
      role_code: roleCode,
      expected_minutes: minutes ? Number(minutes) : null,
      shirt_number: shirt ? Number(shirt) : null,
      bench_order: benchOrder ? Number(benchOrder) : null,
      starting_probability: startingProbability ? Number(startingProbability) : null,
      membership_override: membershipOverride,
    };
  });
}

async function loadLineupPlayersForTeam(teamId: string): Promise<number> {
  const requestSequence = ++lineupPlayerLoadSequence;
  const result = await api.listPlayers({
    search: null,
    team_id: teamId,
    position_code: null,
    availability_status: null,
    player_status: "active",
    limit: 200,
    cursor_name: null,
    cursor_id: null,
  });
  if (
    requestSequence !== lineupPlayerLoadSequence ||
    lineupBuilderForm.team_id !== teamId
  ) {
    return 0;
  }
  lineupPlayerCandidates = result.items;
  lineupBuilderPlayers = lineupBuilderPlayers.filter((item) =>
    result.items.some((candidate) => candidate.id === item.player_id),
  );
  return result.items.length;
}

async function autoLoadLineupPlayers(teamId: string): Promise<void> {
  const teamName =
    playerReferences?.teams.find((team) => team.id === teamId)?.canonical_name ??
    "所选球队";
  const traceId = recordUiOperation(
    "ui_action",
    "action:auto-load-lineup-players",
    { page, teamId, teamName },
  );
  try {
    const count = await runBusy(() => loadLineupPlayersForTeam(teamId));
    if (lineupBuilderForm.team_id !== teamId) return;
    render();
    toast(
      count > 0
        ? `${teamName}已自动加载 ${count} 名球员`
        : `${teamName}已完成查询，但没有找到有效球员`,
      count > 0 ? "success" : "normal",
    );
  } catch (error) {
    recordUiOperationFailure(
      "action:auto-load-lineup-players",
      error,
      traceId,
      { page, teamId, teamName },
    );
    recordClientIssue(error, `${pageTitle(page)} / 自动加载球队名单`);
    toast(userFacingError(error), "error");
  }
}

async function loadLineupPlayers(): Promise<void> {
  captureLineupFormFromDom();
  const teamId = lineupBuilderForm.team_id || null;
  if (!teamId) throw new Error("请先选择球队");
  captureLineupBuilderFromDom();
  const count = await runBusy(() => loadLineupPlayersForTeam(teamId));
  render();
  toast(`已加载 ${count} 名球员`, "success");
}

function addLineupPlayer(playerId: string, starter: boolean): void {
  captureLineupFormFromDom();
  captureLineupBuilderFromDom();
  if (lineupBuilderPlayers.some((item) => item.player_id === playerId)) return;
  const player = lineupPlayerCandidates.find((item) => item.id === playerId);
  if (!player) throw new Error("球员不在当前球队名单中");
  if (
    starter &&
    lineupBuilderPlayers.filter((item) => item.is_starter).length >= 11
  ) {
    throw new Error("首发人数不能超过 11 人");
  }
  const playerName = displayPlayerName(player);
  lineupBuilderPlayers.push({
    player_id: player.id,
    player_name: playerName.primary,
    player_secondary_name: playerName.secondary,
    position_code: player.primary_position_code,
    role_code: defaultRoleForPlayer(player, player.primary_position_code),
    is_starter: starter,
    expected_minutes: starter ? 90 : 20,
    shirt_number: null,
    bench_order: starter ? null : lineupBuilderPlayers.filter((item) => !item.is_starter).length + 1,
    starting_probability: starter ? 1 : 0,
    membership_override: false,
    availability_status: player.availability_status,
  });
  render();
}

function removeLineupPlayer(playerId: string): void {
  captureLineupFormFromDom();
  captureLineupBuilderFromDom();
  lineupBuilderPlayers = lineupBuilderPlayers.filter(
    (item) => item.player_id !== playerId,
  );
  render();
}

async function inspectLineupChain(): Promise<void> {
  captureLineupFormFromDom();
  if (!lineupBuilderForm.match_id) throw new Error("请选择比赛");
  selectedMatchLineupChain = await runBusy(() =>
    api.readMatchLineupChain(lineupBuilderForm.match_id, lineupBuilderForm.snapshot_type),
  );
  render();
}

async function createLineup(): Promise<void> {
  captureLineupFormFromDom();
  captureLineupBuilderFromDom();
  const matchId = lineupBuilderForm.match_id || null;
  const teamId = lineupBuilderForm.team_id || null;
  if (!matchId || !teamId) throw new Error("请选择比赛和参赛队");
  if (lineupBuilderPlayers.length === 0)
    throw new Error("请先从球队名单加入球员");
  const starterCount = lineupBuilderPlayers.filter(
    (item) => item.is_starter,
  ).length;
  if (starterCount !== 11) throw new Error(`正式阵容必须恰好 11 名首发，当前为 ${starterCount} 名`);
  const capturedAt = localDateTimeToIso(
    lineupBuilderForm.captured_at || null,
    lineupBuilderForm.snapshot_type === "T-N",
  );
  if (!capturedAt) {
    throw new Error("固定时点阵容必须填写不晚于模型截止时间的历史记录时间");
  }
  const activeChain =
    selectedMatchLineupChain?.match_record.id === matchId &&
    selectedMatchLineupChain.snapshot_type === lineupBuilderForm.snapshot_type
      ? selectedMatchLineupChain
      : null;
  if (
    activeChain &&
    new Date(capturedAt).getTime() >
      new Date(activeChain.data_cutoff_time).getTime()
  ) {
    throw new Error(
      `${lineupBuilderForm.snapshot_type} 阵容记录时间不能晚于模型截止时间 ${new Date(activeChain.data_cutoff_time).toLocaleString()}`,
    );
  }
  const selectedMatch =
    playerReferences?.managed_matches.find((match) => match.id === matchId) ??
    playerReferences?.upcoming_matches.find((match) => match.id === matchId);
  if (
    selectedMatch &&
    lineupBuilderForm.lineup_type !== "actual" &&
    new Date(capturedAt).getTime() >=
      new Date(selectedMatch.kickoff_time).getTime()
  ) {
    throw new Error("预计或确认阵容的记录时间必须早于开球时间");
  }
  const players: LineupPlayerDraft[] = lineupBuilderPlayers.map(
    (item, index) => ({
      player_id: item.player_id,
      position_code: item.position_code,
      role_code: item.role_code,
      is_starter: item.is_starter,
      shirt_number: item.shirt_number,
      expected_minutes: item.expected_minutes,
      actual_minutes: null,
      sequence_no: index + 1,
      bench_order: item.is_starter ? null : item.bench_order,
      availability_status: item.availability_status,
      starting_probability: item.starting_probability,
      membership_override: item.membership_override,
      source_urls: [],
      metadata: {},
    }),
  );
  const draft: LineupDraft = {
    match_id: matchId,
    team_id: teamId,
    lineup_type: lineupBuilderForm.lineup_type,
    snapshot_type: lineupBuilderForm.snapshot_type,
    formation: lineupBuilderForm.formation.trim() || null,
    formation_id: lineupBuilderForm.formation_id || null,
    coach_id: lineupBuilderForm.coach_id || null,
    captured_at: capturedAt,
    source_document_id: null,
    source_urls: lineupBuilderForm.source_urls.split(/[;\n,]/).map((item) => item.trim()).filter(Boolean),
    quality_score: lineupBuilderForm.quality_score,
    metadata: { entry_mode: "visual_builder" },
    players,
  };
  const snapshotType = lineupBuilderForm.snapshot_type;
  await runBusy(async () => {
    const created = await api.createLineup(draft);
    lineupRecords = await api.listLineups(null, 100);
    selectedMatchLineupChain = await api.readMatchLineupChain(
      matchId,
      snapshotType,
    );
    lastPredictionResult = created;
  });
  lineupBuilderPlayers = [];
  lineupPlayerCandidates = [];
  if (page === "review" && selectedReviewMatchId) {
    await loadReviewLineups(selectedReviewMatchId);
    lineupBuilderForm = {
      ...lineupBuilderForm,
      match_id: selectedReviewMatchId,
      team_id: "",
      lineup_type: "actual",
    };
    render();
    toast("阵容已保存，可继续补录另一队或生成复盘", "success");
    return;
  }
  const chain = selectedMatchLineupChain;
  if (!chain) throw new Error("阵容已保存，但未能读取模型链路");
  if (chain.ready_for_model) {
    workspaceState.patchModule("lineups", { active_section: "chain" });
    render();
    toast("双方阵容已保存并通过门禁，可以返回正式推演", "success");
    return;
  }
  const target = missingLineupTeam(chain);
  lineupBuilderForm = {
    ...lineupBuilderForm,
    match_id: matchId,
    team_id: target.team_id,
    lineup_type: "expected",
    snapshot_type: snapshotType,
    captured_at: localDateTimeInputValue(chain.data_cutoff_time),
  };
  const count = await runBusy(() => loadLineupPlayersForTeam(target.team_id));
  workspaceState.patchModule("lineups", {
    active_section: "builder",
    controls: {},
  });
  render();
  toast(
    `当前阵容已保存；继续补齐${target.team_side === "home" ? "主队" : "客队"}${target.team_name}（已加载 ${count} 名球员）`,
    "success",
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
  lineupBuilderForm = {
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
  lineupBuilderPlayers = [];
  lineupPlayerCandidates = [];
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
  if (page === "analytics") await loadAnalysisCenter();
  state = await api.bootstrap();
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
  window.localStorage.setItem(ANALYSIS_PACKAGE_ID_KEY, summary.package_id);
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
  state = await api.bootstrap();
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
  const sequence = ++openAiApiExampleSequence;
  const protocol = value("openai-api-protocol") as OpenAiApiProtocol;
  setOpenAiApiExampleStatus(
    "parsing",
    "正在解析",
    "正在识别 curl、请求体和协议……",
  );
  try {
    const result = await api.parseOpenAiApiExample(example, protocol);
    if (sequence !== openAiApiExampleSequence) return false;
    applyOpenAiApiExample(result);
    openAiApiExampleLastParsed = example;
    return true;
  } catch (error: unknown) {
    if (sequence !== openAiApiExampleSequence) return false;
    setOpenAiApiExampleStatus("error", "无法解析", userFacingError(error));
    return false;
  }
}

function scheduleOpenAiApiExampleParse(): void {
  if (openAiApiExampleTimer !== null)
    window.clearTimeout(openAiApiExampleTimer);
  openAiApiExampleTimer = window.setTimeout(() => {
    openAiApiExampleTimer = null;
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
    if (openAiApiExampleTimer !== null) {
      window.clearTimeout(openAiApiExampleTimer);
      openAiApiExampleTimer = null;
    }
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
  switch (action) {
    case "toggle-global-sidebar":
      workspaceState.setSidebarCollapsed(!workspaceState.sidebarCollapsed());
      render({ preserveForm: true });
      break;
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
      render({ preserveForm: true });
      break;
    case "select-workspace-section": {
      const sectionId = button.dataset.sectionId;
      if (!sectionId) break;
      workspaceState.patchModule(page, { active_section: sectionId });
      render({ preserveForm: true });
      break;
    }
    case "jump-workspace-anchor": {
      const anchorId = button.dataset.anchorId;
      if (!anchorId) break;
      const target = document.getElementById(anchorId);
      if (!target) break;
      if (target instanceof HTMLDetailsElement) target.open = true;
      const nav = button.closest<HTMLElement>(".workspace-anchor-nav, .workspace-task-anchor-nav");
      nav?.querySelectorAll<HTMLElement>("button[data-anchor-id]").forEach((item) => item.classList.toggle("active", item === button));
      target.scrollIntoView({ behavior: "smooth", block: "start" });
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
    case "reset-current-workspace":
      resetCurrentWorkspace();
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
    case "toggle-theme":
      toggleTheme();
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
      await navigator.clipboard.writeText(content);
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
      await connectDatabase();
      break;
    case "disconnect-database":
      await runBusy(() => api.disconnectDatabase());
      await refresh();
      toast("数据库连接配置已清除", "success");
      break;
    case "request-reset-database":
      requestDatabaseReset();
      break;
    case "execute-database-reset":
      await executeDatabaseReset();
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
      await createCompetition();
      break;
    case "show-competition-path":
      showCompetitionPath(button.dataset.competitionId ?? "");
      break;
    case "request-delete-competition":
      requestDeleteCompetition(
        button.dataset.competitionId ?? "",
        button.dataset.competitionName ?? "未命名赛事",
      );
      break;
    case "create-season":
      await createSeason();
      break;
    case "create-stage":
      await createStage();
      break;
    case "create-round":
      await createRound();
      break;
    case "register-rule-package":
      await registerRulePackage();
      break;
    case "clear-rule-package":
      pendingRulePackage = null;
      render();
      break;
    case "show-pending-rule-package":
      if (pendingRulePackage) showModal("规则包只读详情", pendingRulePackage);
      break;
    case "create-binding":
      await createBinding();
      break;
    case "refresh-player-catalog":
      await runBusy(() => loadPlayerCatalog(true));
      render();
      break;
    case "refresh-team-catalog":
      await runBusy(() => loadTeamCatalog(true));
      render();
      break;
    case "refresh-lineup-preset-page":
      await refreshLineupPresetPage();
      break;
    case "select-lineup-preset-team":
      await selectLineupPresetTeam(button.dataset.teamId ?? "");
      break;
    case "search-teams":
      await searchTeams();
      break;
    case "search-teams-from-detail":
      await searchTeams(true);
      break;
    case "clear-team-filters":
      await clearTeamFilters();
      break;
    case "clear-team-filters-from-detail":
      await clearTeamFilters(true);
      break;
    case "next-team-page":
      await nextTeamPage();
      break;
    case "previous-team-page":
      await previousTeamPage();
      break;
    case "open-team":
      await openTeam(button.dataset.teamId ?? "");
      break;
    case "return-team-directory":
      selectedTeam = null;
      selectedPlayer = null;
      selectedTeamLineupHistory = [];
      selectedTeamLineupPresets = [];
      workspaceState.patchModule("teams", { active_tab_id: null, active_section: "directory" });
      render({ preserveForm: true });
      break;
    case "update-team":
      await updateTeam(button.dataset.teamId ?? "");
      break;
    case "add-team-name":
      await addTeamName(button.dataset.teamId ?? "");
      break;
    case "save-team-profile":
      await saveTeamProfile(button.dataset.teamId ?? "");
      break;
    case "save-formation-usage":
      await saveFormationUsage(button.dataset.teamId ?? "");
      break;
    case "export-team-package-template":
      await exportTeamPackageTemplate();
      break;
    case "preview-team-package-import":
      await previewTeamPackageImport();
      break;
    case "resolve-team-package-conflict":
      await resolveTeamPackageConflict(
        button.dataset.packageScope === "player" ? "player" : "team",
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-team-package-conflict":
      await resolveTeamPackageConflict(
        button.dataset.packageScope === "player" ? "player" : "team",
        button.dataset.rowId ?? "",
        null,
        true,
      );
      break;
    case "commit-team-package-import":
      await commitTeamPackageImport();
      break;
    case "show-team-package-preview-json":
      if (teamPackagePreview)
        showModal(
          "球队完整资料包预检结果",
          teamPackagePreview,
          `<button type="button" class="secondary" data-action="close-workspace-detail">关闭</button><button type="button" class="primary" data-action="export-team-package-preview-json">导出完整预检 JSON</button>`,
        );
      break;
    case "export-team-package-preview-json":
      await exportTeamPackagePreviewJson();
      break;
    case "export-team-template":
      await exportTeamTemplate();
      break;
    case "export-team-data":
      await exportTeamData();
      break;
    case "preview-team-import":
      await previewTeamImport();
      break;
    case "resolve-team-import-conflict":
      await resolveTeamImportConflict(
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-team-import-conflict":
      await resolveTeamImportConflict(button.dataset.rowId ?? "", null, true);
      break;
    case "commit-team-import":
      await commitTeamImport();
      break;
    case "show-team-import-preview-json":
      if (teamSpreadsheetPreview)
        showModal("球队月度工作簿预检结果", teamSpreadsheetPreview);
      break;
    case "create-coach":
      await createCoach();
      break;
    case "add-team-coach-period":
      await addTeamCoachPeriod(button.dataset.teamId ?? "");
      break;
    case "bulk-archive-teams":
      await bulkArchiveEntities("team");
      break;
    case "bulk-archive-players":
      await bulkArchiveEntities("player");
      break;
    case "bulk-delete-teams":
      await requestBulkDeleteTeams();
      break;
    case "request-force-delete-team":
      await requestForceDeleteTeam(button.dataset.teamId ?? "");
      break;
    case "confirm-force-delete-team":
      await confirmForceDeleteTeam();
      break;
    case "bulk-delete-players":
      requestBulkDeletePlayers();
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
      await searchTeamOptions();
      break;
    case "search-players":
      await searchPlayers();
      break;
    case "clear-player-filters":
      await clearPlayerFilters();
      break;
    case "next-player-page":
      await nextPlayerPage();
      break;
    case "previous-player-page":
      await previousPlayerPage();
      break;
    case "open-player":
      await openPlayer(button.dataset.playerId ?? "");
      break;
    case "open-player-profile":
      await openPlayerProfile(button.dataset.playerId ?? "");
      break;
    case "open-team-profile":
      await openTeamProfile(button.dataset.teamId ?? "");
      break;
    case "show-player-json":
      if (selectedPlayer) showModal("球员完整档案", selectedPlayer);
      break;
    case "create-team":
      await createTeam();
      break;
    case "create-provider":
      await createProvider();
      break;
    case "create-player":
      await createPlayer();
      break;
    case "create-lineup-player":
      await createLineupPlayerQuick();
      break;
    case "update-player":
      await updatePlayer(button.dataset.playerId ?? "");
      break;
    case "request-delete-player":
      await requestDeletePlayer(
        button.dataset.playerId ?? "",
        button.dataset.playerName ?? "未命名球员",
      );
      break;
    case "export-player-template":
      await exportPlayerTemplate();
      break;
    case "export-player-data":
      await exportPlayerData();
      break;
    case "preview-player-import":
      await previewPlayerImport();
      break;
    case "resolve-import-conflict":
      await resolvePlayerImportConflict(
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-import-conflict":
      await resolvePlayerImportConflict(button.dataset.rowId ?? "", null, true);
      break;
    case "commit-player-import":
      await commitPlayerImport();
      break;
    case "show-import-preview-json":
      if (spreadsheetPreview) showModal("表格导入检查结果", spreadsheetPreview);
      break;
    case "add-player-name":
      await addPlayerName(button.dataset.playerId ?? "");
      break;
    case "assign-player-position":
      await assignPlayerPosition(button.dataset.playerId ?? "");
      break;
    case "add-player-team-period":
      await addPlayerTeamPeriod(button.dataset.playerId ?? "");
      break;
    case "add-player-availability":
      await addPlayerAvailability(button.dataset.playerId ?? "");
      break;
    case "add-player-ability":
      await addPlayerAbility(button.dataset.playerId ?? "");
      break;
    case "add-player-external-id":
      await addPlayerExternalId(button.dataset.playerId ?? "");
      break;
    case "add-player-dynamic-tag":
      await addPlayerDynamicTag(button.dataset.playerId ?? "");
      break;
    case "calculate-player-contribution":
      await calculatePlayerContribution(button.dataset.playerId ?? "");
      break;
    case "export-match-template":
      await exportMatchTemplate();
      break;
    case "export-match-data":
      await exportMatchData();
      break;
    case "export-ai-match-package":
      await exportAiMatchPackage();
      break;
    case "calculate-stored-match":
      await calculateStoredMatch();
      break;
    case "preview-match-import":
      await previewMatchImport(false);
      break;
    case "preview-ai-match-import":
      await previewMatchImport(true);
      break;
    case "resolve-match-import-conflict":
      await resolveMatchImportConflict(
        button.dataset.rowId ?? "",
        button.dataset.entityId ?? null,
        false,
      );
      break;
    case "skip-match-import-conflict":
      await resolveMatchImportConflict(button.dataset.rowId ?? "", null, true);
      break;
    case "commit-match-import":
      await commitMatchImport();
      break;
    case "show-match-import-json":
      if (matchSpreadsheetPreview)
        showModal("比赛与阵容导入预检", matchSpreadsheetPreview);
      break;
    case "select-managed-match": {
      const matchId = button.dataset.matchId ?? "";
      selectedManagedMatchId = matchId || null;
      render({ preserveForm: true });
      queueMicrotask(() => {
        autoSelectMatchSeason();
        filterMatchTeamOptions();
      });
      break;
    }
    case "new-managed-match":
      selectedManagedMatchId = null;
      render({ preserveForm: true });
      break;
    case "open-match-lineups": {
      const matchId = button.dataset.matchId ?? "";
      resetPairedBuilderForMatch(matchId, false);
      workspaceState.patchModule("lineups", { active_section: "builder" });
      await runBusy(loadBothPairedLineupSides);
      render({ preserveForm: true });
      break;
    }
    case "open-match-prediction": {
      const matchId = button.dataset.matchId ?? "";
      selectedP4MatchId = matchId;
      selectedPredictionSnapshot = pairedLineupBuilder.match_id === matchId
        ? pairedLineupBuilder.snapshot_type
        : "T-N";
      workspaceState.patchModule("prediction", { active_section: "formal" });
      await navigateTo("prediction");
      break;
    }
    case "create-match":
      await createMatch();
      break;
    case "request-delete-match":
      requestDeleteMatch(
        button.dataset.matchId ?? "",
        button.dataset.matchLabel ?? "未命名比赛",
      );
      break;
    case "reload-paired-lineup-side": {
      const side = button.dataset.lineupSide as LineupSide;
      const count = await runBusy(() => loadPairedLineupSide(side));
      render();
      toast(`${pairedSide(side).team_name}已加载 ${count} 名球员`, count > 0 ? "success" : "normal");
      break;
    }
    case "clear-paired-lineup-side":
      clearPairedLineupSide(button.dataset.lineupSide as LineupSide);
      break;
    case "open-team-lineup-preset-manager":
      await openTeamLineupPresetManager(
        button.dataset.teamId ?? selectedTeam?.team.id ?? "",
        button.dataset.teamName ?? selectedTeam?.team.canonical_name ?? "球队",
      );
      break;
    case "open-lineup-preset-manager":
      await openLineupPresetManagerForSide(button.dataset.lineupSide as LineupSide);
      break;
    case "request-delete-team-lineup-preset":
      requestDeleteTeamLineupPreset(
        button.dataset.presetId ?? "",
        button.dataset.presetName ?? "阵容预设",
        button.dataset.teamId ?? "",
        button.dataset.teamName ?? "球队",
        button.dataset.presetStatus ?? "active",
        Number(button.dataset.memberCount ?? 0),
      );
      break;
    case "open-team-lineup-preset-editor":
      openTeamLineupPresetEditor(button.dataset.presetId || null);
      break;
    case "auto-assign-preset-formation":
      assignPresetFormationSlots();
      break;
    case "save-team-lineup-preset":
      await saveTeamLineupPreset(button.dataset.presetId || null);
      break;
    case "duplicate-team-lineup-preset":
      openDuplicateLineupPreset(
        button.dataset.presetId ?? "",
        button.dataset.presetName ?? "阵容预设",
      );
      break;
    case "confirm-duplicate-lineup-preset":
      await duplicateLineupPreset(button.dataset.presetId ?? "");
      break;
    case "archive-team-lineup-preset":
      requestArchiveLineupPreset(
        button.dataset.presetId ?? "",
        button.dataset.presetName ?? "阵容预设",
      );
      break;
    case "save-current-lineup-as-preset":
      openSaveCurrentLineupPreset(button.dataset.lineupSide as LineupSide);
      break;
    case "confirm-save-current-lineup-preset":
      await saveCurrentLineupAsPreset(button.dataset.lineupSide as LineupSide);
      break;
    case "preview-apply-lineup-preset":
      await previewApplyLineupPreset(button.dataset.lineupSide as LineupSide);
      break;
    case "confirm-apply-lineup-preset":
      await applyLineupPreset(
        button.dataset.lineupSide as LineupSide,
        button.dataset.presetId ?? "",
      );
      break;
    case "create-lineup-pair":
      await createPairedLineups();
      break;
    case "inspect-paired-lineup-chain":
      await inspectPairedLineupChain();
      break;
    case "load-lineup-players":
      await loadLineupPlayers();
      break;
    case "add-selected-lineup-player":
      addSelectedPairedLineupPlayer(button.dataset.lineupSide as LineupSide);
      break;
    case "open-lineup-player-settings":
      openPairedLineupPlayerSettings(
        button.dataset.lineupSide as LineupSide,
        button.dataset.playerId ?? "",
      );
      break;
    case "save-lineup-player-settings":
      savePairedLineupPlayerSettings(
        button.dataset.lineupSide as LineupSide,
        button.dataset.playerId ?? "",
      );
      break;
    case "add-lineup-player": {
      const side = button.dataset.lineupSide as LineupSide | undefined;
      if (side) {
        addPairedLineupPlayer(
          side,
          button.dataset.playerId ?? "",
          button.dataset.role === "starter",
        );
      } else {
        addLineupPlayer(
          button.dataset.playerId ?? "",
          button.dataset.role === "starter",
        );
      }
      break;
    }
    case "remove-lineup-player": {
      const side = button.dataset.lineupSide as LineupSide | undefined;
      if (side) removePairedLineupPlayer(side, button.dataset.playerId ?? "");
      else removeLineupPlayer(button.dataset.playerId ?? "");
      break;
    }
    case "clear-lineup-builder":
      captureLineupFormFromDom();
      lineupBuilderPlayers = [];
      render();
      break;
    case "inspect-lineup-chain":
      await inspectLineupChain();
      break;
    case "create-lineup":
      await createLineup();
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
      if (!state) return;
      state.data.recent_runs = await runBusy(() => api.listRecentRuns(100));
      render();
      break;
    case "request-remove-lineup-history":
      requestRemoveLineupHistory(
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
    case "workspace-history-back":
      modal.back();
      break;
    case "workspace-history-forward":
      modal.forward();
      break;
    case "confirm-workspace-action":
      await modal.runPendingAction();
      break;
    case "close-workspace-detail":
      closeModal();
      break;
  }
}

function closeAppContextMenu(): void {
  document.querySelector("#app-context-menu")?.remove();
}

function showAppContextMenu(event: MouseEvent, target: HTMLElement): void {
  closeAppContextMenu();
  const kind = target.dataset.contextKind;
  if (kind !== "run" && kind !== "match" && kind !== "lineup") return;
  event.preventDefault();
  const menu = document.createElement("div");
  menu.id = "app-context-menu";
  menu.className = "app-context-menu";
  menu.setAttribute("role", "menu");
  if (kind === "run") {
    menu.innerHTML = `<button type="button" role="menuitem" data-action="request-hide-run-history" data-run-id="${escapeHtml(target.dataset.runId ?? "")}" data-run-label="${escapeHtml(target.dataset.runLabel ?? "未命名推演")}">从历史列表删除</button><small>底层运行、快照和收敛数据保留</small>`;
  } else if (kind === "lineup") {
    menu.innerHTML = `<button type="button" class="danger-action" role="menuitem" data-action="request-remove-lineup-history" data-lineup-id="${escapeHtml(target.dataset.lineupId ?? "")}" data-lineup-label="${escapeHtml(target.dataset.lineupLabel ?? "未命名阵容")}">删除或归档阵容版本</button><small>已引用版本保留模型血缘，未引用版本永久删除</small>`;
  } else {
    menu.innerHTML = `<button type="button" class="danger-action" role="menuitem" data-action="request-delete-match" data-match-id="${escapeHtml(target.dataset.matchId ?? "")}" data-match-label="${escapeHtml(target.dataset.matchLabel ?? "未命名比赛")}">删除比赛</button><small>将同时删除该比赛的阵容和复盘数据</small>`;
  }
  document.body.append(menu);
  const rect = menu.getBoundingClientRect();
  menu.style.left = `${Math.max(8, Math.min(event.clientX, window.innerWidth - rect.width - 8))}px`;
  menu.style.top = `${Math.max(8, Math.min(event.clientY, window.innerHeight - rect.height - 8))}px`;
  menu.querySelector<HTMLElement>("button")?.focus();
}

document.addEventListener("contextmenu", (event) => {
  const target = (event.target as HTMLElement).closest<HTMLElement>("[data-context-kind]");
  if (!target) return;
  showAppContextMenu(event, target);
}, { signal: browserLifecycleController.signal });

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    closeAppContextMenu();
    return;
  }
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

document.addEventListener("scroll", closeAppContextMenu, { capture: true, signal: browserLifecycleController.signal });

function filterRulesCompetitionCatalogue(): void {
  const directory = document.querySelector<HTMLElement>("[data-rules-directory]");
  if (!directory) return;
  const query = (document.querySelector<HTMLInputElement>("#rules-competition-search")?.value ?? "").trim().toLowerCase();
  const kind = document.querySelector<HTMLSelectElement>("#rules-competition-kind")?.value ?? "";
  const rows = Array.from(directory.querySelectorAll<HTMLElement>("[data-rules-competition-row]"));
  const baseMatches = (row: HTMLElement): boolean =>
    (!kind || row.dataset.kind === kind) &&
    (!query || (row.dataset.search ?? "").includes(query));

  let scope = directory.querySelector<HTMLElement>("[data-rules-scope].active")?.dataset.rulesScope ?? "";
  const scopeButtons = Array.from(directory.querySelectorAll<HTMLButtonElement>("[data-rules-scope]"));
  for (const button of scopeButtons) {
    const value = button.dataset.rulesScope ?? "";
    const count = rows.filter((row) => baseMatches(row) && (!value || row.dataset.scope === value)).length;
    const badge = button.querySelector<HTMLElement>("span");
    if (badge) badge.textContent = String(count);
    button.disabled = value !== "" && count === 0;
  }
  const activeScopeButton = scopeButtons.find((button) => button.dataset.rulesScope === scope);
  if (!activeScopeButton || activeScopeButton.disabled) {
    scope = "";
    scopeButtons.forEach((button) => button.classList.toggle("active", (button.dataset.rulesScope ?? "") === ""));
  }

  let region = directory.querySelector<HTMLElement>("[data-rules-region].active")?.dataset.rulesRegion ?? "";
  const regionButtons = Array.from(directory.querySelectorAll<HTMLButtonElement>("[data-rules-region]"));
  for (const button of regionButtons) {
    const value = button.dataset.rulesRegion ?? "";
    const count = rows.filter((row) =>
      baseMatches(row) &&
      (!scope || row.dataset.scope === scope) &&
      (!value || row.dataset.region === value),
    ).length;
    const badge = button.querySelector<HTMLElement>("span");
    if (badge) badge.textContent = String(count);
    button.disabled = value !== "" && count === 0;
    button.hidden = button.disabled;
  }
  const activeRegionButton = regionButtons.find((button) => button.dataset.rulesRegion === region);
  if (!activeRegionButton || activeRegionButton.disabled || activeRegionButton.hidden) {
    region = "";
    regionButtons.forEach((button) => button.classList.toggle("active", (button.dataset.rulesRegion ?? "") === ""));
  }

  let visible = 0;
  for (const row of rows) {
    const matches =
      baseMatches(row) &&
      (!scope || row.dataset.scope === scope) &&
      (!region || row.dataset.region === region);
    row.hidden = !matches;
    if (matches) visible += 1;
  }
  const count = directory.querySelector<HTMLElement>("#rules-visible-count");
  if (count) count.textContent = String(visible);
  const empty = directory.querySelector<HTMLElement>("#rules-empty-filter");
  if (empty) empty.hidden = visible !== 0;
}

function selectRulesDirectoryLevel(target: HTMLElement): boolean {
  const scopeButton = target.closest<HTMLElement>("[data-rules-scope]");
  const regionButton = target.closest<HTMLElement>("[data-rules-region]");
  const directory = target.closest<HTMLElement>("[data-rules-directory]");
  if (!directory || (!scopeButton && !regionButton)) return false;
  if (scopeButton) {
    directory.querySelectorAll<HTMLElement>("[data-rules-scope]").forEach((item) => item.classList.toggle("active", item === scopeButton));
    const scope = scopeButton.dataset.rulesScope ?? "";
    let firstVisibleRegion: HTMLElement | null = null;
    directory.querySelectorAll<HTMLElement>("[data-rules-region]").forEach((item) => {
      const scopes = (item.dataset.regionScopes ?? "").split(",").filter(Boolean);
      const available = !scope || !item.dataset.rulesRegion || scopes.includes(scope);
      item.hidden = !available;
      if (available && !firstVisibleRegion) firstVisibleRegion = item;
      item.classList.remove("active");
    });
    const allRegion = directory.querySelector<HTMLElement>('[data-rules-region=""]');
    (allRegion && !allRegion.hidden ? allRegion : firstVisibleRegion)?.classList.add("active");
  } else if (regionButton) {
    directory.querySelectorAll<HTMLElement>("[data-rules-region]").forEach((item) => item.classList.toggle("active", item === regionButton));
  }
  filterRulesCompetitionCatalogue();
  return true;
}

document.addEventListener("click", (event) => {
  const target = event.target as HTMLElement;
  if (!target.closest("#app-context-menu")) closeAppContextMenu();
  if (selectRulesDirectoryLevel(target)) return;
  const pageButton = target.closest<HTMLElement>("[data-page]");
  if (pageButton?.dataset.page) {
    const targetPage = pageButton.dataset.page as Page;
    if (targetPage === "players") prepareDirectPlayerDirectoryEntry();
    const navigationTraceId = recordUiOperation("navigation", `page:${targetPage}`, {
      fromPage: page,
      toPage: targetPage,
    });
    void navigateTo(targetPage).catch((error: unknown) => {
      recordUiOperationFailure(`page:${targetPage}`, error, navigationTraceId, {
        fromPage: page,
        toPage: targetPage,
      });
      recordClientIssue(error, `打开页面：${pageTitle(targetPage)}`);
      toast(userFacingError(error), "error");
    });
    return;
  }
  const actionButton = target.closest<HTMLElement>("[data-action]");
  if (!actionButton?.dataset.action) return;
  const action = actionButton.dataset.action;
  const actionTraceId = recordUiOperation("ui_action", `action:${action}`, {
    page,
    dataset: { ...actionButton.dataset },
  });
  void handleAction(action, actionButton).catch((error: unknown) => {
    recordUiOperationFailure(`action:${action}`, error, actionTraceId, { page });
    recordClientIssue(error, `${pageTitle(page)} / ${action}`);
    toast(userFacingError(error), "error");
  });
}, { signal: browserLifecycleController.signal });

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
  if (
    target.id === "database-reset-confirmation" &&
    target instanceof HTMLInputElement
  ) {
    const button = document.querySelector<HTMLButtonElement>(
      '[data-action="execute-database-reset"]',
    );
    if (button) {
      button.disabled = target.value.trim() !== (target.dataset.databaseName ?? "");
    }
    return;
  }
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
    filterManagedMatchList(target.value);
    return;
  }
  if (target.id === "rules-competition-search" && target instanceof HTMLInputElement) {
    filterRulesCompetitionCatalogue();
  }
}, { signal: browserLifecycleController.signal });

document.addEventListener("change", (event) => {
  const target = event.target as HTMLInputElement | HTMLSelectElement;
  if (
    target instanceof HTMLInputElement &&
    target.classList.contains("player-select-checkbox")
  ) {
    const playerId = target.dataset.playerId;
    if (playerId)
      target.checked
        ? selectedPlayerIds.add(playerId)
        : selectedPlayerIds.delete(playerId);
    persistWorkspaceSelection("players");
    render({ preserveForm: true });
    return;
  }
  if (target instanceof HTMLInputElement && target.id === "player-select-all") {
    for (const item of playerListPage?.items ?? [])
      target.checked
        ? selectedPlayerIds.add(item.id)
        : selectedPlayerIds.delete(item.id);
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
      target.checked
        ? selectedTeamIds.add(teamId)
        : selectedTeamIds.delete(teamId);
    persistWorkspaceSelection("teams");
    render({ preserveForm: true });
    return;
  }
  if (target instanceof HTMLInputElement && target.id === "team-select-all") {
    for (const item of teamListPage?.items ?? [])
      target.checked
        ? selectedTeamIds.add(item.id)
        : selectedTeamIds.delete(item.id);
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
    if (row) syncPresetMemberRowState(row);
    updatePresetEditorSummary();
    return;
  }
  if (target instanceof HTMLSelectElement && target.id === "lineup-preset-formation") {
    refreshPresetPositionOptions(true);
    return;
  }
  if (target instanceof HTMLSelectElement && target.classList.contains("preset-member-role")) {
    const row = target.closest<HTMLElement>("[data-preset-player-id]");
    if (row) syncPresetMemberRowState(row);
    updatePresetEditorSummary();
    return;
  }
  if (target instanceof HTMLSelectElement && target.classList.contains("preset-member-position")) {
    const row = target.closest<HTMLElement>("[data-preset-player-id]");
    if (row) refreshPresetTacticalRoleSelect(row);
    updatePresetEditorSummary();
    return;
  }
  if (target instanceof HTMLSelectElement && target.classList.contains("preset-member-tactical-role")) {
    updatePresetEditorSummary();
    return;
  }
  if (target.id === "rule-package-file" && target instanceof HTMLInputElement) {
    const file = target.files?.[0];
    if (!file) return;
    void file
      .text()
      .then((text) => {
        const parsed: unknown = JSON.parse(text);
        if (!parsed || typeof parsed !== "object" || Array.isArray(parsed))
          throw new Error("规则包文件格式无效");
        const candidate = parsed as RulePackageDraft;
        if (
          !candidate.display_name ||
          !candidate.package_key ||
          !candidate.version ||
          !candidate.routing ||
          !candidate.competition_profile
        ) {
          throw new Error("规则包缺少名称、版本、赛事 Profile 或模型路由");
        }
        pendingRulePackage = candidate;
        render();
        toast("规则包已读取，请核对摘要后注册", "success");
      })
      .catch((error: unknown) => {
        pendingRulePackage = null;
        recordClientIssue(error, "赛事设置 / 读取规则包文件");
        toast(userFacingError(error), "error");
      });
    return;
  }
  if (target.id === "new-match-kickoff" && target instanceof HTMLInputElement) {
    autoSelectMatchSeason();
    filterMatchTeamOptions();
    return;
  }
  if (!(target instanceof HTMLSelectElement)) return;
  if (target.id === "rules-competition-kind") {
    filterRulesCompetitionCatalogue();
    return;
  }
  if (target.id === "prediction-match-id") {
    const matchId = target.value;
    if (!matchId) return;
    selectedP4MatchId = matchId;
    selectedMatchLineupChain = null;
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
    selectedMatchLineupChain = null;
    selectedPredictionReadiness = null;
    render({ preserveForm: true });
    return;
  }
  if (target.id === "prediction-model-family") {
    selectedPredictionModelFamily = target.value as PredictionModelFamily;
    selectedMatchLineupChain = null;
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
    filterSelectOptions("season-id", "competitionId", target.value || null);
    filterSelectOptions("stage-id", "competitionId", target.value || null);
  }
  if (target.id === "season-id") {
    filterSelectOptions("stage-id", "seasonId", target.value || null);
  }
  if (target.id === "stage-id") {
    const kind = target.selectedOptions[0]?.dataset.kind;
    const kindSelect =
      document.querySelector<HTMLSelectElement>("#competition-kind");
    if (kind && kindSelect) kindSelect.value = kind;
  }
  if (target.id === "binding-competition-id") {
    filterSelectOptions(
      "binding-season-id",
      "competitionId",
      target.value || null,
    );
    filterSelectOptions(
      "binding-stage-id",
      "competitionId",
      target.value || null,
    );
  }
  if (target.id === "binding-season-id") {
    filterSelectOptions("binding-stage-id", "seasonId", target.value || null);
  }
  if (target.id === "new-match-competition-scope") {
    updateCompetitionHierarchy("scope");
    return;
  }
  if (target.id === "new-match-competition-region") {
    updateCompetitionHierarchy("region");
    return;
  }
  if (target.id === "new-match-competition") {
    updateCompetitionHierarchy("competition");
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
    autoSelectMatchSeason();
    filterMatchTeamOptions();
    return;
  }
  if (target.id === "new-match-season") {
    filterSelectOptions("new-match-stage", "seasonId", target.value || null);
    filterMatchTeamOptions();
    return;
  }
  if (target.id === "new-match-stage") {
    filterSelectOptions("new-match-round", "stageId", target.value || null);
    return;
  }
  if (target.id === "new-match-team-scope") {
    filterMatchTeamOptions();
    return;
  }
  if (target.id === "paired-home-formation-level1" || target.id === "paired-away-formation-level1") {
    updateFormationHierarchy(target.id.includes("home") ? "home" : "away", "level1");
    return;
  }
  if (target.id === "paired-home-formation-level2" || target.id === "paired-away-formation-level2") {
    updateFormationHierarchy(target.id.includes("home") ? "home" : "away", "level2");
    return;
  }
  if (target.id === "paired-home-formation-id" || target.id === "paired-away-formation-id") {
    updateFormationHierarchy(target.id.includes("home") ? "home" : "away", "formation");
    return;
  }
  if (target.id === "paired-lineup-match") {
    if (!target.value) return;
    resetPairedBuilderForMatch(target.value, true);
    void runBusy(loadBothPairedLineupSides)
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
    capturePairedLineupFromDom();
    selectedMatchLineupChain = null;
    return;
  }
  if (target.id === "new-lineup-match") {
    const selected = target.selectedOptions[0];
    captureLineupFormFromDom({ allowBlankIdentity: true });
    lineupBuilderForm = {
      ...lineupBuilderForm,
      match_id: target.value,
      team_id: selected?.dataset.homeTeam ?? "",
    };
    lineupBuilderPlayers = [];
    lineupPlayerCandidates = [];
    lineupPlayerLoadSequence += 1;
    const teamId = lineupBuilderForm.team_id;
    render();
    if (teamId) void autoLoadLineupPlayers(teamId);
    return;
  }
  if (["new-lineup-team", "new-lineup-type", "new-lineup-snapshot", "new-lineup-formation-id", "new-lineup-coach"].includes(target.id)) {
    const previousTeam = lineupBuilderForm.team_id;
    captureLineupFormFromDom({
      allowBlankIdentity: target.id === "new-lineup-team",
    });
    if (
      target.id === "new-lineup-team" &&
      previousTeam !== lineupBuilderForm.team_id
    ) {
      lineupBuilderPlayers = [];
      lineupPlayerCandidates = [];
      lineupPlayerLoadSequence += 1;
      const teamId = lineupBuilderForm.team_id;
      render();
      if (teamId) void autoLoadLineupPlayers(teamId);
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
  selectedTeamIds = new Set(workspaceState.module("teams").selected_object_ids);
  selectedPlayerIds = new Set(workspaceState.module("players").selected_object_ids);
  await refresh();
  if (window.sessionStorage.getItem(DATABASE_RESET_COMPLETE_KEY) === "1") {
    window.sessionStorage.removeItem(DATABASE_RESET_COMPLETE_KEY);
    toast("数据库已彻底清空并重建为空白状态", "success");
  }
}

window.addEventListener("beforeunload", () => {
  if (renderedPage) {
    const currentPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
    workspaceState.capture(renderedPage, currentPageRoot, true);
  }
  void workspaceState.flush();
}, { signal: browserLifecycleController.signal });

export interface BrowserApplicationModule {
  readonly name: "browser-application";
  start(): Promise<void>;
  destroy(): Promise<void>;
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
  let startTask: Promise<void> | null = null;
  let destroyed = false;

  async function start(): Promise<void> {
    if (destroyed) throw new Error("浏览器应用生命周期已结束");
    if (startTask) return startTask;
    startTask = (async () => {
      bindSearchableSelectDiagnostics(browserLifecycleController.signal);
      render();
      try {
        await initializeApplication();
      } catch (error: unknown) {
        recordClientIssue(error, "平台启动", "critical");
        throw new Error(userFacingError(error), { cause: error });
      }
    })();
    return startTask;
  }

  async function destroy(): Promise<void> {
    if (destroyed) return;
    destroyed = true;
    browserLifecycleController.abort();
    if (openAiApiExampleTimer !== null) {
      window.clearTimeout(openAiApiExampleTimer);
      openAiApiExampleTimer = null;
    }
    if (renderedPage) {
      const currentPageRoot = app.querySelector<HTMLElement>(".page-container") ?? app;
      workspaceState.capture(renderedPage, currentPageRoot, true);
    }
    await workspaceState.destroy();
  }

  return { name: "browser-application", start, destroy };
}
