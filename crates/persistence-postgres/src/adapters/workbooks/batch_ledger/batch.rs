use crate::{PersistenceError, PersistenceResult};
use football_domain::{
    SpreadsheetImportCommitResult, SpreadsheetImportCounts, SpreadsheetParsedWorkbook,
    PLAYER_MONTHLY_FORMAT,
};
use serde_json::{json, Value};
use sqlx::{postgres::PgRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Clone, Copy)]
pub(crate) enum ImportFamily {
    Player,
    Match,
    Team,
}
impl ImportFamily {
    fn import_types(self) -> Vec<&'static str> {
        match self {
            Self::Player => vec!["player_catalog_xlsx", "player_monthly_xlsx"],
            Self::Match => vec!["match_lineup_xlsx"],
            Self::Team => vec!["team_monthly_xlsx"],
        }
    }
}
pub(crate) fn require_pending(status: &str, message: String) -> PersistenceResult<()> {
    if status == "pending" {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(message))
    }
}
pub(crate) async fn lock_batch_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    family: ImportFamily,
) -> PersistenceResult<PgRow> {
    Ok(sqlx::query("SELECT status,import_mode,inserted_count,updated_count,ended_previous_count,skipped_count,error_count,finished_at FROM catalog.import_batches WHERE id=$1 AND import_type=ANY($2) FOR UPDATE")
        .bind(batch_id).bind(family.import_types()).fetch_one(&mut **tx).await?)
}
pub(crate) async fn start_batch_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
) -> PersistenceResult<()> {
    sqlx::query("UPDATE catalog.import_batches SET status='running' WHERE id=$1")
        .bind(batch_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
pub(crate) async fn find_player_batch(
    pool: &PgPool,
    source_sha256: &str,
    import_type: &str,
) -> PersistenceResult<Option<PgRow>> {
    Ok(sqlx::query("SELECT id,status FROM catalog.import_batches WHERE source_sha256=$1 AND import_type=$2 AND status IN ('pending','running','succeeded') ORDER BY started_at DESC NULLS LAST LIMIT 1")
        .bind(source_sha256).bind(import_type).fetch_optional(pool).await?)
}
pub(crate) async fn cancel_pending_player_batch(
    pool: &PgPool,
    batch_id: Uuid,
    metadata: Value,
) -> PersistenceResult<()> {
    sqlx::query("UPDATE catalog.import_batches SET status='cancelled',finished_at=now(),metadata=metadata||$2 WHERE id=$1 AND status='pending'")
        .bind(batch_id).bind(metadata).execute(pool).await?;
    Ok(())
}

pub(crate) async fn create_player_batch_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    parsed: &SpreadsheetParsedWorkbook,
    counts: &SpreadsheetImportCounts,
    import_type: &str,
    mode_text: &str,
) -> PersistenceResult<()> {
    sqlx::query(
            r#"
            INSERT INTO catalog.import_batches (
                id, import_type, workbook_kind, format_version, status, source_file_name, source_sha256,
                import_mode, started_at, skipped_count, error_count, metadata
            ) VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, now(), $8, $9, $10)
            "#,
        )
        .bind(batch_id)
        .bind(import_type)
        .bind(if parsed.format_version == PLAYER_MONTHLY_FORMAT { "player_monthly" } else { "legacy_player" })
        .bind(&parsed.format_version)
        .bind(&parsed.source_file_name)
        .bind(&parsed.source_sha256)
        .bind(mode_text)
        .bind(counts.skipped as i64)
        .bind((counts.error + counts.conflict) as i64)
        .bind(json!({
            "format_version": parsed.format_version,
            "preview_counts": &counts,
        }))
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(crate) async fn create_match_batch_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    parsed: &SpreadsheetParsedWorkbook,
    counts: &SpreadsheetImportCounts,
    mode: football_domain::SpreadsheetImportMode,
) -> PersistenceResult<()> {
    sqlx::query(
        r#"
            INSERT INTO catalog.import_batches (
                id, import_type, status, source_file_name, source_sha256,
                import_mode, started_at, skipped_count, error_count, metadata
            ) VALUES ($1, $2, 'pending', $3, $4, $5, now(), $6, $7, $8)
            "#,
    )
    .bind(batch_id)
    .bind("match_lineup_xlsx")
    .bind(&parsed.source_file_name)
    .bind(&parsed.source_sha256)
    .bind(super::mapping::match_mode_text(mode))
    .bind(counts.skipped as i64)
    .bind((counts.error + counts.conflict) as i64)
    .bind(json!({"format_version": parsed.format_version, "preview_counts": counts}))
    .execute(&mut **tx)
    .await
    .map_err(map_duplicate_import)?;
    Ok(())
}

pub(crate) fn map_duplicate_import(error: sqlx::Error) -> PersistenceError {
    match error {
        sqlx::Error::Database(database) if database.is_unique_violation() => {
            PersistenceError::InvalidState("该文件已经存在进行中或已完成的导入批次".to_string())
        }
        other => PersistenceError::Sqlx(other),
    }
}

// 使用同一个结果对象更新账本和审计，不能另开或提交业务事务。
pub(crate) async fn finish_batch_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    result: &SpreadsheetImportCommitResult,
    family: ImportFamily,
) -> PersistenceResult<()> {
    sqlx::query("UPDATE catalog.import_batches SET status='succeeded',finished_at=$2,inserted_count=$3,updated_count=$4,ended_previous_count=$5,skipped_count=$6,error_count=$7 WHERE id=$1")
        .bind(result.batch_id).bind(result.finished_at).bind(result.inserted_count as i64).bind(result.updated_count as i64).bind(result.ended_previous_count as i64).bind(result.skipped_count as i64).bind(result.error_count as i64).execute(&mut **tx).await?;
    let (event, payload) = match family {
        ImportFamily::Team => (
            "team_monthly_workbook_imported",
            json!({"inserted":result.inserted_count,"updated":result.updated_count,"ended_previous":result.ended_previous_count,"skipped":result.skipped_count}),
        ),
        ImportFamily::Player => (
            "spreadsheet_import_committed",
            json!({"inserted":result.inserted_count,"updated":result.updated_count,"skipped":result.skipped_count}),
        ),
        ImportFamily::Match => (
            "match_lineup_import_committed",
            json!({"inserted":result.inserted_count,"updated":result.updated_count,"ended_previous":result.ended_previous_count,"skipped":result.skipped_count}),
        ),
    };
    crate::write_audit_event(
        tx,
        event,
        "import_batch",
        result.batch_id.to_string(),
        payload,
    )
    .await
}

pub(crate) async fn find_team_batch(
    pool: &PgPool,
    source_sha256: &str,
) -> PersistenceResult<Option<Uuid>> {
    Ok(sqlx::query_scalar("SELECT id FROM catalog.import_batches WHERE source_sha256=$1 AND import_type=$2 AND status IN ('pending','running','succeeded') ORDER BY started_at DESC NULLS LAST LIMIT 1").bind(source_sha256).bind("team_monthly_xlsx").fetch_optional(pool).await?)
}
pub(crate) async fn create_team_batch_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    parsed: &SpreadsheetParsedWorkbook,
    counts: &SpreadsheetImportCounts,
    mode_text: &str,
) -> PersistenceResult<()> {
    sqlx::query(
        r#"
            INSERT INTO catalog.import_batches (
                id, import_type, workbook_kind, format_version, status,
                source_file_name, source_sha256, import_mode, started_at,
                skipped_count, error_count, metadata
            ) VALUES ($1,$2,'team_monthly',$3,'pending',$4,$5,$6,now(),$7,$8,$9)
            "#,
    )
    .bind(batch_id)
    .bind("team_monthly_xlsx")
    .bind(&parsed.format_version)
    .bind(&parsed.source_file_name)
    .bind(&parsed.source_sha256)
    .bind(mode_text)
    .bind(counts.skipped as i64)
    .bind((counts.error + counts.conflict) as i64)
    .bind(json!({"preview_counts": counts}))
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_pending_batches_can_be_changed_and_rejection_text_is_preserved() {
        assert!(require_pending("pending", "不应返回".into()).is_ok());
        for status in ["running", "succeeded", "failed", "cancelled", "", "unknown"] {
            assert!(
                matches!(require_pending(status, format!("拒绝：{status}")), Err(PersistenceError::InvalidState(message)) if message == format!("拒绝：{status}"))
            );
        }
    }

    #[test]
    fn batch_family_lock_scope_keeps_player_and_match_imports_separate() {
        assert_eq!(
            ImportFamily::Player.import_types(),
            ["player_catalog_xlsx", "player_monthly_xlsx"]
        );
        assert_eq!(ImportFamily::Match.import_types(), ["match_lineup_xlsx"]);
        assert_eq!(ImportFamily::Team.import_types(), ["team_monthly_xlsx"]);
        for family in [ImportFamily::Player, ImportFamily::Match] {
            assert!(family
                .import_types()
                .iter()
                .all(|kind| !ImportFamily::Team.import_types().contains(kind)));
        }
        assert!(ImportFamily::Player
            .import_types()
            .iter()
            .all(|kind| !ImportFamily::Match.import_types().contains(kind)));
    }
}
