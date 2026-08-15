use super::{sha256_json, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{CompetitionProfile, RouteDecision, RouteRequest, RouteSource};
use football_model_api::ModelDescriptor;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Row, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistration {
    pub model_version_id: Uuid,
    pub parameter_set_id: Uuid,
}

impl PostgresStore {
    pub async fn register_model(
        &self,
        descriptor: &ModelDescriptor,
        model_version: &str,
        parameter_version: &str,
        parameters: &Value,
    ) -> PersistenceResult<ModelRegistration> {
        let mut tx = self.pool.begin().await?;
        let registration = register_model_in_tx(
            &mut tx,
            descriptor,
            model_version,
            parameter_version,
            parameters,
        )
        .await?;
        tx.commit().await?;
        Ok(registration)
    }

    pub async fn resolve_route(&self, request: &RouteRequest) -> PersistenceResult<RouteDecision> {
        if let Some(package_id) = request.explicit_rule_package_id {
            let row = sqlx::query(
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
                    p.id AS parameter_set_id, p.parameter_version, p.definition AS parameters
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
            .bind(request.preferred_model_family.as_deref())
            .bind(request.preferred_model_id.as_deref())
            .fetch_optional(&self.pool)
            .await?
            .ok_or(PersistenceError::RouteNotFound)?;
            return route_decision_from_row(&row, RouteSource::ExplicitRulePackage, request);
        }

        let row = sqlx::query(
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
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PersistenceError::RouteNotFound)?;

        let source = if row.try_get::<Option<Uuid>, _>("stage_id")?.is_some() {
            RouteSource::StageBinding
        } else if row.try_get::<Option<Uuid>, _>("season_id")?.is_some() {
            RouteSource::SeasonBinding
        } else if row.try_get::<Option<Uuid>, _>("competition_id")?.is_some() {
            RouteSource::CompetitionBinding
        } else {
            RouteSource::CompetitionKindDefault
        };
        route_decision_from_row(&row, source, request)
    }
}

pub(crate) async fn register_model_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    descriptor: &ModelDescriptor,
    model_version: &str,
    parameter_version: &str,
    parameters: &Value,
) -> PersistenceResult<ModelRegistration> {
    let model_id = upsert_model_definition(tx, descriptor).await?;
    let model_version_id = register_model_version(tx, model_id, descriptor, model_version).await?;
    let parameter_set_id =
        register_parameter_set(tx, model_version_id, parameter_version, parameters).await?;
    Ok(ModelRegistration {
        model_version_id,
        parameter_set_id,
    })
}

async fn upsert_model_definition(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    descriptor: &ModelDescriptor,
) -> PersistenceResult<Uuid> {
    let generated_id = Uuid::new_v4();
    let returned: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO model.definitions (id, model_key, display_name, description)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (model_key) DO UPDATE SET
            display_name = EXCLUDED.display_name,
            description = EXCLUDED.description,
            is_active = true
        RETURNING id
        "#,
    )
    .bind(generated_id)
    .bind(&descriptor.model_id)
    .bind(&descriptor.display_name)
    .bind(format!(
        "{}；引擎 {}",
        descriptor.display_name, descriptor.engine_version
    ))
    .fetch_one(&mut **tx)
    .await?;
    Ok(returned)
}

async fn register_model_version(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    model_id: Uuid,
    descriptor: &ModelDescriptor,
    model_version: &str,
) -> PersistenceResult<Uuid> {
    let generated_id = Uuid::new_v4();
    let inserted: Option<Uuid> = sqlx::query_scalar(
        r#"
        INSERT INTO model.versions (
            id, model_id, version, engine_version,
            input_schema_version, output_schema_version, status
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active')
        ON CONFLICT (model_id, version) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(generated_id)
    .bind(model_id)
    .bind(model_version)
    .bind(&descriptor.engine_version)
    .bind(&descriptor.input_schema_version)
    .bind(&descriptor.output_schema_version)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(inserted_id) = inserted {
        return Ok(inserted_id);
    }

    let row = sqlx::query(
        r#"
        SELECT id, engine_version, input_schema_version, output_schema_version
        FROM model.versions
        WHERE model_id = $1 AND version = $2
        "#,
    )
    .bind(model_id)
    .bind(model_version)
    .fetch_one(&mut **tx)
    .await?;

    let existing_id: Uuid = row.try_get("id")?;
    let existing_engine: String = row.try_get("engine_version")?;
    let existing_input: String = row.try_get("input_schema_version")?;
    let existing_output: String = row.try_get("output_schema_version")?;
    if existing_engine != descriptor.engine_version
        || existing_input != descriptor.input_schema_version
        || existing_output != descriptor.output_schema_version
    {
        return Err(PersistenceError::InvalidState(format!(
            "模型版本 {model_version} 已存在但引擎或 Schema 不一致；请创建新模型版本"
        )));
    }
    Ok(existing_id)
}

async fn register_parameter_set(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    model_version_id: Uuid,
    parameter_version: &str,
    parameters: &Value,
) -> PersistenceResult<Uuid> {
    let generated_id = Uuid::new_v4();
    let definition_hash = sha256_json(parameters)?;
    let inserted: Option<Uuid> = sqlx::query_scalar(
        r#"
        INSERT INTO model.parameter_sets (
            id, model_version_id, parameter_version, name,
            definition, definition_sha256, status
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active')
        ON CONFLICT (model_version_id, parameter_version) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(generated_id)
    .bind(model_version_id)
    .bind(parameter_version)
    .bind(parameter_version)
    .bind(parameters)
    .bind(&definition_hash)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(inserted_id) = inserted {
        return Ok(inserted_id);
    }

    let row = sqlx::query(
        r#"
        SELECT id, definition_sha256
        FROM model.parameter_sets
        WHERE model_version_id = $1 AND parameter_version = $2
        "#,
    )
    .bind(model_version_id)
    .bind(parameter_version)
    .fetch_one(&mut **tx)
    .await?;

    let existing_id: Uuid = row.try_get("id")?;
    let existing_hash: String = row.try_get("definition_sha256")?;
    if existing_hash != definition_hash {
        return Err(PersistenceError::InvalidState(format!(
            "参数版本 {parameter_version} 已存在但内容不同；请创建新参数版本"
        )));
    }
    Ok(existing_id)
}

fn route_decision_from_row(
    row: &sqlx::postgres::PgRow,
    source: RouteSource,
    request: &RouteRequest,
) -> PersistenceResult<RouteDecision> {
    let profile_value: Value = row.try_get("profile")?;
    let competition_profile: CompetitionProfile = serde_json::from_value(profile_value)?;
    let routing = serde_json::from_value(row.try_get::<Value, _>("routing")?)?;
    let binding_id: Option<Uuid> = row.try_get("binding_id")?;
    let rule_package_id: Uuid = row.try_get("rule_package_id")?;
    let package_key: String = row.try_get("package_key")?;
    let package_version: String = row.try_get("package_version")?;
    let package_display_name: String = row.try_get("package_display_name")?;
    let model_id: String = row.try_get("model_key")?;
    let model_version_id: Uuid = row.try_get("model_version_id")?;
    let model_version: String = row.try_get("model_version")?;
    let parameter_set_id: Uuid = row.try_get("parameter_set_id")?;
    let parameter_version: String = row.try_get("parameter_version")?;
    let competition_profile_id: Uuid = row.try_get("competition_profile_id")?;
    let priority: i32 = row.try_get("priority")?;
    let reason = json!({
        "source": &source,
        "binding_id": binding_id,
        "rule_package_id": rule_package_id,
        "package_key": &package_key,
        "package_version": &package_version,
        "preferred_model_family": request.preferred_model_family.as_deref(),
        "preferred_model_id": request.preferred_model_id.as_deref(),
        "competition_id": request.competition_id,
        "season_id": request.season_id,
        "stage_id": request.stage_id,
        "competition_kind": request.competition_kind,
        "priority": priority,
    });
    Ok(RouteDecision {
        source,
        binding_id,
        rule_package_id,
        package_key,
        package_version,
        package_display_name,
        model_id,
        model_version_id,
        model_version,
        parameter_set_id,
        parameter_version,
        competition_profile_id,
        parameters: row.try_get("parameters")?,
        routing,
        competition_profile,
        feature_requirements: row.try_get("feature_requirements")?,
        output_contract: row.try_get("output_contract")?,
        priority,
        reason,
    })
}
