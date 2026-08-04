use crate::{ApplicationError, ApplicationResult, ApplicationService};
use chrono::Utc;
use football_domain::{
    LineupPairDraft, LineupRecord, LineupType, MatchResultDraft, MatchResultRecord,
    MatchReviewPackageCommitRequest, MatchReviewPackageCommitResult, MatchReviewPackageComparison,
    MatchReviewPackageConfirmationRequest, MatchReviewPackageData,
    MatchReviewPackageFactsCommitResult, MatchReviewPackageIdentityCheck,
    MatchReviewPackagePreview, MatchReviewPackageReviewResult, MatchReviewPackageSnapshotSummary,
    MatchReviewPackageSummary, MatchReviewPackageWorkflowAction, MatchReviewPackageWorkflowRecord,
};
use football_spreadsheet_io::{read_match_review_package, write_match_review_package};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use uuid::Uuid;

impl ApplicationService {
    pub async fn export_match_review_package(
        &self,
        output_path: String,
        match_id: Uuid,
    ) -> ApplicationResult<MatchReviewPackageSummary> {
        let path = validate_path(&output_path, true)?;
        let store = self.active_store().await?;
        let context = store.ai_match_package_context(match_id).await?;
        let home_team = store.read_team(context.match_record.home_team_id).await?;
        let away_team = store.read_team(context.match_record.away_team_id).await?;
        let reviewable = store
            .list_reviewable_matches(200)
            .await?
            .into_iter()
            .find(|item| item.match_record.id == match_id);
        let latest_model_run_id = store
            .list_recent_runs(500)
            .await?
            .into_iter()
            .find(|item| item.match_key == context.match_record.external_key)
            .map(|item| item.id);
        let latest_model_run = match latest_model_run_id {
            Some(run_id) => Some(store.read_run(run_id).await?),
            None => None,
        };
        let package_id = Uuid::new_v4();
        let data = MatchReviewPackageData {
            package_id,
            exported_at: Utc::now(),
            match_record: context.match_record.clone(),
            home_team,
            away_team,
            pre_match_lineups: context.lineups.clone(),
            player_context: context.players,
            existing_result: reviewable.as_ref().and_then(|item| item.result.clone()),
            latest_review: reviewable.and_then(|item| item.latest_review),
            latest_model_run,
        };
        let lineup_count = data.pre_match_lineups.len() as u64;
        let player_count = data
            .home_team
            .squad
            .iter()
            .map(|item| item.player_id)
            .chain(data.away_team.squad.iter().map(|item| item.player_id))
            .collect::<HashSet<_>>()
            .len() as u64;
        let pre_match_snapshot = snapshot_from_lineups(
            &data.pre_match_lineups,
            data.match_record.home_team_id,
            data.match_record.away_team_id,
            false,
            None,
        );
        let export_database_snapshot = snapshot_from_lineups(
            &data.pre_match_lineups,
            data.match_record.home_team_id,
            data.match_record.away_team_id,
            true,
            data.existing_result.as_ref(),
        );
        let output = path.clone();
        tokio::task::spawn_blocking(move || write_match_review_package(&output, &data))
            .await
            .map_err(|error| {
                ApplicationError::Validation(format!("赛后复盘资料包导出任务失败：{error}"))
            })??;
        let summary = MatchReviewPackageSummary {
            output_path: path.to_string_lossy().to_string(),
            package_id,
            match_id,
            match_key: context.match_record.external_key,
            lineup_count,
            player_count,
            content_sha256: sha256_file(&path)?,
            pre_match_snapshot,
            export_database_snapshot,
        };
        store.register_match_review_package_export(&summary).await?;
        Ok(summary)
    }

    pub async fn read_match_review_package_workflow(
        &self,
        match_id: Uuid,
    ) -> ApplicationResult<Option<MatchReviewPackageWorkflowRecord>> {
        Ok(self
            .active_store()
            .await?
            .read_active_match_review_package_workflow(match_id)
            .await?)
    }

    pub async fn preview_match_review_package(
        &self,
        input_path: String,
        expected_match_id: Option<Uuid>,
    ) -> ApplicationResult<MatchReviewPackagePreview> {
        self.validate_match_review_package(input_path, expected_match_id, true)
            .await
    }

    pub async fn confirm_match_review_package(
        &self,
        request: MatchReviewPackageConfirmationRequest,
    ) -> ApplicationResult<MatchReviewPackageWorkflowRecord> {
        let store = self.active_store().await?;
        let workflow = store
            .read_match_review_package_workflow(request.package_id)
            .await?;
        workflow
            .require_action(MatchReviewPackageWorkflowAction::ConfirmImport)
            .map_err(ApplicationError::Validation)?;
        if !workflow.preview_ready {
            return Err(ApplicationError::Validation(
                "本轮导出的资料包尚未通过预检，不能人工确认".to_string(),
            ));
        }
        let import_path = workflow.import_path.clone().ok_or_else(|| {
            ApplicationError::Validation("预检记录缺少导入文件路径，请重新预检".to_string())
        })?;
        let preview = self
            .validate_match_review_package(import_path, Some(workflow.match_id), false)
            .await?;
        if preview.package_id != workflow.package_id
            || preview.source_sha256 != workflow.import_sha256.as_deref().unwrap_or_default()
            || !preview.ready
            || !preview.errors.is_empty()
        {
            return Err(ApplicationError::Validation(
                "资料包在预检后已变化或重新校验失败，请重新预检".to_string(),
            ));
        }
        Ok(store
            .confirm_match_review_package_workflow(
                request.package_id,
                request.confirmed_by.as_deref(),
                request.confirmation_note.as_deref(),
            )
            .await?)
    }

    pub async fn commit_match_review_package_facts(
        &self,
        package_id: Uuid,
    ) -> ApplicationResult<MatchReviewPackageFactsCommitResult> {
        let store = self.active_store().await?;
        let workflow = store.read_match_review_package_workflow(package_id).await?;
        workflow
            .require_action(MatchReviewPackageWorkflowAction::CommitFacts)
            .map_err(ApplicationError::Validation)?;
        let import_path = workflow.import_path.clone().ok_or_else(|| {
            ApplicationError::Validation("确认记录缺少导入文件路径，请重新预检".to_string())
        })?;
        let mut preview = self
            .validate_match_review_package(import_path, Some(workflow.match_id), false)
            .await?;
        if preview.package_id != workflow.package_id
            || preview.source_sha256 != workflow.import_sha256.as_deref().unwrap_or_default()
            || !preview.ready
            || !preview.errors.is_empty()
        {
            return Err(ApplicationError::Validation(
                "已确认资料包发生变化，拒绝写入赛后事实".to_string(),
            ));
        }
        apply_confirmation_metadata(&mut preview, &workflow);
        let pair = store.create_lineup_pair(&preview.lineup_pair).await?;
        store.commit_match_review_facts(&preview.review).await?;
        let workflow = store
            .mark_match_review_package_facts_committed(package_id)
            .await?;
        Ok(MatchReviewPackageFactsCommitResult {
            home_lineup_id: pair.home.id,
            away_lineup_id: pair.away.id,
            workflow,
        })
    }

    pub async fn generate_match_review_from_package(
        &self,
        package_id: Uuid,
    ) -> ApplicationResult<MatchReviewPackageReviewResult> {
        let store = self.active_store().await?;
        let workflow = store.read_match_review_package_workflow(package_id).await?;
        workflow
            .require_action(MatchReviewPackageWorkflowAction::GenerateReview)
            .map_err(ApplicationError::Validation)?;
        let mut preview = store.read_match_review_package_preview(package_id).await?;
        apply_confirmation_metadata(&mut preview, &workflow);
        let review = store.generate_match_review(&preview.review).await?;
        let workflow = store
            .mark_match_review_package_review_created(package_id, review.summary.id)
            .await?;
        Ok(MatchReviewPackageReviewResult { review, workflow })
    }

    pub async fn commit_match_review_package(
        &self,
        request: MatchReviewPackageCommitRequest,
    ) -> ApplicationResult<MatchReviewPackageCommitResult> {
        if !request.preview.ready || !request.preview.errors.is_empty() {
            return Err(ApplicationError::Validation(
                "赛后复盘资料包仍有阻断错误，不能确认导入".to_string(),
            ));
        }
        let store = self.active_store().await?;
        let workflow = store
            .read_match_review_package_workflow(request.preview.package_id)
            .await?;
        if workflow
            .require_action(MatchReviewPackageWorkflowAction::ConfirmImport)
            .is_err()
            || workflow.import_sha256.as_deref() != Some(request.preview.source_sha256.as_str())
            || workflow.match_id != request.preview.match_id
        {
            return Err(ApplicationError::Validation(
                "当前预检不属于本轮导出的资料包，请重新导出并预检".to_string(),
            ));
        }
        self.confirm_match_review_package(MatchReviewPackageConfirmationRequest {
            package_id: request.preview.package_id,
            confirmed_by: request.confirmed_by,
            confirmation_note: request.confirmation_note,
        })
        .await?;
        let facts = self
            .commit_match_review_package_facts(request.preview.package_id)
            .await?;
        let generated = self
            .generate_match_review_from_package(request.preview.package_id)
            .await?;
        Ok(MatchReviewPackageCommitResult {
            home_lineup_id: facts.home_lineup_id,
            away_lineup_id: facts.away_lineup_id,
            review: generated.review,
        })
    }

    async fn validate_match_review_package(
        &self,
        input_path: String,
        expected_match_id: Option<Uuid>,
        persist_preview: bool,
    ) -> ApplicationResult<MatchReviewPackagePreview> {
        let path = validate_path(&input_path, false)?;
        let read_path = path.clone();
        let mut preview =
            tokio::task::spawn_blocking(move || read_match_review_package(&read_path))
                .await
                .map_err(|error| {
                    ApplicationError::Validation(format!("赛后复盘资料包读取任务失败：{error}"))
                })??;
        let store = self.active_store().await?;
        let workflow = store
            .read_active_match_review_package_workflow(preview.match_id)
            .await?;
        match workflow.as_ref() {
            None => preview
                .errors
                .push("当前比赛没有有效的导出记录，请先从本页重新导出资料包".to_string()),
            Some(value) if value.package_id != preview.package_id => preview.errors.push(
                "导入文件不是当前比赛最近一次导出的资料包；旧包或其他包不能继续本轮流程"
                    .to_string(),
            ),
            Some(value) if value.match_key != preview.match_key => preview
                .errors
                .push("资料包工作流中的比赛标识与文件不一致".to_string()),
            Some(_) => {}
        }
        let current_export_data = store
            .match_lineup_export_data(Some(preview.match_id))
            .await?;
        let current_match = current_export_data
            .selected_match
            .clone()
            .ok_or_else(|| ApplicationError::Validation("资料包关联的比赛不存在".to_string()))?;
        let current_result = store.read_match_result(preview.match_id).await?;
        let current_database = snapshot_from_lineups(
            &current_export_data.lineups,
            current_match.home_team_id,
            current_match.away_team_id,
            true,
            current_result.as_ref(),
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
                match_key_matches_database: current_match.external_key == preview.match_key,
                team_identity_matches_database: current_match.home_team_id
                    == preview.lineup_pair.home.team_id
                    && current_match.away_team_id == preview.lineup_pair.away.team_id,
            },
        };
        if expected_match_id.is_some_and(|value| value != preview.match_id) {
            preview
                .errors
                .push("导入资料包与当前选择的比赛不一致".to_string());
        }
        if current_match.external_key != preview.match_key
            || current_match.home_team_id != preview.lineup_pair.home.team_id
            || current_match.away_team_id != preview.lineup_pair.away.team_id
        {
            preview
                .errors
                .push("资料包的比赛或主客队身份与当前数据库不一致".to_string());
        }
        if let Some(existing) = store
            .list_reviewable_matches(200)
            .await?
            .into_iter()
            .find(|item| item.match_record.id == preview.match_id)
        {
            if existing.result.is_some() || existing.latest_review.is_some() {
                preview.warnings.push(
                    "当前比赛已有赛果或复盘记录；确认导入会新建实际阵容和复盘修订版本，不覆盖原赛前快照"
                        .to_string(),
                );
            }
        }
        let home_team = store.read_team(current_match.home_team_id).await?;
        let away_team = store.read_team(current_match.away_team_id).await?;
        validate_membership(
            &preview.lineup_pair.home.players,
            &home_team.squad.iter().map(|item| item.player_id).collect(),
            "主队",
            &mut preview.errors,
        );
        validate_membership(
            &preview.lineup_pair.away.players,
            &away_team.squad.iter().map(|item| item.player_id).collect(),
            "客队",
            &mut preview.errors,
        );
        validate_event_identities(&preview.lineup_pair, &preview.events, &mut preview.errors);
        if let Some(run_id) = preview.review.source_run_id {
            match store.read_run(run_id).await {
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
            preview.warnings.push(
                "资料包没有绑定成功赛前推演；可以生成复盘，但正式结算门禁不会通过".to_string(),
            );
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
                store
                    .record_match_review_package_preview(preview.package_id, &preview)
                    .await?;
            }
        }
        Ok(preview)
    }
}

fn apply_confirmation_metadata(
    preview: &mut MatchReviewPackagePreview,
    workflow: &MatchReviewPackageWorkflowRecord,
) {
    let confirmation = json!({
        "package_id": workflow.package_id,
        "source_sha256": workflow.import_sha256.as_deref(),
        "confirmed_by": workflow.confirmed_by.as_deref(),
        "confirmation_note": workflow.confirmation_note.as_deref(),
        "confirmed_at": workflow.confirmed_at.as_ref(),
    });
    merge_metadata(
        &mut preview.review.result.metadata,
        "review_package_confirmation",
        confirmation.clone(),
    );
    merge_metadata(
        &mut preview.lineup_pair.home.metadata,
        "review_package_confirmation",
        confirmation.clone(),
    );
    merge_metadata(
        &mut preview.lineup_pair.away.metadata,
        "review_package_confirmation",
        confirmation,
    );
    if let Some(note) = workflow
        .confirmation_note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        preview.review.notes = Some(match preview.review.notes.take() {
            Some(existing) if !existing.trim().is_empty() => {
                format!("{}\n人工确认：{}", existing.trim(), note)
            }
            _ => format!("人工确认：{note}"),
        });
    }
}

fn validate_membership(
    players: &[football_domain::LineupPlayerDraft],
    registered: &HashSet<Uuid>,
    label: &str,
    errors: &mut Vec<String>,
) {
    for player in players {
        if !registered.contains(&player.player_id) && !player.membership_override {
            errors.push(format!(
                "{label}球员 {} 不在当前球队登记名单；确认真实出场时需将 membership_override 设为 true 并说明",
                player.player_id
            ));
        }
    }
}

fn validate_event_identities(
    lineup_pair: &LineupPairDraft,
    events: &[football_domain::MatchReviewEventDraft],
    errors: &mut Vec<String>,
) {
    let player_teams = lineup_pair
        .home
        .players
        .iter()
        .map(|player| (player.player_id, lineup_pair.home.team_id))
        .chain(
            lineup_pair
                .away
                .players
                .iter()
                .map(|player| (player.player_id, lineup_pair.away.team_id)),
        )
        .collect::<std::collections::HashMap<_, _>>();
    for (index, event) in events.iter().enumerate() {
        for (role, player_id) in [
            ("player_id", event.player_id),
            ("related_player_id", event.related_player_id),
        ] {
            let Some(player_id) = player_id else { continue };
            let Some(player_team_id) = player_teams.get(&player_id) else {
                errors.push(format!(
                    "比赛事件第 {} 行的 {role} 不在准备导入的比赛名单中",
                    index + 2
                ));
                continue;
            };
            let team_matches_event = event.team_id.is_none_or(|team_id| {
                if event.event_type == football_domain::MatchEventType::OwnGoal
                    && role == "player_id"
                {
                    team_id != *player_team_id
                } else {
                    team_id == *player_team_id
                }
            });
            if !team_matches_event {
                let expected = if event.event_type == football_domain::MatchEventType::OwnGoal
                    && role == "player_id"
                {
                    "乌龙球球员应属于事件受益球队的对手"
                } else {
                    "球员应属于事件球队"
                };
                errors.push(format!(
                    "比赛事件第 {} 行的 {role} 与 team_id 不一致：{expected}",
                    index + 2
                ));
            }
        }
    }
}

fn merge_metadata(target: &mut Value, key: &str, value: Value) {
    if !target.is_object() {
        *target = json!({});
    }
    if let Some(object) = target.as_object_mut() {
        object.insert(key.to_string(), value);
    }
}

fn snapshot_from_lineups(
    lineups: &[LineupRecord],
    home_team_id: Uuid,
    away_team_id: Uuid,
    actual: bool,
    result: Option<&MatchResultRecord>,
) -> MatchReviewPackageSnapshotSummary {
    let preferred = |team_id| {
        lineups
            .iter()
            .filter(|lineup| {
                lineup.team_id == team_id
                    && matches!(lineup.lineup_type, LineupType::Actual) == actual
            })
            .min_by_key(|lineup| match lineup.lineup_type {
                LineupType::Confirmed => 0,
                LineupType::Expected => 1,
                LineupType::Actual => 2,
            })
    };
    let home = preferred(home_team_id);
    let away = preferred(away_team_id);
    MatchReviewPackageSnapshotSummary {
        home_goals_90: result.map(|value| value.home_goals_90),
        away_goals_90: result.map(|value| value.away_goals_90),
        home_player_count: home.map_or(0, |value| value.players.len() as u64),
        away_player_count: away.map_or(0, |value| value.players.len() as u64),
        home_starter_count: home.map_or(0, |value| {
            value
                .players
                .iter()
                .filter(|player| player.is_starter)
                .count() as u64
        }),
        away_starter_count: away.map_or(0, |value| {
            value
                .players
                .iter()
                .filter(|player| player.is_starter)
                .count() as u64
        }),
    }
}

fn snapshot_from_pair(
    pair: &LineupPairDraft,
    result: &MatchResultDraft,
) -> MatchReviewPackageSnapshotSummary {
    MatchReviewPackageSnapshotSummary {
        home_goals_90: Some(result.home_goals_90),
        away_goals_90: Some(result.away_goals_90),
        home_player_count: pair.home.players.len() as u64,
        away_player_count: pair.away.players.len() as u64,
        home_starter_count: pair
            .home
            .players
            .iter()
            .filter(|player| player.is_starter)
            .count() as u64,
        away_starter_count: pair
            .away
            .players
            .iter()
            .filter(|player| player.is_starter)
            .count() as u64,
    }
}

fn validate_path(value: &str, output: bool) -> ApplicationResult<PathBuf> {
    let path = PathBuf::from(value.trim());
    if path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
        != Some("xlsx")
    {
        return Err(ApplicationError::Validation(
            "赛后复盘资料包必须使用 .xlsx 扩展名".to_string(),
        ));
    }
    if output {
        if let Some(parent) = path.parent().filter(|value| !value.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|error| {
                ApplicationError::Validation(format!(
                    "无法创建输出目录 {}：{error}",
                    parent.display()
                ))
            })?;
        }
    } else if !path.is_file() {
        return Err(ApplicationError::Validation(format!(
            "文件不存在：{}",
            path.display()
        )));
    }
    Ok(path)
}

fn sha256_file(path: &Path) -> ApplicationResult<String> {
    let bytes = std::fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
