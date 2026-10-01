use super::super::monthly_gaps::monthly_gap_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::MonthlyDataGapRow;

impl PostgresStore {
    pub async fn player_monthly_data_gaps(&self) -> PersistenceResult<Vec<MonthlyDataGapRow>> {
        let rows = sqlx::query(
            r#"
            SELECT 'player'::text AS entity_type, player.id AS entity_id,
                   player.canonical_name AS entity_name, gap.missing_field,
                   player.updated_at AS last_observed_at,
                   GREATEST(0, EXTRACT(day FROM now() - player.updated_at)::bigint) AS stale_days,
                   gap.priority, gap.recommended_action
            FROM football.players player
            CROSS JOIN LATERAL (
                VALUES
                    ('birth_date', CASE WHEN player.date_of_birth IS NULL THEN 'high' ELSE NULL END, '填写出生日期以避免同名误匹配'),
                    ('nationality_code', CASE WHEN player.nationality_code IS NULL THEN 'medium' ELSE NULL END, '填写国籍代码'),
                    ('position', CASE WHEN NOT EXISTS (
                        SELECT 1 FROM football.player_positions position WHERE position.player_id=player.id
                    ) THEN 'high' ELSE NULL END, '维护主要位置与熟练度'),
                    ('team_period', CASE WHEN NOT EXISTS (
                        SELECT 1 FROM football.player_team_periods period
                        WHERE period.player_id=player.id AND period.valid_from<=current_date
                          AND (period.valid_to IS NULL OR period.valid_to>=current_date)
                    ) THEN 'medium' ELSE NULL END, '维护当前球队履历'),
                    ('ability_observation', CASE WHEN NOT EXISTS (
                        SELECT 1 FROM feature.player_ability_observations observation
                        WHERE observation.player_id=player.id AND observation.observed_at>=now()-interval '120 days'
                    ) THEN 'medium' ELSE NULL END, '补充近期能力观察')
            ) AS gap(missing_field, priority, recommended_action)
            WHERE gap.priority IS NOT NULL
            ORDER BY CASE gap.priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 ELSE 2 END,
                     player.canonical_name, gap.missing_field
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(monthly_gap_from_row).collect()
    }
}
