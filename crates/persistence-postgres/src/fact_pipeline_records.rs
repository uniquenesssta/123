use super::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    ConflictEvaluationDraft, ConflictEvaluationRecord, ConflictEvaluationStatus, EntityCandidate,
    EntityResolutionDraft, EntityResolutionRecord, EntityResolutionStatus, EvidenceRouteDraft,
    EvidenceRouteRecord, EvidenceRouteStatus, FactPipelineContext, SourcePolicyVersionDraft,
    SourcePolicyVersionRecord, TimeAuditDraft, TimeAuditRecord, TimeAuditStatus,
};
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::BTreeMap;
use uuid::Uuid;

impl PostgresStore {
    pub async fn register_source_policy_version(
        &self,
        draft: &SourcePolicyVersionDraft,
    ) -> PersistenceResult<SourcePolicyVersionRecord> {
        if draft.policy_key.trim().is_empty() || draft.version.trim().is_empty() {
            return Err(PersistenceError::InvalidState(
                "来源策略键和版本不能为空".to_string(),
            ));
        }
        validate_source_policy_definition(draft)?;
        let content_sha256 = sha256_json(&draft.definition)?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(format!(
                "source-policy:{}@{}",
                draft.policy_key, draft.version
            ))
            .execute(&mut *tx)
            .await?;
        if let Some(row) = sqlx::query(
            r#"
            SELECT id, policy_key, version, content_sha256, created_at
            FROM research.source_policy_versions
            WHERE policy_key = $1 AND version = $2
            "#,
        )
        .bind(&draft.policy_key)
        .bind(&draft.version)
        .fetch_optional(&mut *tx)
        .await?
        {
            let existing: String = row.try_get("content_sha256")?;
            if existing != content_sha256 {
                return Err(PersistenceError::InvalidState(format!(
                    "来源策略{}@{}已经存在但内容指纹不同；必须发布新版本",
                    draft.policy_key, draft.version
                )));
            }
            let record = SourcePolicyVersionRecord {
                id: row.try_get("id")?,
                policy_key: row.try_get("policy_key")?,
                version: row.try_get("version")?,
                content_sha256: existing,
                created_at: row.try_get("created_at")?,
            };
            tx.commit().await?;
            return Ok(record);
        }

        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO research.source_policy_versions (
                id, policy_key, version, competition_profile_id,
                definition, content_sha256, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, policy_key, version, content_sha256, created_at
            "#,
        )
        .bind(id)
        .bind(&draft.policy_key)
        .bind(&draft.version)
        .bind(draft.competition_profile_id)
        .bind(serde_json::to_value(&draft.definition)?)
        .bind(&content_sha256)
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "source_policy_registered",
            "source_policy",
            Some(id.to_string()),
            json!({
                "policy_key": draft.policy_key,
                "version": draft.version,
                "content_sha256": content_sha256,
            }),
        )
        .await?;
        let record = SourcePolicyVersionRecord {
            id: row.try_get("id")?,
            policy_key: row.try_get("policy_key")?,
            version: row.try_get("version")?,
            content_sha256: row.try_get("content_sha256")?,
            created_at: row.try_get("created_at")?,
        };
        tx.commit().await?;
        Ok(record)
    }

    pub async fn fact_pipeline_context(
        &self,
        research_run_id: Uuid,
    ) -> PersistenceResult<FactPipelineContext> {
        let row = sqlx::query(
            r#"
            SELECT
                run.id AS research_run_id,
                run.match_id,
                match.external_key AS match_key,
                run.horizon,
                run.data_cutoff_at,
                run.trace_id,
                run.prompt_version_id,
                prompt.version AS prompt_version,
                run.schema_version_id,
                schema.version AS schema_version,
                match.home_team_id,
                home.canonical_name AS home_team_name,
                match.away_team_id,
                away.canonical_name AS away_team_name,
                match.competition_id,
                competition.name AS competition_name,
                competition.code AS competition_code
            FROM research.runs run
            JOIN football.matches match ON match.id = run.match_id
            JOIN research.schema_versions schema ON schema.id = run.schema_version_id
            LEFT JOIN research.prompt_versions prompt ON prompt.id = run.prompt_version_id
            LEFT JOIN football.teams home ON home.id = match.home_team_id
            LEFT JOIN football.teams away ON away.id = match.away_team_id
            LEFT JOIN football.competitions competition ON competition.id = match.competition_id
            WHERE run.id = $1
            "#,
        )
        .bind(research_run_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("研究任务不存在".to_string()))?;
        Ok(FactPipelineContext {
            research_run_id: row.try_get("research_run_id")?,
            match_id: row.try_get("match_id")?,
            match_key: row.try_get("match_key")?,
            horizon: row.try_get("horizon")?,
            data_cutoff_at: row.try_get("data_cutoff_at")?,
            trace_id: row.try_get("trace_id")?,
            prompt_version_id: row.try_get("prompt_version_id")?,
            prompt_version: row.try_get("prompt_version")?,
            schema_version_id: row.try_get("schema_version_id")?,
            schema_version: row.try_get("schema_version")?,
            home_team_id: row.try_get("home_team_id")?,
            home_team_name: row.try_get("home_team_name")?,
            away_team_id: row.try_get("away_team_id")?,
            away_team_name: row.try_get("away_team_name")?,
            competition_id: row.try_get("competition_id")?,
            competition_name: row.try_get("competition_name")?,
            competition_code: row.try_get("competition_code")?,
        })
    }

    pub async fn find_entity_candidates(
        &self,
        context: &FactPipelineContext,
        entity_type: &str,
        normalized_name: &str,
        compact_name: &str,
        external_id: Option<&str>,
    ) -> PersistenceResult<Vec<EntityCandidate>> {
        if entity_type == "match" {
            return Ok(vec![EntityCandidate {
                entity_id: context.match_id,
                canonical_name: context.match_key.clone(),
                matched_name: context.match_key.clone(),
                strategy: "research_run_match_scope".to_string(),
                score: 100,
                relation: Some("current_match".to_string()),
            }]);
        }
        if entity_type == "competition" {
            let Some(competition_id) = context.competition_id else {
                return Ok(Vec::new());
            };
            let name = context.competition_name.clone().unwrap_or_default();
            let code = context.competition_code.clone().unwrap_or_default();
            let matches = [name.as_str(), code.as_str()]
                .iter()
                .any(|value| normalize_for_lookup(value) == normalized_name);
            return Ok(if matches {
                vec![EntityCandidate {
                    entity_id: competition_id,
                    canonical_name: name.clone(),
                    matched_name: if normalize_for_lookup(&name) == normalized_name {
                        name
                    } else {
                        code
                    },
                    strategy: "match_competition_exact".to_string(),
                    score: 98,
                    relation: Some("competition".to_string()),
                }]
            } else {
                Vec::new()
            });
        }
        if !matches!(entity_type, "team" | "player") {
            return Ok(Vec::new());
        }

        let mut by_id: BTreeMap<Uuid, EntityCandidate> = BTreeMap::new();
        if let Some(external_id) = external_id.map(str::trim).filter(|value| !value.is_empty()) {
            let rows = if entity_type == "team" {
                sqlx::query(
                    r#"
                    SELECT external.entity_id, team.canonical_name,
                           CASE
                               WHEN external.entity_id = $2 THEN 'home'
                               WHEN external.entity_id = $3 THEN 'away'
                           END AS relation
                    FROM football.external_entity_ids external
                    JOIN football.teams team ON team.id = external.entity_id
                    WHERE external.entity_type = 'team'
                      AND external.external_id = $1
                      AND external.entity_id IN ($2, $3)
                    "#,
                )
                .bind(external_id)
                .bind(context.home_team_id)
                .bind(context.away_team_id)
                .fetch_all(&self.pool)
                .await?
            } else {
                sqlx::query(
                    r#"
                    WITH scoped AS (
                        SELECT period.player_id,
                               CASE
                                   WHEN period.team_id = $2 THEN 'home'
                                   WHEN period.team_id = $3 THEN 'away'
                               END AS relation
                        FROM football.player_team_periods period
                        WHERE period.team_id IN ($2, $3)
                          AND period.valid_from <= $4::date
                          AND (period.valid_to IS NULL OR period.valid_to >= $4::date)
                          AND period.registration_status IN ('registered', 'loan', 'trial')
                        UNION
                        SELECT lineup_player.player_id,
                               CASE
                                   WHEN lineup.team_id = $2 THEN 'home'
                                   WHEN lineup.team_id = $3 THEN 'away'
                               END AS relation
                        FROM football.lineups lineup
                        JOIN football.lineup_players lineup_player ON lineup_player.lineup_id = lineup.id
                        WHERE lineup.match_id = $5
                    )
                    SELECT DISTINCT external.entity_id, player.canonical_name, scoped.relation
                    FROM football.external_entity_ids external
                    JOIN football.players player ON player.id = external.entity_id
                    JOIN scoped ON scoped.player_id = external.entity_id
                    WHERE external.entity_type = 'player'
                      AND external.external_id = $1
                    "#,
                )
                .bind(external_id)
                .bind(context.home_team_id)
                .bind(context.away_team_id)
                .bind(context.data_cutoff_at.date_naive())
                .bind(context.match_id)
                .fetch_all(&self.pool)
                .await?
            };
            for row in rows {
                let entity_id: Uuid = row.try_get("entity_id")?;
                let canonical_name: String = row.try_get("canonical_name")?;
                let relation: Option<String> = row.try_get("relation")?;
                keep_best_candidate(
                    &mut by_id,
                    EntityCandidate {
                        entity_id,
                        canonical_name,
                        matched_name: external_id.to_string(),
                        strategy: "external_id_match_scoped".to_string(),
                        score: 100,
                        relation,
                    },
                );
            }
        }

        match entity_type {
            "team" => {
                let rows = sqlx::query(
                    r#"
                    WITH scoped(team_id, relation) AS (
                        VALUES ($1::uuid, 'home'::text), ($2::uuid, 'away'::text)
                    ), names AS (
                        SELECT team.id AS entity_id, team.canonical_name,
                               team.canonical_name AS matched_name,
                               team.normalized_name,
                               relation, 'canonical'::text AS kind
                        FROM scoped
                        JOIN football.teams team ON team.id = scoped.team_id
                        WHERE scoped.team_id IS NOT NULL
                        UNION ALL
                        SELECT team.id, team.canonical_name, alias.name,
                               alias.normalized_name, relation, 'alias'::text
                        FROM scoped
                        JOIN football.teams team ON team.id = scoped.team_id
                        JOIN football.team_names alias ON alias.team_id = team.id
                        WHERE scoped.team_id IS NOT NULL
                          AND (alias.valid_from IS NULL OR alias.valid_from <= $3::date)
                          AND (alias.valid_to IS NULL OR alias.valid_to >= $3::date)
                    )
                    SELECT entity_id, canonical_name, matched_name, relation, kind
                    FROM names
                    WHERE normalized_name = $4
                       OR regexp_replace(normalized_name, '[^[:alnum:]]', '', 'g') = $5
                    "#,
                )
                .bind(context.home_team_id)
                .bind(context.away_team_id)
                .bind(context.data_cutoff_at.date_naive())
                .bind(normalized_name)
                .bind(compact_name)
                .fetch_all(&self.pool)
                .await?;
                for row in rows {
                    let entity_id: Uuid = row.try_get("entity_id")?;
                    let kind: String = row.try_get("kind")?;
                    let candidate = EntityCandidate {
                        entity_id,
                        canonical_name: row.try_get("canonical_name")?,
                        matched_name: row.try_get("matched_name")?,
                        strategy: format!("match_team_{kind}_exact"),
                        score: if kind == "canonical" { 98 } else { 95 },
                        relation: row.try_get("relation")?,
                    };
                    keep_best_candidate(&mut by_id, candidate);
                }
            }
            "player" => {
                let rows = sqlx::query(
                    r#"
                    WITH scoped AS (
                        SELECT period.player_id,
                               CASE
                                   WHEN period.team_id = $1 THEN 'home'
                                   WHEN period.team_id = $2 THEN 'away'
                                   ELSE 'other'
                               END AS relation
                        FROM football.player_team_periods period
                        WHERE period.team_id IN ($1, $2)
                          AND period.valid_from <= $3::date
                          AND (period.valid_to IS NULL OR period.valid_to >= $3::date)
                          AND period.registration_status IN ('registered', 'loan', 'trial')
                        UNION
                        SELECT lineup_player.player_id,
                               CASE
                                   WHEN lineup.team_id = $1 THEN 'home'
                                   WHEN lineup.team_id = $2 THEN 'away'
                                   ELSE 'other'
                               END AS relation
                        FROM football.lineups lineup
                        JOIN football.lineup_players lineup_player ON lineup_player.lineup_id = lineup.id
                        WHERE lineup.match_id = $4
                    ), names AS (
                        SELECT player.id AS entity_id, player.canonical_name,
                               player.canonical_name AS matched_name,
                               player.normalized_name,
                               scoped.relation, 'canonical'::text AS kind
                        FROM scoped
                        JOIN football.players player ON player.id = scoped.player_id
                        UNION ALL
                        SELECT player.id, player.canonical_name, alias.name,
                               alias.normalized_name, scoped.relation, 'alias'::text
                        FROM scoped
                        JOIN football.players player ON player.id = scoped.player_id
                        JOIN football.player_names alias ON alias.player_id = player.id
                        WHERE (alias.valid_from IS NULL OR alias.valid_from <= $3::date)
                          AND (alias.valid_to IS NULL OR alias.valid_to >= $3::date)
                    )
                    SELECT entity_id, canonical_name, matched_name, relation, kind
                    FROM names
                    WHERE normalized_name = $5
                       OR regexp_replace(normalized_name, '[^[:alnum:]]', '', 'g') = $6
                    "#,
                )
                .bind(context.home_team_id)
                .bind(context.away_team_id)
                .bind(context.data_cutoff_at.date_naive())
                .bind(context.match_id)
                .bind(normalized_name)
                .bind(compact_name)
                .fetch_all(&self.pool)
                .await?;
                for row in rows {
                    let entity_id: Uuid = row.try_get("entity_id")?;
                    let kind: String = row.try_get("kind")?;
                    let candidate = EntityCandidate {
                        entity_id,
                        canonical_name: row.try_get("canonical_name")?,
                        matched_name: row.try_get("matched_name")?,
                        strategy: format!("match_player_{kind}_exact"),
                        score: if kind == "canonical" { 98 } else { 95 },
                        relation: row.try_get("relation")?,
                    };
                    keep_best_candidate(&mut by_id, candidate);
                }

                if by_id.is_empty() {
                    let rows = sqlx::query(
                        r#"
                        WITH names AS (
                            SELECT player.id AS entity_id, player.canonical_name,
                                   player.canonical_name AS matched_name,
                                   player.normalized_name, 'canonical'::text AS kind
                            FROM football.players player
                            WHERE player.status <> 'retired'
                            UNION ALL
                            SELECT player.id, player.canonical_name, alias.name,
                                   alias.normalized_name, 'alias'::text
                            FROM football.players player
                            JOIN football.player_names alias ON alias.player_id = player.id
                            WHERE player.status <> 'retired'
                              AND (alias.valid_from IS NULL OR alias.valid_from <= $1::date)
                              AND (alias.valid_to IS NULL OR alias.valid_to >= $1::date)
                        )
                        SELECT entity_id, canonical_name, matched_name, kind
                        FROM names
                        WHERE normalized_name = $2
                           OR regexp_replace(normalized_name, '[^[:alnum:]]', '', 'g') = $3
                        LIMIT 20
                        "#,
                    )
                    .bind(context.data_cutoff_at.date_naive())
                    .bind(normalized_name)
                    .bind(compact_name)
                    .fetch_all(&self.pool)
                    .await?;
                    for row in rows {
                        let entity_id: Uuid = row.try_get("entity_id")?;
                        let kind: String = row.try_get("kind")?;
                        let candidate = EntityCandidate {
                            entity_id,
                            canonical_name: row.try_get("canonical_name")?,
                            matched_name: row.try_get("matched_name")?,
                            strategy: format!("global_player_{kind}_exact"),
                            score: if kind == "canonical" { 78 } else { 72 },
                            relation: None,
                        };
                        keep_best_candidate(&mut by_id, candidate);
                    }
                }
            }
            _ => unreachable!(),
        }
        let mut candidates: Vec<_> = by_id.into_values().collect();
        candidates.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.canonical_name.cmp(&right.canonical_name))
                .then_with(|| left.entity_id.cmp(&right.entity_id))
        });
        Ok(candidates)
    }

    pub async fn append_entity_resolution(
        &self,
        draft: &EntityResolutionDraft,
    ) -> PersistenceResult<EntityResolutionRecord> {
        let fingerprint = sha256_json(&json!({
            "research_run_id": draft.research_run_id,
            "match_id": draft.match_id,
            "trace_id": draft.trace_id,
            "fact_key": draft.fact_key,
            "entity_type": draft.entity_type,
            "raw_name": draft.raw_name,
            "normalized_name": draft.normalized_name,
            "external_id": draft.external_id,
            "status": draft.status.as_str(),
            "resolved_entity_id": draft.resolved_entity_id,
            "resolved_name": draft.resolved_name,
            "strategy": draft.strategy,
            "confidence_score": draft.confidence_score,
            "candidates": draft.candidates,
            "reason": draft.reason,
        }))?;
        let row = sqlx::query(
            r#"
            INSERT INTO research.entity_resolutions (
                id, research_run_id, match_id, trace_id, fact_key,
                entity_type, raw_name, normalized_name, external_id,
                resolution_status, resolved_entity_id, resolved_name,
                strategy, confidence_score, candidates, reason,
                idempotency_key, resolution_fingerprint
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12,
                $13, $14, $15, $16,
                $17, $18
            )
            ON CONFLICT (idempotency_key) DO NOTHING
            RETURNING id, resolution_status, resolved_entity_id,
                      resolution_fingerprint, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.research_run_id)
        .bind(draft.match_id)
        .bind(draft.trace_id)
        .bind(&draft.fact_key)
        .bind(&draft.entity_type)
        .bind(&draft.raw_name)
        .bind(&draft.normalized_name)
        .bind(&draft.external_id)
        .bind(draft.status.as_str())
        .bind(draft.resolved_entity_id)
        .bind(&draft.resolved_name)
        .bind(&draft.strategy)
        .bind(i32::from(draft.confidence_score))
        .bind(serde_json::to_value(&draft.candidates)?)
        .bind(&draft.reason)
        .bind(&draft.idempotency_key)
        .bind(&fingerprint)
        .fetch_optional(&self.pool)
        .await?;
        let row = match row {
            Some(row) => row,
            None => {
                sqlx::query(
                    r#"SELECT id, resolution_status, resolved_entity_id,
                          resolution_fingerprint, created_at
                   FROM research.entity_resolutions
                   WHERE idempotency_key = $1"#,
                )
                .bind(&draft.idempotency_key)
                .fetch_one(&self.pool)
                .await?
            }
        };
        let existing: String = row.try_get("resolution_fingerprint")?;
        ensure_fingerprint("实体解析", &draft.idempotency_key, &existing, &fingerprint)?;
        Ok(EntityResolutionRecord {
            id: row.try_get("id")?,
            status: parse_entity_resolution_status(
                row.try_get::<String, _>("resolution_status")?.as_str(),
            )?,
            resolved_entity_id: row.try_get("resolved_entity_id")?,
            resolution_fingerprint: existing,
            created_at: row.try_get("created_at")?,
        })
    }

    pub async fn append_time_audit(
        &self,
        draft: &TimeAuditDraft,
    ) -> PersistenceResult<TimeAuditRecord> {
        let fingerprint = sha256_json(&json!({
            "research_run_id": draft.research_run_id,
            "match_id": draft.match_id,
            "trace_id": draft.trace_id,
            "fact_key": draft.fact_key,
            "field_key": draft.field_key,
            "data_cutoff_at": draft.data_cutoff_at,
            "published_at": draft.published_at,
            "observed_at": draft.observed_at,
            "effective_at": draft.effective_at,
            "retrieved_at": draft.retrieved_at,
            "timezone": draft.timezone,
            "status": draft.status.as_str(),
            "reason": draft.reason,
        }))?;
        let row = sqlx::query(
            r#"
            INSERT INTO research.time_audits (
                id, research_run_id, match_id, trace_id, fact_key, field_key,
                data_cutoff_at, published_at, observed_at, effective_at,
                retrieved_at, timezone, audit_status, accepted, reason,
                idempotency_key, time_fingerprint
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12, $13, $14, $15,
                $16, $17
            )
            ON CONFLICT (idempotency_key) DO NOTHING
            RETURNING id, audit_status, time_fingerprint, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.research_run_id)
        .bind(draft.match_id)
        .bind(draft.trace_id)
        .bind(&draft.fact_key)
        .bind(&draft.field_key)
        .bind(draft.data_cutoff_at)
        .bind(draft.published_at)
        .bind(draft.observed_at)
        .bind(draft.effective_at)
        .bind(draft.retrieved_at)
        .bind(&draft.timezone)
        .bind(draft.status.as_str())
        .bind(draft.status.accepted())
        .bind(&draft.reason)
        .bind(&draft.idempotency_key)
        .bind(&fingerprint)
        .fetch_optional(&self.pool)
        .await?;
        let row = match row {
            Some(row) => row,
            None => {
                sqlx::query(
                    r#"SELECT id, audit_status, time_fingerprint, created_at
                   FROM research.time_audits
                   WHERE idempotency_key = $1"#,
                )
                .bind(&draft.idempotency_key)
                .fetch_one(&self.pool)
                .await?
            }
        };
        let existing: String = row.try_get("time_fingerprint")?;
        ensure_fingerprint("时间审计", &draft.idempotency_key, &existing, &fingerprint)?;
        Ok(TimeAuditRecord {
            id: row.try_get("id")?,
            status: parse_time_audit_status(row.try_get::<String, _>("audit_status")?.as_str())?,
            time_fingerprint: existing,
            created_at: row.try_get("created_at")?,
        })
    }

    pub async fn append_conflict_evaluation(
        &self,
        draft: &ConflictEvaluationDraft,
    ) -> PersistenceResult<ConflictEvaluationRecord> {
        let fingerprint = sha256_json(&json!({
            "conflict_id": draft.conflict_id,
            "research_run_id": draft.research_run_id,
            "match_id": draft.match_id,
            "trace_id": draft.trace_id,
            "source_policy_key": draft.source_policy_key,
            "source_policy_version": draft.source_policy_version,
            "status": draft.status.as_str(),
            "winning_evidence_ids": draft.winning_evidence_ids,
            "winning_value": draft.winning_value,
            "ranking": draft.ranking,
            "reason": draft.reason,
        }))?;
        let row = sqlx::query(
            r#"
            INSERT INTO research.conflict_evaluations (
                id, conflict_id, research_run_id, match_id, trace_id,
                source_policy_key, source_policy_version, evaluation_status,
                winning_evidence_ids, winning_value, ranking, reason,
                idempotency_key, evaluation_fingerprint
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14
            )
            ON CONFLICT (idempotency_key) DO NOTHING
            RETURNING id, evaluation_status, evaluation_fingerprint, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.conflict_id)
        .bind(draft.research_run_id)
        .bind(draft.match_id)
        .bind(draft.trace_id)
        .bind(&draft.source_policy_key)
        .bind(&draft.source_policy_version)
        .bind(draft.status.as_str())
        .bind(&draft.winning_evidence_ids)
        .bind(&draft.winning_value)
        .bind(&draft.ranking)
        .bind(&draft.reason)
        .bind(&draft.idempotency_key)
        .bind(&fingerprint)
        .fetch_optional(&self.pool)
        .await?;
        let row = match row {
            Some(row) => row,
            None => {
                sqlx::query(
                    r#"SELECT id, evaluation_status, evaluation_fingerprint, created_at
                   FROM research.conflict_evaluations
                   WHERE idempotency_key = $1"#,
                )
                .bind(&draft.idempotency_key)
                .fetch_one(&self.pool)
                .await?
            }
        };
        let existing: String = row.try_get("evaluation_fingerprint")?;
        ensure_fingerprint("冲突评估", &draft.idempotency_key, &existing, &fingerprint)?;
        Ok(ConflictEvaluationRecord {
            id: row.try_get("id")?,
            status: parse_conflict_evaluation_status(
                row.try_get::<String, _>("evaluation_status")?.as_str(),
            )?,
            evaluation_fingerprint: existing,
            created_at: row.try_get("created_at")?,
        })
    }

    pub async fn append_conflict_event(
        &self,
        conflict_id: Uuid,
        event_type: &str,
        actor: &str,
        payload: &Value,
        idempotency_key: &str,
    ) -> PersistenceResult<()> {
        if !matches!(
            event_type,
            "resolved" | "reopened" | "dismissed" | "accepted_unknown"
        ) {
            return Err(PersistenceError::InvalidState(
                "冲突事件类型无效".to_string(),
            ));
        }
        let fingerprint = sha256_json(&json!({
            "event_type": event_type,
            "actor": actor,
            "payload": payload,
        }))?;
        let inserted: Option<Uuid> = sqlx::query_scalar(
            r#"
            INSERT INTO research.evidence_conflict_events (
                id, conflict_id, event_type, actor, payload,
                idempotency_key, event_fingerprint
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (conflict_id, idempotency_key) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(conflict_id)
        .bind(event_type)
        .bind(actor)
        .bind(payload)
        .bind(idempotency_key)
        .bind(&fingerprint)
        .fetch_optional(&self.pool)
        .await?;
        if inserted.is_none() {
            let existing: String = sqlx::query_scalar(
                r#"
                SELECT event_fingerprint
                FROM research.evidence_conflict_events
                WHERE conflict_id = $1 AND idempotency_key = $2
                "#,
            )
            .bind(conflict_id)
            .bind(idempotency_key)
            .fetch_one(&self.pool)
            .await?;
            ensure_fingerprint("冲突事件", idempotency_key, &existing, &fingerprint)?;
        }
        Ok(())
    }

    pub async fn append_evidence_route(
        &self,
        draft: &EvidenceRouteDraft,
    ) -> PersistenceResult<EvidenceRouteRecord> {
        let mut evidence_ids = draft.selected_evidence_ids.clone();
        evidence_ids.sort_unstable();
        evidence_ids.dedup();
        let fingerprint = sha256_json(&json!({
            "research_run_id": draft.research_run_id,
            "match_id": draft.match_id,
            "trace_id": draft.trace_id,
            "route_key": draft.route_key,
            "field_key": draft.field_key,
            "target_module": draft.target_module,
            "target_slot": draft.target_slot,
            "route_registry_version": draft.route_registry_version,
            "entity_type": draft.entity_type,
            "entity_id": draft.entity_id,
            "status": draft.status.as_str(),
            "verification_state": draft.verification_state,
            "selected_evidence_ids": evidence_ids,
            "selected_value": draft.selected_value,
            "reason": draft.reason,
        }))?;
        let row = sqlx::query(
            r#"
            INSERT INTO research.evidence_routes (
                id, research_run_id, match_id, trace_id, route_key,
                field_key, target_module, target_slot, route_registry_version,
                entity_type, entity_id, route_status, verification_state, selected_evidence_ids,
                selected_value, reason, idempotency_key, route_fingerprint
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12, $13, $14,
                $15, $16, $17, $18
            )
            ON CONFLICT (idempotency_key) DO NOTHING
            RETURNING id, route_status, route_fingerprint, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.research_run_id)
        .bind(draft.match_id)
        .bind(draft.trace_id)
        .bind(&draft.route_key)
        .bind(&draft.field_key)
        .bind(&draft.target_module)
        .bind(&draft.target_slot)
        .bind(&draft.route_registry_version)
        .bind(&draft.entity_type)
        .bind(draft.entity_id)
        .bind(draft.status.as_str())
        .bind(&draft.verification_state)
        .bind(&evidence_ids)
        .bind(&draft.selected_value)
        .bind(&draft.reason)
        .bind(&draft.idempotency_key)
        .bind(&fingerprint)
        .fetch_optional(&self.pool)
        .await?;
        let row = match row {
            Some(row) => row,
            None => {
                sqlx::query(
                    r#"SELECT id, route_status, route_fingerprint, created_at
                   FROM research.evidence_routes
                   WHERE idempotency_key = $1"#,
                )
                .bind(&draft.idempotency_key)
                .fetch_one(&self.pool)
                .await?
            }
        };
        let existing: String = row.try_get("route_fingerprint")?;
        ensure_fingerprint("证据路由", &draft.idempotency_key, &existing, &fingerprint)?;
        Ok(EvidenceRouteRecord {
            id: row.try_get("id")?,
            status: parse_evidence_route_status(
                row.try_get::<String, _>("route_status")?.as_str(),
            )?,
            route_fingerprint: existing,
            created_at: row.try_get("created_at")?,
        })
    }
}

fn validate_source_policy_definition(draft: &SourcePolicyVersionDraft) -> PersistenceResult<()> {
    let mut tiers = BTreeMap::new();
    for tier in &draft.definition.tiers {
        if tier.key.trim().is_empty() || tier.rank > 1000 {
            return Err(PersistenceError::InvalidState(
                "来源等级键不能为空且rank不能超过1000".to_string(),
            ));
        }
        if tiers.insert(tier.key.as_str(), tier.rank).is_some() {
            return Err(PersistenceError::InvalidState(
                "来源策略包含重复等级键".to_string(),
            ));
        }
    }
    if !tiers.contains_key(draft.definition.default_tier.as_str()) {
        return Err(PersistenceError::InvalidState(
            "来源策略默认等级未在tiers中定义".to_string(),
        ));
    }
    let mut domains = BTreeMap::new();
    for rule in &draft.definition.domain_rules {
        let domain = normalize_domain(&rule.domain);
        if domain.is_empty() || !tiers.contains_key(rule.tier.as_str()) {
            return Err(PersistenceError::InvalidState(
                "来源域名规则引用了无效域名或等级".to_string(),
            ));
        }
        if domains.insert(domain, rule.tier.as_str()).is_some() {
            return Err(PersistenceError::InvalidState(
                "来源策略包含重复域名规则".to_string(),
            ));
        }
    }
    Ok(())
}

fn keep_best_candidate(map: &mut BTreeMap<Uuid, EntityCandidate>, candidate: EntityCandidate) {
    match map.get(&candidate.entity_id) {
        Some(existing) if existing.score >= candidate.score => {}
        _ => {
            map.insert(candidate.entity_id, candidate);
        }
    }
}

fn normalize_domain(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("*.")
        .trim_start_matches("www.")
        .to_lowercase()
}

fn normalize_for_lookup(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn ensure_fingerprint(
    label: &str,
    idempotency_key: &str,
    existing: &str,
    expected: &str,
) -> PersistenceResult<()> {
    if existing == expected {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(format!(
            "{label}幂等键{idempotency_key}已存在但载荷不同"
        )))
    }
}

fn parse_entity_resolution_status(value: &str) -> PersistenceResult<EntityResolutionStatus> {
    match value {
        "resolved" => Ok(EntityResolutionStatus::Resolved),
        "ambiguous" => Ok(EntityResolutionStatus::Ambiguous),
        "unmatched" => Ok(EntityResolutionStatus::Unmatched),
        "unsupported" => Ok(EntityResolutionStatus::Unsupported),
        other => Err(PersistenceError::InvalidState(format!(
            "未知实体解析状态：{other}"
        ))),
    }
}

fn parse_time_audit_status(value: &str) -> PersistenceResult<TimeAuditStatus> {
    match value {
        "accepted" => Ok(TimeAuditStatus::Accepted),
        "accepted_non_fact" => Ok(TimeAuditStatus::AcceptedNonFact),
        "rejected_future" => Ok(TimeAuditStatus::RejectedFuture),
        "rejected_retrieved_after_cutoff" => Ok(TimeAuditStatus::RejectedRetrievedAfterCutoff),
        "rejected_missing_evidence_time" => Ok(TimeAuditStatus::RejectedMissingEvidenceTime),
        "rejected_missing_timezone" => Ok(TimeAuditStatus::RejectedMissingTimezone),
        "rejected_invalid_order" => Ok(TimeAuditStatus::RejectedInvalidOrder),
        other => Err(PersistenceError::InvalidState(format!(
            "未知时间审计状态：{other}"
        ))),
    }
}

fn parse_conflict_evaluation_status(value: &str) -> PersistenceResult<ConflictEvaluationStatus> {
    match value {
        "auto_resolved" => Ok(ConflictEvaluationStatus::AutoResolved),
        "manual_required" => Ok(ConflictEvaluationStatus::ManualRequired),
        "accepted_unknown" => Ok(ConflictEvaluationStatus::AcceptedUnknown),
        other => Err(PersistenceError::InvalidState(format!(
            "未知冲突评估状态：{other}"
        ))),
    }
}

fn parse_evidence_route_status(value: &str) -> PersistenceResult<EvidenceRouteStatus> {
    match value {
        "routed" => Ok(EvidenceRouteStatus::Routed),
        "missing" => Ok(EvidenceRouteStatus::Missing),
        "blocked_entity" => Ok(EvidenceRouteStatus::BlockedEntity),
        "blocked_time" => Ok(EvidenceRouteStatus::BlockedTime),
        "blocked_conflict" => Ok(EvidenceRouteStatus::BlockedConflict),
        "blocked_unregistered_field" => Ok(EvidenceRouteStatus::BlockedUnregisteredField),
        "ignored_non_model_fact" => Ok(EvidenceRouteStatus::IgnoredNonModelFact),
        other => Err(PersistenceError::InvalidState(format!(
            "未知证据路由状态：{other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::{SourcePolicyDefinition, SourceTierDefinition, SourceTierRule};

    #[test]
    fn source_policy_rejects_duplicate_domains_and_unknown_tiers() {
        let valid = SourcePolicyVersionDraft {
            policy_key: "default".to_string(),
            version: "1.0.0".to_string(),
            competition_profile_id: None,
            definition: SourcePolicyDefinition {
                schema_version: "football.p4-source-policy.v1".to_string(),
                default_tier: "unclassified".to_string(),
                tiers: vec![SourceTierDefinition {
                    key: "unclassified".to_string(),
                    rank: 100,
                }],
                domain_rules: vec![],
            },
            metadata: json!({}),
        };
        validate_source_policy_definition(&valid).expect("valid source policy");

        let mut invalid = valid.clone();
        invalid.definition.domain_rules = vec![SourceTierRule {
            domain: "example.com".to_string(),
            tier: "missing".to_string(),
        }];
        assert!(validate_source_policy_definition(&invalid).is_err());
    }

    #[test]
    fn best_candidate_keeps_highest_score_per_entity() {
        let id = Uuid::new_v4();
        let mut map = BTreeMap::new();
        keep_best_candidate(
            &mut map,
            EntityCandidate {
                entity_id: id,
                canonical_name: "A".to_string(),
                matched_name: "A".to_string(),
                strategy: "global".to_string(),
                score: 70,
                relation: None,
            },
        );
        keep_best_candidate(
            &mut map,
            EntityCandidate {
                entity_id: id,
                canonical_name: "A".to_string(),
                matched_name: "Alias".to_string(),
                strategy: "scoped".to_string(),
                score: 95,
                relation: Some("home".to_string()),
            },
        );
        assert_eq!(map.get(&id).expect("candidate").score, 95);
    }
}
