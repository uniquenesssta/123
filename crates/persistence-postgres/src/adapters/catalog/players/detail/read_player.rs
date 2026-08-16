use super::{
    abilities::{read_ability_observations, read_ability_profile},
    availability::read_availability,
    external_ids::read_external_ids,
    names::read_names,
    positions::read_positions,
    team_periods::read_team_periods,
};
use crate::{
    adapters::catalog::players::record::{map_player_record, PlayerRecordRow},
    PersistenceResult, PostgresStore,
};
use chrono::Utc;
use football_domain::PlayerDetail;
use uuid::Uuid;

impl PostgresStore {
    pub async fn read_player(&self, player_id: Uuid) -> PersistenceResult<PlayerDetail> {
        let row = sqlx::query_as::<_, PlayerRecordRow>(
            r#"
            SELECT
                id, canonical_name, normalized_name, date_of_birth,
                nationality_code, preferred_foot, height_cm, status, created_at
            FROM football.players
            WHERE id = $1
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.pool)
        .await?;
        let player = map_player_record(row)?;
        let names = read_names(&self.pool, player_id).await?;
        let positions = read_positions(&self.pool, player_id).await?;
        let team_periods = read_team_periods(&self.pool, player_id).await?;
        let availability = read_availability(self, player_id).await?;
        let ability_profile = read_ability_profile(&self.pool, player_id).await?;
        let ability_observations = read_ability_observations(&self.pool, player_id).await?;
        let dynamic_tags = self.list_player_dynamic_tags(player_id, Utc::now()).await?;
        let external_ids = read_external_ids(&self.pool, player_id).await?;
        Ok(PlayerDetail {
            player,
            names,
            positions,
            team_periods,
            availability,
            ability_profile,
            ability_observations,
            dynamic_tags,
            external_ids,
        })
    }
}
