use football_domain::{
    CompetitionBindingDraft, CompetitionBindingSummary, ResolvedCompetitionContext,
};
use serde_json::json;
use uuid::Uuid;

use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};

use super::{
    insert_binding::{insert_binding, NewBinding},
    validation::validate_binding_draft,
};
use crate::adapters::competition::bindings::{package_route_metadata, read_binding};

impl PostgresStore {
    pub async fn create_competition_binding(
        &self,
        draft: &CompetitionBindingDraft,
    ) -> PersistenceResult<CompetitionBindingSummary> {
        validate_binding_draft(draft)?;

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

        insert_binding(
            &mut tx,
            NewBinding {
                id,
                binding_name: &binding_name,
                resolved: &resolved,
                model_version_id,
                parameter_set_id,
                rule_package_id: draft.rule_package_id,
                priority: draft.priority,
                valid_from: draft.valid_from,
                valid_to: draft.valid_to,
            },
        )
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

        read_binding(self, id).await
    }
}
