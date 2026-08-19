use super::delete_write::{write_player_delete, write_team_delete};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use uuid::Uuid;

impl PostgresStore {
    pub async fn delete_player(&self, player_id: Uuid) -> PersistenceResult<()> {
        let check = self.check_entity_deletion("player", player_id).await?;
        if !check.can_permanently_delete {
            return Err(PersistenceError::InvalidState(check.reason));
        }
        write_player_delete(&self.pool, player_id).await
    }
}

pub(crate) async fn delete_team(store: &PostgresStore, team_id: Uuid) -> PersistenceResult<()> {
    let check = store.check_entity_deletion("team", team_id).await?;
    if !check.can_permanently_delete {
        return Err(PersistenceError::InvalidState(check.reason));
    }
    write_team_delete(&store.pool, team_id).await
}
