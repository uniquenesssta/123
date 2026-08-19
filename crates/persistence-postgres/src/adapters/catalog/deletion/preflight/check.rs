use super::{
    labels::entity_label,
    references::{coach_reference_counts, player_reference_counts, team_reference_counts},
};
use crate::{
    adapters::catalog::references::validate_entity_type, PersistenceResult, PostgresStore,
};
use football_domain::EntityDeletionCheck;
use uuid::Uuid;

impl PostgresStore {
    pub async fn check_entity_deletion(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> PersistenceResult<EntityDeletionCheck> {
        validate_entity_type(entity_type)?;
        let label = entity_label(&self.pool, entity_type, entity_id).await?;
        let Some(label) = label else {
            return Ok(EntityDeletionCheck {
                entity_type: entity_type.to_string(),
                entity_id,
                label: entity_id.to_string(),
                exists: false,
                can_permanently_delete: false,
                must_archive: false,
                references: Vec::new(),
                reason: "实体不存在".to_string(),
            });
        };
        let references = match entity_type {
            "team" => team_reference_counts(&self.pool, entity_id).await?,
            "player" => player_reference_counts(&self.pool, entity_id).await?,
            "coach" => coach_reference_counts(&self.pool, entity_id).await?,
            _ => unreachable!(),
        };
        let total: i64 = references.iter().map(|item| item.count).sum();
        Ok(EntityDeletionCheck {
            entity_type: entity_type.to_string(),
            entity_id,
            label,
            exists: true,
            can_permanently_delete: total == 0,
            must_archive: total > 0,
            references,
            reason: if total == 0 {
                "没有历史引用，可以永久删除".to_string()
            } else {
                format!("存在 {total} 条历史或业务引用，只允许归档")
            },
        })
    }
}
