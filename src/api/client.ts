import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  BootstrapResponse,
  CompetitionBindingDraft,
  CompetitionBindingSummary,
  CompetitionDraft,
  CompetitionRecord,
  DatabaseHealth,
  DatabaseOptions,
  ModelRunListItem,
  MatchPredictionReadiness,
  PredictionCommand,
  PredictionExecution,
  StoredMatchPredictionCommand,
  RoundDraft,
  RoundRecord,
  RouteDecision,
  RoutePreviewCommand,
  RulePackageDraft,
  RulePackageSummary,
  SeasonDraft,
  SeasonRecord,
  StageDraft,
  StageRecord,
  DataProviderDraft,
  DataProviderRecord,
  ExternalEntityIdDraft,
  LineupDraft,
  LineupPairDraft,
  LineupHistoryRemovalResult,
  LineupPairRecord,
  LineupRecord,
  MatchLineupChain,
  TeamMatchLineupHistoryItem,
  TeamLineupPresetApplicationPreview,
  TeamLineupPresetDraft,
  TeamLineupPresetRecord,
  MatchDraft,
  MatchRecord,
  PlayerAbilityObservationDraft,
  PlayerAvailabilityDraft,
  PlayerCatalogReferenceData,
  PlayerDetail,
  PlayerDraft,
  PlayerListPage,
  PlayerListQuery,
  PlayerNameDraft,
  PlayerPositionDraft,
  PlayerRecord,
  PlayerTeamPeriodDraft,
  TeamDraft,
  TeamOption,
  TeamRecord,
  TeamDetail,
  TeamListPage,
  TeamListQuery,
  TeamNameDraft,
  TeamNameRecord,
  TeamProfileDraft,
  TeamProfileRecord,
  BulkDeleteResult,
  TeamForceDeletePreview,
  TeamForceDeleteRequest,
  TeamForceDeleteResult,
  BulkArchiveResult,
  CoachDraft,
  CoachRecord,
  CoachListQuery,
  CoachListItem,
  CoachDetail,
  CoachNameDraft,
  CoachNameRecord,
  TeamCoachPeriodDraft,
  TeamCoachPeriodRecord,
  FormationRecord,
  FormationUsageDistributionDraft,
  FormationUsageDistributionRecord,
  FormationUsageListQuery,
  FormationDistributionQuery,
  ResolvedFormationDistribution,
  EntityReferenceQuery,
  EntityReferenceRecord,
  EntityMatchRequest,
  EntityMatchResult,
  EntityDeletionCheck,
  EntityReferenceType,
  SpreadsheetExportSummary,
  TeamPackageCommitRequest,
  TeamPackageCommitResult,
  TeamPackageExportSummary,
  TeamPackageImportPreview,
  TeamPackagePreviewExportSummary,
  MonthlyWorkbookExportSummary,
  SpreadsheetImportCommitResult,
  SpreadsheetImportMode,
  SpreadsheetImportPreview,
  SpreadsheetImportResolution,
  PlayerDynamicTagDraft,
  PlayerDynamicTagRecord,
  PlayerMatchContributionRequest,
  PlayerMatchContribution,
  MatchLineupExportSummary,
  AiMatchPackageSummary,
  AbilityCandidateDecisionDraft,
  AbilityCandidateStatus,
  AbilityUpdateCandidateRecord,
  MatchReviewDetail,
  MatchReviewDraft,
  MatchReviewSummary,
  MatchReviewPackageSummary,
  MatchReviewPackagePreview,
  MatchReviewPackageWorkflowRecord,
  MatchReviewPackageConfirmationRequest,
  MatchReviewPackageFactsCommitResult,
  MatchReviewPackageReviewResult,
  MatchReviewPackageCommitRequest,
  MatchReviewPackageCommitResult,
  ReviewableMatch,
  AnalyticsOverview,
  BackgroundJob,
  EnqueueJobDraft,
  AiAnalysisPackageSummary,
  AiAnalysisResponsePreview,
  AiAnalysisSuggestionRecord,
  AiSuggestionDecisionDraft,
  DataQualityDecisionDraft,
  DataQualityFinding,
  ParameterTuningDraft,
  ParameterTuningDecisionDraft,
  ParameterTuningCandidateRecord,
  ParameterLifecycleReadinessRequest,
  ParameterLifecycleReadiness,
  ParameterShadowValidationRequest,
  ParameterShadowValidationRecord,
  ParameterPromotionRequest,
  ParameterRollbackRequest,
  ParameterPromotionDecisionRecord,
  PostmatchSettlementReadiness,
  PostmatchSettlementDraft,
  PostmatchSettlementRecord,
  EvidenceScoringDecisionDraft,
  EvidenceScoringItemRecord,
  PostmatchMonitoringRequest,
  PostmatchOverview,
  IssueLogDraft,
  IssueLogEntry,
  OpenAiApiExampleParseResult,
  OpenAiApiProtocol,
  OpenAiProfileDraft,
  OpenAiProfileSummary,
  OpenAiProfilesState,
  OpenAiProfileTestResult,
  PlanP4HorizonsCommand,
  P4FreezeReadiness,
  P4FreezeTaskEventRecord,
  P4FreezeTaskRecord,
  P4MatchWorkspace,
  P4TaskWorkspace,
  ResolveP4ConflictCommand,
  ApiWorkspacePreset,
  ApiWorkspaceSessionDetail,
  ApiWorkspaceSessionRecord,
  SendApiWorkspaceCommand,
  ReleaseAcceptanceRequest,
  ReleaseAcceptanceRun,
  ReleaseAcceptanceRunSummary,
} from "../types";

class LoggedInvokeError extends Error {
  readonly issueLogged = true;

  constructor(message: string) {
    super(message);
    this.name = "LoggedInvokeError";
  }
}

type RuntimeOperationPhase =
  | "started"
  | "completed"
  | "failed"
  | "ui_action"
  | "navigation"
  | "diagnostic";

interface RuntimeOperationDraft {
  phase: RuntimeOperationPhase;
  operation: string;
  traceId: string | null;
  durationMs: number | null;
  context: unknown;
}

const MAX_LOG_OBJECT_KEYS = 40;
const MAX_LOG_ARRAY_ITEMS = 8;
const MAX_LOG_DEPTH = 4;
const MAX_LOG_STRING_CHARS = 512;

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function createTraceId(prefix: string): string {
  const random =
    typeof crypto !== "undefined" && typeof crypto.randomUUID === "function"
      ? crypto.randomUUID()
      : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  return `${prefix}-${random}`;
}

function sensitiveLogKey(key: string): boolean {
  return /authorization|api[_-]?key|access[_-]?token|refresh[_-]?token|password|secret|credential/i.test(
    key,
  );
}

function contentLogKey(key: string): boolean {
  return /prompt|message|content|body|schema|api[_-]?example|encrypted[_-]?content/i.test(
    key,
  );
}

function summarizeLogValue(
  value: unknown,
  key = "",
  depth = 0,
): unknown {
  if (sensitiveLogKey(key)) return "[REDACTED]";
  if (value === null || value === undefined) return value ?? null;
  if (typeof value === "boolean" || typeof value === "number") return value;
  if (typeof value === "bigint") return value.toString();
  if (typeof value === "string") {
    if (contentLogKey(key)) {
      return { type: "text", chars: [...value].length, contentOmitted: true };
    }
    const chars = [...value];
    if (chars.length <= MAX_LOG_STRING_CHARS) return value;
    return {
      type: "text",
      chars: chars.length,
      preview: chars.slice(0, MAX_LOG_STRING_CHARS).join(""),
      truncated: true,
    };
  }
  if (depth >= MAX_LOG_DEPTH) {
    if (Array.isArray(value)) return { type: "array", length: value.length };
    if (typeof value === "object") {
      return {
        type: "object",
        keys: Object.keys(value as Record<string, unknown>).slice(
          0,
          MAX_LOG_OBJECT_KEYS,
        ),
      };
    }
    return String(value);
  }
  if (Array.isArray(value)) {
    return {
      type: "array",
      length: value.length,
      sample: value
        .slice(0, MAX_LOG_ARRAY_ITEMS)
        .map((item) => summarizeLogValue(item, key, depth + 1)),
      truncated: value.length > MAX_LOG_ARRAY_ITEMS,
    };
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    return Object.fromEntries(
      entries
        .slice(0, MAX_LOG_OBJECT_KEYS)
        .map(([childKey, childValue]) => [
          childKey,
          summarizeLogValue(childValue, childKey, depth + 1),
        ]),
    );
  }
  return String(value);
}

function summarizeResult(value: unknown): unknown {
  if (value === null || value === undefined) return { type: "empty" };
  if (Array.isArray(value)) return { type: "array", length: value.length };
  if (typeof value === "object") {
    const object = value as Record<string, unknown>;
    const entries = Object.entries(object).slice(0, MAX_LOG_OBJECT_KEYS);
    const summary: Record<string, unknown> = {
      type: "object",
      keys: entries.map(([key]) => key),
    };
    for (const [key, childValue] of entries) {
      if (
        childValue === null ||
        ["string", "number", "boolean", "bigint"].includes(typeof childValue) ||
        Array.isArray(childValue)
      ) {
        summary[key] = summarizeLogValue(childValue, key, 1);
      }
    }
    return summary;
  }
  if (typeof value === "string") {
    return { type: "text", chars: [...value].length };
  }
  return { type: typeof value, value };
}

async function recordRuntimeOperationSilently(
  draft: RuntimeOperationDraft,
): Promise<void> {
  try {
    await tauriInvoke("record_runtime_operation", {
      draft: {
        phase: draft.phase,
        operation: draft.operation,
        trace_id: draft.traceId,
        duration_ms: draft.durationMs,
        context: summarizeLogValue(draft.context),
      },
    });
  } catch {
    // 运行日志不可用时不能阻断业务命令。
  }
}

async function recordIssueSilently(draft: IssueLogDraft): Promise<void> {
  try {
    await tauriInvoke("record_issue", { draft });
  } catch {
    // 日志系统失效时不能覆盖原始业务错误。
  }
}

async function invoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  const traceId = createTraceId("ipc");
  const startedAt = performance.now();
  await recordRuntimeOperationSilently({
    phase: "started",
    operation: command,
    traceId,
    durationMs: null,
    context: { args: args ?? null },
  });
  try {
    const result = await tauriInvoke<T>(command, args);
    await recordRuntimeOperationSilently({
      phase: "completed",
      operation: command,
      traceId,
      durationMs: Math.max(0, Math.round(performance.now() - startedAt)),
      context: { result: summarizeResult(result) },
    });
    return result;
  } catch (error: unknown) {
    const message = errorMessage(error);
    await recordRuntimeOperationSilently({
      phase: "failed",
      operation: command,
      traceId,
      durationMs: Math.max(0, Math.round(performance.now() - startedAt)),
      context: { error: message },
    });
    if (command !== "record_issue" && command !== "bootstrap") {
      await recordIssueSilently({
        severity: "error",
        source: "backend",
        operation: command,
        user_message: message,
        technical_message: message,
      });
    }
    throw new LoggedInvokeError(message);
  }
}

export function recordUiOperation(
  phase: "ui_action" | "navigation",
  operation: string,
  context?: Record<string, unknown>,
): string {
  const traceId = createTraceId("ui");
  void recordRuntimeOperationSilently({
    phase,
    operation,
    traceId,
    durationMs: null,
    context: context ?? null,
  });
  return traceId;
}

export function recordUiOperationFailure(
  operation: string,
  error: unknown,
  traceId: string,
  context?: Record<string, unknown>,
): void {
  void recordRuntimeOperationSilently({
    phase: "failed",
    operation,
    traceId,
    durationMs: null,
    context: {
      ...(context ?? {}),
      error: errorMessage(error),
    },
  });
}

export function recordFrontendDiagnostic(
  operation: string,
  context?: Record<string, unknown>,
): void {
  void recordRuntimeOperationSilently({
    phase: "diagnostic",
    operation,
    traceId: createTraceId("diag"),
    durationMs: null,
    context: context ?? null,
  });
}

export function issueWasLogged(error: unknown): boolean {
  return (
    error instanceof LoggedInvokeError ||
    (error instanceof Error &&
      "issueLogged" in error &&
      (error as Error & { issueLogged?: boolean }).issueLogged === true)
  );
}

export const api = {
  readWorkspaceState: <T>(): Promise<T> => invoke("read_workspace_state"),
  saveWorkspaceState: <T>(document: T): Promise<void> => invoke("save_workspace_state", { document }),
  clearWorkspaceState: <T>(): Promise<T> => invoke("clear_workspace_state"),
  bootstrap: (): Promise<BootstrapResponse> => invoke("bootstrap"),
  recordIssue: (draft: IssueLogDraft): Promise<IssueLogEntry> =>
    invoke<IssueLogEntry>("record_issue", { draft }),
  listIssueLogs: (limit = 500): Promise<IssueLogEntry[]> =>
    invoke("list_issue_logs", { limit }),
  clearIssueLogs: (): Promise<void> => invoke("clear_issue_logs"),
  chooseIssueLogExportFile: async (
    defaultPath: string,
  ): Promise<string | null> => {
    const selected = await save({
      defaultPath,
      filters: [{ name: "问题日志报告", extensions: ["txt"] }],
    });
    return typeof selected === "string" ? selected : null;
  },
  exportIssueLogs: (outputPath: string): Promise<void> =>
    invoke("export_issue_logs", { outputPath }),
  configureDatabase: (options: DatabaseOptions): Promise<DatabaseHealth> =>
    invoke("configure_database", { options }),
  disconnectDatabase: (): Promise<void> => invoke("disconnect_database"),
  resetDatabase: (confirmation: string): Promise<DatabaseHealth> =>
    invoke("reset_database", { confirmation }),
  parseOpenAiApiExample: (
    example: string,
    preferredProtocol: OpenAiApiProtocol | null,
  ): Promise<OpenAiApiExampleParseResult> =>
    invoke("parse_openai_api_example", { example, preferredProtocol }),
  listOpenAiProfiles: (): Promise<OpenAiProfilesState> =>
    invoke("list_openai_profiles"),
  saveOpenAiProfile: (
    draft: OpenAiProfileDraft,
  ): Promise<OpenAiProfileSummary> => invoke("save_openai_profile", { draft }),
  setActiveOpenAiProfile: (profileId: string): Promise<OpenAiProfilesState> =>
    invoke("set_active_openai_profile", { profileId }),
  deleteOpenAiProfile: (profileId: string): Promise<OpenAiProfilesState> =>
    invoke("delete_openai_profile", { profileId }),
  clearOpenAiProfileKey: (profileId: string): Promise<OpenAiProfilesState> =>
    invoke("clear_openai_profile_key", { profileId }),
  testOpenAiProfile: (profileId: string): Promise<OpenAiProfileTestResult> =>
    invoke("test_openai_profile", { profileId }),
  listApiWorkspacePresets: (): Promise<ApiWorkspacePreset[]> =>
    invoke("list_api_workspace_presets"),
  listApiWorkspaceSessions: (
    limit = 100,
  ): Promise<ApiWorkspaceSessionRecord[]> =>
    invoke("list_api_workspace_sessions", { limit }),
  readApiWorkspaceSession: (
    sessionId: string,
  ): Promise<ApiWorkspaceSessionDetail> =>
    invoke("read_api_workspace_session", { sessionId }),
  sendApiWorkspaceMessage: (
    command: SendApiWorkspaceCommand,
  ): Promise<ApiWorkspaceSessionDetail> =>
    invoke("send_api_workspace_message", { command }),
  cancelApiWorkspaceRequest: (requestId: string): Promise<boolean> =>
    invoke("cancel_api_workspace_request", { requestId }),
  archiveApiWorkspaceSession: (sessionId: string): Promise<void> =>
    invoke("archive_api_workspace_session", { sessionId }),
  dryRunDefaultFixture: (): Promise<Record<string, unknown>> =>
    invoke("dry_run_default_fixture"),
  executePrediction: (
    command: PredictionCommand,
  ): Promise<PredictionExecution> => invoke("execute_prediction", { command }),
  executePredictionFromMatch: (
    command: StoredMatchPredictionCommand,
  ): Promise<PredictionExecution> =>
    invoke("execute_prediction_from_match", { command }),
  executeShadowPredictionFromMatch: (
    command: StoredMatchPredictionCommand,
  ): Promise<PredictionExecution> =>
    invoke("execute_shadow_prediction_from_match", { command }),
  inspectMatchPredictionReadiness: (
    command: StoredMatchPredictionCommand,
  ): Promise<MatchPredictionReadiness> =>
    invoke("inspect_match_prediction_readiness", { command }),
  previewRoute: (command: RoutePreviewCommand): Promise<RouteDecision> =>
    invoke("preview_route", { command }),
  planP4Horizons: (
    command: PlanP4HorizonsCommand,
  ): Promise<P4FreezeTaskRecord[]> => invoke("plan_p4_horizons", { command }),
  listP4FreezeTasks: (
    matchId: string | null,
    limit = 100,
  ): Promise<P4FreezeTaskRecord[]> =>
    invoke("list_p4_freeze_tasks", { matchId, limit }),
  readP4FreezeTask: (taskId: string): Promise<P4FreezeTaskRecord> =>
    invoke("read_p4_freeze_task", { taskId }),
  listP4FreezeTaskEvents: (
    taskId: string,
  ): Promise<P4FreezeTaskEventRecord[]> =>
    invoke("list_p4_freeze_task_events", { taskId }),
  p4FreezeReadiness: (taskId: string): Promise<P4FreezeReadiness> =>
    invoke("p4_freeze_readiness", { taskId }),
  readP4MatchWorkspace: (matchId: string): Promise<P4MatchWorkspace> =>
    invoke("read_p4_match_workspace", { matchId }),
  readP4TaskWorkspace: (taskId: string): Promise<P4TaskWorkspace> =>
    invoke("read_p4_task_workspace", { taskId }),
  resolveP4Conflict: (
    command: ResolveP4ConflictCommand,
  ): Promise<P4TaskWorkspace> => invoke("resolve_p4_conflict", { command }),
  createCompetition: (draft: CompetitionDraft): Promise<CompetitionRecord> =>
    invoke("create_competition", { draft }),
  deleteCompetition: (competitionId: string): Promise<void> =>
    invoke("delete_competition", { competitionId }),
  createSeason: (draft: SeasonDraft): Promise<SeasonRecord> =>
    invoke("create_season", { draft }),
  createStage: (draft: StageDraft): Promise<StageRecord> =>
    invoke("create_stage", { draft }),
  createRound: (draft: RoundDraft): Promise<RoundRecord> =>
    invoke("create_round", { draft }),
  registerRulePackage: (draft: RulePackageDraft): Promise<RulePackageSummary> =>
    invoke("register_rule_package", { draft }),
  createCompetitionBinding: (
    draft: CompetitionBindingDraft,
  ): Promise<CompetitionBindingSummary> =>
    invoke("create_competition_binding", { draft }),
  listRecentRuns: (limit = 100): Promise<ModelRunListItem[]> =>
    invoke("list_recent_runs", { limit }),
  readRun: (runId: string): Promise<Record<string, unknown>> =>
    invoke("read_run", { runId }),
  hideModelRunHistory: (runId: string, reason: string | null = null): Promise<void> =>
    invoke("hide_model_run_history", { runId, reason }),
  playerCatalogReferenceData: (): Promise<PlayerCatalogReferenceData> =>
    invoke("player_catalog_reference_data"),
  createCoach: (draft: CoachDraft): Promise<CoachRecord> =>
    invoke("create_coach", { draft }),
  listCoaches: (query: CoachListQuery): Promise<CoachListItem[]> =>
    invoke("list_coaches", { query }),
  readCoach: (coachId: string): Promise<CoachDetail> =>
    invoke("read_coach", { coachId }),
  addCoachName: (draft: CoachNameDraft): Promise<CoachNameRecord> =>
    invoke("add_coach_name", { draft }),
  addTeamCoachPeriod: (
    draft: TeamCoachPeriodDraft,
  ): Promise<TeamCoachPeriodRecord> =>
    invoke("add_team_coach_period", { draft }),
  listEntityReferences: (
    query: EntityReferenceQuery,
  ): Promise<EntityReferenceRecord[]> =>
    invoke("list_entity_references", { query }),
  resolveEntityReference: (
    request: EntityMatchRequest,
  ): Promise<EntityMatchResult> =>
    invoke("resolve_entity_reference", { request }),
  checkEntityDeletion: (
    entityType: EntityReferenceType,
    entityId: string,
  ): Promise<EntityDeletionCheck> =>
    invoke("check_entity_deletion", { entityType, entityId }),
  bulkArchiveEntities: (
    entityType: EntityReferenceType,
    entityIds: string[],
  ): Promise<BulkArchiveResult> =>
    invoke("bulk_archive_entities", { entityType, entityIds }),
  createTeam: (draft: TeamDraft): Promise<TeamRecord> =>
    invoke("create_team", { draft }),
  listTeamOptions: (
    search: string | null,
    limit = 100,
  ): Promise<TeamOption[]> => invoke("list_team_options", { search, limit }),
  listTeams: (query: TeamListQuery): Promise<TeamListPage> =>
    invoke("list_teams", { query }),
  readTeam: (teamId: string): Promise<TeamDetail> =>
    invoke("read_team", { teamId }),
  listFormations: (activeOnly = true): Promise<FormationRecord[]> =>
    invoke("list_formations", { activeOnly }),
  saveFormationUsageDistribution: (
    draft: FormationUsageDistributionDraft,
  ): Promise<FormationUsageDistributionRecord> =>
    invoke("save_formation_usage_distribution", { draft }),
  listFormationUsageDistributions: (
    query: FormationUsageListQuery,
  ): Promise<FormationUsageDistributionRecord[]> =>
    invoke("list_formation_usage_distributions", { query }),
  resolveFormationDistribution: (
    query: FormationDistributionQuery,
  ): Promise<ResolvedFormationDistribution> =>
    invoke("resolve_formation_distribution", { query }),
  updateTeam: (teamId: string, draft: TeamDraft): Promise<TeamRecord> =>
    invoke("update_team", { teamId, draft }),
  addTeamName: (draft: TeamNameDraft): Promise<TeamNameRecord> =>
    invoke("add_team_name", { draft }),
  upsertTeamProfile: (
    teamId: string,
    draft: TeamProfileDraft,
  ): Promise<TeamProfileRecord> =>
    invoke("upsert_team_profile", { teamId, draft }),
  bulkDeletePlayers: (playerIds: string[]): Promise<BulkDeleteResult> =>
    invoke("bulk_delete_players", { playerIds }),
  bulkDeleteTeams: (teamIds: string[]): Promise<BulkDeleteResult> =>
    invoke("bulk_delete_teams", { teamIds }),
  previewForceDeleteTeam: (teamId: string): Promise<TeamForceDeletePreview> =>
    invoke("preview_force_delete_team", { teamId }),
  forceDeleteTeam: (request: TeamForceDeleteRequest): Promise<TeamForceDeleteResult> =>
    invoke("force_delete_team", { request }),
  createDataProvider: (draft: DataProviderDraft): Promise<DataProviderRecord> =>
    invoke("create_data_provider", { draft }),
  createPlayer: (draft: PlayerDraft): Promise<PlayerRecord> =>
    invoke("create_player", { draft }),
  deletePlayer: (playerId: string): Promise<void> =>
    invoke("delete_player", { playerId }),
  listPlayers: (query: PlayerListQuery): Promise<PlayerListPage> =>
    invoke("list_players", { query }),
  readPlayer: (playerId: string): Promise<PlayerDetail> =>
    invoke("read_player", { playerId }),
  updatePlayer: (playerId: string, draft: PlayerDraft): Promise<PlayerRecord> =>
    invoke("update_player", { playerId, draft }),
  chooseExcelImportFile: async (): Promise<string | null> => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Excel 工作簿", extensions: ["xlsx"] }],
    });
    return typeof selected === "string" ? selected : null;
  },
  chooseExcelExportFile: async (
    defaultPath: string,
  ): Promise<string | null> => {
    const selected = await save({
      defaultPath,
      filters: [{ name: "Excel 工作簿", extensions: ["xlsx"] }],
    });
    return typeof selected === "string" ? selected : null;
  },
  chooseJsonExportFile: async (
    defaultPath: string,
  ): Promise<string | null> => {
    const selected = await save({
      defaultPath,
      filters: [{ name: "JSON 文件", extensions: ["json"] }],
    });
    return typeof selected === "string" ? selected : null;
  },
  exportTeamPackageTemplate: (
    outputPath: string,
  ): Promise<TeamPackageExportSummary> =>
    invoke("export_team_package_template", { outputPath }),
  exportTeamPackagePreviewJson: (
    outputPath: string,
    preview: TeamPackageImportPreview,
  ): Promise<TeamPackagePreviewExportSummary> =>
    invoke("export_team_package_preview_json", { outputPath, preview }),
  previewTeamPackageImport: (
    inputPath: string,
    mode: SpreadsheetImportMode,
  ): Promise<TeamPackageImportPreview> =>
    invoke("preview_team_package_import", { inputPath, mode }),
  commitTeamPackageImport: (
    request: TeamPackageCommitRequest,
  ): Promise<TeamPackageCommitResult> =>
    invoke("commit_team_package_import", { request }),
  exportPlayerCatalogTemplate: (
    outputPath: string,
  ): Promise<SpreadsheetExportSummary> =>
    invoke("export_player_catalog_template", { outputPath }),
  exportPlayerCatalogData: (
    outputPath: string,
  ): Promise<SpreadsheetExportSummary> =>
    invoke("export_player_catalog_data", { outputPath }),
  previewPlayerCatalogImport: (
    inputPath: string,
    mode: SpreadsheetImportMode,
  ): Promise<SpreadsheetImportPreview> =>
    invoke("preview_player_catalog_import", { inputPath, mode }),
  readPlayerCatalogImportPreview: (
    batchId: string,
  ): Promise<SpreadsheetImportPreview> =>
    invoke("read_player_catalog_import_preview", { batchId }),
  resolvePlayerCatalogImportConflict: (
    batchId: string,
    resolution: SpreadsheetImportResolution,
  ): Promise<SpreadsheetImportPreview> =>
    invoke("resolve_player_catalog_import_conflict", { batchId, resolution }),
  commitPlayerCatalogImport: (
    batchId: string,
  ): Promise<SpreadsheetImportCommitResult> =>
    invoke("commit_player_catalog_import", { batchId }),
  exportTeamMonthlyTemplate: (
    outputPath: string,
  ): Promise<MonthlyWorkbookExportSummary> =>
    invoke("export_team_monthly_template", { outputPath }),
  exportTeamMonthlyData: (
    outputPath: string,
  ): Promise<MonthlyWorkbookExportSummary> =>
    invoke("export_team_monthly_data", { outputPath }),
  previewTeamMonthlyImport: (
    inputPath: string,
    mode: SpreadsheetImportMode,
  ): Promise<SpreadsheetImportPreview> =>
    invoke("preview_team_monthly_import", { inputPath, mode }),
  readTeamMonthlyImportPreview: (
    batchId: string,
  ): Promise<SpreadsheetImportPreview> =>
    invoke("read_team_monthly_import_preview", { batchId }),
  resolveTeamMonthlyImportConflict: (
    batchId: string,
    resolution: SpreadsheetImportResolution,
  ): Promise<SpreadsheetImportPreview> =>
    invoke("resolve_team_monthly_import_conflict", { batchId, resolution }),
  commitTeamMonthlyImport: (
    batchId: string,
  ): Promise<SpreadsheetImportCommitResult> =>
    invoke("commit_team_monthly_import", { batchId }),
  chooseZipImportFile: async (): Promise<string | null> => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "AI 比赛交换包", extensions: ["zip"] }],
    });
    return typeof selected === "string" ? selected : null;
  },
  chooseZipExportFile: async (defaultPath: string): Promise<string | null> => {
    const selected = await save({
      defaultPath,
      filters: [{ name: "AI 比赛交换包", extensions: ["zip"] }],
    });
    return typeof selected === "string" ? selected : null;
  },
  addPlayerDynamicTag: (
    draft: PlayerDynamicTagDraft,
  ): Promise<PlayerDynamicTagRecord> =>
    invoke("add_player_dynamic_tag", { draft }),
  calculatePlayerMatchContribution: (
    request: PlayerMatchContributionRequest,
  ): Promise<PlayerMatchContribution> =>
    invoke("calculate_player_match_contribution", { request }),
  exportMatchLineupTemplate: (
    outputPath: string,
  ): Promise<MatchLineupExportSummary> =>
    invoke("export_match_lineup_template", { outputPath }),
  exportMatchLineupData: (
    outputPath: string,
    matchId: string,
  ): Promise<MatchLineupExportSummary> =>
    invoke("export_match_lineup_data", { outputPath, matchId }),
  previewMatchLineupImport: (
    inputPath: string,
    mode: SpreadsheetImportMode,
  ): Promise<SpreadsheetImportPreview> =>
    invoke("preview_match_lineup_import", { inputPath, mode }),
  readMatchLineupImportPreview: (
    batchId: string,
  ): Promise<SpreadsheetImportPreview> =>
    invoke("read_match_lineup_import_preview", { batchId }),
  resolveMatchLineupImportConflict: (
    batchId: string,
    resolution: SpreadsheetImportResolution,
  ): Promise<SpreadsheetImportPreview> =>
    invoke("resolve_match_lineup_import_conflict", { batchId, resolution }),
  commitMatchLineupImport: (
    batchId: string,
  ): Promise<SpreadsheetImportCommitResult> =>
    invoke("commit_match_lineup_import", { batchId }),
  exportAiMatchPackage: (
    outputPath: string,
    matchId: string,
  ): Promise<AiMatchPackageSummary> =>
    invoke("export_ai_match_package", { outputPath, matchId }),
  previewAiMatchPackage: (
    inputPath: string,
    mode: SpreadsheetImportMode,
  ): Promise<SpreadsheetImportPreview> =>
    invoke("preview_ai_match_package", { inputPath, mode }),
  addPlayerName: (draft: PlayerNameDraft): Promise<Record<string, unknown>> =>
    invoke("add_player_name", { draft }),
  assignPlayerPosition: (
    draft: PlayerPositionDraft,
  ): Promise<Record<string, unknown>> =>
    invoke("assign_player_position", { draft }),
  addPlayerTeamPeriod: (
    draft: PlayerTeamPeriodDraft,
  ): Promise<Record<string, unknown>> =>
    invoke("add_player_team_period", { draft }),
  addPlayerAvailability: (
    draft: PlayerAvailabilityDraft,
  ): Promise<Record<string, unknown>> =>
    invoke("add_player_availability", { draft }),
  addPlayerAbilityObservation: (
    draft: PlayerAbilityObservationDraft,
  ): Promise<Record<string, unknown>> =>
    invoke("add_player_ability_observation", { draft }),
  addExternalEntityId: (
    draft: ExternalEntityIdDraft,
  ): Promise<Record<string, unknown>> =>
    invoke("add_external_entity_id", { draft }),
  createMatch: (draft: MatchDraft): Promise<MatchRecord> =>
    invoke("create_match", { draft }),
  deleteMatch: (matchId: string): Promise<void> =>
    invoke("delete_match", { matchId }),
  saveTeamLineupPreset: (draft: TeamLineupPresetDraft): Promise<TeamLineupPresetRecord> =>
    invoke("save_team_lineup_preset", { draft }),
  listTeamLineupPresets: (teamId: string, includeArchived = false): Promise<TeamLineupPresetRecord[]> =>
    invoke("list_team_lineup_presets", { teamId, includeArchived }),
  previewTeamLineupPresetApplication: (presetId: string): Promise<TeamLineupPresetApplicationPreview> =>
    invoke("preview_team_lineup_preset_application", { presetId }),
  duplicateTeamLineupPreset: (presetId: string, name: string): Promise<TeamLineupPresetRecord> =>
    invoke("duplicate_team_lineup_preset", { presetId, name }),
  archiveTeamLineupPreset: (presetId: string): Promise<TeamLineupPresetRecord> =>
    invoke("archive_team_lineup_preset", { presetId }),
  deleteTeamLineupPreset: (presetId: string): Promise<void> =>
    invoke("delete_team_lineup_preset", { presetId }),
  createLineup: (draft: LineupDraft): Promise<LineupRecord> =>
    invoke("create_lineup", { draft }),
  createLineupPair: (draft: LineupPairDraft): Promise<LineupPairRecord> =>
    invoke("create_lineup_pair", { draft }),
  listLineups: (matchId: string | null, limit = 100): Promise<LineupRecord[]> =>
    invoke("list_lineups", { matchId, limit }),
  readLineup: (lineupId: string): Promise<LineupRecord> =>
    invoke("read_lineup", { lineupId }),
  removeLineupHistory: (lineupId: string, reason: string | null = null): Promise<LineupHistoryRemovalResult> =>
    invoke("remove_lineup_history", { lineupId, reason }),
  readMatchLineupChain: (matchId: string, snapshotType: string): Promise<MatchLineupChain> =>
    invoke("read_match_lineup_chain", { matchId, snapshotType }),
  listTeamMatchLineups: (teamId: string, limit = 100): Promise<TeamMatchLineupHistoryItem[]> =>
    invoke("list_team_match_lineups", { teamId, limit }),
  generateMatchReview: (draft: MatchReviewDraft): Promise<MatchReviewDetail> =>
    invoke("generate_match_review", { draft }),
  listReviewableMatches: (limit = 100): Promise<ReviewableMatch[]> =>
    invoke("list_reviewable_matches", { limit }),
  listMatchReviews: (limit = 100): Promise<MatchReviewSummary[]> =>
    invoke("list_match_reviews", { limit }),
  readMatchReview: (reviewId: string): Promise<MatchReviewDetail> =>
    invoke("read_match_review", { reviewId }),
  listAbilityCandidates: (
    status: AbilityCandidateStatus | null,
    limit = 500,
    matchReviewId: string | null = null,
  ): Promise<AbilityUpdateCandidateRecord[]> =>
    invoke("list_ability_candidates", { status, limit, matchReviewId }),
  decideAbilityCandidate: (
    draft: AbilityCandidateDecisionDraft,
  ): Promise<AbilityUpdateCandidateRecord> =>
    invoke("decide_ability_candidate", { draft }),
  analyticsOverview: (): Promise<AnalyticsOverview> =>
    invoke("analytics_overview"),
  enqueueAnalysisJob: (draft: EnqueueJobDraft): Promise<BackgroundJob> =>
    invoke("enqueue_analysis_job", { draft }),
  listBackgroundJobs: (limit = 100): Promise<BackgroundJob[]> =>
    invoke("list_background_jobs", { limit }),
  cancelBackgroundJob: (jobId: string): Promise<BackgroundJob> =>
    invoke("cancel_background_job", { jobId }),
  retryBackgroundJob: (jobId: string): Promise<BackgroundJob> =>
    invoke("retry_background_job", { jobId }),
  chooseAiAnalysisExportFile: async (
    defaultPath: string,
  ): Promise<string | null> => {
    const selected = await save({
      defaultPath,
      filters: [{ name: "AI 分析包", extensions: ["zip"] }],
    });
    return typeof selected === "string" ? selected : null;
  },
  chooseAiAnalysisResponseFile: async (): Promise<string | null> => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "AI 分析回包", extensions: ["zip"] }],
    });
    return typeof selected === "string" ? selected : null;
  },
  exportAiAnalysisPackage: (
    outputPath: string,
  ): Promise<AiAnalysisPackageSummary> =>
    invoke("export_ai_analysis_package", { outputPath }),
  previewAiAnalysisResponse: (
    inputPath: string,
  ): Promise<AiAnalysisResponsePreview> =>
    invoke("preview_ai_analysis_response", { inputPath }),
  importAiAnalysisResponse: (
    inputPath: string,
  ): Promise<AiAnalysisSuggestionRecord[]> =>
    invoke("import_ai_analysis_response", { inputPath }),
  listAiAnalysisSuggestions: (
    status: string | null,
    limit = 500,
  ): Promise<AiAnalysisSuggestionRecord[]> =>
    invoke("list_ai_analysis_suggestions", { status, limit }),
  decideAiAnalysisSuggestion: (
    draft: AiSuggestionDecisionDraft,
  ): Promise<AiAnalysisSuggestionRecord> =>
    invoke("decide_ai_analysis_suggestion", { draft }),
  decideDataQualityFinding: (
    draft: DataQualityDecisionDraft,
  ): Promise<DataQualityFinding> =>
    invoke("decide_data_quality_finding", { draft }),
  generateParameterTuningCandidate: (
    draft: ParameterTuningDraft,
  ): Promise<ParameterTuningCandidateRecord> =>
    invoke("generate_parameter_tuning_candidate", { draft }),
  listParameterTuningCandidates: (
    limit = 100,
  ): Promise<ParameterTuningCandidateRecord[]> =>
    invoke("list_parameter_tuning_candidates", { limit }),
  decideParameterTuningCandidate: (
    draft: ParameterTuningDecisionDraft,
  ): Promise<ParameterTuningCandidateRecord> =>
    invoke("decide_parameter_tuning_candidate", { draft }),
  parameterLifecycleReadiness: (
    request: ParameterLifecycleReadinessRequest,
  ): Promise<ParameterLifecycleReadiness> =>
    invoke("parameter_lifecycle_readiness", { request }),
  runParameterShadowValidation: (
    request: ParameterShadowValidationRequest,
  ): Promise<ParameterShadowValidationRecord> =>
    invoke("run_parameter_shadow_validation", { request }),
  listParameterShadowValidations: (
    candidateId: string,
  ): Promise<ParameterShadowValidationRecord[]> =>
    invoke("list_parameter_shadow_validations", { candidateId }),
  promoteParameterCandidate: (
    request: ParameterPromotionRequest,
  ): Promise<ParameterPromotionDecisionRecord> =>
    invoke("promote_parameter_candidate", { request }),
  rollbackParameterCandidate: (
    request: ParameterRollbackRequest,
  ): Promise<ParameterPromotionDecisionRecord> =>
    invoke("rollback_parameter_candidate", { request }),
  listParameterPromotionDecisions: (
    candidateId: string,
  ): Promise<ParameterPromotionDecisionRecord[]> =>
    invoke("list_parameter_promotion_decisions", { candidateId }),
  exportMatchReviewPackage: (
    outputPath: string,
    matchId: string,
  ): Promise<MatchReviewPackageSummary> =>
    invoke("export_match_review_package", { outputPath, matchId }),
  previewMatchReviewPackage: (
    inputPath: string,
    expectedMatchId: string | null,
  ): Promise<MatchReviewPackagePreview> =>
    invoke("preview_match_review_package", { inputPath, expectedMatchId }),
  readMatchReviewPackageWorkflow: (
    matchId: string,
  ): Promise<MatchReviewPackageWorkflowRecord | null> =>
    invoke("read_match_review_package_workflow", { matchId }),
  confirmMatchReviewPackage: (
    request: MatchReviewPackageConfirmationRequest,
  ): Promise<MatchReviewPackageWorkflowRecord> =>
    invoke("confirm_match_review_package", { request }),
  commitMatchReviewPackageFacts: (
    packageId: string,
  ): Promise<MatchReviewPackageFactsCommitResult> =>
    invoke("commit_match_review_package_facts", { packageId }),
  generateMatchReviewFromPackage: (
    packageId: string,
  ): Promise<MatchReviewPackageReviewResult> =>
    invoke("generate_match_review_from_package", { packageId }),
  commitMatchReviewPackage: (
    request: MatchReviewPackageCommitRequest,
  ): Promise<MatchReviewPackageCommitResult> =>
    invoke("commit_match_review_package", { request }),
  postmatchSettlementReadiness: (
    matchReviewId: string,
  ): Promise<PostmatchSettlementReadiness> =>
    invoke("postmatch_settlement_readiness", { matchReviewId }),
  settlePostmatchReview: (
    draft: PostmatchSettlementDraft,
  ): Promise<PostmatchSettlementRecord> =>
    invoke("settle_postmatch_review", { draft }),
  listPostmatchSettlements: (limit = 100): Promise<PostmatchSettlementRecord[]> =>
    invoke("list_postmatch_settlements", { limit }),
  listEvidenceScoringItems: (
    status: string | null,
    limit = 100,
  ): Promise<EvidenceScoringItemRecord[]> =>
    invoke("list_evidence_scoring_items", { status, limit }),
  decideEvidenceScoringItem: (
    draft: EvidenceScoringDecisionDraft,
  ): Promise<EvidenceScoringItemRecord> =>
    invoke("decide_evidence_scoring_item", { draft }),
  refreshPostmatchMonitoring: (
    request: PostmatchMonitoringRequest,
  ): Promise<PostmatchOverview> =>
    invoke("refresh_postmatch_monitoring", { request }),
  postmatchOverview: (limit = 100): Promise<PostmatchOverview> =>
    invoke("postmatch_overview", { limit }),
  runReleaseAcceptance: (request: ReleaseAcceptanceRequest): Promise<ReleaseAcceptanceRun> =>
    invoke("run_release_acceptance", { request }),
  listReleaseAcceptanceRuns: (limit = 50): Promise<ReleaseAcceptanceRunSummary[]> =>
    invoke("list_release_acceptance_runs", { limit }),
  readReleaseAcceptanceRun: (runId: string): Promise<ReleaseAcceptanceRun> =>
    invoke("read_release_acceptance_run", { runId }),
  chooseAiAnalysisResponseTemplateFile: async (
    defaultPath: string,
  ): Promise<string | null> => {
    const selected = await save({
      defaultPath,
      filters: [{ name: "AI 分析回包模板", extensions: ["zip"] }],
    });
    return typeof selected === "string" ? selected : null;
  },
  exportAiAnalysisResponseTemplate: (
    outputPath: string,
    sourcePackageId: string | null,
  ): Promise<string> =>
    invoke("export_ai_analysis_response_template", {
      outputPath,
      sourcePackageId,
    }),
};
