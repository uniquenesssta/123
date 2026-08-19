use super::{
    counts::force_delete_counts,
    execute::execute_force_delete,
    targets::{lock_team, prepare_force_delete_targets, temp_ids},
};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{TeamForceDeletePreview, TeamForceDeleteRequest, TeamForceDeleteResult};
use serde_json::json;
use std::collections::BTreeMap;
use uuid::Uuid;

impl PostgresStore {
    pub async fn preview_force_delete_team(
        &self,
        team_id: Uuid,
    ) -> PersistenceResult<TeamForceDeletePreview> {
        let mut tx = self.pool.begin().await?;
        let label = lock_team(&mut tx, team_id).await?;
        prepare_force_delete_targets(&mut tx, team_id, &label).await?;
        let references = force_delete_counts(&mut tx, team_id).await?;
        let total_rows = references.iter().map(|item| item.count.max(0) as u64).sum();
        tx.rollback().await?;

        Ok(TeamForceDeletePreview {
            team_id,
            label: label.clone(),
            confirmation_text: label,
            total_rows,
            references,
            warning: "该操作会永久删除球队、关联球员与教练、相关比赛、P4 快照与运行、评分、动态状态、导入批次及可追溯历史，无法撤销。"
                .to_string(),
        })
    }

    pub async fn force_delete_team(
        &self,
        request: &TeamForceDeleteRequest,
    ) -> PersistenceResult<TeamForceDeleteResult> {
        let mut tx = self.pool.begin().await?;
        let label = lock_team(&mut tx, request.team_id).await?;
        if request.confirmation_text.trim() != label {
            return Err(PersistenceError::InvalidState(format!(
                "确认文字不匹配；请输入完整球队名称：{label}"
            )));
        }

        prepare_force_delete_targets(&mut tx, request.team_id, &label).await?;
        let deleted_match_ids = temp_ids(&mut tx, "purge_matches").await?;
        let deleted_player_ids = temp_ids(&mut tx, "purge_players").await?;
        let deleted_coach_ids = temp_ids(&mut tx, "purge_coaches").await?;
        let deleted_import_batch_ids = temp_ids(&mut tx, "purge_import_batches").await?;
        let deleted_counts = force_delete_counts(&mut tx, request.team_id)
            .await?
            .into_iter()
            .map(|item| (item.relation, item.count.max(0) as u64))
            .collect::<BTreeMap<_, _>>();

        execute_force_delete(&mut tx, request.team_id).await?;

        crate::write_audit_event(
            &mut tx,
            "team_force_deleted",
            "team_purge",
            request.team_id.to_string(),
            json!({
                "team_name": label,
                "deleted_counts": &deleted_counts,
                "deleted_match_ids": &deleted_match_ids,
                "deleted_player_ids": &deleted_player_ids,
                "deleted_coach_ids": &deleted_coach_ids,
                "deleted_import_batch_ids": &deleted_import_batch_ids,
            }),
        )
        .await?;

        tx.commit().await?;
        Ok(TeamForceDeleteResult {
            team_id: request.team_id,
            label,
            deleted_match_ids,
            deleted_player_ids,
            deleted_coach_ids,
            deleted_import_batch_ids,
            deleted_counts,
        })
    }
}
