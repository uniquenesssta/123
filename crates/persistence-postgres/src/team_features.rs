use crate::{PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::MatchRecord;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

const MAX_HISTORY_MATCHES: usize = 12;
const HISTORY_QUERY_LIMIT: i64 = 36;
const MIN_SCOPED_MATCHES: usize = 4;
const DEFAULT_GOAL_BASELINE: f64 = 1.15;

#[derive(Debug, Clone)]
pub(crate) struct TeamPreMatchFeatures {
    pub attack_score: f64,
    pub defence_score: f64,
    pub rating_confidence: f64,
    pub venue_score: f64,
    pub venue_confidence: f64,
    pub history: Value,
    pub evidence: Vec<Value>,
    pub quality: Value,
}

impl TeamPreMatchFeatures {
    pub(crate) fn neutral(team_id: Uuid, reason: &str) -> Self {
        Self {
            attack_score: 50.0,
            defence_score: 50.0,
            rating_confidence: 0.0,
            venue_score: 50.0,
            venue_confidence: 0.0,
            history: json!({
                "score": 50.0,
                "confidence": 0.0,
                "evidence_ids": [],
                "source": "historical_results"
            }),
            evidence: Vec::new(),
            quality: json!({
                "team_id": team_id,
                "history_match_count": 0,
                "feature_scope": "none",
                "neutral_team_ratings": true,
                "warning": reason
            }),
        }
    }
}

#[derive(Debug, Clone)]
struct HistoricalMatch {
    kickoff_time: DateTime<Utc>,
    goals_for: f64,
    goals_against: f64,
    points: f64,
    played_at_target_venue: bool,
    same_competition: bool,
    same_season: bool,
}

#[derive(Debug, Clone, Copy)]
struct GoalBaseline {
    home_goals: f64,
    away_goals: f64,
    match_count: i64,
}

impl GoalBaseline {
    fn team_goals(self) -> f64 {
        ((self.home_goals + self.away_goals) / 2.0).max(0.2)
    }

    fn venue_goals(self, is_home: bool) -> f64 {
        if is_home {
            self.home_goals.max(0.2)
        } else {
            self.away_goals.max(0.2)
        }
    }
}

impl PostgresStore {
    pub(crate) async fn calculate_team_pre_match_features(
        &self,
        fixture: &MatchRecord,
        team_id: Uuid,
        is_home: bool,
        data_cutoff_time: DateTime<Utc>,
    ) -> PersistenceResult<TeamPreMatchFeatures> {
        let rows = sqlx::query(
            r#"
            SELECT historical.kickoff_time,
                   historical.home_team_id, historical.away_team_id,
                   historical.competition_id, historical.season_id,
                   result.home_goals_90, result.away_goals_90
            FROM football.matches historical
            JOIN football.match_results result ON result.match_id = historical.id
            WHERE historical.id <> $1
              AND historical.kickoff_time < $2
              AND result.finalized_at <= $3
              AND result.created_at <= $3
              AND (historical.home_team_id = $4 OR historical.away_team_id = $4)
            ORDER BY historical.kickoff_time DESC, historical.id DESC
            LIMIT $5
            "#,
        )
        .bind(fixture.id)
        .bind(fixture.kickoff_time)
        .bind(data_cutoff_time)
        .bind(team_id)
        .bind(HISTORY_QUERY_LIMIT)
        .fetch_all(&self.pool)
        .await?;

        let all_matches = rows
            .iter()
            .map(|row| {
                let home_team_id: Uuid = row.try_get("home_team_id")?;
                let home_goals = row.try_get::<i16, _>("home_goals_90")? as f64;
                let away_goals = row.try_get::<i16, _>("away_goals_90")? as f64;
                let team_was_home = home_team_id == team_id;
                let (goals_for, goals_against) = if team_was_home {
                    (home_goals, away_goals)
                } else {
                    (away_goals, home_goals)
                };
                let points = if goals_for > goals_against {
                    3.0
                } else if (goals_for - goals_against).abs() < f64::EPSILON {
                    1.0
                } else {
                    0.0
                };
                let competition_id: Option<Uuid> = row.try_get("competition_id")?;
                let season_id: Option<Uuid> = row.try_get("season_id")?;
                Ok(HistoricalMatch {
                    kickoff_time: row.try_get("kickoff_time")?,
                    goals_for,
                    goals_against,
                    points,
                    played_at_target_venue: team_was_home == is_home,
                    same_competition: fixture.competition_id.is_some()
                        && competition_id == fixture.competition_id,
                    same_season: fixture.season_id.is_some() && season_id == fixture.season_id,
                })
            })
            .collect::<PersistenceResult<Vec<_>>>()?;

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
        let baseline = self
            .goal_baseline(
                fixture.competition_id,
                fixture.kickoff_time,
                data_cutoff_time,
            )
            .await?;

        let mut total_weight = 0.0;
        let mut goals_for = 0.0;
        let mut goals_against = 0.0;
        let mut points = 0.0;
        let mut venue_weight = 0.0;
        let mut venue_goals_for = 0.0;
        let mut venue_goals_against = 0.0;
        let mut venue_count = 0usize;

        for item in &selected {
            let age_days = (fixture.kickoff_time - item.kickoff_time)
                .num_seconds()
                .max(0) as f64
                / 86_400.0;
            let weight = 0.5_f64.powf(age_days / 90.0).max(0.05);
            total_weight += weight;
            goals_for += item.goals_for * weight;
            goals_against += item.goals_against * weight;
            points += item.points * weight;
            if item.played_at_target_venue {
                venue_weight += weight;
                venue_goals_for += item.goals_for * weight;
                venue_goals_against += item.goals_against * weight;
                venue_count += 1;
            }
        }

        if total_weight <= 0.0 {
            return Ok(TeamPreMatchFeatures::neutral(team_id, "历史赛果权重无效"));
        }

        let average_goals_for = goals_for / total_weight;
        let average_goals_against = goals_against / total_weight;
        let average_points = points / total_weight;
        let team_goal_baseline = baseline.team_goals();
        let attack_score = ratio_score(average_goals_for / team_goal_baseline);
        let defence_score = ratio_score(team_goal_baseline / average_goals_against.max(0.2));
        let history_score = history_score(
            average_points,
            average_goals_for - average_goals_against,
            team_goal_baseline,
        );
        let baseline_confidence = (baseline.match_count as f64 / 40.0).clamp(0.25, 1.0);
        let sample_confidence =
            (1.0 - (-(selected.len() as f64) / 6.0).exp()) * baseline_confidence * scope_factor;
        let rating_confidence = sample_confidence.clamp(0.0, 0.9);
        // history 与攻防强度共享同一批赛果，只保留部分独立置信度，避免重复放大。
        let history_confidence = (rating_confidence * 0.65).clamp(0.0, 0.75);

        let (venue_score, venue_confidence, venue_average_goals_for, venue_average_goals_against) =
            if venue_count >= 2 && venue_weight > 0.0 {
                let average_for = venue_goals_for / venue_weight;
                let average_against = venue_goals_against / venue_weight;
                let score = venue_score(
                    average_for,
                    average_against,
                    baseline.venue_goals(is_home),
                    team_goal_baseline,
                );
                let confidence = ((1.0 - (-(venue_count as f64) / 4.0).exp())
                    * baseline_confidence
                    * scope_factor)
                    .clamp(0.0, 0.85);
                (score, confidence, Some(average_for), Some(average_against))
            } else {
                (50.0, 0.0, None, None)
            };

        let evidence_suffix = format!("{}_{}", team_id.simple(), data_cutoff_time.timestamp());
        let rating_evidence_id = format!("TEAM_RATING_{evidence_suffix}");
        let history_evidence_id = format!("TEAM_HISTORY_{evidence_suffix}");
        let venue_evidence_id = format!("TEAM_VENUE_{evidence_suffix}");
        let mut evidence = vec![
            json!({
                "evidence_id": rating_evidence_id,
                "module": "team_rating",
                "score": (attack_score + defence_score) / 2.0,
                "confidence": rating_confidence,
                "source_id": format!("POSTGRES_MATCH_{}", fixture.id),
                "note": format!("基于截止时间之前 {} 场正式赛果计算攻防强度", selected.len())
            }),
            json!({
                "evidence_id": history_evidence_id,
                "module": "history",
                "score": history_score,
                "confidence": history_confidence,
                "source_id": format!("POSTGRES_MATCH_{}", fixture.id),
                "note": "近期赛果按 90 天半衰期连续衰减；因与攻防特征共享样本，已降低独立置信度"
            }),
        ];
        if venue_confidence > 0.0 {
            evidence.push(json!({
                "evidence_id": venue_evidence_id,
                "module": "venue",
                "score": venue_score,
                "confidence": venue_confidence,
                "source_id": format!("POSTGRES_MATCH_{}", fixture.id),
                "note": if is_home { "主场历史表现" } else { "客场历史表现" }
            }));
        }

        Ok(TeamPreMatchFeatures {
            attack_score,
            defence_score,
            rating_confidence,
            venue_score,
            venue_confidence,
            history: json!({
                "score": history_score,
                "confidence": history_confidence,
                "evidence_ids": [history_evidence_id],
                "source": "historical_results"
            }),
            evidence,
            quality: json!({
                "team_id": team_id,
                "history_match_count": selected.len(),
                "feature_scope": scope,
                "scope_factor": scope_factor,
                "baseline_match_count": baseline.match_count,
                "baseline_home_goals": baseline.home_goals,
                "baseline_away_goals": baseline.away_goals,
                "weighted_goals_for": average_goals_for,
                "weighted_goals_against": average_goals_against,
                "weighted_points_per_match": average_points,
                "venue_match_count": venue_count,
                "venue_weighted_goals_for": venue_average_goals_for,
                "venue_weighted_goals_against": venue_average_goals_against,
                "attack_score": attack_score,
                "defence_score": defence_score,
                "rating_confidence": rating_confidence,
                "history_confidence": history_confidence,
                "shared_sample_deduplication": "history_confidence_x0.65",
                "venue_score": venue_score,
                "venue_confidence": venue_confidence,
                "neutral_team_ratings": false,
                "calculation": "recency_weighted_results_v1"
            }),
        })
    }

    async fn goal_baseline(
        &self,
        competition_id: Option<Uuid>,
        kickoff_time: DateTime<Utc>,
        data_cutoff_time: DateTime<Utc>,
    ) -> PersistenceResult<GoalBaseline> {
        let row = sqlx::query(
            r#"
            SELECT AVG(result.home_goals_90::double precision) AS home_goals,
                   AVG(result.away_goals_90::double precision) AS away_goals,
                   COUNT(*)::bigint AS match_count
            FROM football.matches historical
            JOIN football.match_results result ON result.match_id = historical.id
            WHERE historical.kickoff_time < $1
              AND result.finalized_at <= $2
              AND result.created_at <= $2
              AND ($3::uuid IS NULL OR historical.competition_id = $3)
            "#,
        )
        .bind(kickoff_time)
        .bind(data_cutoff_time)
        .bind(competition_id)
        .fetch_one(&self.pool)
        .await?;
        let match_count: i64 = row.try_get("match_count")?;
        let home_goals = row
            .try_get::<Option<f64>, _>("home_goals")?
            .unwrap_or(DEFAULT_GOAL_BASELINE);
        let away_goals = row
            .try_get::<Option<f64>, _>("away_goals")?
            .unwrap_or(DEFAULT_GOAL_BASELINE);
        Ok(GoalBaseline {
            home_goals: finite_or_default(home_goals, DEFAULT_GOAL_BASELINE),
            away_goals: finite_or_default(away_goals, DEFAULT_GOAL_BASELINE),
            match_count,
        })
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

fn ratio_score(ratio: f64) -> f64 {
    let safe_ratio = finite_or_default(ratio, 1.0).clamp(0.1, 10.0);
    (50.0 + 30.0 * (safe_ratio.ln() / 0.55).tanh()).clamp(5.0, 95.0)
}

fn history_score(points_per_match: f64, goal_difference: f64, goal_baseline: f64) -> f64 {
    let points_signal = (finite_or_default(points_per_match, 1.35) - 1.35) / 1.0;
    let goal_signal = finite_or_default(goal_difference, 0.0) / goal_baseline.max(0.5);
    (50.0 + 30.0 * (0.72 * points_signal + 0.28 * goal_signal).tanh()).clamp(5.0, 95.0)
}

fn venue_score(
    goals_for: f64,
    goals_against: f64,
    venue_goal_baseline: f64,
    team_goal_baseline: f64,
) -> f64 {
    let attack_signal = (finite_or_default(goals_for, venue_goal_baseline)
        / venue_goal_baseline.max(0.2))
    .clamp(0.1, 10.0)
    .ln();
    let balance_signal = (finite_or_default(goals_for - goals_against, 0.0)
        / team_goal_baseline.max(0.5))
    .clamp(-3.0, 3.0);
    (50.0 + 28.0 * (0.65 * attack_signal + 0.35 * balance_signal).tanh()).clamp(5.0, 95.0)
}

fn finite_or_default(value: f64, default: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        default
    }
}

#[cfg(test)]
mod tests {
    use super::{history_score, ratio_score, venue_score};

    #[test]
    fn ratio_curve_is_continuous_and_centered() {
        assert!((ratio_score(1.0) - 50.0).abs() < 1e-12);
        assert!(ratio_score(1.2) > ratio_score(1.1));
        assert!(ratio_score(0.8) < ratio_score(0.9));
    }

    #[test]
    fn history_curve_rewards_better_results() {
        assert!(history_score(2.2, 0.8, 1.2) > history_score(1.2, 0.0, 1.2));
        assert!(history_score(0.5, -0.8, 1.2) < 50.0);
    }

    #[test]
    fn venue_curve_stays_bounded() {
        let score = venue_score(6.0, 0.0, 1.2, 1.2);
        assert!((5.0..=95.0).contains(&score));
    }
}
