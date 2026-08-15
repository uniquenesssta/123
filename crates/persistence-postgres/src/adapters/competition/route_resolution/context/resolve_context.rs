use super::record_mapper::{map_season_context, map_stage_context};
use super::season_context::read_season_context;
use super::stage_context::read_stage_context;
use super::validate_scope::ensure_scope_id;
use crate::{PersistenceResult, PostgresStore};
use football_domain::{CompetitionKind, ResolvedCompetitionContext};
use uuid::Uuid;

impl PostgresStore {
    pub async fn resolve_competition_context(
        &self,
        competition_id: Option<Uuid>,
        season_id: Option<Uuid>,
        stage_id: Option<Uuid>,
        fallback_kind: CompetitionKind,
    ) -> PersistenceResult<ResolvedCompetitionContext> {
        if let Some(stage_id_value) = stage_id {
            let row = read_stage_context(&self.pool, stage_id_value).await?;
            ensure_scope_id("赛事", competition_id, row.competition_id)?;
            ensure_scope_id("赛季", season_id, row.season_id)?;
            return map_stage_context(row);
        }

        if let Some(season_id_value) = season_id {
            let row = read_season_context(&self.pool, season_id_value).await?;
            ensure_scope_id("赛事", competition_id, row.competition_id)?;
            return map_season_context(row);
        }

        if let Some(competition_id_value) = competition_id {
            let competition = self.read_competition(competition_id_value).await?;
            return Ok(ResolvedCompetitionContext {
                competition_id: Some(competition.id),
                season_id: None,
                stage_id: None,
                competition_kind: competition.competition_kind,
            });
        }

        Ok(ResolvedCompetitionContext {
            competition_id: None,
            season_id: None,
            stage_id: None,
            competition_kind: fallback_kind,
        })
    }
}
