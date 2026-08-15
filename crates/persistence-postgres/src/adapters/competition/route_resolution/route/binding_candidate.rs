use super::record_row::RouteRow;
use crate::{PersistenceError, PersistenceResult};
use football_domain::RouteRequest;
use sqlx::PgPool;

pub(super) async fn read_binding_candidate(
    pool: &PgPool,
    request: &RouteRequest,
) -> PersistenceResult<RouteRow> {
    sqlx::query_as::<_, RouteRow>(
        r#"
        SELECT
            b.id AS binding_id,
            rp.id AS rule_package_id,
            rp.package_key, rp.version AS package_version,
            rp.display_name AS package_display_name,
            rp.profile, rp.competition_profile_id, rp.routing, rp.feature_requirements, rp.output_contract,
            b.priority,
            d.model_key, v.id AS model_version_id,
            v.version AS model_version,
            p.id AS parameter_set_id, p.parameter_version, p.definition AS parameters,
            b.competition_id, b.season_id, b.stage_id, b.competition_kind
        FROM model.competition_bindings b
        JOIN model.rule_packages rp ON rp.id = b.rule_package_id
        JOIN model.versions v ON v.id = b.model_version_id
        JOIN model.definitions d ON d.id = v.model_id
        JOIN model.parameter_sets p ON p.id = b.parameter_set_id
        WHERE b.is_active = true
          AND rp.status = 'active'
          AND (b.valid_from IS NULL OR b.valid_from <= $5)
          AND (b.valid_to IS NULL OR b.valid_to >= $5)
          AND (b.competition_id IS NULL OR b.competition_id = $1)
          AND (b.season_id IS NULL OR b.season_id = $2)
          AND (b.stage_id IS NULL OR b.stage_id = $3)
          AND (b.competition_kind IS NULL OR b.competition_kind = $4)
          AND ($6::text IS NULL OR split_part(d.model_key, '_', 1) = $6)
          AND ($7::text IS NULL OR d.model_key = $7)
        ORDER BY
            (b.stage_id IS NOT NULL) DESC,
            (b.season_id IS NOT NULL) DESC,
            (b.competition_id IS NOT NULL) DESC,
            b.priority DESC,
            b.created_at DESC,
            b.id DESC
        LIMIT 1
        "#,
    )
    .bind(request.competition_id)
    .bind(request.season_id)
    .bind(request.stage_id)
    .bind(request.competition_kind.as_str())
    .bind(request.kickoff_time)
    .bind(request.preferred_model_family.as_deref())
    .bind(request.preferred_model_id.as_deref())
    .fetch_optional(pool)
    .await?
    .ok_or(PersistenceError::RouteNotFound)
}
