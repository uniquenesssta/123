use super::{mapper::map_player_dynamic_tag, row::PlayerDynamicTagRow};
use crate::{PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::PlayerDynamicTagRecord;
use uuid::Uuid;

impl PostgresStore {
    pub async fn list_player_dynamic_tags(
        &self,
        player_id: Uuid,
        as_of: DateTime<Utc>,
    ) -> PersistenceResult<Vec<PlayerDynamicTagRecord>> {
        let rows = sqlx::query_as::<_, PlayerDynamicTagRow>(
            r#"
            SELECT DISTINCT ON (
                       tag.tag_code, tag.competition_id,
                       tag.position_code, tag.opponent_team_id
                   )
                   tag.id, tag.player_id, tag.tag_code,
                   definition.name AS tag_name, definition.category,
                   tag.value, tag.label, tag.confidence,
                   tag.observed_at, tag.valid_from, tag.valid_to,
                   tag.competition_id, competition.name AS competition_name,
                   tag.position_code, tag.opponent_team_id,
                   opponent.canonical_name AS opponent_team_name,
                   tag.sample_size, tag.source_type,
                   tag.calculation_version, tag.metadata
            FROM feature.player_dynamic_tags tag
            JOIN feature.player_dynamic_tag_definitions definition
              ON definition.code = tag.tag_code
            LEFT JOIN football.competitions competition ON competition.id = tag.competition_id
            LEFT JOIN football.teams opponent ON opponent.id = tag.opponent_team_id
            WHERE tag.player_id = $1
              AND tag.valid_from <= $2
              AND tag.valid_to >= $2
            ORDER BY tag.tag_code, tag.competition_id NULLS FIRST,
                     tag.position_code NULLS FIRST,
                     tag.opponent_team_id NULLS FIRST,
                     tag.observed_at DESC, tag.id DESC
            "#,
        )
        .bind(player_id)
        .bind(as_of)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(map_player_dynamic_tag).collect())
    }
}
