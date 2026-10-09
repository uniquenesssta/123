use super::input::PreparedSnapshot;
use crate::{sha256_json, PersistenceResult};
use serde_json::Value;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn write_snapshot_details(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    prepared: &PreparedSnapshot,
) -> PersistenceResult<()> {
    for feature in &prepared.features {
        let value_sha256 = sha256_json(&feature.value)?;
        sqlx::query(
            r#"
                INSERT INTO feature.snapshot_features (
                    snapshot_id, field_order, field_key, value,
                    verification_state, evidence_ids, value_sha256, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
        )
        .bind(id)
        .bind(i16::from(feature.field_order))
        .bind(&feature.field_key)
        .bind(&feature.value)
        .bind(feature.verification_state.as_str())
        .bind(&feature.evidence_ids)
        .bind(value_sha256)
        .bind(&feature.metadata)
        .execute(&mut **tx)
        .await?;
        for evidence_id in &feature.evidence_ids {
            sqlx::query(
                r#"
                    INSERT INTO feature.snapshot_evidence (snapshot_id, field_key, evidence_id)
                    VALUES ($1, $2, $3)
                    "#,
            )
            .bind(id)
            .bind(&feature.field_key)
            .bind(evidence_id)
            .execute(&mut **tx)
            .await?;
        }
    }

    for probability in &prepared.probabilities {
        let is_formal = probability
            .metadata
            .get("formal")
            .and_then(Value::as_bool)
            .unwrap_or(probability.chain_key == "full");
        let shadow_status: Option<&str> = None;
        sqlx::query(
            r#"
                INSERT INTO model.snapshot_probabilities (
                    snapshot_id, chain_key, home_win, draw, away_win,
                    btts, over_2_5, clean_sheet_home, clean_sheet_away,
                    matrix_sha256, matrix_cell_count, is_formal, shadow_status, metadata
                ) VALUES (
                    $1, $2, $3, $4, $5,
                    $6, $7, $8, $9,
                    $10, $11, $12, $13, $14
                )
                "#,
        )
        .bind(id)
        .bind(&probability.chain_key)
        .bind(probability.home_win)
        .bind(probability.draw)
        .bind(probability.away_win)
        .bind(probability.btts)
        .bind(probability.over_2_5)
        .bind(probability.clean_sheet_home)
        .bind(probability.clean_sheet_away)
        .bind(&probability.matrix_sha256)
        .bind(i32::from(probability.matrix_cell_count))
        .bind(is_formal)
        .bind(shadow_status)
        .bind(&probability.metadata)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}
