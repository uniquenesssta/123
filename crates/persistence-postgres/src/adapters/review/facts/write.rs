use super::{
    identity::{match_event_key, match_review_package_id},
    read::list_substitutions_in_tx,
};
use crate::{review::match_record_from_review_row, PersistenceError, PersistenceResult};
use football_domain::{
    MatchEventType, MatchRecord, MatchResultRecord, MatchReviewDraft, SubstitutionRecord,
};
use sqlx::{Postgres, Row, Transaction};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub(crate) async fn write_facts_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    draft: &MatchReviewDraft,
) -> PersistenceResult<(MatchRecord, MatchResultRecord, Vec<SubstitutionRecord>)> {
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
        if event
            .team_id
            .is_some_and(|team_id| !valid_teams.contains(&team_id))
        {
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
        let rows =
            sqlx::query("SELECT id, match_id FROM review.match_events WHERE id = ANY($1::uuid[])")
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
    Ok((match_record, result, substitutions))
}
