use crate::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{SourcePolicyVersionDraft, SourcePolicyVersionRecord};
use serde_json::json;
use sqlx::Row;
use std::collections::BTreeMap;
use uuid::Uuid;

impl PostgresStore {
    pub async fn register_source_policy_version(
        &self,
        draft: &SourcePolicyVersionDraft,
    ) -> PersistenceResult<SourcePolicyVersionRecord> {
        if draft.policy_key.trim().is_empty() || draft.version.trim().is_empty() {
            return Err(PersistenceError::InvalidState(
                "来源策略键和版本不能为空".to_string(),
            ));
        }
        validate_source_policy_definition(draft)?;
        let content_sha256 = sha256_json(&draft.definition)?;
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(format!(
                "source-policy:{}@{}",
                draft.policy_key, draft.version
            ))
            .execute(&mut *tx)
            .await?;
        if let Some(row) = sqlx::query(
            r#"
            SELECT id, policy_key, version, content_sha256, created_at
            FROM research.source_policy_versions
            WHERE policy_key = $1 AND version = $2
            "#,
        )
        .bind(&draft.policy_key)
        .bind(&draft.version)
        .fetch_optional(&mut *tx)
        .await?
        {
            let existing: String = row.try_get("content_sha256")?;
            if existing != content_sha256 {
                return Err(PersistenceError::InvalidState(format!(
                    "来源策略{}@{}已经存在但内容指纹不同；必须发布新版本",
                    draft.policy_key, draft.version
                )));
            }
            let record = SourcePolicyVersionRecord {
                id: row.try_get("id")?,
                policy_key: row.try_get("policy_key")?,
                version: row.try_get("version")?,
                content_sha256: existing,
                created_at: row.try_get("created_at")?,
            };
            tx.commit().await?;
            return Ok(record);
        }

        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO research.source_policy_versions (
                id, policy_key, version, competition_profile_id,
                definition, content_sha256, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, policy_key, version, content_sha256, created_at
            "#,
        )
        .bind(id)
        .bind(&draft.policy_key)
        .bind(&draft.version)
        .bind(draft.competition_profile_id)
        .bind(serde_json::to_value(&draft.definition)?)
        .bind(&content_sha256)
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "source_policy_registered",
            "source_policy",
            Some(id.to_string()),
            json!({
                "policy_key": draft.policy_key,
                "version": draft.version,
                "content_sha256": content_sha256,
            }),
        )
        .await?;
        let record = SourcePolicyVersionRecord {
            id: row.try_get("id")?,
            policy_key: row.try_get("policy_key")?,
            version: row.try_get("version")?,
            content_sha256: row.try_get("content_sha256")?,
            created_at: row.try_get("created_at")?,
        };
        tx.commit().await?;
        Ok(record)
    }
}

fn validate_source_policy_definition(draft: &SourcePolicyVersionDraft) -> PersistenceResult<()> {
    let mut tiers = BTreeMap::new();
    for tier in &draft.definition.tiers {
        if tier.key.trim().is_empty() || tier.rank > 1000 {
            return Err(PersistenceError::InvalidState(
                "来源等级键不能为空且rank不能超过1000".to_string(),
            ));
        }
        if tiers.insert(tier.key.as_str(), tier.rank).is_some() {
            return Err(PersistenceError::InvalidState(
                "来源策略包含重复等级键".to_string(),
            ));
        }
    }
    if !tiers.contains_key(draft.definition.default_tier.as_str()) {
        return Err(PersistenceError::InvalidState(
            "来源策略默认等级未在tiers中定义".to_string(),
        ));
    }
    let mut domains = BTreeMap::new();
    for rule in &draft.definition.domain_rules {
        let domain = normalize_domain(&rule.domain);
        if domain.is_empty() || !tiers.contains_key(rule.tier.as_str()) {
            return Err(PersistenceError::InvalidState(
                "来源域名规则引用了无效域名或等级".to_string(),
            ));
        }
        if domains.insert(domain, rule.tier.as_str()).is_some() {
            return Err(PersistenceError::InvalidState(
                "来源策略包含重复域名规则".to_string(),
            ));
        }
    }
    Ok(())
}

fn normalize_domain(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("*.")
        .trim_start_matches("www.")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::validate_source_policy_definition;
    use football_domain::SourcePolicyVersionDraft;
    use football_domain::{SourcePolicyDefinition, SourceTierDefinition, SourceTierRule};
    use serde_json::json;

    #[test]
    fn source_policy_rejects_duplicate_domains_and_unknown_tiers() {
        let valid = SourcePolicyVersionDraft {
            policy_key: "default".to_string(),
            version: "1.0.0".to_string(),
            competition_profile_id: None,
            definition: SourcePolicyDefinition {
                schema_version: "football.p4-source-policy.v1".to_string(),
                default_tier: "unclassified".to_string(),
                tiers: vec![SourceTierDefinition {
                    key: "unclassified".to_string(),
                    rank: 100,
                }],
                domain_rules: vec![],
            },
            metadata: json!({}),
        };
        validate_source_policy_definition(&valid).expect("valid source policy");

        let mut invalid = valid.clone();
        invalid.definition.domain_rules = vec![SourceTierRule {
            domain: "example.com".to_string(),
            tier: "missing".to_string(),
        }];
        assert!(validate_source_policy_definition(&invalid).is_err());
    }
    #[test]
    fn policy_preflight_rejects_duplicate_normalized_domains_and_invalid_tiers() {
        let valid = SourcePolicyVersionDraft {
            policy_key: "test".into(),
            version: "1.0.0".into(),
            competition_profile_id: None,
            definition: SourcePolicyDefinition {
                schema_version: "football.p4-source-policy.v1".into(),
                default_tier: "official".into(),
                tiers: vec![SourceTierDefinition {
                    key: "official".into(),
                    rank: 1000,
                }],
                domain_rules: vec![SourceTierRule {
                    domain: "example.com".into(),
                    tier: "official".into(),
                }],
            },
            metadata: json!({}),
        };
        validate_source_policy_definition(&valid).unwrap();
        for invalid_case in [
            "rank",
            "duplicate_tier",
            "default",
            "empty_tier",
            "duplicate_domain",
            "empty_domain",
        ] {
            let mut invalid = valid.clone();
            match invalid_case {
                "rank" => invalid.definition.tiers[0].rank = 1001,
                "duplicate_tier" => invalid
                    .definition
                    .tiers
                    .push(valid.definition.tiers[0].clone()),
                "default" => invalid.definition.default_tier = "absent".into(),
                "empty_tier" => invalid.definition.tiers[0].key = " ".into(),
                "duplicate_domain" => invalid.definition.domain_rules.push(SourceTierRule {
                    domain: " *.example.com ".into(),
                    tier: "official".into(),
                }),
                "empty_domain" => invalid.definition.domain_rules[0].domain = " ".into(),
                _ => unreachable!(),
            }
            assert!(
                validate_source_policy_definition(&invalid).is_err(),
                "{invalid_case}"
            );
        }
    }
}
