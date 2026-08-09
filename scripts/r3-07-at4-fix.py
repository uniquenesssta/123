from pathlib import Path

GENERATOR = Path(__file__).resolve().with_name("r3-07-at4.py")
text = GENERATOR.read_text(encoding="utf-8")


def replace_generator_once(old: str, new: str, label: str) -> None:
    global text
    if text.count(old) != 1:
        raise RuntimeError(f"AT4 fix anchor mismatch: {label}")
    text = text.replace(old, new, 1)


replace_generator_once(
    '''replace_once(
    "crates/application/src/p4_workbench.rs",
    "reconcile_p4_task_after_manual_decision(&store, task.id, research_run_id).await?;",
    "reconcile_p4_task_after_manual_decision(self, &store, task.id, research_run_id).await?;",
    "manual conflict idempotent reconciliation call",
)
replace_once(
    "crates/application/src/p4_workbench.rs",
    "reconcile_p4_task_after_manual_decision(&store, task.id, research_run_id).await?;",
    "reconcile_p4_task_after_manual_decision(self, &store, task.id, research_run_id).await?;",
    "manual conflict post-write reconciliation call",
)
''',
    '''workbench_path = ROOT / "crates/application/src/p4_workbench.rs"
workbench = workbench_path.read_text(encoding="utf-8")
old_reconcile_call = "reconcile_p4_task_after_manual_decision(&store, task.id, research_run_id).await?;"
new_reconcile_call = "reconcile_p4_task_after_manual_decision(self, &store, task.id, research_run_id).await?;"
if workbench.count(old_reconcile_call) != 2:
    raise RuntimeError("AT4 anchor mismatch: expected exactly two manual conflict reconciliation calls")
workbench_path.write_text(
    workbench.replace(old_reconcile_call, new_reconcile_call, 2),
    encoding="utf-8",
    newline="\\n",
)
''',
    "two intentional workbench reconciliation calls",
)

replace_generator_once(
    '''mod context;
mod execution;
mod transitions;

pub(crate) async fn execute(
    workflow: &dyn PredictionWorkflowPort,
    jobs: &dyn JobQueuePort,
    artifacts: &dyn ResearchArtifactPort,
    audit: &dyn ResearchGatewayAuditPort,
    pipeline: &dyn FactPipelineAccess,
    task_id: Uuid,
    job_id: Uuid,
) -> ApplicationResult<Value> {
    execution::execute(workflow, jobs, artifacts, audit, pipeline, task_id, job_id).await
}
''',
    '''mod context;
mod execution;
mod transitions;

pub(crate) struct P4ResearchWorkerAccess<'a> {
    pub(crate) workflow: &'a dyn PredictionWorkflowPort,
    pub(crate) jobs: &'a dyn JobQueuePort,
    pub(crate) artifacts: &'a dyn ResearchArtifactPort,
    pub(crate) audit: &'a dyn ResearchGatewayAuditPort,
    pub(crate) pipeline: &'a dyn FactPipelineAccess,
}

pub(crate) async fn execute(
    access: P4ResearchWorkerAccess<'_>,
    task_id: Uuid,
    job_id: Uuid,
) -> ApplicationResult<Value> {
    execution::execute(
        access.workflow,
        access.jobs,
        access.artifacts,
        access.audit,
        access.pipeline,
        task_id,
        job_id,
    )
    .await
}
''',
    "P4 worker access bundle",
)

replace_generator_once(
    '''    ledger,
    openai_gateway::{self, OpenAiResearchCommand},
    p4_worker,
};
''',
    '''    ledger,
    openai_gateway::{self, OpenAiResearchCommand},
    p4_worker::{self, P4ResearchWorkerAccess},
};
''',
    "ResearchService worker access import",
)

replace_generator_once(
    '''    pub(crate) async fn execute_p4_research_task(
        &self,
        workflow: &dyn PredictionWorkflowPort,
        jobs: &dyn JobQueuePort,
        artifacts: &dyn ResearchArtifactPort,
        audit: &dyn ResearchGatewayAuditPort,
        pipeline: &dyn FactPipelineAccess,
        task_id: Uuid,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        p4_worker::execute(
            workflow, jobs, artifacts, audit, pipeline, task_id, job_id,
        )
        .await
    }
''',
    '''    pub(crate) async fn execute_p4_research_task(
        &self,
        access: P4ResearchWorkerAccess<'_>,
        task_id: Uuid,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        p4_worker::execute(access, task_id, job_id).await
    }
''',
    "ResearchService worker access signature",
)

replace_generator_once(
    '''replace_once(
    "crates/application/src/services/research/facade.rs",
    "use football_research_gateway::{CancellationToken, GatewayExecution};\\n",
    "use football_research_gateway::{CancellationToken, GatewayExecution};\\nuse serde_json::Value;\\nuse uuid::Uuid;\\n",
    "research facade worker imports",
)
''',
    '''replace_once(
    "crates/application/src/services/research/facade.rs",
    "use football_research_gateway::{CancellationToken, GatewayExecution};\\n",
    "use crate::use_cases::research::p4_worker::P4ResearchWorkerAccess;\\nuse football_research_gateway::{CancellationToken, GatewayExecution};\\nuse serde_json::Value;\\nuse uuid::Uuid;\\n",
    "research facade worker imports",
)
''',
    "Research facade worker access import",
)

replace_generator_once(
    '''        let session = self.research_session().await?;
        self.research
            .execute_p4_research_task(
                &session,
                &session,
                &session,
                &session,
                &session,
                task_id,
                job_id,
            )
            .await
''',
    '''        let session = self.research_session().await?;
        let access = P4ResearchWorkerAccess {
            workflow: &session,
            jobs: &session,
            artifacts: &session,
            audit: &session,
            pipeline: &session,
        };
        self.research
            .execute_p4_research_task(access, task_id, job_id)
            .await
''',
    "Research facade worker access construction",
)

GENERATOR.write_text(text, encoding="utf-8", newline="\n")
print("AT4 generator patched for workbench calls and cohesive P4 worker access bundle")
