use super::{mapper::map_player_dynamic_tag, row::PlayerDynamicTagRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::PlayerDynamicTagRecord;
use uuid::Uuid;

impl PostgresStore {
    pub async fn read_player_dynamic_tag(
        &self,
        tag_id: Uuid,
    ) -> PersistenceResult<PlayerDynamicTagRecord> {
        let row = sqlx::query_as::<_, PlayerDynamicTagRow>(
            r#"
            SELECT tag.id, tag.player_id, tag.tag_code, definition.name AS tag_name,
                   definition.category, tag.value, tag.label, tag.confidence,
                   tag.observed_at, tag.valid_from, tag.valid_to,
                   tag.competition_id, competition.name AS competition_name,
                   tag.position_code, tag.opponent_team_id,
                   opponent.canonical_name AS opponent_team_name,
                   tag.sample_size, tag.source_type, tag.calculation_version,
                   tag.metadata
            FROM feature.player_dynamic_tags tag
            JOIN feature.player_dynamic_tag_definitions definition
              ON definition.code = tag.tag_code
            LEFT JOIN football.competitions competition ON competition.id = tag.competition_id
            LEFT JOIN football.teams opponent ON opponent.id = tag.opponent_team_id
            WHERE tag.id = $1
            "#,
        )
        .bind(tag_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(map_player_dynamic_tag(row))
    }
}
