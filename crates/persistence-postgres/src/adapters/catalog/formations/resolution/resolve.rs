use crate::{PersistenceResult, PostgresStore};
use chrono::Utc;
use football_domain::{
    FormationDistributionQuery, FormationUsageEntryRecord, ResolvedFormationDistribution,
};
use uuid::Uuid;

impl PostgresStore {
    pub async fn resolve_formation_distribution(
        &self,
        query: &FormationDistributionQuery,
    ) -> PersistenceResult<ResolvedFormationDistribution> {
        let as_of = query.as_of.unwrap_or_else(Utc::now);
        let mut competition_id = query.competition_id;

        if let Some(match_id) = query.match_id {
            if let Some(observation) = self
                .read_match_formation_observation(match_id, query.team_id, as_of)
                .await?
            {
                let (source_level, source_label) = if observation.lineup_type == "actual" {
                    ("actual_lineup", "当前比赛实际阵型")
                } else {
                    ("confirmed_lineup", "当前比赛确认阵型")
                };
                competition_id = competition_id.or(observation.competition_id);
                return Ok(ResolvedFormationDistribution {
                    source_level: source_level.to_string(),
                    source_label: source_label.to_string(),
                    team_id: query.team_id,
                    coach_id: query.coach_id,
                    competition_id,
                    window_start: None,
                    window_end: None,
                    observed_matches: 1,
                    confidence: observation.quality_score.unwrap_or(1.0),
                    entries: vec![FormationUsageEntryRecord {
                        id: Uuid::nil(),
                        formation_id: observation.formation_id,
                        formation_code: observation.formation_code,
                        formation_name: observation.formation_name,
                        usage_count: 1,
                        raw_probability: 1.0,
                        smoothed_probability: 1.0,
                    }],
                });
            }
            if competition_id.is_none() {
                competition_id = self.read_match_competition_id(match_id).await?;
            }
        }

        let coach_id = match query.coach_id {
            Some(coach_id) => Some(coach_id),
            None => {
                self.read_current_head_coach_id(query.team_id, as_of.date_naive())
                    .await?
            }
        };

        let candidates = [
            (
                "team_coach",
                Some(query.team_id),
                coach_id,
                None,
                "球队 + 教练",
            ),
            ("team", Some(query.team_id), None, None, "球队"),
            ("coach", None, coach_id, None, "教练"),
            (
                "competition_default",
                None,
                None,
                competition_id,
                "赛事默认",
            ),
            ("system_default", None, None, None, "系统默认"),
        ];
        for (scope, team, coach, competition, label) in candidates {
            if (scope == "team_coach" || scope == "coach") && coach.is_none() {
                continue;
            }
            if scope == "competition_default" && competition.is_none() {
                continue;
            }
            if let Some(distribution) = self
                .read_latest_distribution(scope, team, coach, competition, as_of)
                .await?
            {
                return Ok(ResolvedFormationDistribution {
                    source_level: scope.to_string(),
                    source_label: label.to_string(),
                    team_id: query.team_id,
                    coach_id,
                    competition_id,
                    window_start: Some(distribution.window_start),
                    window_end: Some(distribution.window_end),
                    observed_matches: distribution.observed_matches,
                    confidence: distribution.confidence,
                    entries: distribution.entries,
                });
            }
        }

        Ok(ResolvedFormationDistribution {
            source_level: "unknown".to_string(),
            source_label: "无可用观察，回退未知".to_string(),
            team_id: query.team_id,
            coach_id,
            competition_id,
            window_start: None,
            window_end: None,
            observed_matches: 0,
            confidence: 0.0,
            entries: vec![self.read_unknown_formation_entry().await?],
        })
    }
}
