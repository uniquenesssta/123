use crate::{ports::exchange::MatchLineupExchangePort, ApplicationError, ApplicationResult};
use football_domain::MatchLineupExportSummary;
use football_spreadsheet_io::write_match_lineup_export;
use std::future::Future;
use uuid::Uuid;

use super::file_validation;

pub(crate) async fn execute<P, F>(
    session: F,
    output_path: String,
    match_id: Uuid,
) -> ApplicationResult<MatchLineupExportSummary>
where
    P: MatchLineupExchangePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let path = file_validation::validate_output(&output_path, "xlsx")?;
    let port = session.await?;
    let data = port.export_match_lineup(Some(match_id)).await?;
    let summary = MatchLineupExportSummary {
        output_path: path.to_string_lossy().to_string(),
        match_count: u64::from(data.selected_match.is_some()),
        lineup_count: data.lineups.len() as u64,
        player_count: data.players.len() as u64,
    };
    let output = path.clone();
    tokio::task::spawn_blocking(move || write_match_lineup_export(&output, &data))
        .await
        .map_err(|error| {
            ApplicationError::Validation(format!("比赛数据导出任务失败：{error}"))
        })??;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{PortError, PortErrorKind, PortResult};
    use crate::use_cases::prediction::tests::Probe;
    use async_trait::async_trait;
    use football_domain::{
        AiMatchPackageContext, MatchLineupExportData, SpreadsheetImportCommitResult,
        SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution,
        SpreadsheetParsedWorkbook,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn invalid_export_path_is_rejected_before_session_future_is_polled() {
        let port = Arc::new(Probe::new());
        let session = async {
            port.call("session")?;
            Ok(Arc::clone(&port))
        };
        let error = execute(session, "invalid.json".into(), Uuid::from_u128(31))
            .await
            .unwrap_err();
        assert!(matches!(error, ApplicationError::Validation(_)));
        assert!(port.calls().is_empty());
    }

    #[tokio::test]
    async fn failed_export_session_or_data_port_cannot_create_workbook() {
        for failure in ["session", "export_data"] {
            let port = Arc::new(Probe::new());
            port.fail_at(Some(failure));
            let directory = tempfile::tempdir().unwrap();
            let output = directory.path().join("match.xlsx");
            let session = async {
                port.call("session")?;
                Ok(Arc::clone(&port))
            };
            assert!(matches!(
                execute(
                    session,
                    output.to_string_lossy().into(),
                    Uuid::from_u128(31)
                )
                .await,
                Err(ApplicationError::Port(_))
            ));
            assert_eq!(
                port.calls(),
                if failure == "session" {
                    vec!["session"]
                } else {
                    vec!["session", "export_data"]
                }
            );
            assert!(!output.exists());
        }
    }

    #[tokio::test]
    async fn malformed_import_workbook_never_acquires_session_or_previews_batch() {
        let port = Arc::new(Probe::new());
        let directory = tempfile::tempdir().unwrap();
        let input = directory.path().join("malformed.xlsx");
        std::fs::write(&input, b"not an XLSX ZIP archive").unwrap();
        let session = async {
            port.call("session")?;
            Ok(Arc::clone(&port))
        };
        let error = super::super::preview_match_lineup_import::execute(
            session,
            input.to_string_lossy().into(),
            SpreadsheetImportMode::AddAndUpdate,
        )
        .await
        .unwrap_err();
        assert!(matches!(error, ApplicationError::Spreadsheet(_)));
        assert!(port.calls().is_empty());
    }

    #[async_trait]
    impl MatchLineupExchangePort for Arc<Probe> {
        async fn export_match_lineup(
            &self,
            match_id: Option<Uuid>,
        ) -> PortResult<MatchLineupExportData> {
            self.call("export_data")?;
            assert_eq!(match_id, Some(Uuid::from_u128(31)));
            Err(PortError::new(
                PortErrorKind::Unavailable,
                "unexpected successful export",
            ))
        }
        async fn preview_import(
            &self,
            _workbook: &SpreadsheetParsedWorkbook,
            _mode: SpreadsheetImportMode,
        ) -> PortResult<SpreadsheetImportPreview> {
            panic!("forbidden Port call: preview_import")
        }
        async fn read_import_preview(
            &self,
            _preview_id: Uuid,
        ) -> PortResult<SpreadsheetImportPreview> {
            panic!("forbidden Port call: read_import_preview")
        }
        async fn resolve_import_conflict(
            &self,
            _preview_id: Uuid,
            _resolution: SpreadsheetImportResolution,
        ) -> PortResult<SpreadsheetImportPreview> {
            panic!("forbidden Port call: resolve_import_conflict")
        }
        async fn commit_import(
            &self,
            _preview_id: Uuid,
        ) -> PortResult<SpreadsheetImportCommitResult> {
            panic!("forbidden Port call: commit_import")
        }
        async fn ai_match_package_context(
            &self,
            _match_id: Uuid,
        ) -> PortResult<AiMatchPackageContext> {
            panic!("forbidden Port call: ai_match_package_context")
        }
    }
}
