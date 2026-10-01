use super::{
    mapping::{
        match_parse_mode as parse_mode, match_row_from_db as row_from_db,
        player_import_row_from_row as import_row_from_row,
        player_parse_import_mode as parse_import_mode,
    },
    rows::count_preview_rows,
};
use crate::{PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::SpreadsheetImportPreview;
use sqlx::Row;
use uuid::Uuid;
const IMPORT_TYPE: &str = "match_lineup_xlsx";
impl PostgresStore {
    pub async fn read_spreadsheet_import_preview(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let batch = sqlx::query(
            r#"
            SELECT source_file_name, source_sha256, import_mode, started_at
            FROM catalog.import_batches
            WHERE id = $1 AND import_type IN ('player_catalog_xlsx','player_monthly_xlsx')
            "#,
        )
        .bind(batch_id)
        .fetch_one(&self.pool)
        .await?;
        let rows = sqlx::query(
            r#"
            SELECT id, sheet_name, row_number, entity_type, requested_action,
                   status, message, payload, matched_entity_id, conflict_candidates
            FROM catalog.import_rows
            WHERE batch_id = $1
            ORDER BY row_number, sheet_name, id
            "#,
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(import_row_from_row)
        .collect::<PersistenceResult<Vec<_>>>()?;
        let counts = count_preview_rows(&rows);
        Ok(SpreadsheetImportPreview {
            batch_id,
            source_file_name: batch
                .try_get::<Option<String>, _>("source_file_name")?
                .unwrap_or_default(),
            source_sha256: batch
                .try_get::<Option<String>, _>("source_sha256")?
                .unwrap_or_default(),
            import_mode: parse_import_mode(
                batch
                    .try_get::<Option<String>, _>("import_mode")?
                    .as_deref(),
            )?,
            counts,
            rows,
            created_at: batch
                .try_get::<Option<DateTime<Utc>>, _>("started_at")?
                .unwrap_or_else(Utc::now),
        })
    }
    pub async fn read_match_lineup_import_preview(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let batch = sqlx::query(
            "SELECT source_file_name, source_sha256, import_mode, started_at FROM catalog.import_batches WHERE id=$1 AND import_type=$2",
        ).bind(batch_id).bind(IMPORT_TYPE).fetch_one(&self.pool).await?;
        let rows = sqlx::query(
            "SELECT id,sheet_name,row_number,entity_type,requested_action,status,message,payload,matched_entity_id,conflict_candidates FROM catalog.import_rows WHERE batch_id=$1 ORDER BY row_number,sheet_name,id",
        ).bind(batch_id).fetch_all(&self.pool).await?.iter().map(row_from_db).collect::<PersistenceResult<Vec<_>>>()?;
        Ok(SpreadsheetImportPreview {
            batch_id,
            source_file_name: batch
                .try_get::<Option<String>, _>("source_file_name")?
                .unwrap_or_default(),
            source_sha256: batch
                .try_get::<Option<String>, _>("source_sha256")?
                .unwrap_or_default(),
            import_mode: parse_mode(
                batch
                    .try_get::<Option<String>, _>("import_mode")?
                    .as_deref(),
            )?,
            counts: count_preview_rows(&rows),
            rows,
            created_at: batch
                .try_get::<Option<DateTime<Utc>>, _>("started_at")?
                .unwrap_or_else(Utc::now),
        })
    }
}

impl PostgresStore {
    pub async fn read_team_monthly_import_preview(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let batch = sqlx::query(
            "SELECT source_file_name,source_sha256,import_mode,started_at FROM catalog.import_batches WHERE id=$1 AND import_type=$2",
        ).bind(batch_id).bind("team_monthly_xlsx").fetch_one(&self.pool).await?;
        let rows = sqlx::query(
            r#"SELECT id,sheet_name,row_number,entity_type,requested_action,status,message,payload,matched_entity_id,conflict_candidates
               FROM catalog.import_rows WHERE batch_id=$1 ORDER BY
               CASE entity_type WHEN 'team' THEN 0 WHEN 'coach' THEN 1 WHEN 'team_name' THEN 2
                    WHEN 'team_coach_period' THEN 3 WHEN 'formation_usage' THEN 4 ELSE 5 END,
               row_number, id"#,
        ).bind(batch_id).fetch_all(&self.pool).await?
        .iter().map(super::mapping::team_import_row_from_db).collect::<PersistenceResult<Vec<_>>>()?;
        Ok(SpreadsheetImportPreview {
            batch_id,
            source_file_name: batch
                .try_get::<Option<String>, _>("source_file_name")?
                .unwrap_or_default(),
            source_sha256: batch
                .try_get::<Option<String>, _>("source_sha256")?
                .unwrap_or_default(),
            import_mode: super::mapping::team_parse_import_mode(
                batch
                    .try_get::<Option<String>, _>("import_mode")?
                    .as_deref(),
            )?,
            counts: count_preview_rows(&rows),
            rows,
            created_at: batch
                .try_get::<Option<DateTime<Utc>>, _>("started_at")?
                .unwrap_or_else(Utc::now),
        })
    }
}
