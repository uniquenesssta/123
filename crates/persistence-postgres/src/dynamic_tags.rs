use crate::{
    role_resolution::{
        normalize_role_origin, resolve_tactical_role, tactical_role_confidence,
        DefaultTacticalRole, ResolvedTacticalRole, ROLE_ORIGIN_PLAYER_POSITION_DEFAULT,
    },
    PersistenceError, PersistenceResult, PostgresStore,
};
use chrono::{DateTime, Utc};
use football_domain::{
    ContributionComponent, PlayerDynamicTagDefinitionRecord, PlayerDynamicTagDraft,
    PlayerDynamicTagRecord, PlayerMatchContribution, PlayerMatchContributionRequest,
};
use serde_json::Value;
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

const CONTRIBUTION_VERSION: &str = "match-contribution-v2-role-context";

impl PostgresStore {
    pub async fn list_dynamic_tag_definitions(
        &self,
    ) -> PersistenceResult<Vec<PlayerDynamicTagDefinitionRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT code, name, category, minimum_value, maximum_value,
                   default_value, default_ttl_hours, is_multiplier, description
            FROM feature.player_dynamic_tag_definitions
            ORDER BY category, code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(dynamic_tag_definition_from_row).collect()
    }

    pub async fn add_player_dynamic_tag(
        &self,
        draft: &PlayerDynamicTagDraft,
    ) -> PersistenceResult<PlayerDynamicTagRecord> {
        validate_dynamic_tag_draft(self, draft).await?;
        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO feature.player_dynamic_tags (
                id, player_id, tag_code, value, label, confidence,
                observed_at, valid_from, valid_to, competition_id,
                position_code, opponent_team_id, sample_size, source_type,
                source_document_id, calculation_version, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12, $13, $14,
                $15, $16, $17
            )
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(draft.player_id)
        .bind(draft.tag_code.trim())
        .bind(draft.value)
        .bind(draft.label.as_deref())
        .bind(draft.confidence)
        .bind(draft.observed_at)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .bind(draft.competition_id)
        .bind(draft.position_code.as_deref())
        .bind(draft.opponent_team_id)
        .bind(draft.sample_size)
        .bind(draft.source_type.trim())
        .bind(draft.source_document_id)
        .bind(draft.calculation_version.trim())
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        let inserted_id: Uuid = row.try_get("id")?;
        self.read_player_dynamic_tag(inserted_id).await
    }

    pub async fn read_player_dynamic_tag(
        &self,
        tag_id: Uuid,
    ) -> PersistenceResult<PlayerDynamicTagRecord> {
        let row = sqlx::query(
            r#"
            SELECT tag.id, tag.player_id, tag.tag_code, definition.name AS tag_name,
                   definition.category, tag.value, tag.label, tag.confidence,
                   tag.observed_at, tag.valid_from, tag.valid_to,
                   tag.competition_id, competition.name AS competition_name,
                   tag.position_code, tag.opponent_team_id,
                   opponent.canonical_name AS opponent_team_name,
                   tag.sample_size, tag.source_type, tag.calculation_version,
                   tag.metadata
            FROM feature.player_dynamic_tags tag
            JOIN feature.player_dynamic_tag_definitions definition
              ON definition.code = tag.tag_code
            LEFT JOIN football.competitions competition ON competition.id = tag.competition_id
            LEFT JOIN football.teams opponent ON opponent.id = tag.opponent_team_id
            WHERE tag.id = $1
            "#,
        )
        .bind(tag_id)
        .fetch_one(&self.pool)
        .await?;
        player_dynamic_tag_from_row(&row)
    }

    pub async fn list_player_dynamic_tags(
        &self,
        player_id: Uuid,
        as_of: DateTime<Utc>,
    ) -> PersistenceResult<Vec<PlayerDynamicTagRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (
                       tag.tag_code, tag.competition_id,
                       tag.position_code, tag.opponent_team_id
                   )
                   tag.id, tag.player_id, tag.tag_code,
                   definition.name AS tag_name, definition.category,
                   tag.value, tag.label, tag.confidence,
                   tag.observed_at, tag.valid_from, tag.valid_to,
                   tag.competition_id, competition.name AS competition_name,
                   tag.position_code, tag.opponent_team_id,
                   opponent.canonical_name AS opponent_team_name,
                   tag.sample_size, tag.source_type,
                   tag.calculation_version, tag.metadata
            FROM feature.player_dynamic_tags tag
            JOIN feature.player_dynamic_tag_definitions definition
              ON definition.code = tag.tag_code
            LEFT JOIN football.competitions competition ON competition.id = tag.competition_id
            LEFT JOIN football.teams opponent ON opponent.id = tag.opponent_team_id
            WHERE tag.player_id = $1
              AND tag.valid_from <= $2
              AND tag.valid_to >= $2
            ORDER BY tag.tag_code, tag.competition_id NULLS FIRST,
                     tag.position_code NULLS FIRST,
                     tag.opponent_team_id NULLS FIRST,
                     tag.observed_at DESC, tag.id DESC
            "#,
        )
        .bind(player_id)
        .bind(as_of)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(player_dynamic_tag_from_row).collect()
    }

    pub async fn calculate_player_match_contribution(
        &self,
        request: &PlayerMatchContributionRequest,
    ) -> PersistenceResult<PlayerMatchContribution> {
        let player_row = sqlx::query(
            r#"
            SELECT player.canonical_name,
                   COALESCE(ability.average_value, 50) AS base_ability,
                   COALESCE(ability.average_confidence, 0.5) AS base_confidence
            FROM football.players player
            LEFT JOIN LATERAL (
                SELECT avg(latest.value) AS average_value,
                       avg(latest.confidence) AS average_confidence
                FROM (
                    SELECT DISTINCT ON (observation.dimension_code)
                           observation.value, observation.confidence
                    FROM feature.player_ability_observations observation
                    WHERE observation.player_id = player.id
                      AND observation.observed_at <= COALESCE($3, $2)
                      AND observation.created_at <= COALESCE($3, $2)
                      AND observation.effective_from <= $2
                      AND (observation.effective_to IS NULL OR observation.effective_to >= $2)
                    ORDER BY observation.dimension_code,
                             observation.effective_from DESC,
                             observation.observed_at DESC,
                             observation.id DESC
                ) latest
            ) ability ON true
            WHERE player.id = $1
            "#,
        )
        .bind(request.player_id)
        .bind(request.as_of)
        .bind(request.data_cutoff_time)
        .fetch_one(&self.pool)
        .await?;
        let player_name: String = player_row.try_get("canonical_name")?;
        let base_ability: f64 = player_row.try_get("base_ability")?;
        let base_confidence: f64 = player_row.try_get("base_confidence")?;

        let requested_tactical_role_code = request
            .role_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let requested_role_source_position_code = request
            .role_source_position_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .filter(|_| {
                matches!(
                    request.role_origin.as_deref(),
                    Some(ROLE_ORIGIN_PLAYER_POSITION_DEFAULT)
                )
            })
            .map(str::to_uppercase);
        let role_position_row = sqlx::query(
            r#"
            SELECT position.proficiency, position.default_role_code, position.position_code
            FROM football.player_positions position
            WHERE position.player_id = $1
              AND (position.valid_from IS NULL OR position.valid_from <= $2)
              AND (position.valid_to IS NULL OR position.valid_to >= $2)
            ORDER BY
              CASE
                WHEN $3::text IS NOT NULL
                 AND upper(position.position_code) = upper($3) THEN 0
                ELSE 1
              END,
              CASE
                WHEN $4::text IS NOT NULL
                 AND lower(btrim(position.default_role_code)) = lower(btrim($4)) THEN 0
                ELSE 1
              END,
              CASE
                WHEN $5::text IS NOT NULL
                 AND upper(position.position_code) = upper($5) THEN 0
                ELSE 1
              END,
              position.is_primary DESC,
              position.proficiency DESC,
              position.valid_from DESC NULLS LAST
            LIMIT 1
            "#,
        )
        .bind(request.player_id)
        .bind(request.as_of.date_naive())
        .bind(requested_role_source_position_code.as_deref())
        .bind(requested_tactical_role_code.as_deref())
        .bind(request.position_code.as_deref())
        .fetch_optional(&self.pool)
        .await?;
        let role_position_proficiency = role_position_row
            .as_ref()
            .map(|row| row.try_get::<f64, _>("proficiency"))
            .transpose()?;
        let role_position_code = role_position_row
            .as_ref()
            .map(|row| row.try_get::<String, _>("position_code"))
            .transpose()?;
        let inherited_role_code = role_position_row
            .as_ref()
            .map(|row| row.try_get::<Option<String>, _>("default_role_code"))
            .transpose()?
            .flatten()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let inherited_role = inherited_role_code
            .as_ref()
            .zip(role_position_code.as_ref())
            .map(|(role_code, position_code)| DefaultTacticalRole {
                role_code: role_code.clone(),
                position_code: position_code.clone(),
            });
        let role_resolution = match requested_tactical_role_code {
            Some(role_code) => {
                let origin = normalize_role_origin(
                    Some(role_code.as_str()),
                    request.role_origin.as_deref(),
                    inherited_role_code.as_deref(),
                );
                ResolvedTacticalRole {
                    role_code: Some(role_code),
                    origin,
                    source_position_code: if origin == ROLE_ORIGIN_PLAYER_POSITION_DEFAULT {
                        requested_role_source_position_code
                            .clone()
                            .or_else(|| role_position_code.clone())
                    } else {
                        None
                    },
                }
            }
            None => resolve_tactical_role(None, inherited_role.as_ref()),
        };
        let tactical_role_confidence = tactical_role_confidence(
            role_resolution.role_code.as_deref(),
            role_resolution.origin,
            role_position_proficiency,
        );
        let tactical_role_code = role_resolution.role_code;
        let tactical_role_origin = role_resolution.origin;
        let tactical_role_source_position_code = role_resolution.source_position_code;

        let availability_status: Option<String> = sqlx::query_scalar(
            r#"
            SELECT status
            FROM football.player_availability
            WHERE player_id = $1
              AND valid_from <= $2
              AND (valid_to IS NULL OR valid_to >= $2)
              AND ($3::uuid IS NULL OR competition_id IS NULL OR competition_id = $3)
              AND created_at <= COALESCE($4, $2)
            ORDER BY (competition_id IS NOT NULL) DESC, valid_from DESC, created_at DESC
            LIMIT 1
            "#,
        )
        .bind(request.player_id)
        .bind(request.as_of)
        .bind(request.competition_id)
        .bind(request.data_cutoff_time)
        .fetch_optional(&self.pool)
        .await?;

        let tag_rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (tag.tag_code)
                   tag.id, tag.player_id, tag.tag_code,
                   definition.name AS tag_name, definition.category,
                   tag.value, tag.label, tag.confidence,
                   tag.observed_at, tag.valid_from, tag.valid_to,
                   tag.competition_id, competition.name AS competition_name,
                   tag.position_code, tag.opponent_team_id,
                   opponent.canonical_name AS opponent_team_name,
                   tag.sample_size, tag.source_type,
                   tag.calculation_version, tag.metadata,
                   ((tag.competition_id IS NOT NULL)::int
                    + (tag.position_code IS NOT NULL)::int
                    + (tag.opponent_team_id IS NOT NULL)::int
                    + (NULLIF(btrim(tag.metadata->>'role_code'), '') IS NOT NULL)::int)
                       AS specificity
            FROM feature.player_dynamic_tags tag
            JOIN feature.player_dynamic_tag_definitions definition
              ON definition.code = tag.tag_code
            LEFT JOIN football.competitions competition ON competition.id = tag.competition_id
            LEFT JOIN football.teams opponent ON opponent.id = tag.opponent_team_id
            WHERE tag.player_id = $1
              AND tag.valid_from <= $2
              AND tag.valid_to >= $2
              AND (tag.competition_id IS NULL OR tag.competition_id = $3)
              AND (tag.position_code IS NULL OR tag.position_code = $4)
              AND (tag.opponent_team_id IS NULL OR tag.opponent_team_id = $5)
              AND (
                    NULLIF(btrim(tag.metadata->>'role_code'), '') IS NULL
                    OR (
                        $6::text IS NOT NULL
                        AND lower(btrim(tag.metadata->>'role_code')) = lower(btrim($6))
                    )
                  )
              AND tag.observed_at <= COALESCE($7, $2)
              AND tag.created_at <= COALESCE($7, $2)
            ORDER BY tag.tag_code, specificity DESC,
                     tag.observed_at DESC, tag.id DESC
            "#,
        )
        .bind(request.player_id)
        .bind(request.as_of)
        .bind(request.competition_id)
        .bind(request.position_code.as_deref())
        .bind(request.opponent_team_id)
        .bind(tactical_role_code.as_deref())
        .bind(request.data_cutoff_time)
        .fetch_all(&self.pool)
        .await?;
        let applied_tags = tag_rows
            .iter()
            .map(player_dynamic_tag_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        let tag_map = applied_tags
            .iter()
            .map(|tag| (tag.tag_code.as_str(), tag))
            .collect::<HashMap<_, _>>();

        let availability_value = availability_multiplier(availability_status.as_deref());
        let readiness = tag_value(&tag_map, "match_readiness", 1.0);
        let form = tag_value(&tag_map, "form_multiplier", 1.0);
        let fatigue = tag_value(&tag_map, "fatigue_multiplier", 1.0);
        let position_fit = tag_value(&tag_map, "position_fit", 1.0);
        let tactical_fit = tag_value(&tag_map, "tactical_fit", 1.0);
        let chemistry_fit = tag_value(&tag_map, "chemistry_fit", 1.0);
        let realization = tag_value(&tag_map, "realization_multiplier", 1.0);
        let minute_share = request
            .expected_minutes
            .map(|minutes| (f64::from(minutes) / 90.0).clamp(0.0, 1.0))
            .unwrap_or_else(|| tag_value(&tag_map, "expected_minutes_share", 1.0));
        let starting_probability = tag_map.get("starting_probability").map(|tag| tag.value);
        let volatility = tag_value(&tag_map, "volatility", 0.0);
        let average_tag_confidence = if applied_tags.is_empty() {
            0.5
        } else {
            applied_tags.iter().map(|tag| tag.confidence).sum::<f64>() / applied_tags.len() as f64
        };
        let overall_confidence = ((base_confidence + average_tag_confidence) / 2.0
            * (1.0 - 0.35 * volatility))
            .clamp(0.0, 1.0);
        let confidence_multiplier = 0.7 + 0.3 * overall_confidence;

        let components = vec![
            component(
                "availability",
                "可用性",
                availability_value,
                1.0,
                availability_status.unwrap_or_else(|| "unknown".to_string()),
            ),
            component_from_tag(&tag_map, "match_readiness", "比赛准备度", readiness),
            component_from_tag(&tag_map, "form_multiplier", "近期状态", form),
            component_from_tag(&tag_map, "fatigue_multiplier", "体能负荷", fatigue),
            component_from_tag(&tag_map, "position_fit", "位置适配", position_fit),
            component_from_tag(&tag_map, "tactical_fit", "战术适配", tactical_fit),
            component_from_tag(&tag_map, "chemistry_fit", "组合熟悉度", chemistry_fit),
            component_from_tag(
                &tag_map,
                "realization_multiplier",
                "兑现率修正",
                realization,
            ),
            component(
                "expected_minutes_share",
                "预计分钟比例",
                minute_share,
                1.0,
                if request.expected_minutes.is_some() {
                    "request"
                } else {
                    "dynamic_tag"
                },
            ),
            component(
                "data_confidence",
                "数据可信度",
                confidence_multiplier,
                overall_confidence,
                "calculation",
            ),
        ];
        let multiplier_product = components.iter().map(|item| item.value).product::<f64>();
        let effective_contribution = (base_ability * multiplier_product).clamp(0.0, 125.0);

        Ok(PlayerMatchContribution {
            player_id: request.player_id,
            player_name,
            match_id: request.match_id,
            as_of: request.as_of,
            position_code: request.position_code.clone(),
            tactical_role_code,
            tactical_role_origin: tactical_role_origin.to_string(),
            tactical_role_source_position_code,
            tactical_role_confidence,
            base_ability,
            base_ability_confidence: base_confidence,
            effective_contribution,
            overall_confidence,
            expected_minutes_share: minute_share,
            starting_probability,
            components,
            applied_tags,
            calculation_version: CONTRIBUTION_VERSION.to_string(),
        })
    }
}

async fn validate_dynamic_tag_draft(
    store: &PostgresStore,
    draft: &PlayerDynamicTagDraft,
) -> PersistenceResult<()> {
    if draft.valid_to <= draft.valid_from {
        return Err(PersistenceError::InvalidState(
            "动态标签失效时间必须晚于生效时间".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&draft.confidence) {
        return Err(PersistenceError::InvalidState(
            "动态标签 confidence 必须在 0–1 之间".to_string(),
        ));
    }
    if draft.sample_size < 0 {
        return Err(PersistenceError::InvalidState(
            "动态标签 sample_size 不能为负数".to_string(),
        ));
    }
    if draft.calculation_version.trim().is_empty() {
        return Err(PersistenceError::InvalidState(
            "动态标签 calculation_version 不能为空".to_string(),
        ));
    }
    let range = sqlx::query(
        r#"
        SELECT minimum_value, maximum_value
        FROM feature.player_dynamic_tag_definitions
        WHERE code = $1
        "#,
    )
    .bind(draft.tag_code.trim())
    .fetch_optional(&store.pool)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState(format!("未知动态标签：{}", draft.tag_code)))?;
    let minimum: f64 = range.try_get("minimum_value")?;
    let maximum: f64 = range.try_get("maximum_value")?;
    if draft.value < minimum || draft.value > maximum {
        return Err(PersistenceError::InvalidState(format!(
            "动态标签 {} 的值必须在 {}–{} 之间",
            draft.tag_code, minimum, maximum
        )));
    }
    Ok(())
}

fn dynamic_tag_definition_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PlayerDynamicTagDefinitionRecord> {
    Ok(PlayerDynamicTagDefinitionRecord {
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        category: row.try_get("category")?,
        minimum_value: row.try_get("minimum_value")?,
        maximum_value: row.try_get("maximum_value")?,
        default_value: row.try_get("default_value")?,
        default_ttl_hours: row.try_get("default_ttl_hours")?,
        is_multiplier: row.try_get("is_multiplier")?,
        description: row.try_get("description")?,
    })
}

pub(crate) fn player_dynamic_tag_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PlayerDynamicTagRecord> {
    Ok(PlayerDynamicTagRecord {
        id: row.try_get("id")?,
        player_id: row.try_get("player_id")?,
        tag_code: row.try_get("tag_code")?,
        tag_name: row.try_get("tag_name")?,
        category: row.try_get("category")?,
        value: row.try_get("value")?,
        label: row.try_get("label")?,
        confidence: row.try_get("confidence")?,
        observed_at: row.try_get("observed_at")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        position_code: row.try_get("position_code")?,
        opponent_team_id: row.try_get("opponent_team_id")?,
        opponent_team_name: row.try_get("opponent_team_name")?,
        sample_size: row.try_get("sample_size")?,
        source_type: row.try_get("source_type")?,
        calculation_version: row.try_get("calculation_version")?,
        metadata: row.try_get::<Value, _>("metadata")?,
    })
}

fn availability_multiplier(status: Option<&str>) -> f64 {
    match status.unwrap_or("unknown") {
        "available" => 1.0,
        "doubtful" => 0.75,
        "unavailable" => 0.0,
        "injured" => 0.15,
        "suspended" => 0.0,
        "rested" => 0.85,
        "returning" => 0.70,
        _ => 0.80,
    }
}

fn tag_value(tags: &HashMap<&str, &PlayerDynamicTagRecord>, code: &str, default: f64) -> f64 {
    tags.get(code).map(|tag| tag.value).unwrap_or(default)
}

fn component(
    code: &str,
    label: &str,
    value: f64,
    confidence: f64,
    source: impl Into<String>,
) -> ContributionComponent {
    ContributionComponent {
        code: code.to_string(),
        label: label.to_string(),
        value,
        confidence,
        source: source.into(),
    }
}

fn component_from_tag(
    tags: &HashMap<&str, &PlayerDynamicTagRecord>,
    code: &str,
    label: &str,
    value: f64,
) -> ContributionComponent {
    if let Some(tag) = tags.get(code) {
        component(code, label, value, tag.confidence, tag.source_type.clone())
    } else {
        component(code, label, value, 0.5, "default")
    }
}
