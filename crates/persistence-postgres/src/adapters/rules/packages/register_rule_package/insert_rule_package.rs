use crate::{routing::ModelRegistration, PersistenceResult};
use football_domain::RulePackageDraft;
use serde_json::Value;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) struct RulePackageInsertValues<'a> {
    pub(super) draft: &'a RulePackageDraft,
    pub(super) registration: &'a ModelRegistration,
    pub(super) source_document_id: Option<Uuid>,
    pub(super) competition_profile_id: Uuid,
    pub(super) content_sha256: &'a str,
    pub(super) manifest: &'a Value,
    pub(super) profile: &'a Value,
    pub(super) routing: &'a Value,
}

pub(super) async fn insert_rule_package(
    tx: &mut Transaction<'_, Postgres>,
    values: RulePackageInsertValues<'_>,
) -> PersistenceResult<Option<Uuid>> {
    let generated_id = Uuid::new_v4();
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO model.rule_packages (
            id, package_key, version, display_name, competition_kind,
            content_sha256, manifest, profile, routing,
            feature_requirements, output_contract,
            model_version_id, parameter_set_id, source_document_id, priority, format_version,
            competition_profile_id, status
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9,
            $10, $11,
            $12, $13, $14, $15, $16,
            $17, 'active'
        )
        ON CONFLICT (package_key, version) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(generated_id)
    .bind(&values.draft.package_key)
    .bind(&values.draft.version)
    .bind(&values.draft.display_name)
    .bind(values.draft.competition_profile.competition_kind.as_str())
    .bind(values.content_sha256)
    .bind(values.manifest)
    .bind(values.profile)
    .bind(values.routing)
    .bind(&values.draft.feature_requirements)
    .bind(&values.draft.output_contract)
    .bind(values.registration.model_version_id)
    .bind(values.registration.parameter_set_id)
    .bind(values.source_document_id)
    .bind(values.draft.routing.priority)
    .bind(&values.draft.format_version)
    .bind(values.competition_profile_id)
    .fetch_optional(&mut **tx)
    .await?)
}
