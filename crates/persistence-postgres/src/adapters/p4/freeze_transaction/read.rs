use crate::adapters::p4::{
    evidence_ledger::parse_verification_state, idempotency::ensure_idempotent_fingerprint,
};
use crate::p4_records::parse_horizon;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    PrematchSnapshotBundle, PrematchSnapshotRecord, SnapshotFeatureDraft, SnapshotProbabilityDraft,
    SnapshotSourceKind,
};
use serde_json::Value;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

impl PostgresStore {
    pub async fn read_prematch_snapshot(
        &self,
        snapshot_id: Uuid,
    ) -> PersistenceResult<PrematchSnapshotBundle> {
        let mut tx = self.pool.begin().await?;
        let snapshot = snapshot_record_by_id(&mut tx, snapshot_id, false).await?;
        let input_payload: Value =
            sqlx::query_scalar("SELECT input_payload FROM feature.snapshots WHERE id = $1")
                .bind(snapshot_id)
                .fetch_one(&mut *tx)
                .await?;
        let feature_rows = sqlx::query(
            r#"
            SELECT field_order, field_key, value, verification_state, evidence_ids, metadata
            FROM feature.snapshot_features
            WHERE snapshot_id = $1
            ORDER BY field_order
            "#,
        )
        .bind(snapshot_id)
        .fetch_all(&mut *tx)
        .await?;
        let features = feature_rows
            .iter()
            .map(snapshot_feature_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        let probability_rows = sqlx::query(
            r#"
            SELECT chain_key, home_win, draw, away_win, btts, over_2_5,
                   clean_sheet_home, clean_sheet_away, matrix_sha256,
                   matrix_cell_count, metadata
            FROM model.snapshot_probabilities
            WHERE snapshot_id = $1
            ORDER BY chain_key
            "#,
        )
        .bind(snapshot_id)
        .fetch_all(&mut *tx)
        .await?;
        let probabilities = probability_rows
            .iter()
            .map(snapshot_probability_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        tx.commit().await?;
        Ok(PrematchSnapshotBundle {
            snapshot,
            input_payload,
            features,
            probabilities,
        })
    }
}

pub(super) async fn existing_snapshot_by_idempotency(
    tx: &mut Transaction<'_, Postgres>,
    idempotency_key: &str,
    expected_fingerprint: &str,
) -> PersistenceResult<Option<PrematchSnapshotRecord>> {
    let row = sqlx::query(
        r#"
        SELECT id, snapshot_fingerprint
        FROM feature.snapshots
        WHERE idempotency_key = $1
        "#,
    )
    .bind(idempotency_key)
    .fetch_optional(&mut **tx)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let existing: Option<String> = row.try_get("snapshot_fingerprint")?;
    ensure_idempotent_fingerprint(
        "赛前快照",
        idempotency_key,
        existing.as_deref().unwrap_or_default(),
        expected_fingerprint,
    )?;
    let id: Uuid = row.try_get("id")?;
    snapshot_record_by_id(tx, id, false).await.map(Some)
}

pub(super) async fn snapshot_record_by_id(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    created: bool,
) -> PersistenceResult<PrematchSnapshotRecord> {
    let row = sqlx::query(
        r#"
        SELECT id, match_id, match_key, snapshot_type, data_cutoff_time, frozen_at,
               snapshot_fingerprint, idempotency_key, source_kind, evidence_scope, created_at
        FROM feature.snapshots
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&mut **tx)
    .await?;
    let match_id = row
        .try_get::<Option<Uuid>, _>("match_id")?
        .ok_or_else(|| PersistenceError::InvalidState("P4赛前快照缺少比赛关联".to_string()))?;
    Ok(PrematchSnapshotRecord {
        id: row.try_get("id")?,
        match_id,
        match_key: row.try_get("match_key")?,
        horizon: parse_horizon(row.try_get::<String, _>("snapshot_type")?.as_str())?,
        data_cutoff_at: row.try_get("data_cutoff_time")?,
        frozen_at: row.try_get("frozen_at")?,
        snapshot_fingerprint: row
            .try_get::<Option<String>, _>("snapshot_fingerprint")?
            .ok_or_else(|| PersistenceError::InvalidState("P4赛前快照缺少指纹".to_string()))?,
        idempotency_key: row
            .try_get::<Option<String>, _>("idempotency_key")?
            .ok_or_else(|| PersistenceError::InvalidState("P4赛前快照缺少幂等键".to_string()))?,
        source_kind: parse_source_kind(row.try_get::<String, _>("source_kind")?.as_str())?,
        evidence_scope: row.try_get("evidence_scope")?,
        created,
        created_at: row.try_get("created_at")?,
    })
}
fn snapshot_feature_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<SnapshotFeatureDraft> {
    let field_order: i16 = row.try_get("field_order")?;
    Ok(SnapshotFeatureDraft {
        field_order: u8::try_from(field_order)
            .map_err(|_| PersistenceError::InvalidState("快照字段顺序超出u8范围".to_string()))?,
        field_key: row.try_get("field_key")?,
        value: row.try_get("value")?,
        verification_state: parse_verification_state(
            row.try_get::<String, _>("verification_state")?.as_str(),
        )?,
        evidence_ids: row.try_get("evidence_ids")?,
        metadata: row.try_get("metadata")?,
    })
}

fn snapshot_probability_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<SnapshotProbabilityDraft> {
    let matrix_cell_count: i32 = row.try_get("matrix_cell_count")?;
    Ok(SnapshotProbabilityDraft {
        chain_key: row.try_get("chain_key")?,
        home_win: row.try_get("home_win")?,
        draw: row.try_get("draw")?,
        away_win: row.try_get("away_win")?,
        btts: row.try_get("btts")?,
        over_2_5: row.try_get("over_2_5")?,
        clean_sheet_home: row.try_get("clean_sheet_home")?,
        clean_sheet_away: row.try_get("clean_sheet_away")?,
        matrix_sha256: row.try_get("matrix_sha256")?,
        matrix_cell_count: u16::try_from(matrix_cell_count)
            .map_err(|_| PersistenceError::InvalidState("矩阵单元数超出u16范围".to_string()))?,
        metadata: row.try_get("metadata")?,
    })
}
fn parse_source_kind(value: &str) -> PersistenceResult<SnapshotSourceKind> {
    match value {
        "real" => Ok(SnapshotSourceKind::Real),
        "manual" => Ok(SnapshotSourceKind::Manual),
        "synthetic_fixture" => Ok(SnapshotSourceKind::SyntheticFixture),
        other => Err(PersistenceError::InvalidState(format!(
            "记录不是P4正式快照来源：{other}"
        ))),
    }
}
