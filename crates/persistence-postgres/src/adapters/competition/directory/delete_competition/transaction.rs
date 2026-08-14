use super::{
    deactivate_bindings::deactivate_bindings,
    delete_external_entity_ids::delete_external_entity_ids,
    lock_active_competition::lock_active_competition,
    soft_delete_competition::soft_delete_competition,
};
use crate::{write_audit_event, PersistenceResult, PostgresStore};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn delete_competition(&self, id: Uuid) -> PersistenceResult<()> {
        let mut tx = self.pool.begin().await?;
        let competition_name = lock_active_competition(&mut tx, id).await?;
        soft_delete_competition(&mut tx, id).await?;
        delete_external_entity_ids(&mut tx, id).await?;
        deactivate_bindings(&mut tx, id).await?;
        write_audit_event(
            &mut tx,
            "competition_deleted",
            "competition",
            Some(id.to_string()),
            json!({"name": competition_name, "deletion_mode": "soft_delete"}),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
