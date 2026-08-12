use super::policy::ensure_preview_committable;
use crate::{
    ports::exchange::{MonthlyWorkbookPort, SpreadsheetExchangePort},
    ApplicationError, ApplicationResult,
};
use football_domain::{TeamPackageCommitRequest, TeamPackageCommitResult};
use std::future::Future;

pub(crate) async fn execute<P, F>(
    session: F,
    request: TeamPackageCommitRequest,
) -> ApplicationResult<TeamPackageCommitResult>
where
    P: SpreadsheetExchangePort + MonthlyWorkbookPort,
    F: Future<Output = ApplicationResult<P>>,
{
    if request.team_batch_id.is_none() && request.player_batch_id.is_none() {
        return Err(ApplicationError::Validation(
            "球队完整资料包没有可提交的预检批次".to_string(),
        ));
    }
    let port = session.await?;
    if let Some(batch_id) = request.team_batch_id {
        let preview = MonthlyWorkbookPort::read_import_preview(&port, batch_id)
            .await
            .map_err(|error| {
                ApplicationError::Validation(format!(
                    "完整资料包球队链预检批次 {batch_id} 无法读取：{error}"
                ))
            })?;
        ensure_preview_committable(&preview, "球队、教练和阵型")?;
    }
    if let Some(batch_id) = request.player_batch_id {
        let preview = SpreadsheetExchangePort::read_import_preview(&port, batch_id)
            .await
            .map_err(|error| {
                ApplicationError::Validation(format!(
                    "完整资料包球员链预检批次 {batch_id} 无法读取：{error}"
                ))
            })?;
        ensure_preview_committable(&preview, "球员、评分和动态状态")?;
    }
    let team_result = match request.team_batch_id {
        Some(batch_id) => Some(
            MonthlyWorkbookPort::commit_import(&port, batch_id)
                .await
                .map_err(|error| {
                    ApplicationError::Validation(format!(
                        "完整资料包球队、教练与阵型链提交失败（批次 {batch_id}）：{error}"
                    ))
                })?,
        ),
        None => None,
    };
    let player_result = match request.player_batch_id {
        Some(batch_id) => Some(
            SpreadsheetExchangePort::commit_import(&port, batch_id)
                .await
                .map_err(|error| {
                    let team_state = if team_result.is_some() {
                        "球队、教练与阵型链已经提交成功；可修复后直接重试同一完整资料包批次。"
                    } else {
                        ""
                    };
                    ApplicationError::Validation(format!(
                        "{team_state}完整资料包球员、评分与动态状态链提交失败（批次 {batch_id}）：{error}"
                    ))
                })?,
        ),
        None => None,
    };
    let inserted_count = team_result
        .iter()
        .chain(player_result.iter())
        .map(|value| value.inserted_count)
        .sum();
    let updated_count = team_result
        .iter()
        .chain(player_result.iter())
        .map(|value| value.updated_count)
        .sum();
    let ended_previous_count = team_result
        .iter()
        .chain(player_result.iter())
        .map(|value| value.ended_previous_count)
        .sum();
    let skipped_count = team_result
        .iter()
        .chain(player_result.iter())
        .map(|value| value.skipped_count)
        .sum();
    let error_count = team_result
        .iter()
        .chain(player_result.iter())
        .map(|value| value.error_count)
        .sum();
    Ok(TeamPackageCommitResult {
        team_result,
        player_result,
        inserted_count,
        updated_count,
        ended_previous_count,
        skipped_count,
        error_count,
    })
}
