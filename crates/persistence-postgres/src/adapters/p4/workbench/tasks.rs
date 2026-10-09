use super::{conflicts, evidence, research};
use crate::{PersistenceResult, PostgresStore};
use football_domain::P4TaskWorkspace;
use uuid::Uuid;

impl PostgresStore {
    pub async fn read_p4_task_workspace(
        &self,
        task_id: Uuid,
    ) -> PersistenceResult<P4TaskWorkspace> {
        let task = self.read_p4_freeze_task(task_id).await?;
        let readiness = self.p4_freeze_readiness(task_id).await?;
        let events = self.list_p4_freeze_task_events(task_id).await?;
        let routes = self.p4_routed_facts(task_id).await?;
        let research_run = if let Some(research_run_id) = task.research_run_id {
            Some(research::read(self, research_run_id).await?)
        } else {
            None
        };
        let evidence = if let Some(research_run_id) = task.research_run_id {
            evidence::read(self, research_run_id).await?
        } else {
            Vec::new()
        };
        let conflicts = if let Some(research_run_id) = task.research_run_id {
            conflicts::read(self, research_run_id, task_id).await?
        } else {
            Vec::new()
        };
        let snapshot = if let Some(snapshot_id) = task.snapshot_id {
            Some(self.read_prematch_snapshot(snapshot_id).await?)
        } else {
            None
        };
        Ok(P4TaskWorkspace {
            task,
            readiness,
            events,
            research_run,
            routes,
            evidence,
            conflicts,
            snapshot,
        })
    }
}
