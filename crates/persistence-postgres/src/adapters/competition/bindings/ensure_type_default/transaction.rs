use football_domain::CompetitionKind;
use serde_json::json;
use uuid::Uuid;

use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};

use super::{
    find_existing_binding::find_existing_binding,
    insert_binding::{insert_binding, NewTypeDefaultBinding},
};
use crate::adapters::competition::bindings::package_route_metadata;

impl PostgresStore {
    pub async fn ensure_type_default_binding(
        &self,
        package_id: Uuid,
        competition_kind: CompetitionKind,
        priority: i32,
        binding_name: &str,
    ) -> PersistenceResult<Uuid> {
        let mut tx = self.pool.begin().await?;
        let (model_version_id, parameter_set_id, package_kind) =
            package_route_metadata(&mut tx, package_id).await?;
        if package_kind != competition_kind {
            return Err(PersistenceError::InvalidState(format!(
                "规则包赛事类型 {} 不能作为 {} 的默认规则",
                package_kind.as_str(),
                competition_kind.as_str()
            )));
        }

        if let Some(id) = find_existing_binding(&mut tx, package_id, competition_kind).await? {
            tx.commit().await?;
            return Ok(id);
        }

        let id = Uuid::new_v4();
        insert_binding(
            &mut tx,
            NewTypeDefaultBinding {
                id,
                binding_name,
                competition_kind,
                model_version_id,
                parameter_set_id,
                rule_package_id: package_id,
                priority,
            },
        )
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
}
