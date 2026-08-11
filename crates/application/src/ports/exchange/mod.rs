use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    AiMatchPackageContext, MatchLineupExportData, MonthlyDataGapRow, PlayerCatalogReferenceData,
    SpreadsheetExportData, SpreadsheetImportCommitResult, SpreadsheetImportMode,
    SpreadsheetImportPreview, SpreadsheetImportResolution, SpreadsheetParsedWorkbook,
    TeamMonthlyWorkbookData,
};
use std::collections::HashMap;
use uuid::Uuid;

#[async_trait]
pub trait MatchLineupExchangePort: Send + Sync {
    async fn export_match_lineup(
        &self,
        match_id: Option<Uuid>,
    ) -> PortResult<MatchLineupExportData>;

    async fn preview_import(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PortResult<SpreadsheetImportPreview>;

    async fn read_import_preview(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportPreview>;

    async fn resolve_import_conflict(
        &self,
        preview_id: Uuid,
        resolution: SpreadsheetImportResolution,
    ) -> PortResult<SpreadsheetImportPreview>;

    async fn commit_import(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportCommitResult>;

    async fn ai_match_package_context(&self, match_id: Uuid) -> PortResult<AiMatchPackageContext>;
}

#[async_trait]
pub trait SpreadsheetExchangePort: Send + Sync {
    async fn reference_data(&self) -> PortResult<PlayerCatalogReferenceData>;
    async fn export_data(&self) -> PortResult<SpreadsheetExportData>;
    async fn data_gaps(&self) -> PortResult<Vec<MonthlyDataGapRow>>;
    async fn preview_import(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PortResult<SpreadsheetImportPreview>;
    async fn preview_import_with_team_references(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
        package_team_references: &HashMap<String, String>,
    ) -> PortResult<SpreadsheetImportPreview>;
    async fn read_import_preview(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportPreview>;
    async fn resolve_conflict(
        &self,
        preview_id: Uuid,
        resolution: &SpreadsheetImportResolution,
    ) -> PortResult<SpreadsheetImportPreview>;
    async fn commit_import(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportCommitResult>;
}

#[async_trait]
pub trait MonthlyWorkbookPort: Send + Sync {
    async fn export_data(&self) -> PortResult<TeamMonthlyWorkbookData>;
    async fn preview_import(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PortResult<SpreadsheetImportPreview>;
    async fn read_import_preview(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportPreview>;
    async fn resolve_conflict(
        &self,
        preview_id: Uuid,
        resolution: &SpreadsheetImportResolution,
    ) -> PortResult<SpreadsheetImportPreview>;
    async fn commit_import(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportCommitResult>;
}
