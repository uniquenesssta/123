use chrono::{DateTime, Utc};
use football_domain::MatchRecord;
use serde_json::{json, Value};
use uuid::Uuid;

pub(crate) const DEFAULT_GOAL_BASELINE: f64 = 1.15;

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
pub(crate) struct HistoricalMatch {
    pub(crate) kickoff_time: DateTime<Utc>,
    pub(crate) goals_for: f64,
    pub(crate) goals_against: f64,
    pub(crate) points: f64,
    pub(crate) played_at_target_venue: bool,
    pub(crate) same_competition: bool,
    pub(crate) same_season: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GoalBaseline {
    pub(crate) home_goals: f64,
    pub(crate) away_goals: f64,
    pub(crate) match_count: i64,
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

pub(crate) struct HistoricalFeatureSample<'a> {
    pub(crate) matches: Vec<&'a HistoricalMatch>,
    pub(crate) scope: &'static str,
    pub(crate) scope_factor: f64,
    pub(crate) baseline: GoalBaseline,
}

pub(crate) fn project_team_features(
    fixture: &MatchRecord,
    team_id: Uuid,
    is_home: bool,
    data_cutoff_time: DateTime<Utc>,
    sample: HistoricalFeatureSample<'_>,
) -> TeamPreMatchFeatures {
    let HistoricalFeatureSample {
        matches: selected,
        scope,
        scope_factor,
        baseline,
    } = sample;
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
        return TeamPreMatchFeatures::neutral(team_id, "历史赛果权重无效");
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
            let confidence =
                ((1.0 - (-(venue_count as f64) / 4.0).exp()) * baseline_confidence * scope_factor)
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

    TeamPreMatchFeatures {
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
    }
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

pub(crate) fn finite_or_default(value: f64, default: f64) -> f64 {
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
    fn fixture() -> super::MatchRecord {
        serde_json::from_value(serde_json::json!({
            "id":super::Uuid::from_u128(1),"external_key":"HISTORY","competition_id":null,"competition_name":null,"season_id":null,"stage_id":null,"round_id":null,
            "home_team_id":super::Uuid::from_u128(2),"home_team_name":"Home","away_team_id":super::Uuid::from_u128(3),"away_team_name":"Away",
            "kickoff_time":"2026-10-01T12:00:00Z","status":"scheduled","venue":null
        })).unwrap()
    }

    #[test]
    fn historical_feature_projection_keeps_fixed_platform_scores_and_evidence() {
        let fixture = fixture();
        let cutoff = fixture.kickoff_time - chrono::Duration::hours(1);
        let rows = (0..4)
            .map(|_| super::HistoricalMatch {
                kickoff_time: fixture.kickoff_time - chrono::Duration::days(90),
                goals_for: 2.0,
                goals_against: 1.0,
                points: 3.0,
                played_at_target_venue: true,
                same_competition: true,
                same_season: true,
            })
            .collect::<Vec<_>>();
        let feature = super::project_team_features(
            &fixture,
            fixture.home_team_id,
            true,
            cutoff,
            super::HistoricalFeatureSample {
                matches: rows.iter().collect(),
                scope: "same_season",
                scope_factor: 1.0,
                baseline: super::GoalBaseline {
                    home_goals: 1.4,
                    away_goals: 1.0,
                    match_count: 40,
                },
            },
        );
        assert!((feature.attack_score - 71.90065154239097).abs() < 1e-12);
        assert!((feature.defence_score - 59.59586987475388).abs() < 1e-12);
        assert!((feature.history["score"].as_f64().unwrap() - 76.69630239222556).abs() < 1e-12);
        assert!((feature.rating_confidence - 0.486582880967408).abs() < 1e-12);
        assert!((feature.venue_score - 63.45122632742265).abs() < 1e-12);
        assert!((feature.venue_confidence - 0.6321205588285577).abs() < 1e-12);

        assert!(
            (feature.history["confidence"].as_f64().unwrap() - feature.rating_confidence * 0.65)
                .abs()
                < 1e-12
        );
        assert_eq!(feature.quality["history_match_count"], serde_json::json!(4));
        assert_eq!(feature.quality["venue_match_count"], serde_json::json!(4));
        assert_eq!(
            feature.quality["feature_scope"],
            serde_json::json!("same_season")
        );
        assert_eq!(
            feature.quality["weighted_goals_for"],
            serde_json::json!(2.0)
        );
        assert_eq!(feature.evidence.len(), 3);
        assert_eq!(
            feature.evidence[1]["evidence_id"],
            serde_json::json!(format!(
                "TEAM_HISTORY_{}_{}",
                fixture.home_team_id.simple(),
                cutoff.timestamp()
            ))
        );
        assert_eq!(
            feature.evidence[0]["source_id"],
            serde_json::json!(format!("POSTGRES_MATCH_{}", fixture.id))
        );
    }

    #[test]
    fn empty_history_keeps_neutral_features_without_evidence() {
        let fixture = fixture();
        let feature = super::project_team_features(
            &fixture,
            fixture.home_team_id,
            true,
            fixture.kickoff_time,
            super::HistoricalFeatureSample {
                matches: vec![],
                scope: "none",
                scope_factor: 1.0,
                baseline: super::GoalBaseline {
                    home_goals: 1.15,
                    away_goals: 1.15,
                    match_count: 0,
                },
            },
        );
        assert_eq!(feature.attack_score, 50.0);
        assert_eq!(feature.defence_score, 50.0);
        assert_eq!(feature.venue_score, 50.0);
        assert_eq!(feature.rating_confidence, 0.0);
        assert_eq!(feature.venue_confidence, 0.0);
        assert!(feature.evidence.is_empty());
        assert_eq!(
            feature.quality["neutral_team_ratings"],
            serde_json::json!(true)
        );
        assert_eq!(feature.quality["history_match_count"], serde_json::json!(0));
        assert_eq!(feature.history["evidence_ids"], serde_json::json!([]));
    }
}
