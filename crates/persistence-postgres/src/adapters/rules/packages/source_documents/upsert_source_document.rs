use crate::PersistenceResult;
use football_domain::RulePackageDraft;
use serde_json::json;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(in crate::adapters::rules::packages) async fn upsert_rule_source_document(
    tx: &mut Transaction<'_, Postgres>,
    draft: &RulePackageDraft,
) -> PersistenceResult<Option<Uuid>> {
    let Some(source) = &draft.source_document else {
        return Ok(None);
    };
    let Some(content_sha256) = source
        .content_sha256
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(None);
    };

    let generated_id = Uuid::new_v4();
    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO catalog.source_documents (
            id, source_type, source_uri, content_sha256, accessed_at, metadata
        ) VALUES ($1, 'competition_rule_standard', $2, $3, now(), $4)
        ON CONFLICT (content_sha256) DO UPDATE SET
            source_uri = COALESCE(catalog.source_documents.source_uri, EXCLUDED.source_uri),
            metadata = catalog.source_documents.metadata || EXCLUDED.metadata
        RETURNING id
        "#,
    )
    .bind(generated_id)
    .bind(source.source_uri.as_deref())
    .bind(content_sha256)
    .bind(json!({
        "title": &source.title,
        "notes": &source.notes,
        "package_key": &draft.package_key,
        "package_version": &draft.version,
    }))
    .fetch_one(&mut **tx)
    .await?;
    Ok(Some(id))
}
