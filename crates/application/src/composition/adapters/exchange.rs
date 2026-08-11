use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{exchange::MatchLineupExchangePort, PortResult};
use async_trait::async_trait;
use football_domain::{
    AiMatchPackageContext, MatchLineupExportData, SpreadsheetImportCommitResult,
    SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution,
    SpreadsheetParsedWorkbook,
};
use uuid::Uuid;

#[async_trait]
impl MatchLineupExchangePort for ActiveDatabase {
    async fn export_match_lineup(
        &self,
        match_id: Option<Uuid>,
    ) -> PortResult<MatchLineupExportData> {
        self.transition_store()
            .match_lineup_export_data(match_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn preview_import(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.transition_store()
            .preview_match_lineup_import(workbook, mode)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_import_preview(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportPreview> {
        self.transition_store()
            .read_match_lineup_import_preview(preview_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn resolve_import_conflict(
        &self,
        preview_id: Uuid,
        resolution: SpreadsheetImportResolution,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.transition_store()
            .resolve_match_lineup_import_conflict(preview_id, resolution)
            .await
            .map_err(map_persistence_error)
    }

    async fn commit_import(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportCommitResult> {
        self.transition_store()
            .commit_match_lineup_import(preview_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn ai_match_package_context(&self, match_id: Uuid) -> PortResult<AiMatchPackageContext> {
        self.transition_store()
            .ai_match_package_context(match_id)
            .await
            .map_err(map_persistence_error)
    }
}
