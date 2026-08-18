use crate::{
    adapters::catalog::coaches::{
        existence::ensure_coach_exists,
        mapping::coach_name_from_row,
        normalization::{normalize_name, trim_option},
        validation::validate_date_range,
    },
    write_audit_event, PersistenceError, PersistenceResult, PostgresStore,
};
use football_domain::{CoachNameDraft, CoachNameRecord};
use serde_json::json;
use uuid::Uuid;
impl PostgresStore {
    pub async fn add_coach_name(
        &self,
        draft: &CoachNameDraft,
    ) -> PersistenceResult<CoachNameRecord> {
        let name = draft.name.trim();
        if name.is_empty() {
            return Err(PersistenceError::InvalidState(
                "教练名称不能为空".to_string(),
            ));
        }
        validate_date_range(draft.valid_from, draft.valid_to, "教练名称")?;
        let mut tx = self.pool.begin().await?;
        ensure_coach_exists(&mut tx, draft.coach_id).await?;
        if draft.is_primary {
            sqlx::query("UPDATE football.coach_names SET is_primary=false WHERE coach_id=$1")
                .bind(draft.coach_id)
                .execute(&mut *tx)
                .await?;
        }
        let row = sqlx::query(
            r#"
                INSERT INTO football.coach_names (
                    id, coach_id, name, normalized_name, language_code, is_primary,
                    valid_from, valid_to
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
                RETURNING id, coach_id, name, normalized_name, language_code, is_primary,
                          valid_from, valid_to
                "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.coach_id)
        .bind(name)
        .bind(normalize_name(name))
        .bind(trim_option(&draft.language_code))
        .bind(draft.is_primary)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .fetch_one(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "coach_name_added",
            "coach",
            Some(draft.coach_id.to_string()),
            json!({"name": name, "is_primary": draft.is_primary}),
        )
        .await?;
        tx.commit().await?;
        coach_name_from_row(&row)
    }
}
