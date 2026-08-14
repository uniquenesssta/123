use super::super::source_documents::upsert_rule_source_document;
use super::{
    attach_competition_profile::attach_competition_profile,
    find_existing_package::find_existing_package,
    insert_rule_package::{insert_rule_package, RulePackageInsertValues},
};
use crate::{
    p4_records::register_competition_profile_in_tx, routing::register_model_in_tx, sha256_json,
    write_audit_event, PersistenceError, PersistenceResult, PostgresStore,
};
use chrono::Utc;
use football_domain::{CompetitionProfileVersionDraft, RulePackageDraft, RulePackageSummary};
use football_model_api::ModelDescriptor;
use serde_json::json;

impl PostgresStore {
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
        let source_document_id = upsert_rule_source_document(&mut tx, draft).await?;
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

        let inserted = insert_rule_package(
            &mut tx,
            RulePackageInsertValues {
                draft,
                registration: &registration,
                source_document_id,
                competition_profile_id: competition_profile.id,
                content_sha256: &content_sha256,
                manifest: &manifest,
                profile: &profile,
                routing: &routing,
            },
        )
        .await?;
        let was_inserted = inserted.is_some();
        let package_id = if let Some(id) = inserted {
            id
        } else {
            let existing =
                find_existing_package(&mut tx, &draft.package_key, &draft.version).await?;
            if existing.content_sha256 != content_sha256 {
                return Err(PersistenceError::InvalidState(format!(
                    "规则包 {}@{} 已存在但内容不同；请创建新版本",
                    draft.package_key, draft.version
                )));
            }
            match existing.competition_profile_id {
                Some(profile_id) if profile_id != competition_profile.id => {
                    return Err(PersistenceError::InvalidState(format!(
                        "规则包 {}@{} 已绑定不同赛事Profile版本",
                        draft.package_key, draft.version
                    )));
                }
                None => {
                    attach_competition_profile(&mut tx, existing.id, competition_profile.id)
                        .await?;
                }
                Some(_) => {}
            }
            existing.id
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
}
