use super::super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{
    exchange::{MonthlyWorkbookPort, SpreadsheetExchangePort},
    PortResult,
};
use async_trait::async_trait;
use football_domain::{
    MonthlyDataGapRow, PlayerCatalogReferenceData, SpreadsheetExportData,
    SpreadsheetImportCommitResult, SpreadsheetImportMode, SpreadsheetImportPreview,
    SpreadsheetImportResolution, SpreadsheetParsedWorkbook, TeamMonthlyWorkbookData,
};
use std::collections::HashMap;
use uuid::Uuid;

#[async_trait]
impl SpreadsheetExchangePort for ActiveDatabase {
    async fn reference_data(&self) -> PortResult<PlayerCatalogReferenceData> {
        self.transition_store()
            .player_catalog_reference_data()
            .await
            .map_err(map_persistence_error)
    }

    async fn export_data(&self) -> PortResult<SpreadsheetExportData> {
        self.transition_store()
            .spreadsheet_export_data()
            .await
            .map_err(map_persistence_error)
    }

    async fn data_gaps(&self) -> PortResult<Vec<MonthlyDataGapRow>> {
        self.transition_store()
            .player_monthly_data_gaps()
            .await
            .map_err(map_persistence_error)
    }

    async fn preview_import(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.transition_store()
            .preview_spreadsheet_import(workbook, mode)
            .await
            .map_err(map_persistence_error)
    }

    async fn preview_import_with_team_references(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
        package_team_references: &HashMap<String, String>,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.transition_store()
            .preview_spreadsheet_import_with_team_references(
                workbook,
                mode,
                package_team_references,
            )
            .await
            .map_err(map_persistence_error)
    }

    async fn read_import_preview(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportPreview> {
        self.transition_store()
            .read_spreadsheet_import_preview(preview_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn resolve_conflict(
        &self,
        preview_id: Uuid,
        resolution: &SpreadsheetImportResolution,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.transition_store()
            .resolve_spreadsheet_import_conflict(preview_id, resolution.clone())
            .await
            .map_err(map_persistence_error)
    }

    async fn commit_import(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportCommitResult> {
        self.transition_store()
            .commit_spreadsheet_import(preview_id)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl MonthlyWorkbookPort for ActiveDatabase {
    async fn export_data(&self) -> PortResult<TeamMonthlyWorkbookData> {
        self.transition_store()
            .team_monthly_workbook_data()
            .await
            .map_err(map_persistence_error)
    }

    async fn preview_import(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.transition_store()
            .preview_team_monthly_import(workbook, mode)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_import_preview(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportPreview> {
        self.transition_store()
            .read_team_monthly_import_preview(preview_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn resolve_conflict(
        &self,
        preview_id: Uuid,
        resolution: &SpreadsheetImportResolution,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.transition_store()
            .resolve_team_monthly_import_conflict(preview_id, resolution.clone())
            .await
            .map_err(map_persistence_error)
    }

    async fn commit_import(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportCommitResult> {
        self.transition_store()
            .commit_team_monthly_import(preview_id)
            .await
            .map_err(map_persistence_error)
    }
}
