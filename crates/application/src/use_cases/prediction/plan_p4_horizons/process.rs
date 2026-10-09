use super::{prepare, queue, schedule};
use crate::use_cases::prediction::shared::p4_planning::validate_existing_task_identity;
use crate::use_cases::prediction::P4PlanningAccess;
use crate::ApplicationResult;
use chrono::Utc;
use football_domain::{P4FreezeTaskRecord, P4FreezeTaskState, P4Horizon, PlanP4HorizonsCommand};
use uuid::Uuid;

pub(crate) async fn execute<P: P4PlanningAccess + ?Sized>(
    port: &P,
    command: PlanP4HorizonsCommand,
) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
    let plan = prepare::prepare(port, command).await?;
    let now = Utc::now();
    let mut tasks = Vec::with_capacity(P4Horizon::CANONICAL.len());
    for horizon in P4Horizon::CANONICAL {
        let (data_cutoff_at, idempotency_key) = schedule::horizon_identity(&plan, horizon)?;
        if let Some(existing) = port
            .find_freeze_task_by_idempotency(&idempotency_key)
            .await?
        {
            validate_existing_task_identity(
                &existing,
                &plan.decision,
                plan.research_schema.id,
                plan.snapshot_schema.id,
                &plan.requested_fact_keys,
            )?;
            tasks.push(queue::resume(port, &existing, now).await?);
            continue;
        }
        let draft = schedule::task_draft(
            &plan,
            horizon,
            data_cutoff_at,
            idempotency_key,
            now,
            Uuid::new_v4(),
        );
        let task = port.create_freeze_task(&draft).await?;
        if draft.state == P4FreezeTaskState::Missed {
            tasks.push(task);
            continue;
        }
        tasks.push(queue::resume(port, &task, now).await?);
    }
    Ok(tasks)
}
