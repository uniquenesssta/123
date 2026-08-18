use crate::PersistenceResult;
use chrono::{DateTime, NaiveDate, Utc};
use football_domain::{FormationUsageDistributionRecord, FormationUsageEntryRecord};
use sqlx::Row;
use uuid::Uuid;

pub(crate) fn group_distribution_rows(
    rows: &[sqlx::postgres::PgRow],
) -> PersistenceResult<Vec<FormationUsageDistributionRecord>> {
    let mut result: Vec<FormationUsageDistributionRecord> = Vec::new();
    for row in rows {
        let scope_type: String = row.try_get("scope_type")?;
        let team_id: Option<Uuid> = row.try_get("team_id")?;
        let coach_id: Option<Uuid> = row.try_get("coach_id")?;
        let competition_id: Option<Uuid> = row.try_get("competition_id")?;
        let window_start: NaiveDate = row.try_get("window_start")?;
        let window_end: NaiveDate = row.try_get("window_end")?;
        let observed_at: DateTime<Utc> = row.try_get("observed_at")?;
        let index = result.iter().position(|item| {
            item.scope_type == scope_type
                && item.team_id == team_id
                && item.coach_id == coach_id
                && item.competition_id == competition_id
                && item.window_start == window_start
                && item.window_end == window_end
                && item.observed_at == observed_at
        });
        let entry = FormationUsageEntryRecord {
            id: row.try_get("id")?,
            formation_id: row.try_get("formation_id")?,
            formation_code: row.try_get("formation_code")?,
            formation_name: row.try_get("formation_name")?,
            usage_count: row.try_get("usage_count")?,
            raw_probability: row.try_get("raw_probability")?,
            smoothed_probability: row.try_get("smoothed_probability")?,
        };
        if let Some(index) = index {
            result[index].entries.push(entry);
        } else {
            result.push(FormationUsageDistributionRecord {
                scope_type,
                team_id,
                team_name: row.try_get("team_name")?,
                coach_id,
                coach_name: row.try_get("coach_name")?,
                competition_id,
                competition_name: row.try_get("competition_name")?,
                window_preset: row.try_get("window_preset")?,
                window_start,
                window_end,
                observed_matches: row.try_get("observed_matches")?,
                confidence: row.try_get("confidence")?,
                alpha: row.try_get("smoothing_alpha")?,
                observed_at,
                entries: vec![entry],
            });
        }
    }
    Ok(result)
}
