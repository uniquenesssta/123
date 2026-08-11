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
