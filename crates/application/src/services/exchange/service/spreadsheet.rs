use super::ExchangeService;
use crate::{
    ports::exchange::{MonthlyWorkbookPort, SpreadsheetExchangePort},
    use_cases::exchange,
    ApplicationResult,
};
use football_domain::{
    MonthlyWorkbookExportSummary, SpreadsheetExportSummary, SpreadsheetImportCommitResult,
    SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution,
    TeamPackageCommitRequest, TeamPackageCommitResult, TeamPackageExportSummary,
    TeamPackageImportPreview, TeamPackagePreviewExportSummary,
};
use std::future::Future;
use uuid::Uuid;

impl ExchangeService {
    pub(crate) async fn export_team_package_template<P, F>(&self, session: F, output_path: String) -> ApplicationResult<TeamPackageExportSummary>
    where P: SpreadsheetExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::export_team_package_template::execute(session, output_path).await
    }
    pub(crate) async fn export_team_package_preview_json(&self, output_path: String, preview: TeamPackageImportPreview) -> ApplicationResult<TeamPackagePreviewExportSummary> {
        exchange::export_team_package_preview_json::execute(output_path, preview).await
    }
    pub(crate) async fn preview_team_package_import<P, F>(&self, session: F, input_path: String, mode: SpreadsheetImportMode) -> ApplicationResult<TeamPackageImportPreview>
    where P: SpreadsheetExchangePort + MonthlyWorkbookPort, F: Future<Output = ApplicationResult<P>> {
        exchange::preview_team_package_import::execute(session, input_path, mode).await
    }
    pub(crate) async fn commit_team_package_import<P, F>(&self, session: F, request: TeamPackageCommitRequest) -> ApplicationResult<TeamPackageCommitResult>
    where P: SpreadsheetExchangePort + MonthlyWorkbookPort, F: Future<Output = ApplicationResult<P>> {
        exchange::commit_team_package_import::execute(session, request).await
    }
    pub(crate) async fn export_player_catalog_template<P, F>(&self, session: F, output_path: String) -> ApplicationResult<SpreadsheetExportSummary>
    where P: SpreadsheetExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::export_player_catalog_template::execute(session, output_path).await
    }
    pub(crate) async fn export_player_catalog_data<P, F>(&self, session: F, output_path: String) -> ApplicationResult<SpreadsheetExportSummary>
    where P: SpreadsheetExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::export_player_catalog_data::execute(session, output_path).await
    }
    pub(crate) async fn preview_player_catalog_import<P, F>(&self, session: F, input_path: String, mode: SpreadsheetImportMode) -> ApplicationResult<SpreadsheetImportPreview>
    where P: SpreadsheetExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::preview_player_catalog_import::execute(session, input_path, mode).await
    }
    pub(crate) async fn resolve_player_catalog_import_conflict<P, F>(&self, session: F, batch_id: Uuid, resolution: SpreadsheetImportResolution) -> ApplicationResult<SpreadsheetImportPreview>
    where P: SpreadsheetExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::resolve_player_catalog_import_conflict::execute(session, batch_id, resolution).await
    }
    pub(crate) async fn commit_player_catalog_import<P, F>(&self, session: F, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportCommitResult>
    where P: SpreadsheetExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::commit_player_catalog_import::execute(session, batch_id).await
    }
    pub(crate) async fn read_player_catalog_import_preview<P, F>(&self, session: F, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportPreview>
    where P: SpreadsheetExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::read_player_catalog_import_preview::execute(session, batch_id).await
    }
    pub(crate) async fn export_team_monthly_template<P, F>(&self, session: F, output_path: String) -> ApplicationResult<MonthlyWorkbookExportSummary>
    where P: SpreadsheetExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::export_team_monthly_template::execute(session, output_path).await
    }
    pub(crate) async fn export_team_monthly_data<P, F>(&self, session: F, output_path: String) -> ApplicationResult<MonthlyWorkbookExportSummary>
    where P: SpreadsheetExchangePort + MonthlyWorkbookPort, F: Future<Output = ApplicationResult<P>> {
        exchange::export_team_monthly_data::execute(session, output_path).await
    }
    pub(crate) async fn preview_team_monthly_import<P, F>(&self, session: F, input_path: String, mode: SpreadsheetImportMode) -> ApplicationResult<SpreadsheetImportPreview>
    where P: MonthlyWorkbookPort, F: Future<Output = ApplicationResult<P>> {
        exchange::preview_team_monthly_import::execute(session, input_path, mode).await
    }
    pub(crate) async fn read_team_monthly_import_preview<P, F>(&self, session: F, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportPreview>
    where P: MonthlyWorkbookPort, F: Future<Output = ApplicationResult<P>> {
        exchange::read_team_monthly_import_preview::execute(session, batch_id).await
    }
    pub(crate) async fn resolve_team_monthly_import_conflict<P, F>(&self, session: F, batch_id: Uuid, resolution: SpreadsheetImportResolution) -> ApplicationResult<SpreadsheetImportPreview>
    where P: MonthlyWorkbookPort, F: Future<Output = ApplicationResult<P>> {
        exchange::resolve_team_monthly_import_conflict::execute(session, batch_id, resolution).await
    }
    pub(crate) async fn commit_team_monthly_import<P, F>(&self, session: F, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportCommitResult>
    where P: MonthlyWorkbookPort, F: Future<Output = ApplicationResult<P>> {
        exchange::commit_team_monthly_import::execute(session, batch_id).await
    }
}
