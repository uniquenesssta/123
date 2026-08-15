use super::record_row::RouteRow;
use crate::{PersistenceError, PersistenceResult};
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_explicit_rule_package(
    pool: &PgPool,
    package_id: Uuid,
    preferred_model_family: Option<&str>,
    preferred_model_id: Option<&str>,
) -> PersistenceResult<RouteRow> {
    sqlx::query_as::<_, RouteRow>(
        r#"
        SELECT
            NULL::uuid AS binding_id,
            rp.id AS rule_package_id,
            rp.package_key, rp.version AS package_version,
            rp.display_name AS package_display_name,
            rp.profile, rp.competition_profile_id, rp.routing, rp.feature_requirements, rp.output_contract,
            rp.priority,
            d.model_key, v.id AS model_version_id,
            v.version AS model_version,
            p.id AS parameter_set_id, p.parameter_version, p.definition AS parameters,
            NULL::uuid AS competition_id, NULL::uuid AS season_id,
            NULL::uuid AS stage_id
        FROM model.rule_packages rp
        JOIN model.versions v ON v.id = rp.model_version_id
        JOIN model.definitions d ON d.id = v.model_id
        JOIN model.parameter_sets p ON p.id = rp.parameter_set_id
        WHERE rp.id = $1 AND rp.status = 'active'
          AND ($2::text IS NULL OR split_part(d.model_key, '_', 1) = $2)
          AND ($3::text IS NULL OR d.model_key = $3)
        "#,
    )
    .bind(package_id)
    .bind(preferred_model_family)
    .bind(preferred_model_id)
    .fetch_optional(pool)
    .await?
    .ok_or(PersistenceError::RouteNotFound)
}
