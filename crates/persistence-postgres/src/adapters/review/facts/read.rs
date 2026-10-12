use super::mapping::{match_event_from_row, match_result_from_row, substitution_from_row};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{
    MatchEventRevisionStatus, MatchEventSummary, MatchEventVerificationStatus, MatchResultRecord,
    MatchReviewEventRecord, SubstitutionRecord,
};
use sqlx::{Postgres, Transaction};
use std::collections::BTreeMap;
use uuid::Uuid;

impl PostgresStore {
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

pub(super) async fn list_substitutions_in_tx(
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

pub(crate) fn summarize_match_events(events: &[MatchReviewEventRecord]) -> MatchEventSummary {
    let mut summary = MatchEventSummary::default();
    let mut counts = BTreeMap::new();
    for event in events {
        summary.total_count += 1;
        if event.revision_status.is_effective() {
            summary.effective_count += 1;
            *counts
                .entry(event.event_type.as_str().to_string())
                .or_insert(0) += 1;
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
