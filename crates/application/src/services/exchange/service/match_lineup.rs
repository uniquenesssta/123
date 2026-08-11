use super::ExchangeService;
use crate::{ports::exchange::MatchLineupExchangePort, use_cases::exchange, ApplicationResult};
use football_domain::{
    AiMatchPackageSummary, MatchLineupExportSummary, SpreadsheetImportCommitResult,
    SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution,
};
use std::future::Future;
use uuid::Uuid;

impl ExchangeService {
    pub(crate) async fn export_match_lineup_template<P, F>(&self, session: F, output_path: String) -> ApplicationResult<MatchLineupExportSummary>
    where P: MatchLineupExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::export_match_lineup_template::execute(session, output_path).await
    }
    pub(crate) async fn export_match_lineup_data<P, F>(&self, session: F, output_path: String, match_id: Uuid) -> ApplicationResult<MatchLineupExportSummary>
    where P: MatchLineupExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::export_match_lineup_data::execute(session, output_path, match_id).await
    }
    pub(crate) async fn preview_match_lineup_import<P, F>(&self, session: F, input_path: String, mode: SpreadsheetImportMode) -> ApplicationResult<SpreadsheetImportPreview>
    where P: MatchLineupExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::preview_match_lineup_import::execute(session, input_path, mode).await
    }
    pub(crate) async fn read_match_lineup_import_preview<P, F>(&self, session: F, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportPreview>
    where P: MatchLineupExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::read_match_lineup_import_preview::execute(session, batch_id).await
    }
    pub(crate) async fn resolve_match_lineup_import_conflict<P, F>(&self, session: F, batch_id: Uuid, resolution: SpreadsheetImportResolution) -> ApplicationResult<SpreadsheetImportPreview>
    where P: MatchLineupExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::resolve_match_lineup_import_conflict::execute(session, batch_id, resolution).await
    }
    pub(crate) async fn commit_match_lineup_import<P, F>(&self, session: F, batch_id: Uuid) -> ApplicationResult<SpreadsheetImportCommitResult>
    where P: MatchLineupExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::commit_match_lineup_import::execute(session, batch_id).await
    }
    pub(crate) async fn export_ai_match_package<P, F>(&self, session: F, output_path: String, match_id: Uuid) -> ApplicationResult<AiMatchPackageSummary>
    where P: MatchLineupExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::export_ai_match_package::execute(session, output_path, match_id).await
    }
    pub(crate) async fn preview_ai_match_package<P, F>(&self, session: F, input_path: String, mode: SpreadsheetImportMode) -> ApplicationResult<SpreadsheetImportPreview>
    where P: MatchLineupExchangePort, F: Future<Output = ApplicationResult<P>> {
        exchange::preview_ai_match_package::execute(session, input_path, mode).await
    }
}
