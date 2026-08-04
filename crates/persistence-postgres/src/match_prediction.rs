use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::{
    CompetitionKind, LineupRecord, PlayerMatchContribution, PlayerMatchContributionRequest,
    PreparedMatchPredictionInput,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

const PREPARATION_VERSION: &str = "postgres-match-input-v5-role-context";

struct PreparedTeamRequest<'a> {
    fixture: &'a football_domain::MatchRecord,
    team_id: Uuid,
    team_name: &'a str,
    opponent_team_id: Uuid,
    is_home: bool,
    data_cutoff_time: chrono::DateTime<Utc>,
    lineup: Option<&'a LineupRecord>,
}

impl PostgresStore {
    pub async fn prepare_match_prediction_input(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        model_family: &str,
    ) -> PersistenceResult<PreparedMatchPredictionInput> {
        self.prepare_match_prediction_input_at(match_id, snapshot_type, model_family, Utc::now())
            .await
    }

    pub async fn prepare_match_prediction_input_at(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        model_family: &str,
        reference_time: DateTime<Utc>,
    ) -> PersistenceResult<PreparedMatchPredictionInput> {
        let row = sqlx::query(
            r#"
            SELECT fixture.id, fixture.external_key, fixture.competition_id,
                   competition.name AS competition_name,
                   fixture.season_id, fixture.stage_id, fixture.round_id,
                   fixture.home_team_id, home.canonical_name AS home_team_name,
                   fixture.away_team_id, away.canonical_name AS away_team_name,
                   fixture.kickoff_time, fixture.status, fixture.venue,
                   COALESCE(stage.stage_kind, competition.competition_kind, 'custom') AS effective_kind
            FROM football.matches fixture
            LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
            LEFT JOIN football.competition_stages stage ON stage.id = fixture.stage_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            WHERE fixture.id = $1
            "#,
        )
        .bind(match_id)
        .fetch_one(&self.pool)
        .await?;
        let match_record = super::match_exchange::match_record_from_row(&row)?;
        let competition_kind = parse_kind(&row.try_get::<String, _>("effective_kind")?)?;
        let frozen_at = reference_time;
        let data_window = super::lineup_chain::lineup_snapshot_window_at(
            match_record.kickoff_time,
            snapshot_type,
            frozen_at,
        )?;
        let data_cutoff_time = data_window.cutoff_time;
        let home_lineup = self
            .preferred_pre_match_lineup(match_id, match_record.home_team_id, data_window)
            .await?;
        let away_lineup = self
            .preferred_pre_match_lineup(match_id, match_record.away_team_id, data_window)
            .await?;
        if home_lineup.is_none() || away_lineup.is_none() {
            let chain = self
                .read_match_lineup_chain_at(match_id, snapshot_type, reference_time)
                .await?;
            return Err(PersistenceError::InvalidState(format!(
                "阵容冻结门禁未通过：{}",
                chain.blocking_issues.join("；")
            )));
        }
        let home = self
            .build_prediction_team(PreparedTeamRequest {
                fixture: &match_record,
                team_id: match_record.home_team_id,
                team_name: &match_record.home_team_name,
                opponent_team_id: match_record.away_team_id,
                is_home: true,
                data_cutoff_time,
                lineup: home_lineup.as_ref(),
            })
            .await?;
        let away = self
            .build_prediction_team(PreparedTeamRequest {
                fixture: &match_record,
                team_id: match_record.away_team_id,
                team_name: &match_record.away_team_name,
                opponent_team_id: match_record.home_team_id,
                is_home: false,
                data_cutoff_time,
                lineup: away_lineup.as_ref(),
            })
            .await?;
        let provider_boundary = json!({
            "status": "external_provider_required",
            "provider_state": "NOT_BUNDLED",
            "model_family": model_family,
            "parameter_preparation": "provider_owned"
        });
        let replay_mode = frozen_at >= match_record.kickoff_time;
        let quality_score = ((home.confidence + away.confidence) / 2.0).clamp(0.0, 1.0);
        let data_quality = json!({
            "preparation_version": PREPARATION_VERSION,
            "data_window_start_time": data_window.start_time.map(|value| value.to_rfc3339()),
            "data_cutoff_time": data_cutoff_time.to_rfc3339(),
            "window_semantics": "latest_record_within_selected_pre_match_window",
            "strict_pre_match_cutoff": true,
            "run_mode": if replay_mode { "historical_replay" } else { "pre_match" },
            "team_ratings": "recency_weighted_historical_results",
            "model_provider": provider_boundary,
            "home": home.quality.clone(),
            "away": away.quality.clone(),
            "warning": if home.has_team_history && away.has_team_history {
                Value::Null
            } else {
                json!("部分球队缺少截止时间之前的正式赛果，相应特征已按置信度平滑回中性。")
            }
        });
        let feature_snapshot_id = Uuid::new_v4();
        let match_input = json!({
            "match_id": match_record.external_key.clone(),
            "database_match_id": match_record.id,
            "feature_snapshot_id": feature_snapshot_id,
            "kickoff_time": match_record.kickoff_time.to_rfc3339(),
            "snapshot": {
                "snapshot_id": feature_snapshot_id.to_string(),
                "type": snapshot_type,
                "data_cutoff_time": data_cutoff_time.to_rfc3339(),
                "frozen_at": frozen_at.to_rfc3339()
            },
            "team_a": home.input.clone(),
            "team_b": away.input.clone(),
            "sources": [{
                "source_id": format!("POSTGRES_MATCH_{}", match_record.id),
                "source_name": "PostgreSQL 赛果、阵容、球员能力、伤停与动态标签",
                "published_at": data_cutoff_time.to_rfc3339(),
                "accessed_at": frozen_at.to_rfc3339(),
                "grade": "B",
                "confidence": quality_score,
                "evidence_key": format!("MATCH_INPUT_{}", match_record.id),
                "primary_module": "pre_match_features"
            }],
            "data_quality": data_quality.clone(),
            "feature_quality_score": quality_score,
            "preparation_version": PREPARATION_VERSION
        });

        Ok(PreparedMatchPredictionInput {
            match_record,
            competition_kind,
            snapshot_type: snapshot_type.to_string(),
            match_input,
            data_quality,
        })
    }

    async fn preferred_pre_match_lineup(
        &self,
        match_id: Uuid,
        team_id: Uuid,
        data_window: super::lineup_chain::LineupSnapshotWindow,
    ) -> PersistenceResult<Option<LineupRecord>> {
        match self
            .preferred_lineup_id(match_id, team_id, data_window)
            .await?
        {
            Some(id) => self.read_lineup(id).await.map(Some),
            None => Ok(None),
        }
    }

    async fn build_prediction_team(
        &self,
        request: PreparedTeamRequest<'_>,
    ) -> PersistenceResult<PreparedTeam> {
        let PreparedTeamRequest {
            fixture,
            team_id,
            team_name,
            opponent_team_id,
            is_home,
            data_cutoff_time,
            lineup,
        } = request;
        let team_features = self
            .calculate_team_pre_match_features(fixture, team_id, is_home, data_cutoff_time)
            .await?;
        let (lineup_payload, lineup_confidence, lineup_quality) = if let Some(lineup) = lineup {
            let mut starters = Vec::new();
            let mut bench = Vec::new();
            for player in &lineup.players {
                let contribution = self
                    .calculate_player_match_contribution(&PlayerMatchContributionRequest {
                        player_id: player.player_id,
                        match_id: Some(fixture.id),
                        competition_id: fixture.competition_id,
                        position_code: player.position_code.clone(),
                        role_code: player.role_code.clone(),
                        role_origin: Some(player.role_origin.clone()),
                        role_source_position_code: player.role_source_position_code.clone(),
                        opponent_team_id: Some(opponent_team_id),
                        as_of: fixture.kickoff_time,
                        data_cutoff_time: Some(data_cutoff_time),
                        expected_minutes: player.expected_minutes,
                    })
                    .await?;
                if player.is_starter {
                    starters.push(contribution);
                } else {
                    bench.push(contribution);
                }
            }

            let coverage = (starters.len() as f64 / 11.0).clamp(0.0, 1.0);
            let average_confidence =
                average(starters.iter().map(|item| item.overall_confidence)).unwrap_or(0.0);
            let quality_score = lineup.quality_score.unwrap_or(0.5).clamp(0.0, 1.0);
            let confidence =
                (coverage * (0.6 * quality_score + 0.4 * average_confidence)).clamp(0.0, 1.0);
            let core_starters = average(
                starters
                    .iter()
                    .map(|item| item.effective_contribution.clamp(0.0, 100.0)),
            )
            .unwrap_or(50.0);
            let natural_positions = average(
                starters
                    .iter()
                    .map(|item| component_score(item, "position_fit")),
            )
            .unwrap_or(50.0);
            let supply_chain = average(
                starters
                    .iter()
                    .map(|item| component_score(item, "chemistry_fit")),
            )
            .unwrap_or(50.0);
            let tactical_match = average(
                starters
                    .iter()
                    .map(|item| component_score(item, "tactical_fit")),
            )
            .unwrap_or(50.0);
            let role_certainty = average(
                starters
                    .iter()
                    .map(|item| item.tactical_role_confidence * 100.0),
            )
            .unwrap_or(50.0);
            let inherited_role_count = starters
                .iter()
                .filter(|item| item.tactical_role_origin == "player_position_default")
                .count();
            let overridden_role_count = starters
                .iter()
                .filter(|item| item.tactical_role_origin == "lineup_override")
                .count();
            let missing_role_count = starters
                .iter()
                .filter(|item| item.tactical_role_origin == "missing")
                .count();
            let injury_risk = average(starters.iter().map(|item| risk_score(item, "availability")))
                .unwrap_or(50.0);
            let fatigue_risk = average(
                starters
                    .iter()
                    .map(|item| risk_score(item, "fatigue_multiplier")),
            )
            .unwrap_or(50.0);
            let bench_continuity = average(
                bench
                    .iter()
                    .map(|item| item.effective_contribution.clamp(0.0, 100.0)),
            )
            .unwrap_or(50.0);
            let player_contributions = starters
                .iter()
                .chain(bench.iter())
                .map(|item| {
                    json!({
                        "player_id": item.player_id,
                        "player_name": &item.player_name,
                        "position_code": &item.position_code,
                        "tactical_role_code": &item.tactical_role_code,
                        "tactical_role_origin": &item.tactical_role_origin,
                        "tactical_role_source_position_code": &item.tactical_role_source_position_code,
                        "tactical_role_confidence": item.tactical_role_confidence,
                        "base_ability": item.base_ability,
                        "effective_contribution": item.effective_contribution,
                        "overall_confidence": item.overall_confidence,
                        "expected_minutes_share": item.expected_minutes_share,
                        "starting_probability": item.starting_probability,
                        "calculation_version": &item.calculation_version,
                        "components": &item.components,
                        "applied_tags": &item.applied_tags,
                    })
                })
                .collect::<Vec<_>>();
            let components = json!({
                "core_starters": score_component(core_starters),
                "natural_positions": score_component(natural_positions),
                "supply_chain": score_component(supply_chain),
                "role_certainty": score_component(role_certainty),
                "injury_uncertainty_risk": score_component(injury_risk),
                "fatigue_risk": score_component(fatigue_risk),
                "tactical_match": score_component(tactical_match),
                "bench_continuity": score_component(bench_continuity)
            });
            (
                json!({
                    "confidence": confidence,
                    "components": components,
                    "lineup_id": lineup.id,
                    "lineup_type": lineup.lineup_type.as_str(),
                    "snapshot_type": lineup.snapshot_type.clone(),
                    "formation": lineup.formation.clone(),
                    "formation_id": lineup.formation_id,
                    "coach_id": lineup.coach_id,
                    "player_count": lineup.player_count,
                    "starter_count": lineup.starter_count,
                    "player_contributions": player_contributions
                }),
                confidence,
                json!({
                    "lineup_present": true,
                    "lineup_id": lineup.id,
                    "lineup_type": lineup.lineup_type.as_str(),
                    "snapshot_type": lineup.snapshot_type.clone(),
                    "formation_id": lineup.formation_id,
                    "model_validation_status": lineup.model_validation_status.clone(),
                    "quality_score": quality_score,
                    "player_count": lineup.player_count,
                    "starter_count": lineup.starter_count,
                    "starter_coverage": coverage,
                    "average_player_confidence": average_confidence,
                    "role_certainty_score": role_certainty,
                    "inherited_role_count": inherited_role_count,
                    "overridden_role_count": overridden_role_count,
                    "missing_role_count": missing_role_count
                }),
            )
        } else {
            (
                json!({"confidence": 0.0, "components": {}}),
                0.0,
                json!({
                    "lineup_present": false,
                    "warning": "截止时间之前没有有效的预计或确认阵容"
                }),
            )
        };

        let confidence = combine_confidences(
            team_features.rating_confidence,
            lineup_confidence,
            team_features.venue_confidence,
        );
        let has_team_history = team_features
            .quality
            .get("history_match_count")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            > 0;
        let input = json!({
            "team_id": team_id,
            "name": team_name,
            "attack_score": team_features.attack_score,
            "defence_score": team_features.defence_score,
            "rating_confidence": team_features.rating_confidence,
            "venue_score": team_features.venue_score,
            "venue_confidence": team_features.venue_confidence,
            "data_confidence": confidence,
            "lineup": lineup_payload,
            "history": team_features.history.clone(),
            "path": {},
            "lcs": {"score": 50.0, "confidence": 0.0},
            "cpn": {"score": 50.0, "confidence": 0.0},
            "complexity": {"score": 50.0, "confidence": 0.0},
            "state": {},
            "evidence": team_features.evidence.clone()
        });
        Ok(PreparedTeam {
            input,
            confidence,
            has_team_history,
            quality: json!({
                "overall_confidence": confidence,
                "team_features": team_features.quality,
                "lineup": lineup_quality
            }),
        })
    }
}

struct PreparedTeam {
    input: Value,
    confidence: f64,
    has_team_history: bool,
    quality: Value,
}

fn combine_confidences(rating: f64, lineup: f64, venue: f64) -> f64 {
    // 缺失维度按 0 参与，避免仅凭一类高质量数据把整体质量抬高到同等水平。
    (0.45 * rating.clamp(0.0, 1.0) + 0.40 * lineup.clamp(0.0, 1.0) + 0.15 * venue.clamp(0.0, 1.0))
        .clamp(0.0, 1.0)
}

fn score_component(score: f64) -> Value {
    json!({"score": score.clamp(0.0, 100.0), "evidence_ids": []})
}

fn component_score(contribution: &PlayerMatchContribution, code: &str) -> f64 {
    contribution
        .components
        .iter()
        .find(|item| item.code == code)
        .map(|item| {
            if item.source == "default" {
                50.0
            } else {
                (50.0 + (item.value - 1.0) * 100.0).clamp(0.0, 100.0)
            }
        })
        .unwrap_or(50.0)
}

fn risk_score(contribution: &PlayerMatchContribution, code: &str) -> f64 {
    contribution
        .components
        .iter()
        .find(|item| item.code == code)
        .map(|item| {
            if item.source == "default" || item.source == "unknown" {
                50.0
            } else {
                ((1.0 - item.value) * 100.0).clamp(0.0, 100.0)
            }
        })
        .unwrap_or(50.0)
}

fn average(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut total = 0.0;
    let mut count = 0usize;
    for value in values {
        total += value;
        count += 1;
    }
    (count > 0).then_some(total / count as f64)
}

fn parse_kind(value: &str) -> PersistenceResult<CompetitionKind> {
    match value {
        "league" => Ok(CompetitionKind::League),
        "group_stage" => Ok(CompetitionKind::GroupStage),
        "knockout_single_leg" => Ok(CompetitionKind::KnockoutSingleLeg),
        "knockout_two_leg" => Ok(CompetitionKind::KnockoutTwoLeg),
        "friendly" => Ok(CompetitionKind::Friendly),
        "custom" => Ok(CompetitionKind::Custom),
        other => Err(PersistenceError::InvalidState(format!(
            "未知赛事类型：{other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::combine_confidences;

    #[test]
    fn missing_quality_dimensions_are_not_ignored() {
        assert!((combine_confidences(0.0, 0.8, 0.0) - 0.32).abs() < 1e-12);
        assert!((combine_confidences(1.0, 1.0, 1.0) - 1.0).abs() < 1e-12);
    }
}
