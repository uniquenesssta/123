import type * as T from "../../types";
/** Existing wire contracts, injected by the composition root. */
export interface LineupFiles {
    chooseExcelImportFile(): Promise<string | null>;
    chooseZipImportFile(): Promise<string | null>;
    chooseExcelExportFile(defaultPath: string): Promise<string | null>;
    chooseZipExportFile(defaultPath: string): Promise<string | null>;
}
export interface LineupApi {
    exportMatchLineupTemplate(outputPath: string): Promise<T.MatchLineupExportSummary>;
    exportMatchLineupData(outputPath: string, matchId: string): Promise<T.MatchLineupExportSummary>;
    exportAiMatchPackage(outputPath: string, matchId: string): Promise<T.AiMatchPackageSummary>;
    previewMatchLineupImport(inputPath: string, mode: T.SpreadsheetImportMode): Promise<T.SpreadsheetImportPreview>;
    previewAiMatchPackage(inputPath: string, mode: T.SpreadsheetImportMode): Promise<T.SpreadsheetImportPreview>;
    readMatchLineupImportPreview(batchId: string): Promise<T.SpreadsheetImportPreview>;
    resolveMatchLineupImportConflict(batchId: string, resolution: T.SpreadsheetImportResolution): Promise<T.SpreadsheetImportPreview>;
    commitMatchLineupImport(batchId: string): Promise<T.SpreadsheetImportCommitResult>;
    listPlayers: (query: T.PlayerListQuery) => Promise<T.PlayerListPage>;
    readMatchLineupChain: (matchId: string, snapshotType: string) => Promise<T.MatchLineupChain>;
    createLineup: (draft: T.LineupDraft) => Promise<T.LineupRecord>;
    listLineups: (matchId: string | null, limit?: number) => Promise<T.LineupRecord[]>;
    createMatch: (draft: T.MatchDraft) => Promise<T.MatchRecord>;
    playerCatalogReferenceData: () => Promise<T.PlayerCatalogReferenceData>;
    removeLineupHistory: (lineupId: string, reason?: string | null) => Promise<T.LineupHistoryRemovalResult>;
    deleteMatch: (matchId: string) => Promise<void>;
    createLineupPair: (draft: T.LineupPairDraft) => Promise<T.LineupPairRecord>;
    listTeamLineupPresets: (teamId: string, includeArchived?: boolean) => Promise<T.TeamLineupPresetRecord[]>;
    deleteTeamLineupPreset: (presetId: string) => Promise<void>;
    saveTeamLineupPreset: (draft: T.TeamLineupPresetDraft) => Promise<T.TeamLineupPresetRecord>;
    duplicateTeamLineupPreset: (presetId: string, name: string) => Promise<T.TeamLineupPresetRecord>;
    archiveTeamLineupPreset: (presetId: string) => Promise<T.TeamLineupPresetRecord>;
    previewTeamLineupPresetApplication: (presetId: string) => Promise<T.TeamLineupPresetApplicationPreview>;
}
