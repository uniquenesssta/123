use super::{
    parse_competition_kind, sha256_json, write_audit_event, PersistenceError, PersistenceResult,
    PostgresStore,
};
use chrono::{DateTime, Utc};
use football_domain::{
    P4FreezeReadiness, P4FreezeTaskDraft, P4FreezeTaskEventRecord, P4FreezeTaskRecord,
    P4FreezeTaskState, P4FreezeTaskTransition, P4Horizon, P4PlanningMatchContext, P4RoutedFact,
    ResearchRunRecord, ResearchRunStatus, SchemaVersionRecord,
};
use serde_json::{json, Value};
use sqlx::{Postgres, Row, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

impl PostgresStore {
    pub async fn p4_planning_match_context(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<P4PlanningMatchContext> {
        let row = sqlx::query(
            r#"
            SELECT fixture.id, fixture.external_key, fixture.kickoff_time,
                   fixture.competition_id, fixture.season_id, fixture.stage_id,
                   COALESCE(stage.stage_kind, competition.competition_kind, 'custom') AS effective_kind,
                   home.canonical_name AS home_team_name,
                   away.canonical_name AS away_team_name
            FROM football.matches fixture
            LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
            LEFT JOIN football.competition_stages stage ON stage.id = fixture.stage_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            WHERE fixture.id = $1
            "#,
        )
        .bind(match_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛不存在".to_string()))?;
        Ok(P4PlanningMatchContext {
            match_id: row.try_get("id")?,
            match_key: row.try_get("external_key")?,
            kickoff_at: row.try_get("kickoff_time")?,
            competition_id: row.try_get("competition_id")?,
            season_id: row.try_get("season_id")?,
            stage_id: row.try_get("stage_id")?,
            competition_kind: parse_competition_kind(
                row.try_get::<String, _>("effective_kind")?.as_str(),
            )?,
            home_team_name: row.try_get("home_team_name")?,
            away_team_name: row.try_get("away_team_name")?,
        })
    }

    pub async fn read_schema_version_by_key(
        &self,
        schema_key: &str,
        version: &str,
    ) -> PersistenceResult<SchemaVersionRecord> {
        let row = sqlx::query(
            r#"
            SELECT id, schema_key, version, schema_kind, content_sha256, created_at
            FROM research.schema_versions
            WHERE schema_key = $1 AND version = $2
            "#,
        )
        .bind(schema_key)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            PersistenceError::InvalidState(format!("未登记Schema版本：{schema_key}@{version}"))
        })?;
        Ok(SchemaVersionRecord {
            id: row.try_get("id")?,
            schema_key: row.try_get("schema_key")?,
            version: row.try_get("version")?,
            schema_kind: row.try_get("schema_kind")?,
            content_sha256: row.try_get("content_sha256")?,
            created_at: row.try_get("created_at")?,
        })
    }

    pub async fn read_research_run(
        &self,
        research_run_id: Uuid,
    ) -> PersistenceResult<ResearchRunRecord> {
        let row = sqlx::query(
            r#"
            SELECT id, match_id, horizon, data_cutoff_at, trace_id, idempotency_key,
                   request_fingerprint, status, created_at
            FROM research.runs
            WHERE id = $1
            "#,
        )
        .bind(research_run_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("研究任务不存在".to_string()))?;
        research_run_from_row(&row)
    }

    pub async fn find_p4_freeze_task_by_idempotency(
        &self,
        idempotency_key: &str,
    ) -> PersistenceResult<Option<P4FreezeTaskRecord>> {
        let row = sqlx::query("SELECT * FROM platform.p4_freeze_tasks WHERE idempotency_key = $1")
            .bind(idempotency_key)
            .fetch_optional(&self.pool)
            .await?;
        row.as_ref().map(task_from_row).transpose()
    }

    pub async fn create_p4_freeze_task(
        &self,
        draft: &P4FreezeTaskDraft,
    ) -> PersistenceResult<P4FreezeTaskRecord> {
        if !draft.horizon.is_canonical() {
            return Err(PersistenceError::InvalidState(
                "T-N兼容时点不得进入P4正式冻结队列".to_string(),
            ));
        }
        if draft.requested_fact_keys.is_empty() {
            return Err(PersistenceError::InvalidState(
                "P4冻结任务至少需要一个事实字段".to_string(),
            ));
        }
        let mut requested_fact_keys = draft.requested_fact_keys.clone();
        requested_fact_keys.sort();
        requested_fact_keys.dedup();
        let task_fingerprint = sha256_json(&json!({
            "match_id": draft.match_id,
            "match_key": draft.match_key,
            "horizon": draft.horizon.as_str(),
            "kickoff_at": draft.kickoff_at,
            "data_cutoff_at": draft.data_cutoff_at,
            "research_due_at": draft.research_due_at,
            "freeze_deadline_at": draft.freeze_deadline_at,
            "rule_package_id": draft.rule_package_id,
            "model_version_id": draft.model_version_id,
            "parameter_set_id": draft.parameter_set_id,
            "competition_profile_id": draft.competition_profile_id,
            "research_schema_version_id": draft.research_schema_version_id,
            "snapshot_schema_version_id": draft.snapshot_schema_version_id,
            "requested_fact_keys": &requested_fact_keys,
            "trace_id": draft.trace_id,
            "state": draft.state.as_str(),
            "metadata": &draft.metadata,
        }))?;
        let mut tx = self.pool.begin().await?;
        lock_key(
            &mut tx,
            &format!("p4-freeze-task:{}", draft.idempotency_key),
        )
        .await?;
        if let Some(row) = select_task_by_idempotency(&mut tx, &draft.idempotency_key).await? {
            let existing: String = row.try_get("task_fingerprint")?;
            if existing != task_fingerprint {
                return Err(PersistenceError::InvalidState(format!(
                    "P4冻结任务幂等键 {} 已存在但载荷不同",
                    draft.idempotency_key
                )));
            }
            let record = task_from_row(&row)?;
            tx.commit().await?;
            return Ok(record);
        }

        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO platform.p4_freeze_tasks (
                id, match_id, match_key, horizon, kickoff_at, data_cutoff_at,
                research_due_at, freeze_deadline_at, rule_package_id,
                model_version_id, parameter_set_id, competition_profile_id,
                research_schema_version_id, snapshot_schema_version_id,
                requested_fact_keys, trace_id, state, task_fingerprint,
                idempotency_key, metadata, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9,
                $10, $11, $12,
                $13, $14,
                $15, $16, $17, $18,
                $19, $20, now()
            )
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(draft.match_id)
        .bind(&draft.match_key)
        .bind(draft.horizon.as_str())
        .bind(draft.kickoff_at)
        .bind(draft.data_cutoff_at)
        .bind(draft.research_due_at)
        .bind(draft.freeze_deadline_at)
        .bind(draft.rule_package_id)
        .bind(draft.model_version_id)
        .bind(draft.parameter_set_id)
        .bind(draft.competition_profile_id)
        .bind(draft.research_schema_version_id)
        .bind(draft.snapshot_schema_version_id)
        .bind(&requested_fact_keys)
        .bind(draft.trace_id)
        .bind(draft.state.as_str())
        .bind(&task_fingerprint)
        .bind(&draft.idempotency_key)
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        append_task_event(
            &mut tx,
            id,
            None,
            draft.state,
            "三个计划窗口规划器创建任务",
            json!({
                "planner_version": football_domain::P4_ORCHESTRATION_PLANNER_VERSION,
                "horizon": draft.horizon.as_str(),
                "data_cutoff_at": draft.data_cutoff_at,
            }),
        )
        .await?;
        write_audit_event(
            &mut tx,
            "p4_freeze_task_created",
            "p4_freeze_task",
            Some(id.to_string()),
            json!({
                "match_id": draft.match_id,
                "horizon": draft.horizon.as_str(),
                "data_cutoff_at": draft.data_cutoff_at,
                "state": draft.state.as_str(),
                "trace_id": draft.trace_id,
                "task_fingerprint": task_fingerprint,
            }),
        )
        .await?;
        let record = task_from_row(&row)?;
        tx.commit().await?;
        Ok(record)
    }

    pub async fn transition_p4_freeze_task(
        &self,
        transition: &P4FreezeTaskTransition,
    ) -> PersistenceResult<P4FreezeTaskRecord> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query("SELECT * FROM platform.p4_freeze_tasks WHERE id = $1 FOR UPDATE")
            .bind(transition.task_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("P4冻结任务不存在".to_string()))?;
        let current = parse_state(row.try_get::<String, _>("state")?.as_str())?;
        if current == transition.next_state {
            let record = task_from_row(&row)?;
            tx.commit().await?;
            return Ok(record);
        }
        if current != transition.expected_state {
            return Err(PersistenceError::InvalidState(format!(
                "P4冻结任务状态并发冲突：预期 {}，实际 {}",
                transition.expected_state.as_str(),
                current.as_str()
            )));
        }
        if !current.can_transition_to(transition.next_state) {
            return Err(PersistenceError::InvalidState(format!(
                "不允许的P4冻结状态迁移：{} -> {}",
                current.as_str(),
                transition.next_state.as_str()
            )));
        }

        sqlx::query(
            r#"
            UPDATE platform.p4_freeze_tasks
            SET state = $2,
                blockers = CASE WHEN $3 = 'null'::jsonb THEN blockers ELSE $3 END,
                research_run_id = COALESCE($4, research_run_id),
                research_job_id = COALESCE($5, research_job_id),
                freeze_job_id = COALESCE($6, freeze_job_id),
                snapshot_id = COALESCE($7, snapshot_id),
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(transition.task_id)
        .bind(transition.next_state.as_str())
        .bind(&transition.blockers)
        .bind(transition.research_run_id)
        .bind(transition.research_job_id)
        .bind(transition.freeze_job_id)
        .bind(transition.snapshot_id)
        .execute(&mut *tx)
        .await?;
        append_task_event(
            &mut tx,
            transition.task_id,
            Some(current),
            transition.next_state,
            &transition.reason,
            transition.payload.clone(),
        )
        .await?;
        write_audit_event(
            &mut tx,
            "p4_freeze_task_transitioned",
            "p4_freeze_task",
            Some(transition.task_id.to_string()),
            json!({
                "from_state": current.as_str(),
                "to_state": transition.next_state.as_str(),
                "reason": transition.reason,
            }),
        )
        .await?;
        let row = sqlx::query("SELECT * FROM platform.p4_freeze_tasks WHERE id = $1")
            .bind(transition.task_id)
            .fetch_one(&mut *tx)
            .await?;
        let record = task_from_row(&row)?;
        tx.commit().await?;
        Ok(record)
    }

    pub async fn read_p4_freeze_task(
        &self,
        task_id: Uuid,
    ) -> PersistenceResult<P4FreezeTaskRecord> {
        let row = sqlx::query("SELECT * FROM platform.p4_freeze_tasks WHERE id = $1")
            .bind(task_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("P4冻结任务不存在".to_string()))?;
        task_from_row(&row)
    }

    pub async fn list_p4_freeze_tasks(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> PersistenceResult<Vec<P4FreezeTaskRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM platform.p4_freeze_tasks
            WHERE $1::uuid IS NULL OR match_id = $1
            ORDER BY kickoff_at DESC,
                     CASE horizon
                       WHEN 'T-24h' THEN 1
                       WHEN 'T-6h' THEN 2
                       WHEN 'T-90m' THEN 3
                       WHEN 'T-1h' THEN 4
                       ELSE 5
                     END,
                     created_at DESC
            LIMIT $2
            "#,
        )
        .bind(match_id)
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(task_from_row).collect()
    }

    pub async fn list_p4_freeze_task_events(
        &self,
        task_id: Uuid,
    ) -> PersistenceResult<Vec<P4FreezeTaskEventRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, task_id, from_state, to_state, reason, payload, occurred_at
            FROM platform.p4_freeze_task_events
            WHERE task_id = $1
            ORDER BY occurred_at, id
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(task_event_from_row).collect()
    }

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

async fn lock_key(tx: &mut Transaction<'_, Postgres>, key: &str) -> PersistenceResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::bigint)")
        .bind(key)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn select_task_by_idempotency(
    tx: &mut Transaction<'_, Postgres>,
    idempotency_key: &str,
) -> PersistenceResult<Option<sqlx::postgres::PgRow>> {
    Ok(
        sqlx::query("SELECT * FROM platform.p4_freeze_tasks WHERE idempotency_key = $1")
            .bind(idempotency_key)
            .fetch_optional(&mut **tx)
            .await?,
    )
}

async fn append_task_event(
    tx: &mut Transaction<'_, Postgres>,
    task_id: Uuid,
    from_state: Option<P4FreezeTaskState>,
    to_state: P4FreezeTaskState,
    reason: &str,
    payload: Value,
) -> PersistenceResult<()> {
    let idempotency_key = format!(
        "{}:{}:{}",
        from_state.map(P4FreezeTaskState::as_str).unwrap_or("NONE"),
        to_state.as_str(),
        sha256_json(&json!({"reason": reason, "payload": &payload}))?
    );
    let event_fingerprint = sha256_json(&json!({
        "task_id": task_id,
        "from_state": from_state.map(P4FreezeTaskState::as_str),
        "to_state": to_state.as_str(),
        "reason": reason,
        "payload": &payload,
    }))?;
    let inserted: Option<Uuid> = sqlx::query_scalar(
        r#"
        INSERT INTO platform.p4_freeze_task_events (
            id, task_id, from_state, to_state, reason, payload,
            idempotency_key, event_fingerprint
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (task_id, idempotency_key) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(task_id)
    .bind(from_state.map(P4FreezeTaskState::as_str))
    .bind(to_state.as_str())
    .bind(reason)
    .bind(&payload)
    .bind(&idempotency_key)
    .bind(&event_fingerprint)
    .fetch_optional(&mut **tx)
    .await?;
    if inserted.is_none() {
        let existing: String = sqlx::query_scalar(
            r#"
            SELECT event_fingerprint
            FROM platform.p4_freeze_task_events
            WHERE task_id = $1 AND idempotency_key = $2
            "#,
        )
        .bind(task_id)
        .bind(&idempotency_key)
        .fetch_one(&mut **tx)
        .await?;
        if existing != event_fingerprint {
            return Err(PersistenceError::InvalidState(
                "P4冻结任务事件幂等冲突".to_string(),
            ));
        }
    }
    Ok(())
}

fn task_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<P4FreezeTaskRecord> {
    Ok(P4FreezeTaskRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        match_key: row.try_get("match_key")?,
        horizon: parse_horizon(row.try_get::<String, _>("horizon")?.as_str())?,
        kickoff_at: row.try_get("kickoff_at")?,
        data_cutoff_at: row.try_get("data_cutoff_at")?,
        research_due_at: row.try_get("research_due_at")?,
        freeze_deadline_at: row.try_get("freeze_deadline_at")?,
        rule_package_id: row.try_get("rule_package_id")?,
        model_version_id: row.try_get("model_version_id")?,
        parameter_set_id: row.try_get("parameter_set_id")?,
        competition_profile_id: row.try_get("competition_profile_id")?,
        research_schema_version_id: row.try_get("research_schema_version_id")?,
        snapshot_schema_version_id: row.try_get("snapshot_schema_version_id")?,
        requested_fact_keys: row.try_get("requested_fact_keys")?,
        trace_id: row.try_get("trace_id")?,
        state: parse_state(row.try_get::<String, _>("state")?.as_str())?,
        research_run_id: row.try_get("research_run_id")?,
        research_job_id: row.try_get("research_job_id")?,
        freeze_job_id: row.try_get("freeze_job_id")?,
        snapshot_id: row.try_get("snapshot_id")?,
        blockers: row.try_get("blockers")?,
        task_fingerprint: row.try_get("task_fingerprint")?,
        idempotency_key: row.try_get("idempotency_key")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn task_event_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<P4FreezeTaskEventRecord> {
    let from_state = row
        .try_get::<Option<String>, _>("from_state")?
        .map(|value| parse_state(&value))
        .transpose()?;
    Ok(P4FreezeTaskEventRecord {
        id: row.try_get("id")?,
        task_id: row.try_get("task_id")?,
        from_state,
        to_state: parse_state(row.try_get::<String, _>("to_state")?.as_str())?,
        reason: row.try_get("reason")?,
        payload: row.try_get("payload")?,
        occurred_at: row.try_get("occurred_at")?,
    })
}

fn research_run_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<ResearchRunRecord> {
    Ok(ResearchRunRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        horizon: parse_horizon(row.try_get::<String, _>("horizon")?.as_str())?,
        data_cutoff_at: row.try_get("data_cutoff_at")?,
        trace_id: row.try_get("trace_id")?,
        idempotency_key: row.try_get("idempotency_key")?,
        request_fingerprint: row.try_get("request_fingerprint")?,
        status: parse_research_status(row.try_get::<String, _>("status")?.as_str())?,
        created_at: row.try_get("created_at")?,
    })
}

fn parse_horizon(value: &str) -> PersistenceResult<P4Horizon> {
    match value {
        "T-24h" => Ok(P4Horizon::T24h),
        "T-6h" => Ok(P4Horizon::T6h),
        "T-90m" => Ok(P4Horizon::T90m),
        "T-1h" => Ok(P4Horizon::T1h),
        "T-N" => Ok(P4Horizon::LegacyTN),
        other => Err(PersistenceError::InvalidState(format!(
            "未知P4时点：{other}"
        ))),
    }
}

fn parse_state(value: &str) -> PersistenceResult<P4FreezeTaskState> {
    P4FreezeTaskState::parse(value)
        .ok_or_else(|| PersistenceError::InvalidState(format!("未知P4冻结任务状态：{value}")))
}

fn parse_research_status(value: &str) -> PersistenceResult<ResearchRunStatus> {
    match value {
        "planned" => Ok(ResearchRunStatus::Planned),
        "running" => Ok(ResearchRunStatus::Running),
        "succeeded" => Ok(ResearchRunStatus::Succeeded),
        "partial" => Ok(ResearchRunStatus::Partial),
        "failed" => Ok(ResearchRunStatus::Failed),
        "cancelled" => Ok(ResearchRunStatus::Cancelled),
        other => Err(PersistenceError::InvalidState(format!(
            "未知研究任务状态：{other}"
        ))),
    }
}
