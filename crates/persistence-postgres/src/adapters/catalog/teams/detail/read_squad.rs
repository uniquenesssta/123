use super::{squad_mapper::map_team_squad_row, squad_row::TeamSquadRow};
use crate::PersistenceResult;
use football_domain::TeamSquadPlayer;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_squad(
    pool: &PgPool,
    team_id: Uuid,
) -> PersistenceResult<Vec<TeamSquadPlayer>> {
    let rows = sqlx::query_as::<_, TeamSquadRow>(
        r#"
        SELECT player.id AS player_id, player.canonical_name AS player_name,
               localized_name.name AS localized_name,
               position.position_code, position.default_role_code AS role_code,
               period.squad_number, period.registration_status,
               availability.status AS availability_status,
               ability.average_value AS ability_average
        FROM football.player_team_periods period
        JOIN football.players player ON player.id = period.player_id
        LEFT JOIN LATERAL (
            SELECT alias.name
            FROM football.player_names alias
            WHERE alias.player_id = player.id
              AND (
                lower(COALESCE(alias.language_code, '')) IN ('zh-cn', 'zh-hans', 'zh')
                OR alias.name ~ '[一-龥]'
              )
            ORDER BY
              CASE lower(COALESCE(alias.language_code, ''))
                WHEN 'zh-cn' THEN 0 WHEN 'zh-hans' THEN 1 WHEN 'zh' THEN 2 ELSE 3
              END,
              alias.is_primary DESC,
              alias.valid_from DESC NULLS LAST,
              alias.id DESC
            LIMIT 1
        ) localized_name ON true
        LEFT JOIN LATERAL (
            SELECT item.position_code, item.default_role_code
            FROM football.player_positions item
            WHERE item.player_id = player.id
              AND (item.valid_from IS NULL OR item.valid_from <= current_date)
              AND (item.valid_to IS NULL OR item.valid_to >= current_date)
            ORDER BY item.is_primary DESC, item.proficiency DESC, item.position_code
            LIMIT 1
        ) position ON true
        LEFT JOIN LATERAL (
            SELECT item.status
            FROM football.player_availability item
            WHERE item.player_id = player.id
              AND item.valid_from <= now()
              AND (item.valid_to IS NULL OR item.valid_to >= now())
            ORDER BY item.valid_from DESC, item.created_at DESC
            LIMIT 1
        ) availability ON true
        LEFT JOIN feature.player_ability_profiles ability ON ability.player_id = player.id
        WHERE period.team_id = $1
          AND period.valid_from <= current_date
          AND (period.valid_to IS NULL OR period.valid_to >= current_date)
          AND period.registration_status IN ('registered','loan','trial')
          AND player.status = 'active'
        ORDER BY position.position_code NULLS LAST, period.squad_number NULLS LAST,
                 player.normalized_name, player.id
        "#,
    )
    .bind(team_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(map_team_squad_row).collect()
}
