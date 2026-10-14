use super::policy::{ensure_promotable, ensure_rollback_allowed, required_uuid};
use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    ParameterPromotionDecisionRecord, ParameterPromotionRequest, ParameterRollbackRequest,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn promote_parameter_candidate(
        &self,
        request: &ParameterPromotionRequest,
    ) -> PersistenceResult<ParameterPromotionDecisionRecord> {
        let candidate = self
            .read_parameter_tuning_candidate(request.candidate_id)
            .await?;
        ensure_promotable(&candidate.status)?;
        let competition_id = required_uuid(candidate.competition_id, "候选缺少赛事范围")?;
        let competition_profile_id =
            required_uuid(candidate.competition_profile_id, "候选缺少赛事 Profile")?;
        let baseline_model_version_id =
            required_uuid(candidate.baseline_model_version_id, "候选缺少基线模型版本")?;
        let baseline_parameter_set_id =
            required_uuid(candidate.baseline_parameter_set_id, "候选缺少基线参数版本")?;
        let candidate_model_version_id =
            required_uuid(candidate.candidate_model_version_id, "候选缺少候选模型版本")?;
        let candidate_parameter_set_id =
            required_uuid(candidate.candidate_parameter_set_id, "候选缺少候选参数版本")?;
        let mut tx = self.pool.begin().await?;
        let bindings = sqlx::query(
            r#"
            SELECT binding.id, binding.model_version_id, binding.parameter_set_id
            FROM model.competition_bindings binding
            JOIN model.rule_packages package ON package.id = binding.rule_package_id
            WHERE binding.is_active = true
              AND binding.competition_id = $1
              AND binding.model_version_id = $2
              AND binding.parameter_set_id = $3
              AND package.competition_profile_id = $4
            FOR UPDATE OF binding
            "#,
        )
        .bind(competition_id)
        .bind(baseline_model_version_id)
        .bind(baseline_parameter_set_id)
        .bind(competition_profile_id)
        .fetch_all(&mut *tx)
        .await?;
        if bindings.is_empty() {
            return Err(PersistenceError::InvalidState(
                "活动绑定已经变化，拒绝覆盖；请重新生成候选".to_string(),
            ));
        }
        let decision_id = Uuid::new_v4();
        let previous_state = Value::Array(
            bindings
                .iter()
                .map(|row| {
                    json!({
                        "binding_id": row.try_get::<Uuid, _>("id").ok(),
                        "model_version_id": row.try_get::<Uuid, _>("model_version_id").ok(),
                        "parameter_set_id": row.try_get::<Uuid, _>("parameter_set_id").ok(),
                    })
                })
                .collect(),
        );
        let new_state = Value::Array(
            bindings
                .iter()
                .map(|row| {
                    json!({
                        "binding_id": row.try_get::<Uuid, _>("id").ok(),
                        "model_version_id": candidate_model_version_id,
                        "parameter_set_id": candidate_parameter_set_id,
                    })
                })
                .collect(),
        );
        sqlx::query(
            r#"
            INSERT INTO analytics.parameter_promotion_decisions (
                id, candidate_id, decision, previous_binding_state, new_binding_state,
                decided_by, decision_note
            ) VALUES ($1,$2,'promote',$3,$4,$5,$6)
            "#,
        )
        .bind(decision_id)
        .bind(candidate.id)
        .bind(&previous_state)
        .bind(&new_state)
        .bind(request.decided_by.as_deref())
        .bind(&request.decision_note)
        .execute(&mut *tx)
        .await?;
        for row in &bindings {
            let binding_id: Uuid = row.try_get("id")?;
            sqlx::query(
                r#"
                INSERT INTO analytics.parameter_binding_changes (
                    decision_id, binding_id,
                    previous_model_version_id, previous_parameter_set_id,
                    new_model_version_id, new_parameter_set_id
                ) VALUES ($1,$2,$3,$4,$5,$6)
                "#,
            )
            .bind(decision_id)
            .bind(binding_id)
            .bind(baseline_model_version_id)
            .bind(baseline_parameter_set_id)
            .bind(candidate_model_version_id)
            .bind(candidate_parameter_set_id)
            .execute(&mut *tx)
            .await?;
            let updated = sqlx::query(
                r#"
                UPDATE model.competition_bindings
                SET model_version_id = $2, parameter_set_id = $3
                WHERE id = $1
                  AND model_version_id = $4
                  AND parameter_set_id = $5
                "#,
            )
            .bind(binding_id)
            .bind(candidate_model_version_id)
            .bind(candidate_parameter_set_id)
            .bind(baseline_model_version_id)
            .bind(baseline_parameter_set_id)
            .execute(&mut *tx)
            .await?;
            if updated.rows_affected() != 1 {
                return Err(PersistenceError::InvalidState(format!(
                    "绑定 {binding_id} 已被其他版本修改，拒绝覆盖式晋升"
                )));
            }
        }
        sqlx::query("UPDATE model.versions SET status='active' WHERE id=$1")
            .bind(candidate_model_version_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE model.parameter_sets SET status='active' WHERE id=$1")
            .bind(candidate_parameter_set_id)
            .execute(&mut *tx)
            .await?;
        let promoted = sqlx::query(
            "UPDATE analytics.parameter_tuning_candidates SET status='promoted', decided_at=now(), decision_note=$2 WHERE id=$1 AND status='shadow_passed'",
        )
        .bind(candidate.id)
        .bind(&request.decision_note)
        .execute(&mut *tx)
        .await?;
        if promoted.rows_affected() != 1 {
            return Err(PersistenceError::InvalidState(
                "候选状态已变化，拒绝提交晋升决策".to_string(),
            ));
        }
        write_audit_event(
            &mut tx,
            "parameter_candidate_promoted",
            "parameter_tuning_candidate",
            Some(candidate.id.to_string()),
            json!({
                "decision_id": decision_id,
                "binding_count": bindings.len(),
                "manual": true,
                "automatic_promotion": false,
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_parameter_promotion_decision(decision_id).await
    }

    pub async fn rollback_parameter_candidate(
        &self,
        request: &ParameterRollbackRequest,
    ) -> PersistenceResult<ParameterPromotionDecisionRecord> {
        let candidate = self
            .read_parameter_tuning_candidate(request.candidate_id)
            .await?;
        ensure_rollback_allowed(&candidate.status)?;
        let candidate_model_version_id =
            required_uuid(candidate.candidate_model_version_id, "候选缺少候选模型版本")?;
        let candidate_parameter_set_id =
            required_uuid(candidate.candidate_parameter_set_id, "候选缺少候选参数版本")?;
        let promotion = sqlx::query(
            r#"
            SELECT id
            FROM analytics.parameter_promotion_decisions
            WHERE candidate_id = $1 AND decision = 'promote'
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(candidate.id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("未找到该候选的晋升决策记录".to_string()))?;
        let promotion_id: Uuid = promotion.try_get("id")?;
        let mut tx = self.pool.begin().await?;
        let changes = sqlx::query(
            r#"
            SELECT binding_id, previous_model_version_id, previous_parameter_set_id,
                   new_model_version_id, new_parameter_set_id
            FROM analytics.parameter_binding_changes
            WHERE decision_id = $1
            ORDER BY binding_id
            "#,
        )
        .bind(promotion_id)
        .fetch_all(&mut *tx)
        .await?;
        if changes.is_empty() {
            return Err(PersistenceError::InvalidState(
                "晋升决策没有可回滚的绑定快照".to_string(),
            ));
        }
        let rollback_id = Uuid::new_v4();
        let previous_state = Value::Array(
            changes
                .iter()
                .map(|row| {
                    json!({
                        "binding_id": row.try_get::<Uuid, _>("binding_id").ok(),
                        "model_version_id": candidate_model_version_id,
                        "parameter_set_id": candidate_parameter_set_id,
                    })
                })
                .collect(),
        );
        let new_state = Value::Array(
            changes
                .iter()
                .map(|row| {
                    json!({
                        "binding_id": row.try_get::<Uuid, _>("binding_id").ok(),
                        "model_version_id": row.try_get::<Uuid, _>("previous_model_version_id").ok(),
                        "parameter_set_id": row.try_get::<Uuid, _>("previous_parameter_set_id").ok(),
                    })
                })
                .collect(),
        );
        sqlx::query(
            r#"
            INSERT INTO analytics.parameter_promotion_decisions (
                id, candidate_id, decision, previous_binding_state, new_binding_state,
                decided_by, decision_note
            ) VALUES ($1,$2,'rollback',$3,$4,$5,$6)
            "#,
        )
        .bind(rollback_id)
        .bind(candidate.id)
        .bind(&previous_state)
        .bind(&new_state)
        .bind(request.decided_by.as_deref())
        .bind(&request.decision_note)
        .execute(&mut *tx)
        .await?;
        for row in &changes {
            let binding_id: Uuid = row.try_get("binding_id")?;
            let previous_model_version_id: Uuid = row.try_get("previous_model_version_id")?;
            let previous_parameter_set_id: Uuid = row.try_get("previous_parameter_set_id")?;
            let updated = sqlx::query(
                r#"
                UPDATE model.competition_bindings
                SET model_version_id = $2, parameter_set_id = $3
                WHERE id = $1
                  AND model_version_id = $4
                  AND parameter_set_id = $5
                "#,
            )
            .bind(binding_id)
            .bind(previous_model_version_id)
            .bind(previous_parameter_set_id)
            .bind(candidate_model_version_id)
            .bind(candidate_parameter_set_id)
            .execute(&mut *tx)
            .await?;
            if updated.rows_affected() != 1 {
                return Err(PersistenceError::InvalidState(format!(
                    "绑定 {binding_id} 已被其他版本修改，拒绝覆盖式回滚"
                )));
            }
        }
        sqlx::query("UPDATE model.versions SET status='deprecated' WHERE id=$1")
            .bind(candidate_model_version_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE model.parameter_sets SET status='deprecated' WHERE id=$1")
            .bind(candidate_parameter_set_id)
            .execute(&mut *tx)
            .await?;
        let rolled_back = sqlx::query(
            "UPDATE analytics.parameter_tuning_candidates SET status='rolled_back', decided_at=now(), decision_note=$2 WHERE id=$1 AND status='promoted'",
        )
        .bind(candidate.id)
        .bind(&request.decision_note)
        .execute(&mut *tx)
        .await?;
        if rolled_back.rows_affected() != 1 {
            return Err(PersistenceError::InvalidState(
                "候选状态已变化，拒绝提交回滚决策".to_string(),
            ));
        }
        write_audit_event(
            &mut tx,
            "parameter_candidate_rolled_back",
            "parameter_tuning_candidate",
            Some(candidate.id.to_string()),
            json!({
                "promotion_decision_id": promotion_id,
                "rollback_decision_id": rollback_id,
                "binding_count": changes.len(),
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_parameter_promotion_decision(rollback_id).await
    }
}
