from pathlib import Path
import os
import sys

ROOT = Path(__file__).resolve().parents[1]


def read(rel: str) -> str:
    return (ROOT / rel).read_text(encoding="utf-8").replace("\r\n", "\n")


def write(rel: str, content: str) -> None:
    path = ROOT / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8", newline="\n")


def replace_once(rel: str, old: str, new: str) -> None:
    content = read(rel)
    if old not in content:
        raise RuntimeError(f"{rel}: expected source block not found")
    if content.count(old) != 1:
        raise RuntimeError(f"{rel}: expected source block is not unique")
    write(rel, content.replace(old, new, 1))


def set_status_line(rel: str, line: str) -> None:
    content = read(rel)
    lines = content.splitlines()
    for index, current in enumerate(lines):
        if current.startswith("- Atomic Task 2C "):
            lines[index] = line
            write(rel, "\n".join(lines) + "\n")
            return
    for index, current in enumerate(lines):
        if current.startswith("- Atomic Task 2B 已正式关闭为 `DONE`。"):
            lines.insert(index + 1, line)
            write(rel, "\n".join(lines) + "\n")
            return
    raise RuntimeError(f"{rel}: Atomic Task 2B status anchor not found")


def apply_source_changes() -> None:
    replace_once(
        "crates/application/src/ports/prediction/mod.rs",
        "    P4TaskWorkspace, PredictionSummary, PrematchSnapshotDraft, PrematchSnapshotRecord,\n    PreparedMatchPredictionInput, RouteDecision,\n",
        "    P4TaskWorkspace, PredictionSummary, PrematchSnapshotBundle, PrematchSnapshotDraft,\n    PrematchSnapshotRecord, PreparedMatchPredictionInput, RouteDecision,\n",
    )
    replace_once(
        "crates/application/src/ports/prediction/mod.rs",
        "    async fn freeze_snapshot(\n        &self,\n        draft: &PrematchSnapshotDraft,\n    ) -> PortResult<PrematchSnapshotRecord>;\n}\n",
        "    async fn freeze_snapshot(\n        &self,\n        draft: &PrematchSnapshotDraft,\n    ) -> PortResult<PrematchSnapshotRecord>;\n    async fn read_snapshot(&self, snapshot_id: Uuid) -> PortResult<PrematchSnapshotBundle>;\n}\n",
    )

    replace_once(
        "crates/application/src/composition/adapters/prediction.rs",
        "    P4TaskWorkspace, PredictionSummary, PrematchSnapshotDraft, PrematchSnapshotRecord,\n    PreparedMatchPredictionInput, RouteDecision,\n",
        "    P4TaskWorkspace, PredictionSummary, PrematchSnapshotBundle, PrematchSnapshotDraft,\n    PrematchSnapshotRecord, PreparedMatchPredictionInput, RouteDecision,\n",
    )
    replace_once(
        "crates/application/src/composition/adapters/prediction.rs",
        "    async fn freeze_snapshot(\n        &self,\n        draft: &PrematchSnapshotDraft,\n    ) -> PortResult<PrematchSnapshotRecord> {\n        self.transition_store()\n            .freeze_prematch_snapshot(draft)\n            .await\n            .map_err(map_persistence_error)\n    }\n}\n",
        "    async fn freeze_snapshot(\n        &self,\n        draft: &PrematchSnapshotDraft,\n    ) -> PortResult<PrematchSnapshotRecord> {\n        self.transition_store()\n            .freeze_prematch_snapshot(draft)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn read_snapshot(&self, snapshot_id: Uuid) -> PortResult<PrematchSnapshotBundle> {\n        self.transition_store()\n            .read_prematch_snapshot(snapshot_id)\n            .await\n            .map_err(map_persistence_error)\n    }\n}\n",
    )

    replace_once(
        "crates/application/src/use_cases/prediction/mod.rs",
        "pub(crate) mod p4_freeze_readiness;\n",
        "pub(crate) mod p4_freeze_readiness;\npub(crate) mod p4_snapshot;\n",
    )
    write(
        "crates/application/src/use_cases/prediction/p4_snapshot/mod.rs",
        """use crate::ports::prediction::P4FreezeExecutionPort;
use crate::ApplicationResult;
use football_domain::{PrematchSnapshotBundle, PrematchSnapshotDraft, PrematchSnapshotRecord};
use uuid::Uuid;

pub(crate) async fn freeze<P: P4FreezeExecutionPort + ?Sized>(
    port: &P,
    draft: PrematchSnapshotDraft,
) -> ApplicationResult<PrematchSnapshotRecord> {
    Ok(port.freeze_snapshot(&draft).await?)
}

pub(crate) async fn read<P: P4FreezeExecutionPort + ?Sized>(
    port: &P,
    snapshot_id: Uuid,
) -> ApplicationResult<PrematchSnapshotBundle> {
    Ok(port.read_snapshot(snapshot_id).await?)
}
""",
    )

    replace_once(
        "crates/application/src/services/prediction/service.rs",
        "    list_p4_freeze_tasks, list_recent_runs, p4_freeze_readiness, plan_p4_horizons, preview_route,\n",
        "    list_p4_freeze_tasks, list_recent_runs, p4_freeze_readiness, p4_snapshot, plan_p4_horizons,\n    preview_route,\n",
    )
    replace_once(
        "crates/application/src/services/prediction/service.rs",
        "    P4MatchWorkspace, P4TaskWorkspace, PlanP4HorizonsCommand, RouteDecision,\n",
        "    P4MatchWorkspace, P4TaskWorkspace, PlanP4HorizonsCommand, PrematchSnapshotBundle,\n    PrematchSnapshotDraft, PrematchSnapshotRecord, RouteDecision,\n",
    )
    service_marker = "    pub(crate) async fn plan_p4_horizons<P: P4PlanningAccess + ?Sized>(\n"
    service_content = read("crates/application/src/services/prediction/service.rs")
    if service_marker not in service_content:
        raise RuntimeError("service.rs: plan_p4_horizons anchor not found")
    service_methods = """    pub(crate) async fn freeze_p4_prematch_snapshot<P: P4FreezeExecutionPort + ?Sized>(
        &self,
        port: &P,
        draft: PrematchSnapshotDraft,
    ) -> ApplicationResult<PrematchSnapshotRecord> {
        p4_snapshot::freeze(port, draft).await
    }

    pub(crate) async fn read_p4_prematch_snapshot<P: P4FreezeExecutionPort + ?Sized>(
        &self,
        port: &P,
        snapshot_id: Uuid,
    ) -> ApplicationResult<PrematchSnapshotBundle> {
        p4_snapshot::read(port, snapshot_id).await
    }

"""
    write(
        "crates/application/src/services/prediction/service.rs",
        service_content.replace(service_marker, service_methods + service_marker, 1),
    )

    replace_once(
        "crates/application/src/services/prediction/facade.rs",
        "    P4MatchWorkspace, P4TaskWorkspace, PlanP4HorizonsCommand, RouteDecision,\n",
        "    P4MatchWorkspace, P4TaskWorkspace, PlanP4HorizonsCommand, PrematchSnapshotBundle,\n    PrematchSnapshotDraft, PrematchSnapshotRecord, RouteDecision,\n",
    )
    facade_marker = "    pub async fn plan_p4_horizons(\n"
    facade_content = read("crates/application/src/services/prediction/facade.rs")
    if facade_marker not in facade_content:
        raise RuntimeError("facade.rs: plan_p4_horizons anchor not found")
    facade_methods = """    pub async fn freeze_p4_prematch_snapshot(
        &self,
        draft: PrematchSnapshotDraft,
    ) -> ApplicationResult<PrematchSnapshotRecord> {
        let session = self.prediction_session().await?;
        self.prediction
            .freeze_p4_prematch_snapshot(&session, draft)
            .await
    }

    pub async fn read_p4_prematch_snapshot(
        &self,
        snapshot_id: Uuid,
    ) -> ApplicationResult<PrematchSnapshotBundle> {
        let session = self.prediction_session().await?;
        self.prediction
            .read_p4_prematch_snapshot(&session, snapshot_id)
            .await
    }

"""
    write(
        "crates/application/src/services/prediction/facade.rs",
        facade_content.replace(facade_marker, facade_methods + facade_marker, 1),
    )

    replace_once(
        "crates/application/src/p4_persistence.rs",
        "    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, PrematchSnapshotBundle,\n    PrematchSnapshotDraft, PrematchSnapshotRecord, PromptVersionDraft, PromptVersionRecord,\n",
        "    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, PromptVersionDraft,\n    PromptVersionRecord,\n",
    )
    replace_once("crates/application/src/p4_persistence.rs", "use uuid::Uuid;\n\n", "")
    replace_once(
        "crates/application/src/p4_persistence.rs",
        """    pub async fn freeze_p4_prematch_snapshot(
        &self,
        draft: PrematchSnapshotDraft,
    ) -> ApplicationResult<PrematchSnapshotRecord> {
        let store = self.active_store().await?;
        Ok(store.freeze_prematch_snapshot(&draft).await?)
    }

    pub async fn read_p4_prematch_snapshot(
        &self,
        snapshot_id: Uuid,
    ) -> ApplicationResult<PrematchSnapshotBundle> {
        let store = self.active_store().await?;
        Ok(store.read_prematch_snapshot(snapshot_id).await?)
    }
""",
        "",
    )

    write(
        "scripts/verify-prediction-service.mjs",
        """import { existsSync, readFileSync, readdirSync } from \"node:fs\";
import { dirname, join, relative, resolve } from \"node:path\";
import { fileURLToPath } from \"node:url\";
const root = resolve(dirname(fileURLToPath(import.meta.url)), \"..\");
const read = (path) => readFileSync(join(root, path), \"utf8\").replaceAll(\"\\r\\n\", \"\\n\");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
function rustFiles(path) { const absolute = join(root, path); const result = []; for (const entry of readdirSync(absolute, {withFileTypes:true})) { const child=join(absolute,entry.name); if(entry.isDirectory()) result.push(...rustFiles(relative(root,child))); else if(entry.name.endsWith(\".rs\")) result.push(relative(root,child).replaceAll(\"\\\\\",\"/\")); } return result; }
const publicMethods=[\"execute_prediction\",\"inspect_match_prediction_readiness\",\"execute_prediction_from_match\",\"execute_shadow_prediction_from_match\",\"preview_route\",\"dry_run_default_fixture\",\"list_recent_runs\",\"hide_run_from_history\",\"read_run\",\"plan_p4_horizons\",\"list_p4_freeze_tasks\",\"read_p4_freeze_task\",\"list_p4_freeze_task_events\",\"p4_freeze_readiness\",\"read_p4_match_workspace\",\"read_p4_task_workspace\",\"freeze_p4_prematch_snapshot\",\"read_p4_prematch_snapshot\"];
const required=[\"crates/application/src/services/prediction/mod.rs\",\"crates/application/src/services/prediction/service.rs\",\"crates/application/src/services/prediction/facade.rs\",\"crates/application/src/composition/adapters/prediction.rs\",\"crates/application/src/ports/prediction/mod.rs\",\"crates/application/src/use_cases/prediction/mod.rs\",\"crates/application/src/use_cases/prediction/p4_snapshot/mod.rs\",\"crates/application/src/use_cases/prediction/execute_p4_freeze/mod.rs\",\"crates/application/src/use_cases/prediction/execute_p4_freeze/snapshot_projection.rs\"];
for(const path of required) check(existsSync(join(root,path)),`缺少 R3-06 文件：${path}`);
check(!existsSync(join(root,\"crates/application/src/prediction.rs\")),\"旧 prediction.rs 仍残留 Prediction owner 或空转发层\");
const facade=read(\"crates/application/src/services/prediction/facade.rs\"); const service=read(\"crates/application/src/services/prediction/service.rs\"); const adapter=read(\"crates/application/src/composition/adapters/prediction.rs\"); const ports=read(\"crates/application/src/ports/prediction/mod.rs\"); const useCases=read(\"crates/application/src/use_cases/prediction/mod.rs\"); const legacyPersistence=read(\"crates/application/src/p4_persistence.rs\"); const legacyOrchestration=read(\"crates/application/src/p4_orchestration.rs\"); const legacyWorkbench=read(\"crates/application/src/p4_workbench.rs\"); const packageJson=JSON.parse(read(\"package.json\")); const frontend=read(\"scripts/verify-frontend.mjs\");
for(const method of publicMethods){ check(facade.includes(`fn ${method}`),`Prediction facade 缺少公共兼容方法：${method}`); check(service.includes(`fn ${method}`),`PredictionService 缺少职责：${method}`); }
check(ports.includes(\"trait P4FreezeExecutionPort\"),\"Prediction Ports 缺少 P4FreezeExecutionPort\"); check(ports.includes(\"async fn freeze_snapshot(\"),\"P4FreezeExecutionPort 缺少 freeze_snapshot\"); check(ports.includes(\"async fn read_snapshot(\"),\"P4FreezeExecutionPort 缺少 read_snapshot\");
check(adapter.includes(\"impl P4FreezeExecutionPort for ActiveDatabase\"),\"Prediction 组合适配器缺少 P4FreezeExecutionPort\"); check(adapter.includes(\".freeze_prematch_snapshot(draft)\"),\"Prediction 组合适配器未复用既有快照写入\"); check(adapter.includes(\".read_prematch_snapshot(snapshot_id)\"),\"Prediction 组合适配器未复用既有快照读取\");
check(useCases.includes(\"pub(crate) mod p4_snapshot;\"),\"Prediction Use Cases 未登记 P4 Snapshot 模块\");
for(const method of [\"freeze_p4_prematch_snapshot\",\"read_p4_prematch_snapshot\"]) check(!legacyPersistence.includes(`fn ${method}`),`p4_persistence.rs 仍直接持有 Prediction 快照职责：${method}`);
check(!legacyOrchestration.includes(\"async fn execute_p4_freeze_task(\"),\"p4_orchestration.rs 仍实现 P4 freeze execution\"); check(legacyOrchestration.includes(\"self.execute_p4_freeze_task(payload.task_id, job_id)\"),\"旧混合 worker 未委托 Prediction Service 的 freeze use case\");
for(const method of [\"read_p4_match_workspace\",\"read_p4_task_workspace\"]) check(!legacyWorkbench.includes(`fn ${method}`),`p4_workbench.rs 仍持有只读 Prediction workspace 职责：${method}`); check(legacyWorkbench.includes(\"fn resolve_p4_conflict\"),\"R3-07 冲突写入职责被意外移除\");
const predictionFiles=[...rustFiles(\"crates/application/src/services/prediction\"),...rustFiles(\"crates/application/src/use_cases/prediction\")];
for(const path of predictionFiles){const source=read(path); for(const token of [\"football_persistence_postgres\",\"PostgresStore\",\"sqlx::\",\"PgPool\",\"PersistenceStore\"]) check(!source.includes(token),`${path} 泄漏具体持久化实现：${token}`); for(const token of [\"football_model_stub\",\"model_p4\",\"private_model\"]) check(!source.includes(token),`${path} 绕过 model-api/registry 边界：${token}`);}
check(packageJson.scripts?.[\"verify:prediction-service\"]===\"node scripts/verify-prediction-service.mjs\",\"package.json 未登记 R3-06 专项门禁\"); check(packageJson.scripts?.[\"verify:architecture\"]?.includes(\"verify-prediction-service.mjs\"),\"verify:architecture 未接入 R3-06 门禁\"); check(frontend.includes('\"verify-prediction-service.mjs\"'),\"verify:frontend 未接入 R3-06 门禁\");
if(failures.length) throw new Error(`Prediction Service 验证失败\\n${failures.map((item)=>`- ${item}`).join(\"\\n\")}`);
console.log(`Prediction Service 验证通过：${predictionFiles.length} 个 Service/Use Case Rust 文件，18 个公开 Application 职责已进入 Prediction Service/Ports 边界，P4 freeze execution 与 snapshot persistence 均不再由旧混合 owner 直接实现。`);
""",
    )

    replace_once(
        "package.json",
        "node scripts/verify-lineups-service.mjs\",\n",
        "node scripts/verify-lineups-service.mjs && node scripts/verify-prediction-service.mjs\",\n",
    )
    replace_once(
        "package.json",
        "    \"verify:lineups-service\": \"node scripts/verify-lineups-service.mjs\"\n",
        "    \"verify:lineups-service\": \"node scripts/verify-lineups-service.mjs\",\n    \"verify:prediction-service\": \"node scripts/verify-prediction-service.mjs\"\n",
    )
    replace_once(
        "scripts/verify-frontend.mjs",
        "  \"verify-lineups-service.mjs\",\n",
        "  \"verify-lineups-service.mjs\",\n  \"verify-prediction-service.mjs\",\n",
    )

    status = (
        "- Atomic Task 2C 已完成源码迁移：`freeze_p4_prematch_snapshot` / "
        "`read_p4_prematch_snapshot` 已从旧 `p4_persistence.rs` 迁入 Prediction Service / "
        "`use_cases/prediction/p4_snapshot/`，并复用扩展后的 `P4FreezeExecutionPort`；"
        "长期 `verify:prediction-service` 门禁已接入 architecture/frontend。当前专项 Windows 硬门禁尚未完成，"
        "因此 R3-06 继续为 `IN_PROGRESS`、R3-07 继续为 `BLOCKED`。"
    )
    set_status_line("README.md", status)
    set_status_line("docs/modular-rewrite/R03-application-services/README.md", status)


def finalize_docs() -> None:
    run_id = os.environ.get("GITHUB_RUN_ID", "unknown")
    status = (
        f"- Atomic Task 2C 已完成并通过 Windows hard gate run `{run_id}`：公开 P4 Snapshot 写入/读取 API 已从旧 "
        "`p4_persistence.rs` 迁入 Prediction Service / `use_cases/prediction/p4_snapshot/`，"
        "`P4FreezeExecutionPort` 同时承担不可变快照写入与读取，ApplicationService 方法名、参数和返回类型保持兼容；"
        "`verify:prediction-service` 已接入 `verify:architecture` 与 `verify:frontend`。专项验证覆盖 Prediction Service、"
        "Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 "
        "workspace tests。Research schema/prompt/run/evidence/conflict 写入仍保留给 R3-07。当前仅等待 clean 源码树的最终 "
        "Public Platform CI，因此 R3-06 继续为 `IN_PROGRESS`、R3-07 继续为 `BLOCKED`。"
    )
    set_status_line("README.md", status)
    set_status_line("docs/modular-rewrite/R03-application-services/README.md", status)


if __name__ == "__main__":
    if "--finalize" in sys.argv:
        finalize_docs()
    else:
        apply_source_changes()
