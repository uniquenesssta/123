use super::context::{
    calculate_recent_starter_overlap, load_prediction_context_in_tx, load_review_players_in_tx,
};
use crate::{adapters::review::facts::write_facts_in_tx, PersistenceResult};
use football_domain::{MatchReviewDraft, ReviewPreparationData, ReviewTeamContext};
use sqlx::{Postgres, Transaction};

pub(crate) async fn prepare_review_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    draft: &MatchReviewDraft,
) -> PersistenceResult<ReviewPreparationData> {
    let (match_record, result, substitutions) = write_facts_in_tx(tx, draft).await?;
    let players = load_review_players_in_tx(tx, draft.match_id).await?;
    let mut teams = Vec::with_capacity(2);
    for (team_id, team_name) in [
        (
            match_record.home_team_id,
            match_record.home_team_name.clone(),
        ),
        (
            match_record.away_team_id,
            match_record.away_team_name.clone(),
        ),
    ] {
        let continuity = calculate_recent_starter_overlap(
            tx,
            draft.match_id,
            team_id,
            match_record.kickoff_time,
        )
        .await?;
        teams.push(ReviewTeamContext {
            team_id,
            team_name,
            recent_starter_overlap: continuity,
        });
    }
    let prediction = load_prediction_context_in_tx(
        tx,
        draft.match_id,
        draft.source_run_id,
        draft.result.home_goals_90,
        draft.result.away_goals_90,
    )
    .await?;

    Ok(ReviewPreparationData {
        data_coverage: draft.data_coverage,
        match_record,
        result,
        substitutions,
        players,
        teams,
        prediction,
    })
}
