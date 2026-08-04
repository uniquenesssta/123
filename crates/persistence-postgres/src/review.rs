use super::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Duration, Utc};
use football_domain::{
    AbilityCandidateDecision, AbilityCandidateDecisionDraft, AbilityCandidateStatus,
    AbilityUpdateCandidateRecord, CalculatedMatchReview, MatchEventRevisionStatus,
    MatchEventSummary, MatchEventType, MatchEventVerificationStatus, MatchRecord,
    MatchResultRecord, MatchReviewDetail, MatchReviewDraft, MatchReviewEventDraft,
    MatchReviewEventRecord, MatchReviewSummary,
    PlayerMatchReviewRecord,
    ReviewPlayerBaseline, ReviewPreparationData, ReviewTeamContext, ReviewableMatch,
    SubstitutionRecord, TeamMatchReviewRecord,
};
use football_review_engine::calculate_review;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use std::collections::{BTreeMap, HashMap, HashSet};
use uuid::Uuid;

impl PostgresStore {
    pub async fn generate_match_review(
        &self,
        draft: &MatchReviewDraft,
    ) -> PersistenceResult<MatchReviewDetail> {
        validate_review_draft(draft)?;
        let mut tx = self.pool.begin().await?;
        let preparation = prepare_review_in_tx(&mut tx, draft).await?;
        let calculation = calculate_review(&preparation);
        let review_id =
            persist_calculation_in_tx(&mut tx, draft, &preparation, &calculation).await?;
        write_audit_event(
            &mut tx,
            "match_review_generated",
            "match_review",
            Some(review_id.to_string()),
            json!({
                "match_id": draft.match_id,
                "review_version": draft.review_version,
                "calculation_version": calculation.calculation_version,
                "player_count": calculation.player_reviews.len(),
                "team_count": calculation.team_reviews.len(),
                "ability_candidate_count": calculation.player_reviews.iter().map(|item| item.ability_candidates.len()).sum::<usize>(),
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_match_review(review_id).await
    }

    pub async fn commit_match_review_facts(
        &self,
        draft: &MatchReviewDraft,
    ) -> PersistenceResult<()> {
        validate_review_draft(draft)?;
        let mut tx = self.pool.begin().await?;
        let preparation = prepare_review_in_tx(&mut tx, draft).await?;
        write_audit_event(
            &mut tx,
            "match_review_facts_committed",
            "match",
            Some(draft.match_id.to_string()),
            json!({
                "result": preparation.result,
                "substitution_count": draft.substitutions.len(),
                "event_count": draft.events.len(),
                "player_observation_count": draft.player_observations.len(),
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_reviewable_matches(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<ReviewableMatch>> {
        let rows = sqlx::query(
            r#"
            WITH selected_fixtures AS (
                SELECT fixture.*
                FROM football.matches fixture
                WHERE fixture.status IN ('live', 'finished')
                   OR fixture.kickoff_time <= now()
                   OR EXISTS (
                       SELECT 1 FROM football.match_results result
                       WHERE result.match_id = fixture.id
                   )
                ORDER BY fixture.kickoff_time DESC, fixture.id DESC
                LIMIT $1
            )
            SELECT
                fixture.id, fixture.external_key,
                fixture.competition_id, competition.name AS competition_name,
                fixture.season_id, fixture.stage_id, fixture.round_id,
                fixture.home_team_id, home.canonical_name AS home_team_name,
                fixture.away_team_id, away.canonical_name AS away_team_name,
                fixture.kickoff_time, fixture.status, fixture.venue,
                result.home_goals_90, result.away_goals_90,
                result.home_goals_extra_time, result.away_goals_extra_time,
                result.home_penalties, result.away_penalties,
                result.finalized_at, result.metadata AS result_metadata,
                latest_review.id AS review_id,
                latest_review.review_version,
                latest_review.status AS review_status,
                latest_review.data_coverage::double precision AS review_data_coverage,
                latest_review.source_run_id,
                latest_review.calculation_version,
                latest_review.result_snapshot,
                latest_review.substitutions_snapshot,
                latest_review.prediction_evaluation,
                latest_review.conclusions,
                latest_review.created_at AS review_created_at,
                latest_review.finalized_at AS review_finalized_at,
                COALESCE(observation_count.value, 0)::bigint AS player_observation_count,
                COALESCE(lineup_count.value, 0)::bigint AS actual_lineup_count
            FROM selected_fixtures fixture
            LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            LEFT JOIN football.match_results result ON result.match_id = fixture.id
            LEFT JOIN LATERAL (
                SELECT review.*
                FROM review.match_reviews review
                WHERE review.match_id = fixture.id
                ORDER BY review.created_at DESC, review.id DESC
                LIMIT 1
            ) latest_review ON true
            LEFT JOIN LATERAL (
                SELECT COUNT(*) AS value
                FROM review.player_match_observations observation
                WHERE observation.match_id = fixture.id
            ) observation_count ON true
            LEFT JOIN LATERAL (
                SELECT COUNT(*) AS value
                FROM football.lineups lineup
                WHERE lineup.match_id = fixture.id
                  AND lineup.lineup_type = 'actual'
                  AND lineup.status = 'active'
            ) lineup_count ON true
            ORDER BY fixture.kickoff_time DESC, fixture.id DESC
            "#,
        )
        .bind(i64::from(limit.clamp(1, 200)))
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(reviewable_match_from_row).collect()
    }

    pub async fn list_match_reviews(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<MatchReviewSummary>> {
        let rows = sqlx::query(
            r#"
            SELECT
                review.id, review.match_id, fixture.external_key AS match_key,
                home.canonical_name AS home_team_name,
                away.canonical_name AS away_team_name,
                review.review_version, review.status,
                review.data_coverage::double precision AS data_coverage,
                review.source_run_id, review.calculation_version,
                review.result_snapshot, review.substitutions_snapshot, review.prediction_evaluation,
                review.conclusions, review.created_at, review.finalized_at
            FROM review.match_reviews review
            JOIN football.matches fixture ON fixture.id = review.match_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            ORDER BY review.created_at DESC, review.id DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 200)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(match_review_summary_from_row).collect()
    }

    pub async fn read_match_review(&self, review_id: Uuid) -> PersistenceResult<MatchReviewDetail> {
        let summary_row = sqlx::query(
            r#"
            SELECT
                review.id, review.match_id, fixture.external_key AS match_key,
                home.canonical_name AS home_team_name,
                away.canonical_name AS away_team_name,
                review.review_version, review.status,
                review.data_coverage::double precision AS data_coverage,
                review.source_run_id, review.calculation_version,
                review.result_snapshot, review.substitutions_snapshot, review.prediction_evaluation,
                review.conclusions, review.created_at, review.finalized_at
            FROM review.match_reviews review
            JOIN football.matches fixture ON fixture.id = review.match_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            WHERE review.id = $1
            "#,
        )
        .bind(review_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("复盘记录不存在".to_string()))?;
        let summary = match_review_summary_from_row(&summary_row)?;
        let result: MatchResultRecord = serde_json::from_value(summary.result_snapshot.clone())?;
        let substitutions: Vec<SubstitutionRecord> =
            serde_json::from_value(summary.substitutions_snapshot.clone())?;

        let player_rows = sqlx::query(
            r#"
            SELECT
                player_review.id, player_review.match_review_id,
                player_review.player_id, player.canonical_name AS player_name,
                player_review.team_id, team.canonical_name AS team_name,
                player_review.role_code, player_review.started, player_review.entry_type,
                player_review.minutes_played, player_review.expected_performance,
                player_review.actual_performance, player_review.realization_ratio,
                player_review.confidence, player_review.contribution_weight,
                player_review.ability_candidate_count, player_review.metrics
            FROM review.player_match_reviews player_review
            JOIN football.players player ON player.id = player_review.player_id
            JOIN football.teams team ON team.id = player_review.team_id
            WHERE player_review.match_review_id = $1
            ORDER BY player_review.team_id, player_review.started DESC,
                     player_review.minutes_played DESC NULLS LAST, player.canonical_name
            "#,
        )
        .bind(review_id)
        .fetch_all(&self.pool)
        .await?;
        let player_reviews = player_rows
            .iter()
            .map(player_match_review_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;

        let team_rows = sqlx::query(
            r#"
            SELECT
                team_review.id, team_review.match_review_id,
                team_review.team_id, team.canonical_name AS team_name,
                team_review.chemistry_score, team_review.lineup_continuity,
                team_review.performance_cohesion, team_review.bench_strength,
                team_review.bench_dropoff, team_review.substitution_impact,
                team_review.substitute_count, team_review.realization_score,
                team_review.confidence, team_review.metrics
            FROM review.team_match_reviews team_review
            JOIN football.teams team ON team.id = team_review.team_id
            WHERE team_review.match_review_id = $1
            ORDER BY team.canonical_name
            "#,
        )
        .bind(review_id)
        .fetch_all(&self.pool)
        .await?;
        let team_reviews = team_rows
            .iter()
            .map(team_match_review_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        let events = self.list_match_events(summary.match_id).await?;
        let event_summary = summarize_match_events(&events);
        let ability_candidates = self
            .list_ability_candidates(None, 1000, Some(review_id))
            .await?;

        Ok(MatchReviewDetail {
            summary,
            result,
            substitutions,
            events,
            event_summary,
            player_reviews,
            team_reviews,
            ability_candidates,
        })
    }

    pub async fn list_ability_candidates(
        &self,
        status: Option<AbilityCandidateStatus>,
        limit: u32,
        match_review_id: Option<Uuid>,
    ) -> PersistenceResult<Vec<AbilityUpdateCandidateRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                candidate.id, candidate.match_review_id, candidate.player_match_review_id,
                candidate.player_id, player.canonical_name AS player_name,
                candidate.dimension_code, dimension.name AS dimension_name,
                candidate.current_value, candidate.proposed_value,
                candidate.confidence, candidate.sample_size, candidate.evidence,
                candidate.calculation_version, candidate.status,
                candidate.created_at, candidate.decided_at,
                candidate.decided_by, candidate.decision_note,
                candidate.accepted_observation_id
            FROM review.ability_update_candidates candidate
            JOIN football.players player ON player.id = candidate.player_id
            JOIN feature.player_ability_dimensions dimension ON dimension.code = candidate.dimension_code
            WHERE ($1::text IS NULL OR candidate.status = $1)
              AND ($2::uuid IS NULL OR candidate.match_review_id = $2)
            ORDER BY
                CASE candidate.status WHEN 'pending' THEN 0 ELSE 1 END,
                candidate.confidence DESC, candidate.created_at DESC
            LIMIT $3
            "#,
        )
        .bind(status.map(AbilityCandidateStatus::as_str))
        .bind(match_review_id)
        .bind(i64::from(limit.clamp(1, 2000)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(ability_candidate_from_row).collect()
    }

    pub async fn decide_ability_candidate(
        &self,
        draft: &AbilityCandidateDecisionDraft,
    ) -> PersistenceResult<AbilityUpdateCandidateRecord> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            SELECT
                candidate.id, candidate.match_review_id, candidate.player_match_review_id,
                candidate.player_id, candidate.dimension_code,
                candidate.current_value, candidate.proposed_value,
                candidate.confidence, candidate.sample_size, candidate.evidence,
                candidate.calculation_version, candidate.status,
                review.match_id, fixture.kickoff_time
            FROM review.ability_update_candidates candidate
            LEFT JOIN review.match_reviews review ON review.id = candidate.match_review_id
            LEFT JOIN football.matches fixture ON fixture.id = review.match_id
            WHERE candidate.id = $1
            FOR UPDATE OF candidate
            "#,
        )
        .bind(draft.candidate_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("能力更新候选不存在".to_string()))?;
        let status: String = row.try_get("status")?;
        if status != "pending" {
            return Err(PersistenceError::InvalidState(format!(
                "能力更新候选已经处理，当前状态：{status}"
            )));
        }
        let decided_by = draft
            .decided_by
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("local_user");
        let player_id: Uuid = row.try_get("player_id")?;
        let proposed_value: f64 = row.try_get("proposed_value")?;
        let current_value: Option<f64> = row.try_get("current_value")?;
        let calculation_version: String = row.try_get("calculation_version")?;
        let match_id: Option<Uuid> = row.try_get("match_id")?;
        let observed_at: Option<DateTime<Utc>> = row.try_get("kickoff_time")?;
        let evidence: Value = row.try_get("evidence")?;
        let confidence: f64 = row.try_get("confidence")?;
        let sample_size: i32 = row.try_get("sample_size")?;
        let dimension_code: String = row.try_get("dimension_code")?;

        let (decision_value, candidate_status, observation_id) = match draft.decision {
            AbilityCandidateDecision::Accept => {
                let current_projection: Option<f64> = sqlx::query_scalar(
                    r#"
                    SELECT value
                    FROM feature.player_current_abilities
                    WHERE player_id = $1 AND dimension_code = $2
                    "#,
                )
                .bind(player_id)
                .bind(&dimension_code)
                .fetch_optional(&mut *tx)
                .await?;
                let candidate_is_current = match (current_value, current_projection) {
                    (Some(expected), Some(actual)) => (expected - actual).abs() <= 0.01,
                    (None, None) => true,
                    _ => false,
                };
                if !candidate_is_current {
                    return Err(PersistenceError::InvalidState(
                        "球员当前能力已发生变化，该候选已经过期，请重新生成复盘".to_string(),
                    ));
                }
                let observation_id = Uuid::new_v4();
                sqlx::query(
                    r#"
                    INSERT INTO feature.player_ability_observations (
                        id, player_id, dimension_code, context_type, context_id,
                        value, confidence, sample_size, observed_at,
                        effective_from, effective_to, calculation_version,
                        source_document_id, metadata
                    ) VALUES (
                        $1, $2, $3, 'match_review', $4,
                        $5, $6, $7, $8,
                        now(), NULL, $9, NULL, $10
                    )
                    "#,
                )
                .bind(observation_id)
                .bind(player_id)
                .bind(&dimension_code)
                .bind(match_id)
                .bind(proposed_value)
                .bind(confidence)
                .bind(sample_size)
                .bind(observed_at.unwrap_or_else(Utc::now))
                .bind(format!("{calculation_version}:accepted"))
                .bind(json!({
                    "candidate_id": draft.candidate_id,
                    "evidence": evidence,
                    "decision_note": draft.decision_note,
                    "decided_by": decided_by,
                }))
                .execute(&mut *tx)
                .await?;
                ("accepted", "accepted", Some(observation_id))
            }
            AbilityCandidateDecision::Reject => ("rejected", "rejected", None),
        };

        sqlx::query(
            r#"
            UPDATE review.ability_update_candidates
            SET status = $2, decided_at = now(), decided_by = $3,
                decision_note = $4, accepted_observation_id = $5
            WHERE id = $1
            "#,
        )
        .bind(draft.candidate_id)
        .bind(candidate_status)
        .bind(decided_by)
        .bind(draft.decision_note.as_deref())
        .bind(observation_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO review.ability_update_decisions (
                id, candidate_id, decision, previous_value, proposed_value,
                applied_observation_id, decided_by, decision_note
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.candidate_id)
        .bind(decision_value)
        .bind(current_value)
        .bind(proposed_value)
        .bind(observation_id)
        .bind(decided_by)
        .bind(draft.decision_note.as_deref())
        .execute(&mut *tx)
        .await?;

        write_audit_event(
            &mut tx,
            "ability_update_candidate_decided",
            "ability_update_candidate",
            Some(draft.candidate_id.to_string()),
            json!({
                "decision": decision_value,
                "player_id": player_id,
                "dimension_code": dimension_code,
                "previous_value": current_value,
                "proposed_value": proposed_value,
                "observation_id": observation_id,
                "decided_by": decided_by,
            }),
        )
        .await?;
        tx.commit().await?;

        read_ability_candidate(&self.pool, draft.candidate_id).await
    }

    pub async fn read_match_result(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<Option<MatchResultRecord>> {
        let row = sqlx::query(
            r#"
            SELECT match_id, home_goals_90, away_goals_90,
                   home_goals_extra_time, away_goals_extra_time,
                   home_penalties, away_penalties, finalized_at, metadata
            FROM football.match_results
            WHERE match_id = $1
            "#,
        )
        .bind(match_id)
        .fetch_optional(&self.pool)
        .await?;
        row.as_ref().map(match_result_from_row).transpose()
    }

    pub async fn list_match_events(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<Vec<MatchReviewEventRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT event.id, event.match_id, event.event_key, event.sequence_no,
                   event.event_type, event.team_id, team.canonical_name AS team_name,
                   event.player_id, player.canonical_name AS player_name,
                   event.related_player_id, related_player.canonical_name AS related_player_name,
                   event.minute, event.stoppage_minute, event.period,
                   event.home_score, event.away_score,
                   event.verification_status, event.revision_status, event.verified_at,
                   event.source_document_id, event.source_package_id, event.revision_of_event_id,
                   event.description, event.source_urls, event.confidence,
                   event.metadata, event.recorded_at, event.updated_at
            FROM review.match_events event
            LEFT JOIN football.teams team ON team.id = event.team_id
            LEFT JOIN football.players player ON player.id = event.player_id
            LEFT JOIN football.players related_player ON related_player.id = event.related_player_id
            WHERE event.match_id = $1
              AND event.revision_status <> 'superseded'
            ORDER BY event.sequence_no, event.minute, event.stoppage_minute NULLS FIRST, event.id
            "#,
        )
        .bind(match_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(match_event_from_row).collect()
    }

    pub async fn list_substitutions(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<Vec<SubstitutionRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT substitution.id, substitution.match_id,
                   substitution.team_id, team.canonical_name AS team_name,
                   substitution.player_out_id, player_out.canonical_name AS player_out_name,
                   substitution.player_in_id, player_in.canonical_name AS player_in_name,
                   substitution.minute, substitution.period, substitution.reason
            FROM football.substitutions substitution
            JOIN football.teams team ON team.id = substitution.team_id
            LEFT JOIN football.players player_out ON player_out.id = substitution.player_out_id
            LEFT JOIN football.players player_in ON player_in.id = substitution.player_in_id
            WHERE substitution.match_id = $1
            ORDER BY substitution.minute, substitution.id
            "#,
        )
        .bind(match_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(substitution_from_row).collect()
    }
}

async fn prepare_review_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    draft: &MatchReviewDraft,
) -> PersistenceResult<ReviewPreparationData> {
    let match_row = sqlx::query(
        r#"
        SELECT
            fixture.id, fixture.external_key,
            fixture.competition_id, competition.name AS competition_name,
            fixture.season_id, fixture.stage_id, fixture.round_id,
            fixture.home_team_id, home.canonical_name AS home_team_name,
            fixture.away_team_id, away.canonical_name AS away_team_name,
            fixture.kickoff_time, fixture.status, fixture.venue
        FROM football.matches fixture
        LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
        JOIN football.teams home ON home.id = fixture.home_team_id
        JOIN football.teams away ON away.id = fixture.away_team_id
        WHERE fixture.id = $1
        FOR UPDATE OF fixture
        "#,
    )
    .bind(draft.match_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("比赛不存在".to_string()))?;
    let match_record = match_record_from_review_row(&match_row)?;
    let valid_teams = HashSet::from([match_record.home_team_id, match_record.away_team_id]);
    let player_teams: HashMap<Uuid, Uuid> = draft
        .player_observations
        .iter()
        .map(|observation| (observation.player_id, observation.team_id))
        .collect();
    for observation in &draft.player_observations {
        if !valid_teams.contains(&observation.team_id) {
            return Err(PersistenceError::InvalidState(format!(
                "球员 {} 的球队不是本场参赛队",
                observation.player_id
            )));
        }
    }
    for team_id in &valid_teams {
        if !draft
            .player_observations
            .iter()
            .any(|observation| observation.team_id == *team_id)
        {
            return Err(PersistenceError::InvalidState(
                "主客队都必须至少包含一条球员赛后表现".to_string(),
            ));
        }
    }
    for substitution in &draft.substitutions {
        if !valid_teams.contains(&substitution.team_id) {
            return Err(PersistenceError::InvalidState(
                "换人记录中的球队不是本场参赛队".to_string(),
            ));
        }
        for player_id in [substitution.player_out_id, substitution.player_in_id]
            .into_iter()
            .flatten()
        {
            if player_teams
                .get(&player_id)
                .is_some_and(|team_id| *team_id != substitution.team_id)
            {
                return Err(PersistenceError::InvalidState(
                    "换人球员与换人球队不一致".to_string(),
                ));
            }
        }
    }
    let revision_event_ids = draft
        .events
        .iter()
        .filter_map(|event| event.revision_of_event_id)
        .collect::<HashSet<_>>();
    for event in &draft.events {
        if event.team_id.is_some_and(|team_id| !valid_teams.contains(&team_id)) {
            return Err(PersistenceError::InvalidState(
                "比赛事件中的球队不是本场参赛队".to_string(),
            ));
        }
        if let (Some(team_id), Some(player_id)) = (event.team_id, event.player_id) {
            let player_matches_team = player_teams.get(&player_id).is_none_or(|value| {
                if event.event_type == MatchEventType::OwnGoal {
                    *value != team_id
                } else {
                    *value == team_id
                }
            });
            if !player_matches_team {
                let message = if event.event_type == MatchEventType::OwnGoal {
                    "乌龙球球员必须属于事件受益球队的对手"
                } else {
                    "比赛事件球员与事件球队不一致"
                };
                return Err(PersistenceError::InvalidState(message.to_string()));
            }
        }
        if let (Some(team_id), Some(player_id)) = (event.team_id, event.related_player_id) {
            if player_teams
                .get(&player_id)
                .is_some_and(|value| *value != team_id)
            {
                return Err(PersistenceError::InvalidState(
                    "比赛事件关联球员与事件球队不一致".to_string(),
                ));
            }
        }
    }
    if !revision_event_ids.is_empty() {
        let revision_ids = revision_event_ids.iter().copied().collect::<Vec<_>>();
        let rows = sqlx::query(
            "SELECT id, match_id FROM review.match_events WHERE id = ANY($1::uuid[])",
        )
        .bind(&revision_ids)
        .fetch_all(&mut **tx)
        .await?;
        if rows.len() != revision_event_ids.len() {
            return Err(PersistenceError::InvalidState(
                "比赛事件 revision_of_event_id 引用了不存在的历史事件".to_string(),
            ));
        }
        for row in &rows {
            let referenced_match_id: Uuid = row.try_get("match_id")?;
            if referenced_match_id != draft.match_id {
                return Err(PersistenceError::InvalidState(
                    "比赛事件只能修订同一场比赛的历史事件".to_string(),
                ));
            }
        }
    }

    sqlx::query(
        r#"
        INSERT INTO football.match_results (
            match_id, home_goals_90, away_goals_90,
            home_goals_extra_time, away_goals_extra_time,
            home_penalties, away_penalties, finalized_at,
            source_document_id, metadata
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
        ON CONFLICT (match_id) DO UPDATE SET
            home_goals_90 = EXCLUDED.home_goals_90,
            away_goals_90 = EXCLUDED.away_goals_90,
            home_goals_extra_time = EXCLUDED.home_goals_extra_time,
            away_goals_extra_time = EXCLUDED.away_goals_extra_time,
            home_penalties = EXCLUDED.home_penalties,
            away_penalties = EXCLUDED.away_penalties,
            finalized_at = EXCLUDED.finalized_at,
            source_document_id = EXCLUDED.source_document_id,
            metadata = football.match_results.metadata || EXCLUDED.metadata
        "#,
    )
    .bind(draft.match_id)
    .bind(draft.result.home_goals_90)
    .bind(draft.result.away_goals_90)
    .bind(draft.result.home_goals_extra_time)
    .bind(draft.result.away_goals_extra_time)
    .bind(draft.result.home_penalties)
    .bind(draft.result.away_penalties)
    .bind(draft.result.finalized_at)
    .bind(draft.result.source_document_id)
    .bind(&draft.result.metadata)
    .execute(&mut **tx)
    .await?;
    sqlx::query("UPDATE football.matches SET status='finished', updated_at=now() WHERE id=$1")
        .bind(draft.match_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query("DELETE FROM football.substitutions WHERE match_id = $1")
        .bind(draft.match_id)
        .execute(&mut **tx)
        .await?;
    for substitution in &draft.substitutions {
        sqlx::query(
            r#"
            INSERT INTO football.substitutions (
                id, match_id, team_id, player_out_id, player_in_id,
                minute, period, reason, source_document_id, metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.match_id)
        .bind(substitution.team_id)
        .bind(substitution.player_out_id)
        .bind(substitution.player_in_id)
        .bind(substitution.minute)
        .bind(substitution.period.trim())
        .bind(substitution.reason.as_deref())
        .bind(substitution.source_document_id)
        .bind(&substitution.metadata)
        .execute(&mut **tx)
        .await?;
    }

    let package_id = match_review_package_id(&draft.result.metadata);
    let mut incoming_event_keys = Vec::with_capacity(draft.events.len());
    for (index, event) in draft.events.iter().enumerate() {
        let event_key = match_event_key(event, draft.match_id, index)?;
        let sequence_no = event.sequence_no.unwrap_or(index as i32 + 1);
        incoming_event_keys.push(event_key.clone());
        sqlx::query(
            r#"
            INSERT INTO review.match_events (
                id, match_id, event_key, sequence_no, event_type,
                team_id, player_id, related_player_id,
                minute, stoppage_minute, period, home_score, away_score,
                verification_status, revision_status, verified_at,
                source_document_id, source_package_id, revision_of_event_id,
                description, source_urls, confidence, metadata
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,
                $14,$15,$16,$17,$18,$19,$20,$21,$22,$23
            )
            ON CONFLICT (match_id, event_key) DO UPDATE SET
                sequence_no = EXCLUDED.sequence_no,
                event_type = EXCLUDED.event_type,
                team_id = EXCLUDED.team_id,
                player_id = EXCLUDED.player_id,
                related_player_id = EXCLUDED.related_player_id,
                minute = EXCLUDED.minute,
                stoppage_minute = EXCLUDED.stoppage_minute,
                period = EXCLUDED.period,
                home_score = EXCLUDED.home_score,
                away_score = EXCLUDED.away_score,
                verification_status = EXCLUDED.verification_status,
                revision_status = EXCLUDED.revision_status,
                verified_at = EXCLUDED.verified_at,
                source_document_id = EXCLUDED.source_document_id,
                source_package_id = EXCLUDED.source_package_id,
                revision_of_event_id = EXCLUDED.revision_of_event_id,
                description = EXCLUDED.description,
                source_urls = EXCLUDED.source_urls,
                confidence = EXCLUDED.confidence,
                metadata = review.match_events.metadata || EXCLUDED.metadata,
                updated_at = now()
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.match_id)
        .bind(&event_key)
        .bind(sequence_no)
        .bind(event.event_type.as_str())
        .bind(event.team_id)
        .bind(event.player_id)
        .bind(event.related_player_id)
        .bind(event.minute)
        .bind(event.stoppage_minute)
        .bind(event.period.trim())
        .bind(event.home_score)
        .bind(event.away_score)
        .bind(event.verification_status.as_str())
        .bind(event.revision_status.as_str())
        .bind(event.verified_at)
        .bind(event.source_document_id)
        .bind(event.source_package_id.or(package_id))
        .bind(event.revision_of_event_id)
        .bind(event.description.as_deref())
        .bind(&event.source_urls)
        .bind(event.confidence)
        .bind(&event.metadata)
        .execute(&mut **tx)
        .await?;
    }
    sqlx::query(
        r#"
        UPDATE review.match_events
        SET revision_status = 'superseded', updated_at = now()
        WHERE match_id = $1
          AND revision_status <> 'superseded'
          AND NOT (event_key = ANY($2::text[]))
        "#,
    )
    .bind(draft.match_id)
    .bind(&incoming_event_keys)
    .execute(&mut **tx)
    .await?;

    sqlx::query("DELETE FROM review.player_match_observations WHERE match_id = $1")
        .bind(draft.match_id)
        .execute(&mut **tx)
        .await?;

    for observation in &draft.player_observations {
        let metrics = serde_json::to_value(&observation.metrics)?;
        sqlx::query(
            r#"
            INSERT INTO review.player_match_observations (
                id, match_id, player_id, team_id, position_code, role_code,
                started, minutes_played, performance_score, input_confidence,
                metrics, source_document_id
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            ON CONFLICT (match_id, player_id) DO UPDATE SET
                team_id = EXCLUDED.team_id,
                position_code = EXCLUDED.position_code,
                role_code = EXCLUDED.role_code,
                started = EXCLUDED.started,
                minutes_played = EXCLUDED.minutes_played,
                performance_score = EXCLUDED.performance_score,
                input_confidence = EXCLUDED.input_confidence,
                metrics = EXCLUDED.metrics,
                source_document_id = EXCLUDED.source_document_id,
                updated_at = now()
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.match_id)
        .bind(observation.player_id)
        .bind(observation.team_id)
        .bind(observation.position_code.as_deref())
        .bind(observation.role_code.as_deref())
        .bind(observation.started)
        .bind(observation.minutes_played)
        .bind(observation.performance_score)
        .bind(observation.input_confidence)
        .bind(metrics)
        .bind(observation.source_document_id)
        .execute(&mut **tx)
        .await?;
    }

    let result = MatchResultRecord {
        match_id: draft.match_id,
        home_goals_90: draft.result.home_goals_90,
        away_goals_90: draft.result.away_goals_90,
        home_goals_extra_time: draft.result.home_goals_extra_time,
        away_goals_extra_time: draft.result.away_goals_extra_time,
        home_penalties: draft.result.home_penalties,
        away_penalties: draft.result.away_penalties,
        finalized_at: draft.result.finalized_at,
        metadata: draft.result.metadata.clone(),
    };
    let substitutions = list_substitutions_in_tx(tx, draft.match_id).await?;
    let players = load_review_players_in_tx(tx, draft.match_id).await?;
    let mut teams = Vec::with_capacity(2);
    for (team_id, team_name) in [
        (
            match_record.home_team_id,
            match_record.home_team_name.clone(),
        ),
        (
            match_record.away_team_id,
            match_record.away_team_name.clone(),
        ),
    ] {
        let continuity = calculate_recent_starter_overlap(
            tx,
            draft.match_id,
            team_id,
            match_record.kickoff_time,
        )
        .await?;
        teams.push(ReviewTeamContext {
            team_id,
            team_name,
            recent_starter_overlap: continuity,
        });
    }
    let prediction = load_prediction_context_in_tx(
        tx,
        draft.match_id,
        draft.source_run_id,
        draft.result.home_goals_90,
        draft.result.away_goals_90,
    )
    .await?;

    Ok(ReviewPreparationData {
        data_coverage: draft.data_coverage,
        match_record,
        result,
        substitutions,
        players,
        teams,
        prediction,
    })
}

async fn persist_calculation_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    draft: &MatchReviewDraft,
    preparation: &ReviewPreparationData,
    calculation: &CalculatedMatchReview,
) -> PersistenceResult<Uuid> {
    sqlx::query(
        r#"
        WITH superseded AS (
            UPDATE review.ability_update_candidates candidate
            SET status = 'superseded', decided_at = now(),
                decided_by = 'system', decision_note = '同场比赛生成了新的复盘版本'
            WHERE candidate.status = 'pending'
              AND candidate.match_review_id IN (
                  SELECT id FROM review.match_reviews
                  WHERE match_id = $1 AND status = 'finalized'
              )
            RETURNING candidate.id, candidate.current_value, candidate.proposed_value
        )
        INSERT INTO review.ability_update_decisions (
            id, candidate_id, decision, previous_value, proposed_value,
            applied_observation_id, decided_by, decision_note
        )
        SELECT gen_random_uuid(), superseded.id, 'superseded',
               superseded.current_value, superseded.proposed_value,
               NULL, 'system', '同场比赛生成了新的复盘版本'
        FROM superseded
        "#,
    )
    .bind(draft.match_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "UPDATE review.match_reviews SET status='superseded' WHERE match_id=$1 AND status='finalized'",
    )
    .bind(draft.match_id)
    .execute(&mut **tx)
    .await?;

    let review_id = Uuid::new_v4();
    let review_version = draft.review_version.clone().unwrap_or_else(|| {
        format!(
            "review-{}-{}",
            Utc::now().format("%Y%m%d%H%M%S%.3f"),
            review_id.simple()
        )
    });
    let result_snapshot = serde_json::to_value(&preparation.result)?;
    let mut conclusions = calculation.conclusions.clone();
    if let (Some(notes), Some(object)) = (draft.notes.as_deref(), conclusions.as_object_mut()) {
        if !notes.trim().is_empty() {
            object.insert("notes".to_string(), Value::String(notes.trim().to_string()));
        }
    }
    sqlx::query(
        r#"
        INSERT INTO review.match_reviews (
            id, match_id, review_version, data_coverage, conclusions,
            source_run_id, status, calculation_version,
            result_snapshot, substitutions_snapshot, prediction_evaluation, finalized_at
        ) VALUES ($1,$2,$3,$4,$5,$6,'finalized',$7,$8,$9,$10,now())
        "#,
    )
    .bind(review_id)
    .bind(draft.match_id)
    .bind(&review_version)
    .bind(draft.data_coverage)
    .bind(&conclusions)
    .bind(preparation.prediction.as_ref().map(|item| item.run_id))
    .bind(&calculation.calculation_version)
    .bind(result_snapshot)
    .bind(serde_json::to_value(&preparation.substitutions)?)
    .bind(&calculation.prediction_evaluation)
    .execute(&mut **tx)
    .await?;

    for player in &calculation.player_reviews {
        let player_review_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO review.player_match_reviews (
                id, match_review_id, player_id, team_id, role_code,
                started, minutes_played, expected_performance,
                actual_performance, realization_ratio, confidence, metrics,
                observation_id, entry_type, contribution_weight,
                ability_candidate_count
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            "#,
        )
        .bind(player_review_id)
        .bind(review_id)
        .bind(player.player_id)
        .bind(player.team_id)
        .bind(player.role_code.as_deref())
        .bind(player.started)
        .bind(player.minutes_played)
        .bind(player.expected_performance)
        .bind(player.actual_performance)
        .bind(player.realization_ratio)
        .bind(player.confidence)
        .bind(&player.metrics)
        .bind(player.observation_id)
        .bind(&player.entry_type)
        .bind(player.contribution_weight)
        .bind(player.ability_candidates.len() as i32)
        .execute(&mut **tx)
        .await?;

        for candidate in &player.ability_candidates {
            sqlx::query(
                r#"
                INSERT INTO review.ability_update_candidates (
                    id, player_id, dimension_code, current_value,
                    proposed_value, confidence, sample_size, evidence,
                    calculation_version, status, match_review_id,
                    player_match_review_id
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'pending',$10,$11)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(candidate.player_id)
            .bind(&candidate.dimension_code)
            .bind(candidate.current_value)
            .bind(candidate.proposed_value)
            .bind(candidate.confidence)
            .bind(candidate.sample_size)
            .bind(&candidate.evidence)
            .bind(&calculation.calculation_version)
            .bind(review_id)
            .bind(player_review_id)
            .execute(&mut **tx)
            .await?;
        }
    }

    for team in &calculation.team_reviews {
        sqlx::query(
            r#"
            INSERT INTO review.team_match_reviews (
                id, match_review_id, team_id, chemistry_score,
                bench_strength, substitution_impact, realization_score,
                confidence, metrics, lineup_continuity,
                performance_cohesion, bench_dropoff, substitute_count
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(review_id)
        .bind(team.team_id)
        .bind(team.chemistry_score)
        .bind(team.bench_strength)
        .bind(team.substitution_impact)
        .bind(team.realization_score)
        .bind(team.confidence)
        .bind(&team.metrics)
        .bind(team.lineup_continuity)
        .bind(team.performance_cohesion)
        .bind(team.bench_dropoff)
        .bind(team.substitute_count)
        .execute(&mut **tx)
        .await?;
    }

    let observed_at = preparation.result.finalized_at;
    let valid_to = observed_at + Duration::days(14);
    for player in calculation
        .player_reviews
        .iter()
        .filter(|item| item.minutes_played > 0)
    {
        let position_code = preparation
            .players
            .iter()
            .find(|item| item.player_id == player.player_id)
            .and_then(|item| item.position_code.as_deref());
        let chemistry_score = calculation
            .team_reviews
            .iter()
            .find(|item| item.team_id == player.team_id)
            .map(|item| item.chemistry_score)
            .unwrap_or(50.0);
        let dynamic_values = [
            (
                "realization_multiplier",
                player.realization_ratio.clamp(0.50, 1.25),
                json!({"realization_ratio": player.realization_ratio}),
            ),
            (
                "chemistry_fit",
                (0.50 + chemistry_score.clamp(0.0, 100.0) * 0.006).clamp(0.50, 1.10),
                json!({"team_chemistry_score": chemistry_score}),
            ),
        ];
        for (tag_code, value, evidence) in dynamic_values {
            sqlx::query(
                r#"
                INSERT INTO feature.player_dynamic_tags (
                    id, player_id, tag_code, value, label, confidence,
                    observed_at, valid_from, valid_to, competition_id,
                    position_code, opponent_team_id, sample_size, source_type,
                    source_document_id, calculation_version, metadata
                ) VALUES (
                    $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,NULL,1,
                    'match_review',NULL,$12,$13
                )
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(player.player_id)
            .bind(tag_code)
            .bind(value)
            .bind(if tag_code == "realization_multiplier" {
                "赛后兑现率"
            } else {
                "赛后配合度"
            })
            .bind(player.confidence)
            .bind(observed_at)
            .bind(observed_at)
            .bind(valid_to)
            .bind(preparation.match_record.competition_id)
            .bind(position_code)
            .bind(&calculation.calculation_version)
            .bind(json!({
                "match_review_id": review_id,
                "match_id": draft.match_id,
                "team_id": player.team_id,
                "evidence": evidence,
            }))
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(review_id)
}

async fn load_review_players_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    match_id: Uuid,
) -> PersistenceResult<Vec<ReviewPlayerBaseline>> {
    let rows = sqlx::query(
        r#"
        SELECT
            observation.id AS observation_id,
            observation.player_id, player.canonical_name AS player_name,
            observation.team_id, team.canonical_name AS team_name,
            observation.position_code, observation.role_code,
            observation.started, observation.minutes_played,
            observation.performance_score, observation.input_confidence,
            observation.metrics,
            COALESCE(contribution.effective_contribution, profile.average_value, 50)::double precision AS expected_performance,
            COALESCE(contribution.confidence, profile.average_confidence, 0.50)::double precision AS expected_confidence,
            COALESCE(profile.abilities, '{}'::jsonb) AS current_abilities,
            COALESCE(history.reviewed_match_count, 0)::integer AS reviewed_match_count,
            COALESCE(history.substitute_appearances, 0)::integer AS substitute_appearances
        FROM review.player_match_observations observation
        JOIN football.players player ON player.id = observation.player_id
        JOIN football.teams team ON team.id = observation.team_id
        LEFT JOIN LATERAL (
            SELECT snapshot.effective_contribution, snapshot.confidence
            FROM feature.match_player_contributions snapshot
            WHERE snapshot.match_id = observation.match_id
              AND snapshot.player_id = observation.player_id
            ORDER BY snapshot.as_of DESC, snapshot.created_at DESC
            LIMIT 1
        ) contribution ON true
        LEFT JOIN feature.player_ability_profiles profile ON profile.player_id = observation.player_id
        LEFT JOIN LATERAL (
            SELECT
                COUNT(DISTINCT match_review.match_id)::integer AS reviewed_match_count,
                COUNT(DISTINCT match_review.match_id) FILTER (
                    WHERE player_review.entry_type = 'substitute'
                )::integer AS substitute_appearances
            FROM review.player_match_reviews player_review
            JOIN review.match_reviews match_review
              ON match_review.id = player_review.match_review_id
            WHERE player_review.player_id = observation.player_id
              AND match_review.match_id <> observation.match_id
              AND match_review.status = 'finalized'
        ) history ON true
        WHERE observation.match_id = $1
        ORDER BY observation.team_id, observation.started DESC, observation.minutes_played DESC
        "#,
    )
    .bind(match_id)
    .fetch_all(&mut **tx)
    .await?;
    rows.iter()
        .map(|row| {
            Ok(ReviewPlayerBaseline {
                observation_id: row.try_get("observation_id")?,
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                position_code: row.try_get("position_code")?,
                role_code: row.try_get("role_code")?,
                started: row.try_get("started")?,
                minutes_played: row.try_get("minutes_played")?,
                performance_score: row.try_get("performance_score")?,
                input_confidence: row.try_get("input_confidence")?,
                metrics: serde_json::from_value(row.try_get("metrics")?)?,
                expected_performance: row.try_get("expected_performance")?,
                expected_confidence: row.try_get("expected_confidence")?,
                current_abilities: row.try_get("current_abilities")?,
                reviewed_match_count: row.try_get("reviewed_match_count")?,
                substitute_appearances: row.try_get("substitute_appearances")?,
            })
        })
        .collect()
}

async fn calculate_recent_starter_overlap(
    tx: &mut Transaction<'_, Postgres>,
    match_id: Uuid,
    team_id: Uuid,
    kickoff_time: DateTime<Utc>,
) -> PersistenceResult<f64> {
    let value: f64 = sqlx::query_scalar(
        r#"
        WITH current_starters AS (
            SELECT player_id
            FROM review.player_match_observations
            WHERE match_id = $1 AND team_id = $2 AND started
        ), previous_lineups AS (
            SELECT DISTINCT ON (lineup.match_id)
                lineup.id, lineup.match_id, fixture.kickoff_time
            FROM football.lineups lineup
            JOIN football.matches fixture ON fixture.id = lineup.match_id
            WHERE lineup.team_id = $2
              AND lineup.lineup_type = 'actual'
              AND lineup.status = 'active'
              AND fixture.id <> $1
              AND fixture.kickoff_time < $3
            ORDER BY lineup.match_id, lineup.captured_at DESC, lineup.id DESC
        ), recent_lineups AS (
            SELECT * FROM previous_lineups
            ORDER BY kickoff_time DESC
            LIMIT 10
        ), overlap_by_lineup AS (
            SELECT
                recent.id,
                COUNT(lineup_player.player_id) FILTER (
                    WHERE lineup_player.player_id IN (SELECT player_id FROM current_starters)
                      AND lineup_player.is_starter
                )::double precision
                / GREATEST((SELECT COUNT(*) FROM current_starters), 1)::double precision AS overlap
            FROM recent_lineups recent
            JOIN football.lineup_players lineup_player ON lineup_player.lineup_id = recent.id
            GROUP BY recent.id
        )
        SELECT COALESCE(AVG(overlap), 0.50)::double precision
        FROM overlap_by_lineup
        "#,
    )
    .bind(match_id)
    .bind(team_id)
    .bind(kickoff_time)
    .fetch_one(&mut **tx)
    .await?;
    Ok(value.clamp(0.0, 1.0))
}

async fn load_prediction_context_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    match_id: Uuid,
    source_run_id: Option<Uuid>,
    home_goals: i16,
    away_goals: i16,
) -> PersistenceResult<Option<football_domain::PredictionReviewContext>> {
    let row = if let Some(run_id) = source_run_id {
        sqlx::query(
            "SELECT id, summary FROM model.runs WHERE id=$1 AND match_id=$2 AND status='succeeded'",
        )
        .bind(run_id)
        .bind(match_id)
        .fetch_optional(&mut **tx)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT id, summary
            FROM model.runs
            WHERE match_id=$1 AND status='succeeded'
            ORDER BY completed_at DESC NULLS LAST, created_at DESC
            LIMIT 1
            "#,
        )
        .bind(match_id)
        .fetch_optional(&mut **tx)
        .await?
    };
    let Some(row) = row else {
        if source_run_id.is_some() {
            return Err(PersistenceError::InvalidState(
                "指定的模型运行不存在、未成功或不属于该场比赛".to_string(),
            ));
        }
        return Ok(None);
    };
    let run_id: Uuid = row.try_get("id")?;
    let scoreline_probability: Option<f64> = sqlx::query_scalar(
        "SELECT probability FROM model.run_scorelines WHERE run_id=$1 AND home_goals=$2 AND away_goals=$3",
    )
    .bind(run_id)
    .bind(home_goals)
    .bind(away_goals)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(Some(football_domain::PredictionReviewContext {
        run_id,
        summary: row.try_get("summary")?,
        actual_scoreline_probability: scoreline_probability,
    }))
}

async fn list_substitutions_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    match_id: Uuid,
) -> PersistenceResult<Vec<SubstitutionRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT substitution.id, substitution.match_id,
               substitution.team_id, team.canonical_name AS team_name,
               substitution.player_out_id, player_out.canonical_name AS player_out_name,
               substitution.player_in_id, player_in.canonical_name AS player_in_name,
               substitution.minute, substitution.period, substitution.reason
        FROM football.substitutions substitution
        JOIN football.teams team ON team.id = substitution.team_id
        LEFT JOIN football.players player_out ON player_out.id = substitution.player_out_id
        LEFT JOIN football.players player_in ON player_in.id = substitution.player_in_id
        WHERE substitution.match_id = $1
        ORDER BY substitution.minute, substitution.id
        "#,
    )
    .bind(match_id)
    .fetch_all(&mut **tx)
    .await?;
    rows.iter().map(substitution_from_row).collect()
}

fn match_review_package_id(metadata: &Value) -> Option<Uuid> {
    metadata
        .get("package_id")
        .and_then(Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
}

fn match_event_key(
    event: &MatchReviewEventDraft,
    match_id: Uuid,
    index: usize,
) -> PersistenceResult<String> {
    if let Some(value) = event
        .event_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(value.to_string());
    }
    let identity = json!({
        "match_id": match_id,
        "sequence_no": event.sequence_no.unwrap_or(index as i32 + 1),
        "event_type": event.event_type.as_str(),
        "team_id": event.team_id,
        "player_id": event.player_id,
        "related_player_id": event.related_player_id,
        "minute": event.minute,
        "stoppage_minute": event.stoppage_minute,
        "period": event.period.trim(),
    });
    let digest = Sha256::digest(serde_json::to_vec(&identity)?);
    Ok(format!("generated:{}", hex::encode(digest)))
}

fn summarize_match_events(events: &[MatchReviewEventRecord]) -> MatchEventSummary {
    let mut summary = MatchEventSummary::default();
    let mut counts = BTreeMap::new();
    for event in events {
        summary.total_count += 1;
        if event.revision_status.is_effective() {
            summary.effective_count += 1;
            *counts.entry(event.event_type.as_str().to_string()).or_insert(0) += 1;
            summary.last_event_minute = Some(
                summary
                    .last_event_minute
                    .map_or(event.minute, |current| current.max(event.minute)),
            );
            if let (Some(home_score), Some(away_score)) = (event.home_score, event.away_score) {
                summary.latest_home_score = Some(home_score);
                summary.latest_away_score = Some(away_score);
            }
        }
        if event.revision_status == MatchEventRevisionStatus::Cancelled {
            summary.cancelled_count += 1;
        }
        if event.verification_status == MatchEventVerificationStatus::Disputed {
            summary.disputed_count += 1;
        }
        if event.verification_status == MatchEventVerificationStatus::Verified {
            summary.verified_count += 1;
        }
    }
    summary.event_type_counts = counts;
    summary
}

fn validate_review_draft(draft: &MatchReviewDraft) -> PersistenceResult<()> {
    if draft.result.match_id != draft.match_id {
        return Err(PersistenceError::InvalidState(
            "正式赛果与复盘比赛 ID 不一致".to_string(),
        ));
    }
    if draft.result.home_goals_90 < 0 || draft.result.away_goals_90 < 0 {
        return Err(PersistenceError::InvalidState(
            "90 分钟进球不能为负数".to_string(),
        ));
    }
    for value in [
        draft.result.home_goals_extra_time,
        draft.result.away_goals_extra_time,
        draft.result.home_penalties,
        draft.result.away_penalties,
    ]
    .into_iter()
    .flatten()
    {
        if value < 0 {
            return Err(PersistenceError::InvalidState(
                "加时或点球进球不能为负数".to_string(),
            ));
        }
    }
    if draft.result.home_goals_extra_time.is_some()
        != draft.result.away_goals_extra_time.is_some()
    {
        return Err(PersistenceError::InvalidState(
            "主客队加时进球必须同时填写或同时留空".to_string(),
        ));
    }
    if draft.result.home_penalties.is_some() != draft.result.away_penalties.is_some() {
        return Err(PersistenceError::InvalidState(
            "主客队点球大战进球必须同时填写或同时留空".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&draft.data_coverage) {
        return Err(PersistenceError::InvalidState(
            "数据覆盖率必须位于 0–1".to_string(),
        ));
    }
    if draft.player_observations.is_empty() {
        return Err(PersistenceError::InvalidState(
            "至少需要一条球员赛后表现".to_string(),
        ));
    }
    let mut player_ids = HashSet::new();
    for observation in &draft.player_observations {
        if !player_ids.insert(observation.player_id) {
            return Err(PersistenceError::InvalidState(
                "同一球员存在重复赛后表现".to_string(),
            ));
        }
        if !(0..=150).contains(&observation.minutes_played) {
            return Err(PersistenceError::InvalidState(
                "球员分钟数必须位于 0–150".to_string(),
            ));
        }
        if observation
            .performance_score
            .is_some_and(|value| !(0.0..=100.0).contains(&value))
        {
            return Err(PersistenceError::InvalidState(
                "球员表现分必须位于 0–100".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&observation.input_confidence) {
            return Err(PersistenceError::InvalidState(
                "球员数据可信度必须位于 0–1".to_string(),
            ));
        }
        validate_performance_metrics(&observation.metrics)?;
        if observation.minutes_played > 0
            && observation.performance_score.is_none()
            && observation.metrics.provider_rating.is_none()
            && !performance_metrics_have_signal(&observation.metrics)
        {
            return Err(PersistenceError::InvalidState(
                "出场球员必须提供表现分、供应商评分或至少一项有效事件数据".to_string(),
            ));
        }
    }
    const VALID_PERIODS: [&str; 5] = [
        "first_half",
        "second_half",
        "extra_time_first",
        "extra_time_second",
        "normal_time",
    ];
    for substitution in &draft.substitutions {
        if !(0..=150).contains(&substitution.minute) {
            return Err(PersistenceError::InvalidState(
                "换人分钟必须位于 0–150".to_string(),
            ));
        }
        if substitution.player_in_id.is_none() && substitution.player_out_id.is_none() {
            return Err(PersistenceError::InvalidState(
                "换人记录至少需要一名球员".to_string(),
            ));
        }
        if substitution.player_in_id == substitution.player_out_id
            && substitution.player_in_id.is_some()
        {
            return Err(PersistenceError::InvalidState(
                "换入与换出球员不能相同".to_string(),
            ));
        }
        if !VALID_PERIODS.contains(&substitution.period.trim()) {
            return Err(PersistenceError::InvalidState(
                "换人比赛阶段无效".to_string(),
            ));
        }
    }
    let mut event_keys = HashSet::new();
    let mut event_sequences = HashSet::new();
    let mut ordered_scores = Vec::new();
    for (index, event) in draft.events.iter().enumerate() {
        let event_key = match_event_key(event, draft.match_id, index)?;
        if !event_keys.insert(event_key.clone()) {
            return Err(PersistenceError::InvalidState(format!(
                "比赛事件 event_key 重复：{event_key}"
            )));
        }
        let sequence_no = event.sequence_no.unwrap_or(index as i32 + 1);
        if sequence_no <= 0 || !event_sequences.insert(sequence_no) {
            return Err(PersistenceError::InvalidState(
                "比赛事件 sequence_no 必须大于 0 且不能重复".to_string(),
            ));
        }
        if !(0..=150).contains(&event.minute) {
            return Err(PersistenceError::InvalidState(
                "比赛事件分钟必须位于 0–150".to_string(),
            ));
        }
        if event.stoppage_minute.is_some_and(|value| !(0..=30).contains(&value)) {
            return Err(PersistenceError::InvalidState(
                "比赛事件补时分钟必须位于 0–30".to_string(),
            ));
        }
        if !VALID_PERIODS.contains(&event.period.trim()) {
            return Err(PersistenceError::InvalidState(
                "比赛事件阶段无效".to_string(),
            ));
        }
        if !event.confidence.is_finite() || !(0.0..=1.0).contains(&event.confidence) {
            return Err(PersistenceError::InvalidState(
                "比赛事件可信度必须位于 0–1".to_string(),
            ));
        }
        if event.event_type.requires_team() && event.team_id.is_none() {
            return Err(PersistenceError::InvalidState(format!(
                "{} 事件必须关联球队",
                event.event_type.as_str()
            )));
        }
        if event.event_type.requires_player() && event.player_id.is_none() {
            return Err(PersistenceError::InvalidState(format!(
                "{} 事件必须关联球员",
                event.event_type.as_str()
            )));
        }
        if matches!(
            event.event_type,
            MatchEventType::Substitution | MatchEventType::GoalkeeperChange
        ) {
            let label = if event.event_type == MatchEventType::Substitution {
                "换人"
            } else {
                "门将更换"
            };
            if event.player_id.is_none() || event.related_player_id.is_none() {
                return Err(PersistenceError::InvalidState(format!(
                    "{label}事件必须同时关联离场与入场球员"
                )));
            }
            if event.player_id == event.related_player_id {
                return Err(PersistenceError::InvalidState(format!(
                    "{label}事件的离场与入场球员不能相同"
                )));
            }
        }
        if event.event_type == MatchEventType::Assist && event.related_player_id.is_none() {
            return Err(PersistenceError::InvalidState(
                "助攻事件必须关联对应进球球员".to_string(),
            ));
        }
        if event.home_score.is_some() != event.away_score.is_some() {
            return Err(PersistenceError::InvalidState(
                "比赛事件后的主客队比分必须同时存在或同时为空".to_string(),
            ));
        }
        if event.home_score.is_some_and(|value| value < 0)
            || event.away_score.is_some_and(|value| value < 0)
        {
            return Err(PersistenceError::InvalidState(
                "比赛事件后的比分不能为负数".to_string(),
            ));
        }
        if event.verification_status == MatchEventVerificationStatus::Verified
            && event.verified_at.is_none()
        {
            return Err(PersistenceError::InvalidState(
                "已核验比赛事件必须记录核验时间".to_string(),
            ));
        }
        if event.revision_status == MatchEventRevisionStatus::Superseded {
            return Err(PersistenceError::InvalidState(
                "superseded 由系统根据新资料包自动维护，不能作为当前导入状态".to_string(),
            ));
        }
        if event.revision_status.is_effective() {
            if let (Some(home_score), Some(away_score)) = (event.home_score, event.away_score) {
                let is_extra_time = matches!(
                    event.period.trim(),
                    "extra_time_first" | "extra_time_second"
                );
                ordered_scores.push((sequence_no, home_score, away_score, is_extra_time));
            }
        }
    }
    ordered_scores.sort_by_key(|item| item.0);
    let mut previous_score = (0i16, 0i16);
    let mut latest_regulation_score = None;
    let mut latest_overall_score = None;
    let mut has_extra_time_score = false;
    for (_, home_score, away_score, is_extra_time) in &ordered_scores {
        if *home_score < previous_score.0 || *away_score < previous_score.1 {
            return Err(PersistenceError::InvalidState(
                "比赛事件后的比分不能相对前序事件倒退；VAR 取消应使用 cancelled/corrected 修订状态".to_string(),
            ));
        }
        previous_score = (*home_score, *away_score);
        latest_overall_score = Some(previous_score);
        if *is_extra_time {
            has_extra_time_score = true;
        } else {
            latest_regulation_score = Some(previous_score);
        }
    }
    if let Some((home_score, away_score)) = latest_regulation_score {
        if home_score != draft.result.home_goals_90 || away_score != draft.result.away_goals_90 {
            return Err(PersistenceError::InvalidState(format!(
                "最后一条 90 分钟事件后比分 {home_score}-{away_score} 与正式 90 分钟赛果 {}-{} 不一致",
                draft.result.home_goals_90, draft.result.away_goals_90
            )));
        }
    }
    if has_extra_time_score {
        let (Some(home_extra), Some(away_extra)) = (
            draft.result.home_goals_extra_time,
            draft.result.away_goals_extra_time,
        ) else {
            return Err(PersistenceError::InvalidState(
                "存在加时阶段事件后比分，但正式赛果未同时填写主客队加时进球"
                    .to_string(),
            ));
        };
        if let Some((home_score, away_score)) = latest_overall_score {
            let expected_home = draft.result.home_goals_90 + home_extra;
            let expected_away = draft.result.away_goals_90 + away_extra;
            if home_score != expected_home || away_score != expected_away {
                return Err(PersistenceError::InvalidState(format!(
                    "最后一条加时事件后比分 {home_score}-{away_score} 与正式合计赛果 {expected_home}-{expected_away} 不一致"
                )));
            }
        }
    }
    Ok(())
}

fn validate_performance_metrics(
    metrics: &football_domain::PlayerPerformanceMetrics,
) -> PersistenceResult<()> {
    let values = [
        metrics.goals,
        metrics.assists,
        metrics.expected_goals,
        metrics.expected_assists,
        metrics.shots,
        metrics.shots_on_target,
        metrics.key_passes,
        metrics.progressive_actions,
        metrics.tackles,
        metrics.interceptions,
        metrics.clearances,
        metrics.blocks,
        metrics.duels_won,
        metrics.duels_total,
        metrics.fouls,
        metrics.yellow_cards,
        metrics.red_cards,
        metrics.errors_leading_to_shot,
    ];
    if values
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(PersistenceError::InvalidState(
            "球员事件数据必须是有限的非负数".to_string(),
        ));
    }
    if metrics.shots_on_target > metrics.shots {
        return Err(PersistenceError::InvalidState(
            "射正次数不能大于射门次数".to_string(),
        ));
    }
    if metrics.duels_won > metrics.duels_total {
        return Err(PersistenceError::InvalidState(
            "对抗成功次数不能大于对抗总数".to_string(),
        ));
    }
    if let Some(rating) = metrics.provider_rating {
        let valid = rating.is_finite()
            && ((0.0..=10.0).contains(&rating) || (0.0..=100.0).contains(&rating));
        if !valid {
            return Err(PersistenceError::InvalidState(
                "供应商评分必须位于 0–10 或 0–100".to_string(),
            ));
        }
    }
    Ok(())
}

fn performance_metrics_have_signal(metrics: &football_domain::PlayerPerformanceMetrics) -> bool {
    [
        metrics.goals,
        metrics.assists,
        metrics.expected_goals,
        metrics.expected_assists,
        metrics.shots,
        metrics.shots_on_target,
        metrics.key_passes,
        metrics.progressive_actions,
        metrics.tackles,
        metrics.interceptions,
        metrics.clearances,
        metrics.blocks,
        metrics.duels_won,
        metrics.duels_total,
        metrics.fouls,
        metrics.yellow_cards,
        metrics.red_cards,
        metrics.errors_leading_to_shot,
    ]
    .iter()
    .any(|value| *value > 0.0)
}

async fn read_ability_candidate(
    pool: &sqlx::PgPool,
    candidate_id: Uuid,
) -> PersistenceResult<AbilityUpdateCandidateRecord> {
    let row = sqlx::query(
        r#"
        SELECT
            candidate.id, candidate.match_review_id, candidate.player_match_review_id,
            candidate.player_id, player.canonical_name AS player_name,
            candidate.dimension_code, dimension.name AS dimension_name,
            candidate.current_value, candidate.proposed_value,
            candidate.confidence, candidate.sample_size, candidate.evidence,
            candidate.calculation_version, candidate.status,
            candidate.created_at, candidate.decided_at,
            candidate.decided_by, candidate.decision_note,
            candidate.accepted_observation_id
        FROM review.ability_update_candidates candidate
        JOIN football.players player ON player.id = candidate.player_id
        JOIN feature.player_ability_dimensions dimension ON dimension.code = candidate.dimension_code
        WHERE candidate.id = $1
        "#,
    )
    .bind(candidate_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("能力更新候选不存在".to_string()))?;
    ability_candidate_from_row(&row)
}

fn reviewable_match_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<ReviewableMatch> {
    let match_record = match_record_from_review_row(row)?;
    let result = if row.try_get::<Option<i16>, _>("home_goals_90")?.is_some() {
        Some(MatchResultRecord {
            match_id: match_record.id,
            home_goals_90: row.try_get("home_goals_90")?,
            away_goals_90: row.try_get("away_goals_90")?,
            home_goals_extra_time: row.try_get("home_goals_extra_time")?,
            away_goals_extra_time: row.try_get("away_goals_extra_time")?,
            home_penalties: row.try_get("home_penalties")?,
            away_penalties: row.try_get("away_penalties")?,
            finalized_at: row.try_get("finalized_at")?,
            metadata: row.try_get("result_metadata")?,
        })
    } else {
        None
    };
    let latest_review = if row.try_get::<Option<Uuid>, _>("review_id")?.is_some() {
        Some(MatchReviewSummary {
            id: row.try_get("review_id")?,
            match_id: match_record.id,
            match_key: match_record.external_key.clone(),
            home_team_name: match_record.home_team_name.clone(),
            away_team_name: match_record.away_team_name.clone(),
            review_version: row.try_get("review_version")?,
            status: row.try_get("review_status")?,
            data_coverage: row.try_get("review_data_coverage")?,
            source_run_id: row.try_get("source_run_id")?,
            calculation_version: row.try_get("calculation_version")?,
            result_snapshot: row.try_get("result_snapshot")?,
            substitutions_snapshot: row.try_get("substitutions_snapshot")?,
            prediction_evaluation: row.try_get("prediction_evaluation")?,
            conclusions: row.try_get("conclusions")?,
            created_at: row.try_get("review_created_at")?,
            finalized_at: row.try_get("review_finalized_at")?,
        })
    } else {
        None
    };
    Ok(ReviewableMatch {
        match_record,
        result,
        latest_review,
        player_observation_count: row.try_get("player_observation_count")?,
        actual_lineup_count: row.try_get("actual_lineup_count")?,
    })
}

fn match_record_from_review_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<MatchRecord> {
    Ok(MatchRecord {
        id: row.try_get("id")?,
        external_key: row.try_get("external_key")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        season_id: row.try_get("season_id")?,
        stage_id: row.try_get("stage_id")?,
        round_id: row.try_get("round_id")?,
        home_team_id: row.try_get("home_team_id")?,
        home_team_name: row.try_get("home_team_name")?,
        away_team_id: row.try_get("away_team_id")?,
        away_team_name: row.try_get("away_team_name")?,
        kickoff_time: row.try_get("kickoff_time")?,
        status: match row.try_get::<String, _>("status")?.as_str() {
            "scheduled" => football_domain::MatchStatus::Scheduled,
            "live" => football_domain::MatchStatus::Live,
            "finished" => football_domain::MatchStatus::Finished,
            "postponed" => football_domain::MatchStatus::Postponed,
            "cancelled" => football_domain::MatchStatus::Cancelled,
            other => {
                return Err(PersistenceError::InvalidState(format!(
                    "未知比赛状态：{other}"
                )))
            }
        },
        venue: row.try_get("venue")?,
    })
}

fn match_review_summary_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<MatchReviewSummary> {
    Ok(MatchReviewSummary {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        match_key: row.try_get("match_key")?,
        home_team_name: row.try_get("home_team_name")?,
        away_team_name: row.try_get("away_team_name")?,
        review_version: row.try_get("review_version")?,
        status: row.try_get("status")?,
        data_coverage: row.try_get("data_coverage")?,
        source_run_id: row.try_get("source_run_id")?,
        calculation_version: row.try_get("calculation_version")?,
        result_snapshot: row.try_get("result_snapshot")?,
        substitutions_snapshot: row.try_get("substitutions_snapshot")?,
        prediction_evaluation: row.try_get("prediction_evaluation")?,
        conclusions: row.try_get("conclusions")?,
        created_at: row.try_get("created_at")?,
        finalized_at: row.try_get("finalized_at")?,
    })
}

fn match_result_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<MatchResultRecord> {
    Ok(MatchResultRecord {
        match_id: row.try_get("match_id")?,
        home_goals_90: row.try_get("home_goals_90")?,
        away_goals_90: row.try_get("away_goals_90")?,
        home_goals_extra_time: row.try_get("home_goals_extra_time")?,
        away_goals_extra_time: row.try_get("away_goals_extra_time")?,
        home_penalties: row.try_get("home_penalties")?,
        away_penalties: row.try_get("away_penalties")?,
        finalized_at: row.try_get("finalized_at")?,
        metadata: row.try_get("metadata")?,
    })
}

fn match_event_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<MatchReviewEventRecord> {
    let event_type = row
        .try_get::<String, _>("event_type")?
        .parse::<MatchEventType>()
        .map_err(PersistenceError::InvalidState)?;
    let verification_status = row
        .try_get::<String, _>("verification_status")?
        .parse::<MatchEventVerificationStatus>()
        .map_err(PersistenceError::InvalidState)?;
    let revision_status = row
        .try_get::<String, _>("revision_status")?
        .parse::<MatchEventRevisionStatus>()
        .map_err(PersistenceError::InvalidState)?;
    Ok(MatchReviewEventRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        event_key: row.try_get("event_key")?,
        sequence_no: row.try_get("sequence_no")?,
        event_type,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        related_player_id: row.try_get("related_player_id")?,
        related_player_name: row.try_get("related_player_name")?,
        minute: row.try_get("minute")?,
        stoppage_minute: row.try_get("stoppage_minute")?,
        period: row.try_get("period")?,
        home_score: row.try_get("home_score")?,
        away_score: row.try_get("away_score")?,
        verification_status,
        revision_status,
        verified_at: row.try_get("verified_at")?,
        source_document_id: row.try_get("source_document_id")?,
        source_package_id: row.try_get("source_package_id")?,
        revision_of_event_id: row.try_get("revision_of_event_id")?,
        description: row.try_get("description")?,
        source_urls: row.try_get("source_urls")?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        recorded_at: row.try_get("recorded_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn substitution_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<SubstitutionRecord> {
    Ok(SubstitutionRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        player_out_id: row.try_get("player_out_id")?,
        player_out_name: row.try_get("player_out_name")?,
        player_in_id: row.try_get("player_in_id")?,
        player_in_name: row.try_get("player_in_name")?,
        minute: row.try_get("minute")?,
        period: row.try_get("period")?,
        reason: row.try_get("reason")?,
    })
}

fn player_match_review_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PlayerMatchReviewRecord> {
    Ok(PlayerMatchReviewRecord {
        id: row.try_get("id")?,
        match_review_id: row.try_get("match_review_id")?,
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        role_code: row.try_get("role_code")?,
        started: row.try_get("started")?,
        entry_type: row.try_get("entry_type")?,
        minutes_played: row.try_get("minutes_played")?,
        expected_performance: row.try_get("expected_performance")?,
        actual_performance: row.try_get("actual_performance")?,
        realization_ratio: row.try_get("realization_ratio")?,
        confidence: row.try_get("confidence")?,
        contribution_weight: row.try_get("contribution_weight")?,
        ability_candidate_count: row.try_get("ability_candidate_count")?,
        metrics: row.try_get("metrics")?,
    })
}

fn team_match_review_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<TeamMatchReviewRecord> {
    Ok(TeamMatchReviewRecord {
        id: row.try_get("id")?,
        match_review_id: row.try_get("match_review_id")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        chemistry_score: row.try_get("chemistry_score")?,
        lineup_continuity: row.try_get("lineup_continuity")?,
        performance_cohesion: row.try_get("performance_cohesion")?,
        bench_strength: row.try_get("bench_strength")?,
        bench_dropoff: row.try_get("bench_dropoff")?,
        substitution_impact: row.try_get("substitution_impact")?,
        substitute_count: row.try_get("substitute_count")?,
        realization_score: row.try_get("realization_score")?,
        confidence: row.try_get("confidence")?,
        metrics: row.try_get("metrics")?,
    })
}

fn ability_candidate_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<AbilityUpdateCandidateRecord> {
    let status: String = row.try_get("status")?;
    Ok(AbilityUpdateCandidateRecord {
        id: row.try_get("id")?,
        match_review_id: row.try_get("match_review_id")?,
        player_match_review_id: row.try_get("player_match_review_id")?,
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        dimension_code: row.try_get("dimension_code")?,
        dimension_name: row.try_get("dimension_name")?,
        current_value: row.try_get("current_value")?,
        proposed_value: row.try_get("proposed_value")?,
        confidence: row.try_get("confidence")?,
        sample_size: row.try_get("sample_size")?,
        evidence: row.try_get("evidence")?,
        calculation_version: row.try_get("calculation_version")?,
        status: match status.as_str() {
            "pending" => AbilityCandidateStatus::Pending,
            "accepted" => AbilityCandidateStatus::Accepted,
            "rejected" => AbilityCandidateStatus::Rejected,
            "superseded" => AbilityCandidateStatus::Superseded,
            other => {
                return Err(PersistenceError::InvalidState(format!(
                    "未知候选状态：{other}"
                )))
            }
        },
        created_at: row.try_get("created_at")?,
        decided_at: row.try_get("decided_at")?,
        decided_by: row.try_get("decided_by")?,
        decision_note: row.try_get("decision_note")?,
        accepted_observation_id: row.try_get("accepted_observation_id")?,
    })
}
