use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::{P4FreezeReadiness, P4FreezeTaskRecord, P4RoutedFact};
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

impl PostgresStore {
    pub async fn p4_freeze_readiness(&self, task_id: Uuid) -> PersistenceResult<P4FreezeReadiness> {
        self.p4_readiness(task_id, true).await
    }

    pub async fn p4_route_readiness(&self, task_id: Uuid) -> PersistenceResult<P4FreezeReadiness> {
        self.p4_readiness(task_id, false).await
    }

    async fn p4_readiness(
        &self,
        task_id: Uuid,
        require_succeeded_research: bool,
    ) -> PersistenceResult<P4FreezeReadiness> {
        let task = self.read_p4_freeze_task(task_id).await?;
        let Some(research_run_id) = task.research_run_id else {
            return Ok(P4FreezeReadiness {
                task_id,
                ready: false,
                research_status: None,
                requested_fact_count: task.requested_fact_keys.len() as u32,
                routed_fact_count: 0,
                missing_fact_count: 0,
                ignored_fact_count: 0,
                blocked_fact_count: 0,
                blockers: vec!["研究任务尚未创建".to_string()],
            });
        };
        let research_status: String =
            sqlx::query_scalar("SELECT status FROM research.runs WHERE id = $1")
                .bind(research_run_id)
                .fetch_one(&self.pool)
                .await?;
        let routes = self.p4_routed_facts(task_id).await?;
        let mut by_field = HashMap::<String, Vec<&P4RoutedFact>>::new();
        for route in &routes {
            by_field
                .entry(route.field_key.clone())
                .or_default()
                .push(route);
        }
        let mut blockers = Vec::new();
        let mut routed_fact_count = 0_u32;
        let mut missing_fact_count = 0_u32;
        let mut ignored_fact_count = 0_u32;
        let mut blocked_fact_count = 0_u32;
        for route in &routes {
            match route.route_status.as_str() {
                "routed" => routed_fact_count += 1,
                "missing" => missing_fact_count += 1,
                "ignored_non_model_fact" => ignored_fact_count += 1,
                status if status.starts_with("blocked_") => {
                    blocked_fact_count += 1;
                    blockers.push(format!("{}: {}", route.field_key, route.reason));
                }
                _ => {
                    blocked_fact_count += 1;
                    blockers.push(format!(
                        "{}: 未识别路由状态 {}",
                        route.field_key, route.route_status
                    ));
                }
            }
            if matches!(route.verification_state.as_str(), "CONFLICT" | "STALE") {
                blockers.push(format!(
                    "{}: 验证状态 {} 不允许进入READY_TO_FREEZE",
                    route.field_key, route.verification_state
                ));
            }
        }
        for field_key in &task.requested_fact_keys {
            if !by_field.contains_key(field_key) {
                blockers.push(format!("{field_key}: 缺少不可变证据路由记录"));
            }
        }
        if require_succeeded_research && research_status != ResearchRunStatus::Succeeded.as_str() {
            blockers.push(format!("研究任务状态不是succeeded：{research_status}"));
        }
        let ready = blockers.is_empty();
        Ok(P4FreezeReadiness {
            task_id,
            ready,
            research_status: Some(research_status),
            requested_fact_count: task.requested_fact_keys.len() as u32,
            routed_fact_count,
            missing_fact_count,
            ignored_fact_count,
            blocked_fact_count,
            blockers,
        })
    }

    pub async fn find_frozen_p4_snapshot_id(
        &self,
        task: &P4FreezeTaskRecord,
    ) -> PersistenceResult<Option<Uuid>> {
        let idempotency_key = format!("p4-prematch-snapshot:{}", task.id);
        let row = sqlx::query(
            r#"
            SELECT id, match_id, match_key, snapshot_type, data_cutoff_time, frozen_at,
                   model_version_id, parameter_set_id, competition_profile_id,
                   research_run_id, schema_version_id, trace_id, source_kind
            FROM feature.snapshots
            WHERE idempotency_key = $1
            "#,
        )
        .bind(&idempotency_key)
        .fetch_optional(&self.pool)
        .await?;
        let Some(row) = row else {
            return Ok(None);
        };
        let frozen_at: DateTime<Utc> = row.try_get("frozen_at")?;
        let identity_matches = row.try_get::<Option<Uuid>, _>("match_id")? == Some(task.match_id)
            && row.try_get::<String, _>("match_key")?.as_str() == task.match_key.as_str()
            && row.try_get::<String, _>("snapshot_type")? == task.horizon.as_str()
            && row.try_get::<DateTime<Utc>, _>("data_cutoff_time")? == task.data_cutoff_at
            && row.try_get::<Option<Uuid>, _>("model_version_id")? == Some(task.model_version_id)
            && row.try_get::<Option<Uuid>, _>("parameter_set_id")? == Some(task.parameter_set_id)
            && row.try_get::<Option<Uuid>, _>("competition_profile_id")?
                == Some(task.competition_profile_id)
            && row.try_get::<Option<Uuid>, _>("research_run_id")? == task.research_run_id
            && row.try_get::<Option<Uuid>, _>("schema_version_id")?
                == Some(task.snapshot_schema_version_id)
            && row.try_get::<Option<Uuid>, _>("trace_id")? == Some(task.trace_id)
            && row.try_get::<String, _>("source_kind")? == "real";
        if !identity_matches {
            return Err(PersistenceError::InvalidState(
                "已存在的P4快照与冻结任务锁定身份不一致".to_string(),
            ));
        }
        if frozen_at < task.data_cutoff_at || frozen_at > task.freeze_deadline_at {
            return Err(PersistenceError::InvalidState(
                "已存在的P4快照不在任务冻结时间窗口内".to_string(),
            ));
        }
        Ok(Some(row.try_get("id")?))
    }

    pub async fn p4_routed_facts(&self, task_id: Uuid) -> PersistenceResult<Vec<P4RoutedFact>> {
        let research_run_id: Option<Uuid> = sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT research_run_id FROM platform.p4_freeze_tasks WHERE id = $1",
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?
        .flatten();
        let Some(research_run_id) = research_run_id else {
            return Ok(Vec::new());
        };
        let rows = sqlx::query(
            r#"
            SELECT route.route_key, route.field_key, route.target_module, route.target_slot,
                   COALESCE(manual.route_status, route.route_status) AS route_status,
                   COALESCE(manual.verification_state, route.verification_state) AS verification_state,
                   COALESCE(manual.selected_evidence_ids, route.selected_evidence_ids) AS selected_evidence_ids,
                   COALESCE(manual.selected_value, route.selected_value) AS selected_value,
                   COALESCE(manual.reason, route.reason) AS reason
            FROM research.evidence_routes route
            LEFT JOIN LATERAL (
                SELECT manual_override.route_status, manual_override.verification_state,
                       manual_override.selected_evidence_ids, manual_override.selected_value,
                       manual_override.reason
                FROM research.manual_route_overrides manual_override
                WHERE manual_override.task_id = $2
                  AND manual_override.route_key = route.route_key
                ORDER BY manual_override.created_at DESC, manual_override.id DESC
                LIMIT 1
            ) manual ON true
            WHERE route.research_run_id = $1
            ORDER BY route.field_key, route.route_key
            "#,
        )
        .bind(research_run_id)
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(|row| {
                Ok(P4RoutedFact {
                    route_key: row.try_get("route_key")?,
                    field_key: row.try_get("field_key")?,
                    target_module: row.try_get("target_module")?,
                    target_slot: row.try_get("target_slot")?,
                    route_status: row.try_get("route_status")?,
                    verification_state: row.try_get("verification_state")?,
                    selected_evidence_ids: row.try_get("selected_evidence_ids")?,
                    selected_value: row.try_get("selected_value")?,
                    reason: row.try_get("reason")?,
                })
            })
            .collect()
    }
}
