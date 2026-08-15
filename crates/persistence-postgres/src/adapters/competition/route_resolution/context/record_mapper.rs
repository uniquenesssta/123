use super::record_row::{SeasonContextRow, StageContextRow};
use crate::{parse_competition_kind, PersistenceResult};
use football_domain::ResolvedCompetitionContext;

pub(super) fn map_stage_context(
    row: StageContextRow,
) -> PersistenceResult<ResolvedCompetitionContext> {
    Ok(ResolvedCompetitionContext {
        competition_id: Some(row.competition_id),
        season_id: Some(row.season_id),
        stage_id: Some(row.stage_id),
        competition_kind: parse_competition_kind(&row.stage_kind)?,
    })
}

pub(super) fn map_season_context(
    row: SeasonContextRow,
) -> PersistenceResult<ResolvedCompetitionContext> {
    Ok(ResolvedCompetitionContext {
        competition_id: Some(row.competition_id),
        season_id: Some(row.season_id),
        stage_id: None,
        competition_kind: parse_competition_kind(&row.competition_kind)?,
    })
}
