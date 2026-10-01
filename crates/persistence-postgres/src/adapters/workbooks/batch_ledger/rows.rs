use crate::PersistenceResult;
use football_domain::{SpreadsheetImportCounts, SpreadsheetImportRow, SpreadsheetRowStatus};
use sqlx::{postgres::PgRow, Postgres, Transaction};
use uuid::Uuid;
pub(crate) async fn insert_import_row(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    row: &SpreadsheetImportRow,
) -> PersistenceResult<()> {
    sqlx::query(
        r#"
        INSERT INTO catalog.import_rows (
            id, batch_id, sheet_name, row_number, entity_type,
            requested_action, status, message, payload, matched_entity_id,
            conflict_candidates
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        "#,
    )
    .bind(row.id)
    .bind(batch_id)
    .bind(&row.sheet_name)
    .bind(row.row_number as i32)
    .bind(row.entity_type.as_str())
    .bind(row.action.as_str())
    .bind(row.status.as_str())
    .bind(&row.message)
    .bind(&row.payload)
    .bind(row.matched_entity_id)
    .bind(serde_json::to_value(&row.conflict_candidates)?)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
pub(crate) fn count_preview_rows(rows: &[SpreadsheetImportRow]) -> SpreadsheetImportCounts {
    let mut c = SpreadsheetImportCounts {
        total: rows.len() as u64,
        ..Default::default()
    };
    for row in rows {
        match row.status {
            SpreadsheetRowStatus::ReadyAdd => c.ready_add += 1,
            SpreadsheetRowStatus::ReadyUpdate => c.ready_update += 1,
            SpreadsheetRowStatus::ReadyEndPrevious => c.ready_end_previous += 1,
            SpreadsheetRowStatus::Conflict => c.conflict += 1,
            SpreadsheetRowStatus::Error => c.error += 1,
            SpreadsheetRowStatus::Skip => c.skipped += 1,
            SpreadsheetRowStatus::Imported => c.imported += 1,
        }
    }
    c
}

pub(crate) async fn lock_import_row_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    row_id: Uuid,
) -> PersistenceResult<PgRow> {
    Ok(sqlx::query("SELECT entity_type,requested_action,status,payload,conflict_candidates FROM catalog.import_rows WHERE id=$1 AND batch_id=$2 FOR UPDATE")
        .bind(row_id).bind(batch_id).fetch_one(&mut **tx).await?)
}
pub(crate) async fn blocking_rows_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
) -> PersistenceResult<i64> {
    Ok(sqlx::query_scalar("SELECT COUNT(*)::bigint FROM catalog.import_rows WHERE batch_id=$1 AND status IN ('conflict','error')")
        .bind(batch_id).fetch_one(&mut **tx).await?)
}
pub(crate) async fn skipped_rows_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
) -> PersistenceResult<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM catalog.import_rows WHERE batch_id=$1 AND status='skip'",
    )
    .bind(batch_id)
    .fetch_one(&mut **tx)
    .await?)
}
pub(crate) async fn mark_imported_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    row_id: Uuid,
) -> PersistenceResult<()> {
    sqlx::query("UPDATE catalog.import_rows SET status='imported',imported_at=now() WHERE id=$1")
        .bind(row_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
// 行结果及批次暂存计数共用调用方事务；metadata.preview_counts 保留首次预检快照。
pub(crate) async fn refresh_pending_counts_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
) -> PersistenceResult<()> {
    sqlx::query("UPDATE catalog.import_batches SET skipped_count=(SELECT count(*) FROM catalog.import_rows WHERE batch_id=$1 AND status='skip'),error_count=(SELECT count(*) FROM catalog.import_rows WHERE batch_id=$1 AND status IN ('conflict','error')) WHERE id=$1 AND status='pending'")
        .bind(batch_id).execute(&mut **tx).await?;
    Ok(())
}
pub(crate) async fn skip_import_row_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    row_id: Uuid,
    message: &str,
) -> PersistenceResult<()> {
    sqlx::query("UPDATE catalog.import_rows SET status='skip',message=$2,matched_entity_id=NULL,conflict_candidates='[]'::jsonb WHERE id=$1 AND batch_id=$3")
        .bind(row_id).bind(message).bind(batch_id).execute(&mut **tx).await?;
    refresh_pending_counts_in_tx(tx, batch_id).await
}
pub(crate) struct ResolvedImportRow<'a> {
    pub(crate) id: Uuid,
    pub(crate) status: &'a str,
    pub(crate) message: Option<&'a str>,
    pub(crate) payload: &'a serde_json::Value,
    pub(crate) matched_entity_id: Option<Uuid>,
    pub(crate) candidates: &'a [football_domain::SpreadsheetConflictCandidate],
}
pub(crate) async fn resolve_import_row_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    row: ResolvedImportRow<'_>,
) -> PersistenceResult<()> {
    sqlx::query("UPDATE catalog.import_rows SET status=$2,message=$3,payload=$4,matched_entity_id=$5,conflict_candidates=$6 WHERE id=$1 AND batch_id=$7")
        .bind(row.id).bind(row.status).bind(row.message).bind(row.payload).bind(row.matched_entity_id).bind(serde_json::to_value(row.candidates)?).bind(batch_id).execute(&mut **tx).await?;
    refresh_pending_counts_in_tx(tx, batch_id).await
}
pub(crate) async fn commit_rows_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    family: super::batch::ImportFamily,
) -> PersistenceResult<Vec<PgRow>> {
    Ok(match family {
        super::batch::ImportFamily::Team => sqlx::query(
            r#"SELECT id,sheet_name,row_number,entity_type,requested_action,status,message,payload,matched_entity_id,conflict_candidates
               FROM catalog.import_rows WHERE batch_id=$1 ORDER BY
               CASE entity_type WHEN 'team' THEN 0 WHEN 'coach' THEN 1 WHEN 'team_name' THEN 2
                    WHEN 'team_coach_period' THEN 3 WHEN 'team_tactical_observation' THEN 5
                    WHEN 'team_ability_observation' THEN 6 ELSE 7 END,
               row_number,id FOR UPDATE"#,
        ).bind(batch_id).fetch_all(&mut **tx).await?,
        super::batch::ImportFamily::Player => sqlx::query(
            r#"
            SELECT id, entity_type, requested_action, status, payload, matched_entity_id
            FROM catalog.import_rows
            WHERE batch_id = $1 AND status IN ('ready_add', 'ready_update')
            ORDER BY CASE entity_type
                WHEN 'team' THEN 1 WHEN 'player' THEN 2 WHEN 'player_name' THEN 3
                WHEN 'player_position' THEN 4 WHEN 'player_team_period' THEN 5
                WHEN 'player_ability' THEN 6 WHEN 'player_availability' THEN 7
                WHEN 'external_entity_id' THEN 8 ELSE 99 END,
                row_number, id
            "#,
        )
        .bind(batch_id)
        .fetch_all(&mut **tx)
        .await?,
        super::batch::ImportFamily::Match => sqlx::query("SELECT id,entity_type,status,payload,matched_entity_id FROM catalog.import_rows WHERE batch_id=$1 AND status IN ('ready_add','ready_update','skip') ORDER BY CASE entity_type WHEN 'match' THEN 1 WHEN 'lineup' THEN 2 WHEN 'lineup_player' THEN 3 WHEN 'player_dynamic_tag' THEN 4 ELSE 9 END,row_number,id")
            .bind(batch_id).fetch_all(&mut **tx).await?,
    })
}

// 调用方已持有批次及所选行锁；保留原规范化/身份合并消息和物理行身份。
pub(crate) async fn set_import_payload_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    row_id: Uuid,
    payload: &serde_json::Value,
    message: Option<&str>,
) -> PersistenceResult<()> {
    sqlx::query(
        "UPDATE catalog.import_rows SET payload=$2,message=COALESCE($3,message) WHERE id=$1",
    )
    .bind(row_id)
    .bind(payload)
    .bind(message)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
pub(crate) async fn skip_duplicate_import_row_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    row_id: Uuid,
    message: &str,
) -> PersistenceResult<()> {
    sqlx::query("UPDATE catalog.import_rows SET status='skip',message=$2,matched_entity_id=NULL,conflict_candidates='[]'::jsonb WHERE id=$1").bind(row_id).bind(message).execute(&mut **tx).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::{SpreadsheetAction, SpreadsheetEntityType, SpreadsheetRowStatus};
    use serde_json::json;

    #[test]
    fn preview_counts_cover_every_row_state_and_keep_total_conserved() {
        let statuses = [
            SpreadsheetRowStatus::ReadyAdd,
            SpreadsheetRowStatus::ReadyUpdate,
            SpreadsheetRowStatus::ReadyEndPrevious,
            SpreadsheetRowStatus::Conflict,
            SpreadsheetRowStatus::Error,
            SpreadsheetRowStatus::Skip,
            SpreadsheetRowStatus::Imported,
        ];
        let rows: Vec<_> = statuses
            .iter()
            .enumerate()
            .map(|(index, status)| SpreadsheetImportRow {
                id: Uuid::from_u128(index as u128 + 1),
                sheet_name: "原工作表".into(),
                row_number: index as u32 + 2,
                entity_type: SpreadsheetEntityType::Player,
                action: SpreadsheetAction::Add,
                status: *status,
                message: None,
                payload: json!({}),
                matched_entity_id: None,
                conflict_candidates: vec![],
            })
            .collect();
        let counts = count_preview_rows(&rows);
        assert_eq!(counts.total, 7);
        assert_eq!(
            [
                counts.ready_add,
                counts.ready_update,
                counts.ready_end_previous,
                counts.conflict,
                counts.error,
                counts.skipped,
                counts.imported
            ],
            [1; 7]
        );
        assert_eq!(
            counts.total,
            counts.ready_add
                + counts.ready_update
                + counts.ready_end_previous
                + counts.conflict
                + counts.error
                + counts.skipped
                + counts.imported
        );
        let empty = count_preview_rows(&[]);
        assert_eq!(
            [
                empty.total,
                empty.ready_add,
                empty.ready_update,
                empty.ready_end_previous,
                empty.conflict,
                empty.error,
                empty.skipped,
                empty.imported
            ],
            [0; 8]
        );
    }
}
