use super::{
    shared::{
        snapshot_from_lineups, snapshot_from_pair, validate_event_identities, validate_membership,
    },
    workbook,
};
use crate::{
    ports::review::{MatchReviewPackageSourcePort, MatchReviewPackageStatePort},
    ApplicationError, ApplicationResult,
};
use football_domain::{
    MatchReviewPackageComparison, MatchReviewPackageIdentityCheck, MatchReviewPackagePreview,
    MatchReviewPackageWorkflowAction,
};
use serde_json::Value;
use std::collections::HashSet;
use uuid::Uuid;

pub(crate) async fn execute<P>(
    port: &P,
    input_path: String,
    expected_match_id: Option<Uuid>,
) -> ApplicationResult<MatchReviewPackagePreview>
where
    P: MatchReviewPackageSourcePort + MatchReviewPackageStatePort + ?Sized,
{
    validate(port, input_path, expected_match_id, true).await
}

pub(crate) async fn validate<P>(
    port: &P,
    input_path: String,
    expected_match_id: Option<Uuid>,
    persist_preview: bool,
) -> ApplicationResult<MatchReviewPackagePreview>
where
    P: MatchReviewPackageSourcePort + MatchReviewPackageStatePort + ?Sized,
{
    let path = workbook::validate_path(&input_path, false)?;
    let mut preview = workbook::read_package(path).await?;
    let workflow = port.read_active_workflow(preview.match_id).await?;
    match workflow.as_ref() {
        None => preview
            .errors
            .push("当前比赛没有有效的导出记录，请先从本页重新导出资料包".to_string()),
        Some(value) if value.package_id != preview.package_id => preview.errors.push(
            "导入文件不是当前比赛最近一次导出的资料包；旧包或其他包不能继续本轮流程".to_string(),
        ),
        Some(value) if value.match_key != preview.match_key => preview
            .errors
            .push("资料包工作流中的比赛标识与文件不一致".to_string()),
        Some(_) => {}
    }

    let context = port.validation_context(preview.match_id).await?;
    let current_database = snapshot_from_lineups(
        &context.lineups,
        context.current_match.home_team_id,
        context.current_match.away_team_id,
        true,
        context.result.as_ref(),
    );
    let proposed_import = snapshot_from_pair(&preview.lineup_pair, &preview.review.result);
    preview.comparison = MatchReviewPackageComparison {
        pre_match: workflow
            .as_ref()
            .filter(|value| value.package_id == preview.package_id)
            .map(|value| value.pre_match_snapshot.clone())
            .unwrap_or_default(),
        current_database,
        proposed_import,
        identity: MatchReviewPackageIdentityCheck {
            package_id_matches_current_export: workflow
                .as_ref()
                .is_some_and(|value| value.package_id == preview.package_id),
            match_id_matches_selection: expected_match_id
                .is_none_or(|value| value == preview.match_id),
            match_key_matches_database: context.current_match.external_key == preview.match_key,
            team_identity_matches_database: context.current_match.home_team_id
                == preview.lineup_pair.home.team_id
                && context.current_match.away_team_id == preview.lineup_pair.away.team_id,
        },
    };
    if expected_match_id.is_some_and(|value| value != preview.match_id) {
        preview
            .errors
            .push("导入资料包与当前选择的比赛不一致".to_string());
    }
    if context.current_match.external_key != preview.match_key
        || context.current_match.home_team_id != preview.lineup_pair.home.team_id
        || context.current_match.away_team_id != preview.lineup_pair.away.team_id
    {
        preview
            .errors
            .push("资料包的比赛或主客队身份与当前数据库不一致".to_string());
    }
    if context.has_existing_result_or_review {
        preview.warnings.push(
            "当前比赛已有赛果或复盘记录；确认导入会新建实际阵容和复盘修订版本，不覆盖原赛前快照"
                .to_string(),
        );
    }
    let home_registered = context
        .home_registered_player_ids
        .into_iter()
        .collect::<HashSet<_>>();
    let away_registered = context
        .away_registered_player_ids
        .into_iter()
        .collect::<HashSet<_>>();
    validate_membership(
        &preview.lineup_pair.home.players,
        &home_registered,
        "主队",
        &mut preview.errors,
    );
    validate_membership(
        &preview.lineup_pair.away.players,
        &away_registered,
        "客队",
        &mut preview.errors,
    );
    validate_event_identities(&preview.lineup_pair, &preview.events, &mut preview.errors);
    if let Some(run_id) = preview.review.source_run_id {
        match port.read_source_run(run_id).await {
            Ok(run)
                if run.get("match_key").and_then(Value::as_str)
                    == Some(preview.match_key.as_str()) => {}
            Ok(_) => preview
                .errors
                .push("source_run_id 不属于当前比赛".to_string()),
            Err(_) => preview
                .errors
                .push("source_run_id 对应的赛前推演不存在".to_string()),
        }
    } else {
        preview
            .warnings
            .push("资料包没有绑定成功赛前推演；可以生成复盘，但正式结算门禁不会通过".to_string());
    }
    preview.ready = preview.errors.is_empty();
    if persist_preview {
        if let Some(active_workflow) = workflow
            .as_ref()
            .filter(|value| value.package_id == preview.package_id)
        {
            active_workflow
                .require_action(MatchReviewPackageWorkflowAction::PreviewImport)
                .map_err(ApplicationError::Validation)?;
            port.record_preview(preview.package_id, &preview).await?;
        }
    }
    Ok(preview)
}
