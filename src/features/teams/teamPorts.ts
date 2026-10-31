import type * as T from "../../types";

/** Injected existing transport. Signatures and wire DTOs remain the public client contract. */
export interface TeamApi {
  readPlayer(playerId: string): Promise<T.PlayerDetail>;
  listTeams(query: T.TeamListQuery): Promise<T.TeamListPage>;
  readTeam(teamId: string): Promise<T.TeamDetail>;
  listTeamMatchLineups(teamId: string, limit?: number): Promise<T.TeamMatchLineupHistoryItem[]>;
  listTeamLineupPresets(teamId: string, includeArchived?: boolean): Promise<T.TeamLineupPresetRecord[]>;
  listCoaches(query: T.CoachListQuery): Promise<T.CoachListItem[]>;
  listFormations(activeOnly?: boolean): Promise<T.FormationRecord[]>;
  createTeam(draft: T.TeamDraft): Promise<T.TeamRecord>;
  updateTeam(teamId: string, draft: T.TeamDraft): Promise<T.TeamRecord>;
  addTeamName(draft: T.TeamNameDraft): Promise<T.TeamNameRecord>;
  upsertTeamProfile(teamId: string, draft: T.TeamProfileDraft): Promise<T.TeamProfileRecord>;
  saveFormationUsageDistribution(draft: T.FormationUsageDistributionDraft): Promise<T.FormationUsageDistributionRecord>;
  createCoach(draft: T.CoachDraft): Promise<T.CoachRecord>;
  addTeamCoachPeriod(draft: T.TeamCoachPeriodDraft): Promise<T.TeamCoachPeriodRecord>;
  chooseExcelImportFile(): Promise<string | null>;
  chooseExcelExportFile(defaultPath: string): Promise<string | null>;
  chooseJsonExportFile(defaultPath: string): Promise<string | null>;
  exportTeamPackageTemplate(outputPath: string): Promise<T.TeamPackageExportSummary>;
  exportTeamPackagePreviewJson(outputPath: string, preview: T.TeamPackageImportPreview): Promise<T.TeamPackagePreviewExportSummary>;
  previewTeamPackageImport(inputPath: string, mode: T.SpreadsheetImportMode): Promise<T.TeamPackageImportPreview>;
  commitTeamPackageImport(request: T.TeamPackageCommitRequest): Promise<T.TeamPackageCommitResult>;
  previewTeamMonthlyImport(inputPath: string, mode: T.SpreadsheetImportMode): Promise<T.SpreadsheetImportPreview>;
  readTeamMonthlyImportPreview(batchId: string): Promise<T.SpreadsheetImportPreview>;
  resolveTeamMonthlyImportConflict(batchId: string, resolution: T.SpreadsheetImportResolution): Promise<T.SpreadsheetImportPreview>;
  resolvePlayerCatalogImportConflict(batchId: string, resolution: T.SpreadsheetImportResolution): Promise<T.SpreadsheetImportPreview>;
  readPlayerCatalogImportPreview(batchId: string): Promise<T.SpreadsheetImportPreview>;
  commitTeamMonthlyImport(batchId: string): Promise<T.SpreadsheetImportCommitResult>;
  exportTeamMonthlyTemplate(outputPath: string): Promise<T.MonthlyWorkbookExportSummary>;
  exportTeamMonthlyData(outputPath: string): Promise<T.MonthlyWorkbookExportSummary>;
  checkEntityDeletion(entityType: T.EntityReferenceType, entityId: string): Promise<T.EntityDeletionCheck>;
  bulkDeleteTeams(teamIds: string[]): Promise<T.BulkDeleteResult>;
  bulkArchiveEntities(entityType: T.EntityReferenceType, entityIds: string[]): Promise<T.BulkArchiveResult>;
  previewForceDeleteTeam(teamId: string): Promise<T.TeamForceDeletePreview>;
  forceDeleteTeam(request: T.TeamForceDeleteRequest): Promise<T.TeamForceDeleteResult>;
}
