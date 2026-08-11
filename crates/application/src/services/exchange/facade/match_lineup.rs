use crate::{ApplicationResult, ApplicationService};
use football_domain::{
    AiMatchPackageSummary, MatchLineupExportSummary, SpreadsheetImportCommitResult,
    SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution,
};
use uuid::Uuid;

impl ApplicationService {
    pub async fn export_match_lineup_template(&self, output_path: String) -> ApplicationResult<MatchLineupExportSummary> {
        self.exchange.export_match_lineup_template(self.exchange_session(), output_path).await
    }
    pub async fn export_match_lineup_data(&self, output_path: String, match_id: Uuid) -> ApplicationResult<MatchLineupExportSummary> {
        self.exchange.export_match_lineup_data(self.exchange_session(), output_path, match_id).await
    }
    pub async fn preview_match_lineup_import(&self, input_path: String, mode: SpreadsheetImportMode) -> ApplicationResult<SpreadsheetImportPreview> {
        self.exchange.preview_match_lineup_import(self.exchange_session(), input_path, mode).await
    }
    pub async fn read_match_lineup_import_preview(&self, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportPreview> {
        self.exchange.read_match_lineup_import_preview(self.exchange_session(), batch_id).await
    }
    pub async fn resolve_match_lineup_import_conflict(&self, batch_id: Uuid, resolution: SpreadsheetImportResolution) -> ApplicationResult<SpreadsheetImportPreview> {
        self.exchange.resolve_match_lineup_import_conflict(self.exchange_session(), batch_id, resolution).await
    }
    pub async fn commit_match_lineup_import(&self, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportCommitResult> {
        self.exchange.commit_match_lineup_import(self.exchange_session(), batch_id).await
    }
    pub async fn export_ai_match_package(&self, output_path: String, match_id: Uuid) -> ApplicationResult<AiMatchPackageSummary> {
        self.exchange.export_ai_match_package(self.exchange_session(), output_path, match_id).await
    }
    pub async fn preview_ai_match_package(&self, input_path: String, mode: SpreadsheetImportMode) -> ApplicationResult<SpreadsheetImportPreview> {
        self.exchange.preview_ai_match_package(self.exchange_session(), input_path, mode).await
    }
}
