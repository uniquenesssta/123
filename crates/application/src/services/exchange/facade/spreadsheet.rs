use crate::{ApplicationResult, ApplicationService};
use football_domain::{
    MonthlyWorkbookExportSummary, SpreadsheetExportSummary, SpreadsheetImportCommitResult,
    SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution,
    TeamPackageCommitRequest, TeamPackageCommitResult, TeamPackageExportSummary,
    TeamPackageImportPreview, TeamPackagePreviewExportSummary,
};
use uuid::Uuid;

impl ApplicationService {
    pub async fn export_team_package_template(&self, output_path: String) -> ApplicationResult<TeamPackageExportSummary> {
        self.exchange.export_team_package_template(self.exchange_session(), output_path).await
    }
    pub async fn export_team_package_preview_json(&self, output_path: String, preview: TeamPackageImportPreview) -> ApplicationResult<TeamPackagePreviewExportSummary> {
        self.exchange.export_team_package_preview_json(output_path, preview).await
    }
    pub async fn preview_team_package_import(&self, input_path: String, mode: SpreadsheetImportMode) -> ApplicationResult<TeamPackageImportPreview> {
        self.exchange.preview_team_package_import(self.exchange_session(), input_path, mode).await
    }
    pub async fn commit_team_package_import(&self, request: TeamPackageCommitRequest) -> ApplicationResult<TeamPackageCommitResult> {
        self.exchange.commit_team_package_import(self.exchange_session(), request).await
    }
    pub async fn export_player_catalog_template(&self, output_path: String) -> ApplicationResult<SpreadsheetExportSummary> {
        self.exchange.export_player_catalog_template(self.exchange_session(), output_path).await
    }
    pub async fn export_player_catalog_data(&self, output_path: String) -> ApplicationResult<SpreadsheetExportSummary> {
        self.exchange.export_player_catalog_data(self.exchange_session(), output_path).await
    }
    pub async fn preview_player_catalog_import(&self, input_path: String, mode: SpreadsheetImportMode) -> ApplicationResult<SpreadsheetImportPreview> {
        self.exchange.preview_player_catalog_import(self.exchange_session(), input_path, mode).await
    }
    pub async fn resolve_player_catalog_import_conflict(&self, batch_id: Uuid, resolution: SpreadsheetImportResolution) -> ApplicationResult<SpreadsheetImportPreview> {
        self.exchange.resolve_player_catalog_import_conflict(self.exchange_session(), batch_id, resolution).await
    }
    pub async fn commit_player_catalog_import(&self, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportCommitResult> {
        self.exchange.commit_player_catalog_import(self.exchange_session(), batch_id).await
    }
    pub async fn read_player_catalog_import_preview(&self, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportPreview> {
        self.exchange.read_player_catalog_import_preview(self.exchange_session(), batch_id).await
    }
    pub async fn export_team_monthly_template(&self, output_path: String) -> ApplicationResult<MonthlyWorkbookExportSummary> {
        self.exchange.export_team_monthly_template(self.exchange_session(), output_path).await
    }
    pub async fn export_team_monthly_data(&self, output_path: String) -> ApplicationResult<MonthlyWorkbookExportSummary> {
        self.exchange.export_team_monthly_data(self.exchange_session(), output_path).await
    }
    pub async fn preview_team_monthly_import(&self, input_path: String, mode: SpreadsheetImportMode) -> ApplicationResult<SpreadsheetImportPreview> {
        self.exchange.preview_team_monthly_import(self.exchange_session(), input_path, mode).await
    }
    pub async fn read_team_monthly_import_preview(&self, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportPreview> {
        self.exchange.read_team_monthly_import_preview(self.exchange_session(), batch_id).await
    }
    pub async fn resolve_team_monthly_import_conflict(&self, batch_id: Uuid, resolution: SpreadsheetImportResolution) -> ApplicationResult<SpreadsheetImportPreview> {
        self.exchange.resolve_team_monthly_import_conflict(self.exchange_session(), batch_id, resolution).await
    }
    pub async fn commit_team_monthly_import(&self, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportCommitResult> {
        self.exchange.commit_team_monthly_import(self.exchange_session(), batch_id).await
    }
}
