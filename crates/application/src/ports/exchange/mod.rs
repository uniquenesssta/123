use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    MatchLineupExportData, SpreadsheetExportData, SpreadsheetImportCommitResult,
    SpreadsheetImportPreview, SpreadsheetImportResolution, SpreadsheetParsedWorkbook,
    TeamMonthlyWorkbookData,
};
use uuid::Uuid;

#[async_trait]
pub trait MatchLineupExchangePort: Send + Sync {
    async fn export_match_lineup(&self, match_id: Uuid) -> PortResult<MatchLineupExportData>;
    async fn preview_import(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
    ) -> PortResult<SpreadsheetImportPreview>;
    async fn resolve_import_conflict(
        &self,
        preview_id: Uuid,
        resolution: &SpreadsheetImportResolution,
    ) -> PortResult<SpreadsheetImportPreview>;
    async fn commit_import(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportCommitResult>;
}

#[async_trait]
pub trait SpreadsheetExchangePort: Send + Sync {
    async fn export_data(&self) -> PortResult<SpreadsheetExportData>;
    async fn preview_import(
        &self,
        workbook: &SpreadsheetParsedWorkbook,
    ) -> PortResult<SpreadsheetImportPreview>;
    async fn resolve_conflict(
        &self,
        preview_id: Uuid,
        resolution: &SpreadsheetImportResolution,
    ) -> PortResult<SpreadsheetImportPreview>;
    async fn commit_import(&self, preview_id: Uuid) -> PortResult<SpreadsheetImportCommitResult>;
}

#[async_trait]
pub trait MonthlyWorkbookPort: Send + Sync {
    async fn team_monthly_data(&self, team_id: Uuid) -> PortResult<TeamMonthlyWorkbookData>;
}
