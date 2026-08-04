use football_domain::{
    AbilityCandidateProposal, CalculatedMatchReview, CalculatedPlayerReview, CalculatedTeamReview,
    PlayerPerformanceMetrics, ReviewPreparationData,
};
use serde_json::{json, Map, Value};

pub const REVIEW_CALCULATION_VERSION: &str = "phase4-review-v1";

pub fn calculate_review(input: &ReviewPreparationData) -> CalculatedMatchReview {
    let substitution_players: std::collections::HashSet<_> = input
        .substitutions
        .iter()
        .filter_map(|item| item.player_in_id)
        .collect();

    let player_reviews: Vec<CalculatedPlayerReview> = input
        .players
        .iter()
        .map(|player| {
            let expected = player.expected_performance.clamp(1.0, 100.0);
            let actual = if player.minutes_played <= 0 {
                expected
            } else {
                player.performance_score.unwrap_or_else(|| {
                    calculate_performance_score(&player.metrics, player.minutes_played)
                })
            };
            let realization = (actual / expected).clamp(0.0, 2.0);
            let minutes_share = (f64::from(player.minutes_played) / 90.0).clamp(0.0, 1.0);
            let confidence = (player.input_confidence
                * input.data_coverage
                * (0.35 + 0.65 * minutes_share)
                * (0.70 + 0.30 * player.expected_confidence))
                .clamp(0.0, 1.0);
            let entry_type = if player.started {
                "starter"
            } else if substitution_players.contains(&player.player_id) || player.minutes_played > 0
            {
                "substitute"
            } else {
                "unused_substitute"
            }
            .to_string();
            let contribution_weight = minutes_share * (expected / 100.0);
            let candidates = ability_candidates(player, actual, confidence, &entry_type);
            CalculatedPlayerReview {
                observation_id: player.observation_id,
                player_id: player.player_id,
                team_id: player.team_id,
                role_code: player.role_code.clone(),
                started: player.started,
                entry_type,
                minutes_played: player.minutes_played,
                expected_performance: round4(expected),
                actual_performance: round4(actual),
                realization_ratio: round4(realization),
                confidence: round4(confidence),
                contribution_weight: round4(contribution_weight),
                metrics: json!({
                    "input": player.metrics,
                    "calculated_performance": actual,
                    "expected_source": "pre_match_contribution_or_current_ability",
                    "reviewed_match_count": player.reviewed_match_count,
                    "substitute_appearances": player.substitute_appearances,
                }),
                ability_candidates: candidates,
            }
        })
        .collect();

    let team_reviews = input
        .teams
        .iter()
        .map(|team| {
            calculate_team_review(
                input,
                &player_reviews,
                team.team_id,
                team.recent_starter_overlap,
            )
        })
        .collect::<Vec<_>>();

    let prediction_evaluation = calculate_prediction_evaluation(input);
    let conclusions = calculate_conclusions(
        input,
        &player_reviews,
        &team_reviews,
        &prediction_evaluation,
    );

    CalculatedMatchReview {
        calculation_version: REVIEW_CALCULATION_VERSION.to_string(),
        prediction_evaluation,
        conclusions,
        player_reviews,
        team_reviews,
    }
}

fn calculate_performance_score(metrics: &PlayerPerformanceMetrics, minutes: i16) -> f64 {
    if let Some(rating) = metrics.provider_rating {
        if (0.0..=10.0).contains(&rating) {
            return (rating * 10.0).clamp(0.0, 100.0);
        }
        if (0.0..=100.0).contains(&rating) {
            return rating;
        }
    }
    let minutes_share = (f64::from(minutes) / 90.0).clamp(0.0, 1.0);
    let duel_rate = if metrics.duels_total > 0.0 {
        metrics.duels_won / metrics.duels_total
    } else {
        0.5
    };
    let attacking = metrics.goals * 12.0
        + metrics.assists * 8.0
        + (metrics.goals - metrics.expected_goals) * 4.0
        + (metrics.assists - metrics.expected_assists) * 3.0
        + metrics.shots_on_target * 0.9
        + metrics.key_passes * 1.4
        + metrics.progressive_actions * 0.45;
    let defending = metrics.tackles * 0.9
        + metrics.interceptions * 1.0
        + metrics.clearances * 0.45
        + metrics.blocks * 0.8
        + (duel_rate - 0.5) * 12.0;
    let discipline = metrics.fouls * -0.45
        + metrics.yellow_cards * -3.0
        + metrics.red_cards * -12.0
        + metrics.errors_leading_to_shot * -7.0;
    (48.0 + minutes_share * 7.0 + attacking + defending + discipline).clamp(0.0, 100.0)
}

fn ability_candidates(
    player: &football_domain::ReviewPlayerBaseline,
    actual: f64,
    confidence: f64,
    entry_type: &str,
) -> Vec<AbilityCandidateProposal> {
    if player.minutes_played < 15 || confidence < 0.45 {
        return Vec::new();
    }
    let metrics = &player.metrics;
    let mut observations: Vec<(&str, f64, f64, Value)> = Vec::new();

    if metrics.shots > 0.0 || metrics.expected_goals > 0.0 || metrics.goals > 0.0 {
        let finishing = (50.0
            + metrics.goals * 15.0
            + (metrics.goals - metrics.expected_goals) * 12.0
            + metrics.shots_on_target * 2.0
            - (metrics.shots - metrics.shots_on_target).max(0.0))
        .clamp(0.0, 100.0);
        observations.push(("finishing", finishing, 0.90, json!({"goals": metrics.goals, "xg": metrics.expected_goals, "shots": metrics.shots, "shots_on_target": metrics.shots_on_target})));
    }
    if metrics.key_passes > 0.0 || metrics.expected_assists > 0.0 || metrics.assists > 0.0 {
        let creation = (45.0
            + metrics.assists * 13.0
            + metrics.expected_assists * 8.0
            + metrics.key_passes * 3.0
            + metrics.progressive_actions * 0.6)
            .clamp(0.0, 100.0);
        observations.push(("creation", creation, 0.85, json!({"assists": metrics.assists, "xa": metrics.expected_assists, "key_passes": metrics.key_passes, "progressive_actions": metrics.progressive_actions})));
    }
    let defensive_actions =
        metrics.tackles + metrics.interceptions + metrics.clearances + metrics.blocks;
    if defensive_actions > 0.0 {
        let defence = (42.0
            + metrics.tackles * 3.0
            + metrics.interceptions * 3.5
            + metrics.clearances * 1.2
            + metrics.blocks * 2.5
            - metrics.errors_leading_to_shot * 10.0)
            .clamp(0.0, 100.0);
        observations.push(("defence", defence, 0.80, json!({"tackles": metrics.tackles, "interceptions": metrics.interceptions, "clearances": metrics.clearances, "blocks": metrics.blocks, "errors": metrics.errors_leading_to_shot})));
    }
    if metrics.duels_total > 0.0 {
        let duel_rate = metrics.duels_won / metrics.duels_total;
        let physical = (30.0 + duel_rate * 60.0).clamp(0.0, 100.0);
        observations.push(("physical", physical, 0.70, json!({"duels_won": metrics.duels_won, "duels_total": metrics.duels_total, "duel_rate": duel_rate})));
    }
    if metrics.fouls > 0.0 || metrics.yellow_cards > 0.0 || metrics.red_cards > 0.0 {
        let discipline =
            (88.0 - metrics.fouls * 2.0 - metrics.yellow_cards * 12.0 - metrics.red_cards * 35.0)
                .clamp(0.0, 100.0);
        observations.push(("discipline", discipline, 0.65, json!({"fouls": metrics.fouls, "yellow_cards": metrics.yellow_cards, "red_cards": metrics.red_cards})));
    }
    observations.push((
        "tactical_execution",
        actual,
        0.60,
        json!({"performance_score": actual, "minutes": player.minutes_played}),
    ));
    if entry_type == "substitute" {
        observations.push((
            "substitute_impact",
            actual,
            0.85,
            json!({"performance_score": actual, "minutes": player.minutes_played}),
        ));
    }

    observations
        .into_iter()
        .filter_map(|(dimension, observed, metric_confidence, evidence)| {
            let current = current_ability(&player.current_abilities, dimension);
            let base = current.unwrap_or(50.0);
            let sample_size = player.reviewed_match_count.saturating_add(1);
            let learning_rate = (0.035 + confidence * 0.045).clamp(0.04, 0.08);
            let proposed = (base + (observed - base) * learning_rate).clamp(0.0, 100.0);
            if (proposed - base).abs() < 0.35 {
                return None;
            }
            Some(AbilityCandidateProposal {
                player_id: player.player_id,
                dimension_code: dimension.to_string(),
                current_value: current,
                proposed_value: round4(proposed),
                confidence: round4((confidence * metric_confidence).clamp(0.0, 1.0)),
                sample_size,
                evidence: json!({
                    "observed_dimension_score": round4(observed),
                    "learning_rate": round4(learning_rate),
                    "match_performance": round4(actual),
                    "minutes_played": player.minutes_played,
                    "metric_evidence": evidence,
                    "guardrail": "candidate_only_no_automatic_writeback"
                }),
            })
        })
        .collect()
}

fn calculate_team_review(
    input: &ReviewPreparationData,
    players: &[CalculatedPlayerReview],
    team_id: uuid::Uuid,
    continuity: f64,
) -> CalculatedTeamReview {
    let team_players: Vec<_> = players
        .iter()
        .filter(|player| player.team_id == team_id)
        .collect();
    let active_players: Vec<_> = team_players
        .iter()
        .filter(|player| player.minutes_played > 0)
        .collect();
    let starters: Vec<_> = active_players
        .iter()
        .filter(|player| player.started)
        .collect();
    let substitutes: Vec<_> = active_players
        .iter()
        .filter(|player| player.entry_type == "substitute")
        .collect();

    let realization = weighted_average(
        active_players.iter().map(|player| {
            (
                player.realization_ratio,
                player.contribution_weight.max(0.05),
            )
        }),
        1.0,
    );
    let expected_starters = weighted_average(
        starters.iter().map(|player| {
            (
                player.expected_performance,
                player.contribution_weight.max(0.05),
            )
        }),
        50.0,
    );
    let expected_bench = weighted_average(
        team_players
            .iter()
            .filter(|player| !player.started)
            .map(|player| (player.expected_performance, 1.0)),
        expected_starters,
    );
    let bench_dropoff = expected_bench - expected_starters;
    let substitute_realization = weighted_average(
        substitutes.iter().map(|player| {
            (
                player.realization_ratio,
                f64::from(player.minutes_played).max(1.0),
            )
        }),
        1.0,
    );
    let substitution_impact = ((substitute_realization - 1.0) * 100.0).clamp(-100.0, 100.0);
    let variance = if active_players.len() > 1 {
        let mean = realization;
        active_players
            .iter()
            .map(|player| (player.realization_ratio - mean).powi(2))
            .sum::<f64>()
            / active_players.len() as f64
    } else {
        0.25
    };
    let cohesion = (1.0 - variance.sqrt().min(1.0)).clamp(0.0, 1.0);
    let chemistry = (continuity.clamp(0.0, 1.0) * 0.55 + cohesion * 0.45) * 100.0;
    let confidence = if active_players.is_empty() {
        0.0
    } else {
        active_players
            .iter()
            .map(|player| player.confidence)
            .sum::<f64>()
            / active_players.len() as f64
    };
    let sub_minutes: i16 = substitutes.iter().map(|player| player.minutes_played).sum();
    let team_name = input
        .teams
        .iter()
        .find(|team| team.team_id == team_id)
        .map(|team| team.team_name.clone())
        .unwrap_or_default();

    CalculatedTeamReview {
        team_id,
        chemistry_score: round4(chemistry),
        lineup_continuity: round4(continuity.clamp(0.0, 1.0)),
        performance_cohesion: round4(cohesion),
        bench_strength: round4(expected_bench),
        bench_dropoff: round4(bench_dropoff),
        substitution_impact: round4(substitution_impact),
        substitute_count: substitutes.len() as i32,
        realization_score: round4(realization),
        confidence: round4(confidence.clamp(0.0, 1.0)),
        metrics: json!({
            "team_name": team_name,
            "active_player_count": active_players.len(),
            "starter_count": starters.len(),
            "substitute_count": substitutes.len(),
            "substitute_minutes": sub_minutes,
            "starter_expected_average": round4(expected_starters),
            "bench_expected_average": round4(expected_bench),
            "substitute_realization": round4(substitute_realization),
        }),
    }
}

fn calculate_prediction_evaluation(input: &ReviewPreparationData) -> Value {
    let Some(prediction) = &input.prediction else {
        return json!({"available": false, "reason": "no_succeeded_prediction_run"});
    };
    let summary = prediction.summary.as_object().cloned().unwrap_or_default();
    let home = number(&summary, "home_win").unwrap_or(0.0);
    let draw = number(&summary, "draw").unwrap_or(0.0);
    let away = number(&summary, "away_win").unwrap_or(0.0);
    let actual_outcome = if input.result.home_goals_90 > input.result.away_goals_90 {
        "home_win"
    } else if input.result.home_goals_90 < input.result.away_goals_90 {
        "away_win"
    } else {
        "draw"
    };
    let actual_probability = match actual_outcome {
        "home_win" => home,
        "draw" => draw,
        _ => away,
    }
    .clamp(1e-12, 1.0);
    let targets = [
        if actual_outcome == "home_win" {
            1.0
        } else {
            0.0
        },
        if actual_outcome == "draw" { 1.0 } else { 0.0 },
        if actual_outcome == "away_win" {
            1.0
        } else {
            0.0
        },
    ];
    let probabilities = [home, draw, away];
    let brier = probabilities
        .iter()
        .zip(targets.iter())
        .map(|(probability, target)| (probability - target).powi(2))
        .sum::<f64>();
    let scoreline_probability = prediction.actual_scoreline_probability;
    json!({
        "available": true,
        "run_id": prediction.run_id,
        "actual_outcome": actual_outcome,
        "actual_probability": round4(actual_probability),
        "log_loss": round4(-actual_probability.ln()),
        "brier": round4(brier),
        "scoreline_probability": scoreline_probability.map(round4),
        "scoreline_nll": scoreline_probability.map(|value| round4(-value.clamp(1e-12, 1.0).ln())),
        "predicted_probabilities": {"home_win": home, "draw": draw, "away_win": away},
    })
}

fn calculate_conclusions(
    input: &ReviewPreparationData,
    players: &[CalculatedPlayerReview],
    teams: &[CalculatedTeamReview],
    prediction: &Value,
) -> Value {
    let mut sorted_players = players.to_vec();
    sorted_players
        .sort_by(|left, right| right.actual_performance.total_cmp(&left.actual_performance));
    let top_players: Vec<_> = sorted_players
        .iter()
        .take(5)
        .map(|player| {
            json!({
                "player_id": player.player_id,
                "actual_performance": player.actual_performance,
                "realization_ratio": player.realization_ratio,
                "entry_type": player.entry_type,
            })
        })
        .collect();
    let underperformers: Vec<_> = sorted_players
        .iter()
        .rev()
        .filter(|player| player.minutes_played >= 30)
        .take(5)
        .map(|player| {
            json!({
                "player_id": player.player_id,
                "actual_performance": player.actual_performance,
                "realization_ratio": player.realization_ratio,
            })
        })
        .collect();
    let candidate_count: usize = players
        .iter()
        .map(|player| player.ability_candidates.len())
        .sum();
    json!({
        "match_id": input.match_record.id,
        "result": {
            "home_goals_90": input.result.home_goals_90,
            "away_goals_90": input.result.away_goals_90,
        },
        "top_players": top_players,
        "underperformers": underperformers,
        "teams": teams,
        "prediction": prediction,
        "ability_candidate_count": candidate_count,
        "warning": "能力变更仅生成候选，必须人工审核后才能写入正式能力历史",
    })
}

fn current_ability(abilities: &Value, dimension: &str) -> Option<f64> {
    let value = abilities.get(dimension)?;
    if let Some(number) = value.as_f64() {
        return Some(number);
    }
    value.get("value").and_then(Value::as_f64)
}

fn number(map: &Map<String, Value>, key: &str) -> Option<f64> {
    map.get(key).and_then(Value::as_f64)
}

fn weighted_average<I>(values: I, default: f64) -> f64
where
    I: IntoIterator<Item = (f64, f64)>,
{
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    for (value, weight) in values {
        if value.is_finite() && weight.is_finite() && weight > 0.0 {
            numerator += value * weight;
            denominator += weight;
        }
    }
    if denominator > 0.0 {
        numerator / denominator
    } else {
        default
    }
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_rating_is_normalized() {
        let metrics = PlayerPerformanceMetrics {
            provider_rating: Some(7.4),
            ..Default::default()
        };
        assert_eq!(calculate_performance_score(&metrics, 90), 74.0);
    }

    #[test]
    fn event_score_is_bounded() {
        let metrics = PlayerPerformanceMetrics {
            goals: 6.0,
            assists: 4.0,
            ..Default::default()
        };
        assert_eq!(calculate_performance_score(&metrics, 90), 100.0);
    }
}
