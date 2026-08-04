use super::{
    p4_records::register_competition_profile_in_tx, parse_competition_kind, sha256_json,
    write_audit_event, PersistenceError, PersistenceResult, PostgresStore,
};
use chrono::Utc;
use football_domain::{
    CompetitionBindingDraft, CompetitionBindingSummary, CompetitionKind, CompetitionProfile,
    CompetitionProfileVersionDraft, ResolvedCompetitionContext, RouteDecision, RouteRequest,
    RouteSource, RulePackageDraft, RulePackageSummary,
};
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

    pub async fn register_rule_package(
        &self,
        descriptor: &ModelDescriptor,
        draft: &RulePackageDraft,
    ) -> PersistenceResult<RulePackageSummary> {
        let manifest = serde_json::to_value(draft)?;
        let content_sha256 = sha256_json(&manifest)?;
        let profile = serde_json::to_value(&draft.competition_profile)?;
        let routing = serde_json::to_value(&draft.routing)?;
        let mut tx = self.pool.begin().await?;
        let source_document_id = register_rule_source_document(&mut tx, draft).await?;
        let registration = register_model_in_tx(
            &mut tx,
            descriptor,
            &draft.routing.model_version,
            &draft.routing.parameter_version,
            &draft.parameters,
        )
        .await?;
        let competition_profile = register_competition_profile_in_tx(
            &mut tx,
            &CompetitionProfileVersionDraft {
                profile_key: draft.competition_profile.profile_id.clone(),
                version: draft.version.clone(),
                name: draft.competition_profile.name.clone(),
                competition_kind: draft.competition_profile.competition_kind,
                definition: profile.clone(),
                metadata: json!({
                    "rule_package_key": draft.package_key,
                    "rule_package_version": draft.version,
                }),
            },
        )
        .await?;

        let generated_id = Uuid::new_v4();
        let inserted: Option<Uuid> = sqlx::query_scalar(
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
        .bind(&draft.package_key)
        .bind(&draft.version)
        .bind(&draft.display_name)
        .bind(draft.competition_profile.competition_kind.as_str())
        .bind(&content_sha256)
        .bind(&manifest)
        .bind(&profile)
        .bind(&routing)
        .bind(&draft.feature_requirements)
        .bind(&draft.output_contract)
        .bind(registration.model_version_id)
        .bind(registration.parameter_set_id)
        .bind(source_document_id)
        .bind(draft.routing.priority)
        .bind(&draft.format_version)
        .bind(competition_profile.id)
        .fetch_optional(&mut *tx)
        .await?;

        let was_inserted = inserted.is_some();
        let package_id = if let Some(id) = inserted {
            id
        } else {
            let row = sqlx::query(
                r#"
                SELECT id, content_sha256, competition_profile_id
                FROM model.rule_packages
                WHERE package_key = $1 AND version = $2
                "#,
            )
            .bind(&draft.package_key)
            .bind(&draft.version)
            .fetch_one(&mut *tx)
            .await?;
            let existing_hash: String = row.try_get("content_sha256")?;
            if existing_hash != content_sha256 {
                return Err(PersistenceError::InvalidState(format!(
                    "规则包 {}@{} 已存在但内容不同；请创建新版本",
                    draft.package_key, draft.version
                )));
            }
            let id: Uuid = row.try_get("id")?;
            let existing_profile_id: Option<Uuid> = row.try_get("competition_profile_id")?;
            match existing_profile_id {
                Some(existing) if existing != competition_profile.id => {
                    return Err(PersistenceError::InvalidState(format!(
                        "规则包 {}@{} 已绑定不同赛事Profile版本",
                        draft.package_key, draft.version
                    )));
                }
                None => {
                    sqlx::query(
                        "UPDATE model.rule_packages SET competition_profile_id = $2 WHERE id = $1 AND competition_profile_id IS NULL",
                    )
                    .bind(id)
                    .bind(competition_profile.id)
                    .execute(&mut *tx)
                    .await?;
                }
                Some(_) => {}
            }
            id
        };

        if was_inserted {
            write_audit_event(
                &mut tx,
                "rule_package_registered",
                "rule_package",
                Some(package_id.to_string()),
                json!({
                    "package_key": &draft.package_key,
                    "version": &draft.version,
                    "model_id": &draft.routing.model_id,
                    "competition_kind": draft.competition_profile.competition_kind,
                    "content_sha256": &content_sha256,
                }),
            )
            .await?;
        }
        tx.commit().await?;

        Ok(RulePackageSummary {
            id: package_id,
            format_version: draft.format_version.clone(),
            package_key: draft.package_key.clone(),
            version: draft.version.clone(),
            display_name: draft.display_name.clone(),
            competition_kind: draft.competition_profile.competition_kind,
            model_id: draft.routing.model_id.clone(),
            model_version: draft.routing.model_version.clone(),
            parameter_version: draft.routing.parameter_version.clone(),
            priority: draft.routing.priority,
            content_sha256,
            status: "active".to_string(),
            created_at: Utc::now(),
        })
    }

    pub async fn ensure_type_default_binding(
        &self,
        package_id: Uuid,
        competition_kind: CompetitionKind,
        priority: i32,
        binding_name: &str,
    ) -> PersistenceResult<Uuid> {
        let mut tx = self.pool.begin().await?;
        let package_row = package_route_metadata(&mut tx, package_id).await?;
        if package_row.2 != competition_kind {
            return Err(PersistenceError::InvalidState(format!(
                "规则包赛事类型 {} 不能作为 {} 的默认规则",
                package_row.2.as_str(),
                competition_kind.as_str()
            )));
        }
        let existing: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT id
            FROM model.competition_bindings
            WHERE rule_package_id = $1
              AND competition_id IS NULL
              AND season_id IS NULL
              AND stage_id IS NULL
              AND competition_kind = $2
              AND is_active = true
            ORDER BY priority DESC, created_at DESC
            LIMIT 1
            "#,
        )
        .bind(package_id)
        .bind(competition_kind.as_str())
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(id) = existing {
            tx.commit().await?;
            return Ok(id);
        }

        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO model.competition_bindings (
                id, binding_name, competition_kind,
                model_version_id, parameter_set_id, rule_package_id,
                priority, is_active
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, true)
            "#,
        )
        .bind(id)
        .bind(binding_name)
        .bind(competition_kind.as_str())
        .bind(package_row.0)
        .bind(package_row.1)
        .bind(package_id)
        .bind(priority)
        .execute(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "type_default_binding_created",
            "competition_binding",
            Some(id.to_string()),
            json!({
                "rule_package_id": package_id,
                "competition_kind": competition_kind,
                "priority": priority,
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(id)
    }

    pub async fn list_rule_packages(&self) -> PersistenceResult<Vec<RulePackageSummary>> {
        let rows = sqlx::query(
            r#"
            SELECT
                rp.id, rp.format_version, rp.package_key, rp.version, rp.display_name,
                rp.competition_kind, d.model_key, v.version AS model_version,
                p.parameter_version, rp.priority, rp.content_sha256,
                rp.status, rp.created_at
            FROM model.rule_packages rp
            JOIN model.versions v ON v.id = rp.model_version_id
            JOIN model.definitions d ON d.id = v.model_id
            JOIN model.parameter_sets p ON p.id = rp.parameter_set_id
            ORDER BY rp.created_at DESC, rp.package_key, rp.version
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(rule_package_summary_from_row).collect()
    }

    pub async fn create_competition_binding(
        &self,
        draft: &CompetitionBindingDraft,
    ) -> PersistenceResult<CompetitionBindingSummary> {
        if draft.competition_id.is_none()
            && draft.season_id.is_none()
            && draft.stage_id.is_none()
            && draft.competition_kind.is_none()
        {
            return Err(PersistenceError::InvalidState(
                "绑定范围不能为空；至少指定赛事、赛季、阶段或赛事类型".to_string(),
            ));
        }
        if let (Some(from), Some(to)) = (&draft.valid_from, &draft.valid_to) {
            if to < from {
                return Err(PersistenceError::InvalidState(
                    "绑定结束时间不能早于开始时间".to_string(),
                ));
            }
        }

        let has_specific_scope =
            draft.competition_id.is_some() || draft.season_id.is_some() || draft.stage_id.is_some();
        let resolved = if has_specific_scope {
            let context = self
                .resolve_competition_context(
                    draft.competition_id,
                    draft.season_id,
                    draft.stage_id,
                    draft.competition_kind.unwrap_or_default(),
                )
                .await?;
            if let Some(requested_kind) = draft.competition_kind {
                if requested_kind != context.competition_kind {
                    return Err(PersistenceError::InvalidState(format!(
                        "绑定赛事类型 {} 与赛事层级解析结果 {} 不一致",
                        requested_kind.as_str(),
                        context.competition_kind.as_str()
                    )));
                }
            }
            context
        } else {
            ResolvedCompetitionContext {
                competition_id: None,
                season_id: None,
                stage_id: None,
                competition_kind: draft.competition_kind.ok_or_else(|| {
                    PersistenceError::InvalidState("赛事类型默认绑定缺少赛事类型".to_string())
                })?,
            }
        };

        let mut tx = self.pool.begin().await?;
        let (model_version_id, parameter_set_id, package_kind) =
            package_route_metadata(&mut tx, draft.rule_package_id).await?;
        if package_kind != resolved.competition_kind {
            return Err(PersistenceError::InvalidState(format!(
                "规则包赛事类型 {} 不能绑定到 {}",
                package_kind.as_str(),
                resolved.competition_kind.as_str()
            )));
        }

        let id = Uuid::new_v4();
        let binding_name = draft
            .binding_name
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| format!("赛事规则绑定-{}", &id.to_string()[..8]));
        sqlx::query(
            r#"
            INSERT INTO model.competition_bindings (
                id, binding_name, competition_id, season_id, stage_id,
                competition_kind, model_version_id, parameter_set_id,
                rule_package_id, priority, is_active, valid_from, valid_to
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8,
                $9, $10, true, $11, $12
            )
            "#,
        )
        .bind(id)
        .bind(&binding_name)
        .bind(resolved.competition_id)
        .bind(resolved.season_id)
        .bind(resolved.stage_id)
        .bind(resolved.competition_kind.as_str())
        .bind(model_version_id)
        .bind(parameter_set_id)
        .bind(draft.rule_package_id)
        .bind(draft.priority)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .execute(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "competition_binding_created",
            "competition_binding",
            Some(id.to_string()),
            json!({
                "draft": draft,
                "resolved_scope": resolved,
                "package_kind": package_kind,
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_binding(id).await
    }

    pub async fn list_competition_bindings(
        &self,
    ) -> PersistenceResult<Vec<CompetitionBindingSummary>> {
        let rows = sqlx::query(binding_list_query())
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(binding_summary_from_row).collect()
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

    async fn read_binding(&self, id: Uuid) -> PersistenceResult<CompetitionBindingSummary> {
        let query = format!("{} WHERE b.id = $1", binding_list_query_without_order());
        let row = sqlx::query(&query).bind(id).fetch_one(&self.pool).await?;
        binding_summary_from_row(&row)
    }
}

async fn register_rule_source_document(
    tx: &mut Transaction<'_, sqlx::Postgres>,
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

async fn register_model_in_tx(
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

async fn package_route_metadata(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    package_id: Uuid,
) -> PersistenceResult<(Uuid, Uuid, CompetitionKind)> {
    let row = sqlx::query(
        r#"
        SELECT model_version_id, parameter_set_id, competition_kind
        FROM model.rule_packages
        WHERE id = $1 AND status = 'active'
        "#,
    )
    .bind(package_id)
    .fetch_one(&mut **tx)
    .await?;
    let model_version_id = row
        .try_get::<Option<Uuid>, _>("model_version_id")?
        .ok_or_else(|| PersistenceError::InvalidState("规则包缺少模型版本".to_string()))?;
    let parameter_set_id = row
        .try_get::<Option<Uuid>, _>("parameter_set_id")?
        .ok_or_else(|| PersistenceError::InvalidState("规则包缺少参数版本".to_string()))?;
    let competition_kind = row
        .try_get::<Option<String>, _>("competition_kind")?
        .ok_or_else(|| PersistenceError::InvalidState("规则包缺少赛事类型".to_string()))?;
    Ok((
        model_version_id,
        parameter_set_id,
        parse_competition_kind(&competition_kind)?,
    ))
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

fn rule_package_summary_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<RulePackageSummary> {
    Ok(RulePackageSummary {
        id: row.try_get("id")?,
        format_version: row.try_get("format_version")?,
        package_key: row.try_get("package_key")?,
        version: row.try_get("version")?,
        display_name: row.try_get("display_name")?,
        competition_kind: parse_competition_kind(&row.try_get::<String, _>("competition_kind")?)?,
        model_id: row.try_get("model_key")?,
        model_version: row.try_get("model_version")?,
        parameter_version: row.try_get("parameter_version")?,
        priority: row.try_get("priority")?,
        content_sha256: row.try_get("content_sha256")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
    })
}

fn binding_summary_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<CompetitionBindingSummary> {
    Ok(CompetitionBindingSummary {
        id: row.try_get("id")?,
        binding_name: row.try_get("binding_name")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        season_id: row.try_get("season_id")?,
        stage_id: row.try_get("stage_id")?,
        competition_kind: row
            .try_get::<Option<String>, _>("competition_kind")?
            .map(|value| parse_competition_kind(&value))
            .transpose()?,
        rule_package_id: row.try_get("rule_package_id")?,
        rule_package_name: row.try_get("rule_package_name")?,
        model_id: row.try_get("model_key")?,
        priority: row.try_get("priority")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
    })
}

fn binding_list_query() -> &'static str {
    r#"
    SELECT
        b.id, b.binding_name, b.competition_id, c.name AS competition_name,
        b.season_id, b.stage_id, b.competition_kind,
        b.rule_package_id, rp.display_name AS rule_package_name,
        d.model_key, b.priority, b.is_active, b.created_at
    FROM model.competition_bindings b
    JOIN model.rule_packages rp ON rp.id = b.rule_package_id
    JOIN model.versions v ON v.id = b.model_version_id
    JOIN model.definitions d ON d.id = v.model_id
    LEFT JOIN football.competitions c ON c.id = b.competition_id
    WHERE b.is_active = true
      AND (b.valid_from IS NULL OR b.valid_from <= now())
      AND (b.valid_to IS NULL OR b.valid_to >= now())
    ORDER BY b.priority DESC, b.created_at DESC, b.id DESC
    "#
}

fn binding_list_query_without_order() -> &'static str {
    r#"
    SELECT
        b.id, b.binding_name, b.competition_id, c.name AS competition_name,
        b.season_id, b.stage_id, b.competition_kind,
        b.rule_package_id, rp.display_name AS rule_package_name,
        d.model_key, b.priority, b.is_active, b.created_at
    FROM model.competition_bindings b
    JOIN model.rule_packages rp ON rp.id = b.rule_package_id
    JOIN model.versions v ON v.id = b.model_version_id
    JOIN model.definitions d ON d.id = v.model_id
    LEFT JOIN football.competitions c ON c.id = b.competition_id
    "#
}
