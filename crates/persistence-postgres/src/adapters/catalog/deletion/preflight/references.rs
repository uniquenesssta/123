use crate::PersistenceResult;
use football_domain::EntityReferenceCount;
use uuid::Uuid;

pub(crate) async fn team_reference_counts(
    pool: &sqlx::PgPool,
    id: Uuid,
) -> PersistenceResult<Vec<EntityReferenceCount>> {
    count_relations(
        pool,
        id,
        &[
            ("matches", "SELECT count(*)::bigint FROM football.matches WHERE home_team_id=$1 OR away_team_id=$1"),
            ("lineups", "SELECT count(*)::bigint FROM football.lineups WHERE team_id=$1"),
            ("team_lineup_presets", "SELECT count(*)::bigint FROM football.team_lineup_presets WHERE team_id=$1"),
            ("player_team_periods", "SELECT count(*)::bigint FROM football.player_team_periods WHERE team_id=$1"),
            ("team_coach_periods", "SELECT count(*)::bigint FROM football.team_coach_periods WHERE team_id=$1"),
            ("team_season_memberships", "SELECT count(*)::bigint FROM football.team_season_memberships WHERE team_id=$1"),
            ("player_availability", "SELECT count(*)::bigint FROM football.player_availability WHERE team_id=$1"),
            ("formation_usage", "SELECT count(*)::bigint FROM feature.formation_usage_observations WHERE team_id=$1"),
            ("team_tactical_observations", "SELECT count(*)::bigint FROM feature.team_tactical_observations WHERE team_id=$1"),
            ("team_ability_observations", "SELECT count(*)::bigint FROM feature.team_ability_observations WHERE team_id=$1"),
            ("substitutions", "SELECT count(*)::bigint FROM football.substitutions WHERE team_id=$1"),
            ("match_events", "SELECT count(*)::bigint FROM review.match_events WHERE team_id=$1"),
            ("dynamic_tag_opponents", "SELECT count(*)::bigint FROM feature.player_dynamic_tags WHERE opponent_team_id=$1"),
            ("team_match_reviews", "SELECT count(*)::bigint FROM review.team_match_reviews WHERE team_id=$1"),
            ("player_match_reviews", "SELECT count(*)::bigint FROM review.player_match_reviews WHERE team_id=$1"),
            ("player_match_observations", "SELECT count(*)::bigint FROM review.player_match_observations WHERE team_id=$1"),
        ],
    ).await
}

pub(crate) async fn player_reference_counts(
    pool: &sqlx::PgPool,
    id: Uuid,
) -> PersistenceResult<Vec<EntityReferenceCount>> {
    count_relations(
        pool,
        id,
        &[
            ("lineup_players", "SELECT count(*)::bigint FROM football.lineup_players WHERE player_id=$1"),
            ("team_lineup_preset_members", "SELECT count(*)::bigint FROM football.team_lineup_preset_members WHERE player_id=$1"),
            ("substitutions", "SELECT count(*)::bigint FROM football.substitutions WHERE player_out_id=$1 OR player_in_id=$1"),
            ("match_events", "SELECT count(*)::bigint FROM review.match_events WHERE player_id=$1 OR related_player_id=$1"),
            ("player_team_periods", "SELECT count(*)::bigint FROM football.player_team_periods WHERE player_id=$1"),
            ("player_availability", "SELECT count(*)::bigint FROM football.player_availability WHERE player_id=$1"),
            ("ability_observations", "SELECT count(*)::bigint FROM feature.player_ability_observations WHERE player_id=$1"),
            ("ability_snapshots", "SELECT count(*)::bigint FROM feature.player_ability_snapshots WHERE player_id=$1"),
            ("dynamic_tags", "SELECT count(*)::bigint FROM feature.player_dynamic_tags WHERE player_id=$1"),
            ("match_contributions", "SELECT count(*)::bigint FROM feature.match_player_contributions WHERE player_id=$1"),
            ("player_match_reviews", "SELECT count(*)::bigint FROM review.player_match_reviews WHERE player_id=$1"),
            ("player_match_observations", "SELECT count(*)::bigint FROM review.player_match_observations WHERE player_id=$1"),
            ("ability_candidates", "SELECT count(*)::bigint FROM review.ability_update_candidates WHERE player_id=$1"),
        ],
    ).await
}

pub(crate) async fn coach_reference_counts(
    pool: &sqlx::PgPool,
    id: Uuid,
) -> PersistenceResult<Vec<EntityReferenceCount>> {
    count_relations(
        pool,
        id,
        &[
            (
                "team_coach_periods",
                "SELECT count(*)::bigint FROM football.team_coach_periods WHERE coach_id=$1",
            ),
            (
                "team_lineup_presets",
                "SELECT count(*)::bigint FROM football.team_lineup_presets WHERE coach_id=$1",
            ),
        ],
    )
    .await
}

async fn count_relations(
    pool: &sqlx::PgPool,
    id: Uuid,
    relations: &[(&str, &str)],
) -> PersistenceResult<Vec<EntityReferenceCount>> {
    let mut output = Vec::new();
    for (relation, query) in relations {
        let count: i64 = sqlx::query_scalar(query).bind(id).fetch_one(pool).await?;
        if count > 0 {
            output.push(EntityReferenceCount {
                relation: (*relation).to_string(),
                count,
            });
        }
    }
    Ok(output)
}
