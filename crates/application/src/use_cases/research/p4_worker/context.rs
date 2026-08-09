use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4FreezeTaskRecord;
use serde_json::{json, Value};

pub(super) async fn research_dynamic_context(
    workflow: &dyn PredictionWorkflowPort,
    task: &P4FreezeTaskRecord,
) -> ApplicationResult<Value> {
    let context = workflow.planning_match_context(task.match_id).await?;
    Ok(json!({
        "orchestration_task_id": task.id,
        "match": {
            "match_id": context.match_id,
            "match_key": context.match_key,
            "kickoff_at": context.kickoff_at,
            "home_team": context.home_team_name,
            "away_team": context.away_team_name,
            "competition_id": context.competition_id,
            "season_id": context.season_id,
            "stage_id": context.stage_id,
            "competition_kind": context.competition_kind,
        },
        "horizon": task.horizon.as_str(),
        "data_cutoff_at": task.data_cutoff_at,
        "rules": {
            "facts_only": true,
            "no_external_prediction": true,
            "no_betting_advice": true,
            "missing_facts_must_remain_missing": true,
        }
    }))
}
