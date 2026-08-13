use super::super::super::port_registry::PersistenceStore;
use super::super::map_persistence_error;
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
impl SpreadsheetExchangePort for PersistenceStore {
    async fn reference_data(&self) -> PortResult<PlayerCatalogReferenceData> {
        self.player_catalog_reference_data()
            .await
            .map_err(map_persistence_error)
    }

    async fn export_data(&self) -> PortResult<SpreadsheetExportData> {
        self.spreadsheet_export_data()
            .await
            .map_err(map_persistence_error)
    }

    async fn data_gaps(&self) -> PortResult<Vec<MonthlyDataGapRow>> {
        self.player_monthly_data_gaps()
            .await
            .map_err(map_persistence_error)
    }

    async fn preview_import(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.preview_spreadsheet_import(workbook, mode)
            .await
            .map_err(map_persistence_error)
    }

    async fn preview_import_with_team_references(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
        package_team_references: &HashMap<String, String>,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.preview_spreadsheet_import_with_team_references(
            workbook,
            mode,
            package_team_references,
        )
        .await
        .map_err(map_persistence_error)
    }

    async fn read_import_preview(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportPreview> {
        self.read_spreadsheet_import_preview(preview_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn resolve_conflict(
        &self,
        preview_id: Uuid,
        resolution: &SpreadsheetImportResolution,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.resolve_spreadsheet_import_conflict(preview_id, resolution.clone())
            .await
            .map_err(map_persistence_error)
    }

    async fn commit_import(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportCommitResult> {
        self.commit_spreadsheet_import(preview_id)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl MonthlyWorkbookPort for PersistenceStore {
    async fn export_data(&self) -> PortResult<TeamMonthlyWorkbookData> {
        self.team_monthly_workbook_data()
            .await
            .map_err(map_persistence_error)
    }

    async fn preview_import(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.preview_team_monthly_import(workbook, mode)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_import_preview(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportPreview> {
        self.read_team_monthly_import_preview(preview_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn resolve_conflict(
        &self,
        preview_id: Uuid,
        resolution: &SpreadsheetImportResolution,
    ) -> PortResult<SpreadsheetImportPreview> {
        self.resolve_team_monthly_import_conflict(preview_id, resolution.clone())
            .await
            .map_err(map_persistence_error)
    }

    async fn commit_import(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportCommitResult> {
        self.commit_team_monthly_import(preview_id)
            .await
            .map_err(map_persistence_error)
    }
}
