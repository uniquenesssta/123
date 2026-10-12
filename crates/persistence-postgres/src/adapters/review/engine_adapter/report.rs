use crate::PersistenceResult;
use chrono::{Duration, Utc};
use football_domain::{CalculatedMatchReview, MatchReviewDraft, ReviewPreparationData};
use serde_json::{json, Value};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn persist_calculation_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    draft: &MatchReviewDraft,
    preparation: &ReviewPreparationData,
    calculation: &CalculatedMatchReview,
) -> PersistenceResult<Uuid> {
    sqlx::query(
        r#"
        WITH superseded AS (
            UPDATE review.ability_update_candidates candidate
            SET status = 'superseded', decided_at = now(),
                decided_by = 'system', decision_note = '同场比赛生成了新的复盘版本'
            WHERE candidate.status = 'pending'
              AND candidate.match_review_id IN (
                  SELECT id FROM review.match_reviews
                  WHERE match_id = $1 AND status = 'finalized'
              )
            RETURNING candidate.id, candidate.current_value, candidate.proposed_value
        )
        INSERT INTO review.ability_update_decisions (
            id, candidate_id, decision, previous_value, proposed_value,
            applied_observation_id, decided_by, decision_note
        )
        SELECT gen_random_uuid(), superseded.id, 'superseded',
               superseded.current_value, superseded.proposed_value,
               NULL, 'system', '同场比赛生成了新的复盘版本'
        FROM superseded
        "#,
    )
    .bind(draft.match_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "UPDATE review.match_reviews SET status='superseded' WHERE match_id=$1 AND status='finalized'",
    )
    .bind(draft.match_id)
    .execute(&mut **tx)
    .await?;

    let review_id = Uuid::new_v4();
    let review_version = draft.review_version.clone().unwrap_or_else(|| {
        format!(
            "review-{}-{}",
            Utc::now().format("%Y%m%d%H%M%S%.3f"),
            review_id.simple()
        )
    });
    let result_snapshot = serde_json::to_value(&preparation.result)?;
    let mut conclusions = calculation.conclusions.clone();
    if let (Some(notes), Some(object)) = (draft.notes.as_deref(), conclusions.as_object_mut()) {
        if !notes.trim().is_empty() {
            object.insert("notes".to_string(), Value::String(notes.trim().to_string()));
        }
    }
    sqlx::query(
        r#"
        INSERT INTO review.match_reviews (
            id, match_id, review_version, data_coverage, conclusions,
            source_run_id, status, calculation_version,
            result_snapshot, substitutions_snapshot, prediction_evaluation, finalized_at
        ) VALUES ($1,$2,$3,$4,$5,$6,'finalized',$7,$8,$9,$10,now())
        "#,
    )
    .bind(review_id)
    .bind(draft.match_id)
    .bind(&review_version)
    .bind(draft.data_coverage)
    .bind(&conclusions)
    .bind(preparation.prediction.as_ref().map(|item| item.run_id))
    .bind(&calculation.calculation_version)
    .bind(result_snapshot)
    .bind(serde_json::to_value(&preparation.substitutions)?)
    .bind(&calculation.prediction_evaluation)
    .execute(&mut **tx)
    .await?;

    for player in &calculation.player_reviews {
        let player_review_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO review.player_match_reviews (
                id, match_review_id, player_id, team_id, role_code,
                started, minutes_played, expected_performance,
                actual_performance, realization_ratio, confidence, metrics,
                observation_id, entry_type, contribution_weight,
                ability_candidate_count
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            "#,
        )
        .bind(player_review_id)
        .bind(review_id)
        .bind(player.player_id)
        .bind(player.team_id)
        .bind(player.role_code.as_deref())
        .bind(player.started)
        .bind(player.minutes_played)
        .bind(player.expected_performance)
        .bind(player.actual_performance)
        .bind(player.realization_ratio)
        .bind(player.confidence)
        .bind(&player.metrics)
        .bind(player.observation_id)
        .bind(&player.entry_type)
        .bind(player.contribution_weight)
        .bind(player.ability_candidates.len() as i32)
        .execute(&mut **tx)
        .await?;

        for candidate in &player.ability_candidates {
            sqlx::query(
                r#"
                INSERT INTO review.ability_update_candidates (
                    id, player_id, dimension_code, current_value,
                    proposed_value, confidence, sample_size, evidence,
                    calculation_version, status, match_review_id,
                    player_match_review_id
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'pending',$10,$11)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(candidate.player_id)
            .bind(&candidate.dimension_code)
            .bind(candidate.current_value)
            .bind(candidate.proposed_value)
            .bind(candidate.confidence)
            .bind(candidate.sample_size)
            .bind(&candidate.evidence)
            .bind(&calculation.calculation_version)
            .bind(review_id)
            .bind(player_review_id)
            .execute(&mut **tx)
            .await?;
        }
    }

    for team in &calculation.team_reviews {
        sqlx::query(
            r#"
            INSERT INTO review.team_match_reviews (
                id, match_review_id, team_id, chemistry_score,
                bench_strength, substitution_impact, realization_score,
                confidence, metrics, lineup_continuity,
                performance_cohesion, bench_dropoff, substitute_count
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(review_id)
        .bind(team.team_id)
        .bind(team.chemistry_score)
        .bind(team.bench_strength)
        .bind(team.substitution_impact)
        .bind(team.realization_score)
        .bind(team.confidence)
        .bind(&team.metrics)
        .bind(team.lineup_continuity)
        .bind(team.performance_cohesion)
        .bind(team.bench_dropoff)
        .bind(team.substitute_count)
        .execute(&mut **tx)
        .await?;
    }

    let observed_at = preparation.result.finalized_at;
    let valid_to = observed_at + Duration::days(14);
    for player in calculation
        .player_reviews
        .iter()
        .filter(|item| item.minutes_played > 0)
    {
        let position_code = preparation
            .players
            .iter()
            .find(|item| item.player_id == player.player_id)
            .and_then(|item| item.position_code.as_deref());
        let chemistry_score = calculation
            .team_reviews
            .iter()
            .find(|item| item.team_id == player.team_id)
            .map(|item| item.chemistry_score)
            .unwrap_or(50.0);
        let dynamic_values = [
            (
                "realization_multiplier",
                player.realization_ratio.clamp(0.50, 1.25),
                json!({"realization_ratio": player.realization_ratio}),
            ),
            (
                "chemistry_fit",
                (0.50 + chemistry_score.clamp(0.0, 100.0) * 0.006).clamp(0.50, 1.10),
                json!({"team_chemistry_score": chemistry_score}),
            ),
        ];
        for (tag_code, value, evidence) in dynamic_values {
            sqlx::query(
                r#"
                INSERT INTO feature.player_dynamic_tags (
                    id, player_id, tag_code, value, label, confidence,
                    observed_at, valid_from, valid_to, competition_id,
                    position_code, opponent_team_id, sample_size, source_type,
                    source_document_id, calculation_version, metadata
                ) VALUES (
                    $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,NULL,1,
                    'match_review',NULL,$12,$13
                )
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(player.player_id)
            .bind(tag_code)
            .bind(value)
            .bind(if tag_code == "realization_multiplier" {
                "赛后兑现率"
            } else {
                "赛后配合度"
            })
            .bind(player.confidence)
            .bind(observed_at)
            .bind(observed_at)
            .bind(valid_to)
            .bind(preparation.match_record.competition_id)
            .bind(position_code)
            .bind(&calculation.calculation_version)
            .bind(json!({
                "match_review_id": review_id,
                "match_id": draft.match_id,
                "team_id": player.team_id,
                "evidence": evidence,
            }))
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(review_id)
}
