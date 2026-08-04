use crate::PersistenceResult;
use chrono::NaiveDate;
use serde_json::{json, Value};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

pub(crate) const ROLE_ORIGIN_LINEUP_OVERRIDE: &str = "lineup_override";
pub(crate) const ROLE_ORIGIN_PLAYER_POSITION_DEFAULT: &str = "player_position_default";
pub(crate) const ROLE_ORIGIN_MISSING: &str = "missing";

#[derive(Debug, Clone)]
pub(crate) struct DefaultTacticalRole {
    pub role_code: String,
    pub position_code: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedTacticalRole {
    pub role_code: Option<String>,
    pub origin: &'static str,
    pub source_position_code: Option<String>,
}

pub(crate) async fn resolve_default_tactical_role_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    player_id: Uuid,
    position_code: Option<&str>,
    as_of: NaiveDate,
) -> PersistenceResult<Option<DefaultTacticalRole>> {
    let requested_position = position_code
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_uppercase);
    let row = sqlx::query(
        r#"
        SELECT position.default_role_code, position.position_code
        FROM football.player_positions position
        WHERE position.player_id = $1
          AND position.default_role_code IS NOT NULL
          AND btrim(position.default_role_code) <> ''
          AND (position.valid_from IS NULL OR position.valid_from <= $3)
          AND (position.valid_to IS NULL OR position.valid_to >= $3)
        ORDER BY
          CASE
            WHEN $2::text IS NOT NULL AND upper(position.position_code) = upper($2) THEN 0
            WHEN position.is_primary THEN 1
            ELSE 2
          END,
          position.proficiency DESC,
          position.valid_from DESC NULLS LAST,
          position.id DESC
        LIMIT 1
        "#,
    )
    .bind(player_id)
    .bind(requested_position.as_deref())
    .bind(as_of)
    .fetch_optional(&mut **tx)
    .await?;
    row.map(|row| {
        Ok(DefaultTacticalRole {
            role_code: row.try_get("default_role_code")?,
            position_code: row.try_get("position_code")?,
        })
    })
    .transpose()
}

pub(crate) fn resolve_tactical_role(
    explicit_role_code: Option<&str>,
    inherited_role: Option<&DefaultTacticalRole>,
) -> ResolvedTacticalRole {
    let explicit_role = explicit_role_code
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    match (explicit_role, inherited_role) {
        (Some(role_code), Some(inherited))
            if role_code.eq_ignore_ascii_case(inherited.role_code.trim()) =>
        {
            ResolvedTacticalRole {
                role_code: Some(role_code),
                origin: ROLE_ORIGIN_PLAYER_POSITION_DEFAULT,
                source_position_code: Some(inherited.position_code.clone()),
            }
        }
        (Some(role_code), _) => ResolvedTacticalRole {
            role_code: Some(role_code),
            origin: ROLE_ORIGIN_LINEUP_OVERRIDE,
            source_position_code: None,
        },
        (None, Some(inherited)) => ResolvedTacticalRole {
            role_code: Some(inherited.role_code.trim().to_string()),
            origin: ROLE_ORIGIN_PLAYER_POSITION_DEFAULT,
            source_position_code: Some(inherited.position_code.clone()),
        },
        (None, None) => ResolvedTacticalRole {
            role_code: None,
            origin: ROLE_ORIGIN_MISSING,
            source_position_code: None,
        },
    }
}

pub(crate) fn normalize_role_origin(
    role_code: Option<&str>,
    requested_origin: Option<&str>,
    inherited_role_code: Option<&str>,
) -> &'static str {
    if role_code.is_none() {
        return ROLE_ORIGIN_MISSING;
    }
    if matches!(requested_origin, Some(ROLE_ORIGIN_LINEUP_OVERRIDE)) {
        return ROLE_ORIGIN_LINEUP_OVERRIDE;
    }
    if matches!(requested_origin, Some(ROLE_ORIGIN_PLAYER_POSITION_DEFAULT)) {
        return ROLE_ORIGIN_PLAYER_POSITION_DEFAULT;
    }
    if role_code
        .zip(inherited_role_code)
        .is_some_and(|(role, inherited)| role.eq_ignore_ascii_case(inherited))
    {
        ROLE_ORIGIN_PLAYER_POSITION_DEFAULT
    } else {
        ROLE_ORIGIN_LINEUP_OVERRIDE
    }
}

pub(crate) fn tactical_role_confidence(
    role_code: Option<&str>,
    role_origin: &str,
    position_proficiency: Option<f64>,
) -> f64 {
    if role_code.is_none() {
        return 0.5;
    }
    match role_origin {
        ROLE_ORIGIN_LINEUP_OVERRIDE => 1.0,
        ROLE_ORIGIN_PLAYER_POSITION_DEFAULT => position_proficiency.unwrap_or(0.5).clamp(0.0, 1.0),
        _ => 0.5,
    }
}

pub(crate) fn metadata_with_role_resolution(
    metadata: &Value,
    resolution: &ResolvedTacticalRole,
) -> Value {
    let mut merged = match metadata {
        Value::Object(map) => Value::Object(map.clone()),
        Value::Null => json!({}),
        value => json!({"original_metadata": value}),
    };
    if let Some(object) = merged.as_object_mut() {
        object.insert(
            "role_origin".to_string(),
            Value::String(resolution.origin.to_string()),
        );
        object.insert(
            "role_resolution_version".to_string(),
            Value::String("player-position-default-v1".to_string()),
        );
        match &resolution.source_position_code {
            Some(position_code) => {
                object.insert(
                    "role_source_position_code".to_string(),
                    Value::String(position_code.clone()),
                );
            }
            None => {
                object.remove("role_source_position_code");
            }
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_role() -> DefaultTacticalRole {
        DefaultTacticalRole {
            role_code: "组织核心".to_string(),
            position_code: "CAM".to_string(),
        }
    }

    #[test]
    fn blank_role_inherits_player_position_default() {
        let inherited = default_role();
        let resolved = resolve_tactical_role(None, Some(&inherited));
        assert_eq!(resolved.role_code.as_deref(), Some("组织核心"));
        assert_eq!(resolved.origin, ROLE_ORIGIN_PLAYER_POSITION_DEFAULT);
        assert_eq!(resolved.source_position_code.as_deref(), Some("CAM"));
    }

    #[test]
    fn explicit_different_role_is_lineup_override() {
        let inherited = default_role();
        let resolved = resolve_tactical_role(Some("影锋"), Some(&inherited));
        assert_eq!(resolved.role_code.as_deref(), Some("影锋"));
        assert_eq!(resolved.origin, ROLE_ORIGIN_LINEUP_OVERRIDE);
        assert!(resolved.source_position_code.is_none());
    }

    #[test]
    fn explicit_same_role_keeps_inherited_lineage() {
        let inherited = default_role();
        let resolved = resolve_tactical_role(Some(" 组织核心 "), Some(&inherited));
        assert_eq!(resolved.role_code.as_deref(), Some("组织核心"));
        assert_eq!(resolved.origin, ROLE_ORIGIN_PLAYER_POSITION_DEFAULT);
    }

    #[test]
    fn contribution_role_origin_and_confidence_share_the_same_semantics() {
        assert_eq!(
            normalize_role_origin(
                Some("影锋"),
                Some(ROLE_ORIGIN_LINEUP_OVERRIDE),
                Some("边锋"),
            ),
            ROLE_ORIGIN_LINEUP_OVERRIDE
        );
        assert_eq!(
            tactical_role_confidence(Some("影锋"), ROLE_ORIGIN_LINEUP_OVERRIDE, Some(0.7),),
            1.0
        );
        assert_eq!(
            tactical_role_confidence(
                Some("组织核心"),
                ROLE_ORIGIN_PLAYER_POSITION_DEFAULT,
                Some(0.82),
            ),
            0.82
        );
        assert_eq!(normalize_role_origin(None, None, None), ROLE_ORIGIN_MISSING);
        assert_eq!(
            tactical_role_confidence(None, ROLE_ORIGIN_MISSING, None),
            0.5
        );
    }

    #[test]
    fn metadata_records_role_lineage_without_dropping_existing_fields() {
        let inherited = default_role();
        let resolved = resolve_tactical_role(None, Some(&inherited));
        let metadata = metadata_with_role_resolution(&json!({"source": "test"}), &resolved);
        assert_eq!(metadata["source"], "test");
        assert_eq!(metadata["role_origin"], ROLE_ORIGIN_PLAYER_POSITION_DEFAULT);
        assert_eq!(metadata["role_source_position_code"], "CAM");
    }
}
