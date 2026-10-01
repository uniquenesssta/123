use super::super::monthly_gaps::monthly_gap_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::MonthlyDataGapRow;

impl PostgresStore {
    pub async fn team_monthly_data_gaps(&self) -> PersistenceResult<Vec<MonthlyDataGapRow>> {
        let rows = sqlx::query(
            r#"
            SELECT 'team'::text AS entity_type, team.id AS entity_id,
                   team.canonical_name AS entity_name,
                   gap.missing_field, profile.updated_at AS last_observed_at,
                   CASE WHEN profile.updated_at IS NULL THEN NULL
                        ELSE GREATEST(0, EXTRACT(day FROM now() - profile.updated_at)::bigint) END AS stale_days,
                   gap.priority, gap.recommended_action
            FROM football.teams team
            LEFT JOIN football.team_profiles profile ON profile.team_id = team.id
            CROSS JOIN LATERAL (
                VALUES
                    ('profile', CASE WHEN profile.team_id IS NULL THEN 'high' ELSE NULL END, '补全球队基础资料'),
                    ('country_code', CASE WHEN team.country_code IS NULL THEN 'high' ELSE NULL END, '填写国家或地区代码'),
                    ('current_coach', CASE WHEN NOT EXISTS (
                        SELECT 1 FROM football.team_coach_periods period
                        WHERE period.team_id=team.id AND period.valid_from<=current_date
                          AND (period.valid_to IS NULL OR period.valid_to>=current_date)
                    ) THEN 'medium' ELSE NULL END, '维护当前教练任期'),
                    ('formation_usage', CASE WHEN NOT EXISTS (
                        SELECT 1 FROM feature.formation_usage_observations usage
                        WHERE usage.team_id=team.id AND usage.observed_at>=now()-interval '90 days'
                    ) THEN 'medium' ELSE NULL END, '更新最近阵型使用观察')
            ) AS gap(missing_field, priority, recommended_action)
            WHERE gap.priority IS NOT NULL
            ORDER BY CASE gap.priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 ELSE 2 END,
                     team.canonical_name, gap.missing_field
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(monthly_gap_from_row).collect()
    }
}
