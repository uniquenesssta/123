use super::{ids::unique_ids, preflight::labels::entity_label, safe_delete::delete_team};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{BulkDeleteBlockedItem, BulkDeleteResult};
use uuid::Uuid;

impl PostgresStore {
    pub async fn bulk_delete_players(
        &self,
        player_ids: &[Uuid],
    ) -> PersistenceResult<BulkDeleteResult> {
        let ids = unique_ids(player_ids);
        let mut deleted_ids = Vec::new();
        let mut blocked = Vec::new();
        for player_id in &ids {
            let label = entity_label(&self.pool, "player", *player_id)
                .await?
                .unwrap_or_else(|| player_id.to_string());
            match self.delete_player(*player_id).await {
                Ok(()) => deleted_ids.push(*player_id),
                Err(error) => blocked.push(BulkDeleteBlockedItem {
                    id: *player_id,
                    label,
                    reason: error.to_string(),
                }),
            }
        }
        Ok(BulkDeleteResult {
            requested_count: ids.len() as u64,
            deleted_ids,
            blocked,
        })
    }

    pub async fn bulk_delete_teams(
        &self,
        team_ids: &[Uuid],
    ) -> PersistenceResult<BulkDeleteResult> {
        let ids = unique_ids(team_ids);
        let mut deleted_ids = Vec::new();
        let mut blocked = Vec::new();
        for team_id in &ids {
            match delete_team(self, *team_id).await {
                Ok(()) => deleted_ids.push(*team_id),
                Err(error) => {
                    let label = entity_label(&self.pool, "team", *team_id)
                        .await?
                        .unwrap_or_else(|| team_id.to_string());
                    blocked.push(BulkDeleteBlockedItem {
                        id: *team_id,
                        label,
                        reason: error.to_string(),
                    });
                }
            }
        }
        Ok(BulkDeleteResult {
            requested_count: ids.len() as u64,
            deleted_ids,
            blocked,
        })
    }
}
