mod read;

use crate::team_features::{
    project_team_features, HistoricalFeatureSample, HistoricalMatch, TeamPreMatchFeatures,
};
use crate::{PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::MatchRecord;
use uuid::Uuid;

const MAX_HISTORY_MATCHES: usize = 12;
const MIN_SCOPED_MATCHES: usize = 4;

impl PostgresStore {
    pub(crate) async fn calculate_team_pre_match_features(
        &self,
        fixture: &MatchRecord,
        team_id: Uuid,
        is_home: bool,
        data_cutoff_time: DateTime<Utc>,
    ) -> PersistenceResult<TeamPreMatchFeatures> {
        let all_matches =
            read::team_history(&self.pool, fixture, team_id, is_home, data_cutoff_time).await?;
        if all_matches.is_empty() {
            return Ok(TeamPreMatchFeatures::neutral(
                team_id,
                "截止时间之前没有可用的正式赛果",
            ));
        }
        let (selected, scope, scope_factor) = select_history_scope(&all_matches);
        let selected = selected
            .into_iter()
            .take(MAX_HISTORY_MATCHES)
            .collect::<Vec<_>>();
        let baseline = read::goal_baseline(
            &self.pool,
            fixture.competition_id,
            fixture.kickoff_time,
            data_cutoff_time,
        )
        .await?;
        Ok(project_team_features(
            fixture,
            team_id,
            is_home,
            data_cutoff_time,
            HistoricalFeatureSample {
                matches: selected,
                scope,
                scope_factor,
                baseline,
            },
        ))
    }
}

fn select_history_scope(matches: &[HistoricalMatch]) -> (Vec<&HistoricalMatch>, &'static str, f64) {
    let same_season = matches
        .iter()
        .filter(|item| item.same_season)
        .collect::<Vec<_>>();
    if same_season.len() >= MIN_SCOPED_MATCHES {
        return (same_season, "same_season", 1.0);
    }
    let same_competition = matches
        .iter()
        .filter(|item| item.same_competition)
        .collect::<Vec<_>>();
    if same_competition.len() >= MIN_SCOPED_MATCHES {
        return (same_competition, "same_competition", 0.9);
    }
    (matches.iter().collect(), "cross_competition_fallback", 0.72)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history(same_competition: bool, same_season: bool, index: i64) -> HistoricalMatch {
        HistoricalMatch {
            kickoff_time: DateTime::parse_from_rfc3339("2026-09-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
                - chrono::Duration::days(index),
            goals_for: 2.0,
            goals_against: 1.0,
            points: 3.0,
            played_at_target_venue: true,
            same_competition,
            same_season,
        }
    }

    #[test]
    fn history_scope_requires_four_samples_and_prefers_same_season() {
        let mut rows = (0..3)
            .map(|index| history(true, true, index))
            .collect::<Vec<_>>();
        let (selected, scope, factor) = select_history_scope(&rows);
        assert_eq!(selected.len(), 3);
        assert_eq!(scope, "cross_competition_fallback");
        assert_eq!(factor, 0.72);
        rows.push(history(true, false, 3));
        let (selected, scope, factor) = select_history_scope(&rows);
        assert_eq!(selected.len(), 4);
        assert_eq!(scope, "same_competition");
        assert_eq!(factor, 0.9);
        rows.push(history(true, true, 4));
        rows.push(history(false, false, 5));
        let (selected, scope, factor) = select_history_scope(&rows);
        assert_eq!(selected.len(), 4);
        assert_eq!(scope, "same_season");
        assert_eq!(factor, 1.0);
        assert!(selected.iter().all(|row| row.same_season));
        assert!(std::ptr::eq(selected[0], &rows[0]));
        assert!(std::ptr::eq(selected[3], &rows[4]));
    }

    #[test]
    fn history_scope_filters_before_twelve_match_limit_and_keeps_order() {
        let mut rows = (0..12)
            .map(|index| history(false, false, index))
            .collect::<Vec<_>>();
        rows.extend((12..28).map(|index| history(true, true, index)));
        let (selected, scope, _) = select_history_scope(&rows);
        let selected = selected
            .into_iter()
            .take(MAX_HISTORY_MATCHES)
            .collect::<Vec<_>>();
        assert_eq!(scope, "same_season");
        assert_eq!(selected.len(), 12);
        for (offset, row) in selected.iter().enumerate() {
            assert!(std::ptr::eq(*row, &rows[12 + offset]));
        }
    }
}
