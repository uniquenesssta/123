use super::{
    existence::entity_exists,
    external_id::matching_entity_ids,
    name_candidates::{match_coaches_by_name, match_players_by_name, match_teams_by_name},
    normalization::normalize_name,
    outcome::{ambiguous, exact_match, from_candidates, no_match},
};
use crate::{
    adapters::catalog::references::validate_entity_type, PersistenceResult, PostgresStore,
};
use football_domain::{EntityMatchRequest, EntityMatchResult};

impl PostgresStore {
    pub async fn resolve_entity_reference(
        &self,
        request: &EntityMatchRequest,
    ) -> PersistenceResult<EntityMatchResult> {
        validate_entity_type(&request.entity_type)?;

        if let Some(id) = request.entity_id {
            if entity_exists(&self.pool, &request.entity_type, id).await? {
                return Ok(exact_match(id, "稳定实体 ID 精确匹配"));
            }
        }

        if let (Some(provider_id), Some(external_id)) = (
            request.provider_id,
            request
                .external_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        ) {
            let ids =
                matching_entity_ids(&self.pool, provider_id, &request.entity_type, external_id)
                    .await?;
            if ids.len() == 1 {
                return Ok(exact_match(ids[0], "受信数据源外部 ID 精确匹配"));
            }
            if ids.len() > 1 {
                return Ok(ambiguous(ids, "外部 ID 对应多条实体"));
            }
        }

        let Some(name) = request
            .canonical_name
            .as_deref()
            .map(normalize_name)
            .filter(|value| !value.is_empty())
        else {
            return Ok(no_match());
        };

        let candidates = match request.entity_type.as_str() {
            "team" => {
                match_teams_by_name(&self.pool, &name, request.country_code.as_deref()).await?
            }
            "player" => match_players_by_name(&self.pool, &name, request.date_of_birth).await?,
            "coach" => {
                match_coaches_by_name(&self.pool, &name, request.nationality_code.as_deref())
                    .await?
            }
            _ => unreachable!(),
        };
        Ok(from_candidates(candidates))
    }
}
