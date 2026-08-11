use crate::{ports::exchange::MatchLineupExchangePort, ApplicationError, ApplicationResult};
use football_domain::MatchLineupExportSummary;
use football_spreadsheet_io::write_match_lineup_template;
use std::future::Future;

use super::file_validation;

pub(crate) async fn execute<P, F>(
    session: F,
    output_path: String,
) -> ApplicationResult<MatchLineupExportSummary>
where
    P: MatchLineupExchangePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let path = file_validation::validate_output(&output_path, "xlsx")?;
    let port = session.await?;
    let data = port.export_match_lineup(None).await?;
    let player_count = data.players.len() as u64;
    let output = path.clone();
    tokio::task::spawn_blocking(move || write_match_lineup_template(&output, &data))
        .await
        .map_err(|error| {
            ApplicationError::Validation(format!("比赛模板导出任务失败：{error}"))
        })??;
    Ok(MatchLineupExportSummary {
        output_path: path.to_string_lossy().to_string(),
        match_count: 0,
        lineup_count: 0,
        player_count,
    })
}
