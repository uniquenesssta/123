use super::super::{ids::unique_ids, preflight::labels::entity_label};
use super::write::archive_entity;
use crate::{
    adapters::catalog::references::validate_entity_type, PersistenceResult, PostgresStore,
};
use football_domain::{BulkArchiveFailedItem, BulkArchiveResult};
use uuid::Uuid;

impl PostgresStore {
    pub async fn bulk_archive_entities(
        &self,
        entity_type: &str,
        ids: &[Uuid],
    ) -> PersistenceResult<BulkArchiveResult> {
        validate_entity_type(entity_type)?;
        let ids = unique_ids(ids);
        let mut archived_ids = Vec::new();
        let mut already_archived_ids = Vec::new();
        let mut failed = Vec::new();
        for id in &ids {
            let label = entity_label(&self.pool, entity_type, *id)
                .await?
                .unwrap_or_else(|| id.to_string());
            match archive_entity(&self.pool, entity_type, *id).await {
                Ok(true) => archived_ids.push(*id),
                Ok(false) => already_archived_ids.push(*id),
                Err(error) => failed.push(BulkArchiveFailedItem {
                    id: *id,
                    label,
                    reason: error.to_string(),
                }),
            }
        }
        Ok(BulkArchiveResult {
            entity_type: entity_type.to_string(),
            requested_count: ids.len() as u64,
            archived_ids,
            already_archived_ids,
            failed,
        })
    }
}
