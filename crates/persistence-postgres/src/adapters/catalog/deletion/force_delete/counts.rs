use crate::PersistenceResult;
use football_domain::EntityReferenceCount;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

pub(super) async fn force_delete_counts(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
) -> PersistenceResult<Vec<EntityReferenceCount>> {
    let rows = sqlx::query(
        r#"
        SELECT relation, count FROM (
            SELECT 'teams'::text relation, 1::bigint count
            UNION ALL SELECT 'matches', count(*)::bigint FROM purge_matches
            UNION ALL SELECT 'players', count(*)::bigint FROM purge_players
            UNION ALL SELECT 'coaches', count(*)::bigint FROM purge_coaches
            UNION ALL SELECT 'feature_snapshots', count(*)::bigint FROM purge_snapshots
            UNION ALL SELECT 'model_runs', count(*)::bigint FROM purge_model_runs
            UNION ALL SELECT 'research_runs', count(*)::bigint FROM purge_research_runs
            UNION ALL SELECT 'p4_freeze_tasks', count(*)::bigint FROM purge_tasks
            UNION ALL SELECT 'postmatch_settlements', count(*)::bigint FROM purge_settlements
            UNION ALL SELECT 'match_reviews', count(*)::bigint FROM purge_match_reviews
            UNION ALL SELECT 'ability_update_candidates', count(*)::bigint FROM purge_candidates
            UNION ALL SELECT 'import_batches', count(*)::bigint FROM purge_import_batches
            UNION ALL SELECT 'ai_workspace_sessions', count(*)::bigint FROM purge_ai_sessions
            UNION ALL SELECT 'match_events', count(*)::bigint
                FROM review.match_events
                WHERE match_id IN (SELECT id FROM purge_matches)
                   OR team_id=$1
                   OR player_id IN (SELECT id FROM purge_players)
                   OR related_player_id IN (SELECT id FROM purge_players)
            UNION ALL SELECT 'team_lineup_presets', count(*)::bigint
                FROM football.team_lineup_presets
                WHERE team_id=$1
            UNION ALL SELECT 'team_lineup_preset_members', count(*)::bigint
                FROM football.team_lineup_preset_members
                WHERE preset_id IN (SELECT id FROM football.team_lineup_presets WHERE team_id=$1)
                   OR player_id IN (SELECT id FROM purge_players)
            UNION ALL SELECT 'player_team_periods', count(*)::bigint
                FROM football.player_team_periods
                WHERE team_id=$1 OR player_id IN (SELECT id FROM purge_players)
            UNION ALL SELECT 'player_ability_observations', count(*)::bigint
                FROM feature.player_ability_observations
                WHERE player_id IN (SELECT id FROM purge_players)
            UNION ALL SELECT 'player_dynamic_tags', count(*)::bigint
                FROM feature.player_dynamic_tags
                WHERE player_id IN (SELECT id FROM purge_players) OR opponent_team_id=$1
            UNION ALL SELECT 'formation_usage_observations', count(*)::bigint
                FROM feature.formation_usage_observations
                WHERE team_id=$1 OR coach_id IN (SELECT id FROM purge_coaches)
            UNION ALL SELECT 'team_tactical_observations', count(*)::bigint
                FROM feature.team_tactical_observations
                WHERE team_id=$1 OR coach_id IN (SELECT id FROM purge_coaches)
            UNION ALL SELECT 'team_ability_observations', count(*)::bigint
                FROM feature.team_ability_observations WHERE team_id=$1
        ) summary
        WHERE count > 0
        ORDER BY relation
        "#,
    )
    .bind(team_id)
    .fetch_all(&mut **tx)
    .await?;

    rows.iter()
        .map(|row| {
            Ok(EntityReferenceCount {
                relation: row.try_get("relation")?,
                count: row.try_get("count")?,
            })
        })
        .collect()
}
