use crate::{PersistenceError, PersistenceResult};
use football_domain::EvidenceClaimDraft;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

pub(super) async fn validate_evidence_version_references(
    tx: &mut Transaction<'_, Postgres>,
    draft: &EvidenceClaimDraft,
) -> PersistenceResult<()> {
    let registered_schema_version: Option<String> =
        sqlx::query_scalar("SELECT version FROM research.schema_versions WHERE id = $1")
            .bind(draft.schema_version_id)
            .fetch_optional(&mut **tx)
            .await?;
    if registered_schema_version.as_deref() != Some(draft.schema_version.as_str()) {
        return Err(PersistenceError::InvalidState(
            "证据Schema版本ID与版本号不一致".to_string(),
        ));
    }

    match (draft.prompt_version_id, draft.prompt_version.as_deref()) {
        (None, None) => {}
        (Some(prompt_version_id), Some(prompt_version)) => {
            let registered_prompt_version: Option<String> =
                sqlx::query_scalar("SELECT version FROM research.prompt_versions WHERE id = $1")
                    .bind(prompt_version_id)
                    .fetch_optional(&mut **tx)
                    .await?;
            if registered_prompt_version.as_deref() != Some(prompt_version) {
                return Err(PersistenceError::InvalidState(
                    "证据Prompt版本ID与版本号不一致".to_string(),
                ));
            }
        }
        _ => {
            return Err(PersistenceError::InvalidState(
                "证据Prompt版本ID与版本号必须同时提供或同时为空".to_string(),
            ));
        }
    }

    if let Some(conflict_group_id) = draft.conflict_group_id {
        let conflict = sqlx::query(
            r#"
            SELECT match_id, entity_type, entity_id, field_key
            FROM research.evidence_conflicts
            WHERE id = $1
            "#,
        )
        .bind(conflict_group_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("证据引用的冲突组不存在".to_string()))?;
        if conflict.try_get::<Uuid, _>("match_id")? != draft.match_id
            || conflict.try_get::<String, _>("entity_type")? != draft.entity_type
            || conflict.try_get::<Option<Uuid>, _>("entity_id")? != draft.entity_id
            || conflict.try_get::<String, _>("field_key")? != draft.field_key
        {
            return Err(PersistenceError::InvalidState(
                "证据引用的冲突组必须属于同一比赛、实体和字段".to_string(),
            ));
        }
    }
    Ok(())
}
