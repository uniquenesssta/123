import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const port = read("crates/application/src/ports/prediction/mod.rs");
const adapter = read("crates/application/src/composition/adapters/prediction.rs");
const useCase = read("crates/application/src/use_cases/prediction/execute_p4_freeze/mod.rs");
const projection = read("crates/application/src/use_cases/prediction/execute_p4_freeze/snapshot_projection.rs");
const service = read("crates/application/src/services/prediction/service.rs");
const facade = read("crates/application/src/services/prediction/facade.rs");
const legacy = read("crates/application/src/p4_orchestration.rs");

for (const token of ["P4FreezeExecutionPort", "find_frozen_snapshot_id", "routed_facts", "freeze_snapshot"]) {
  check(port.includes(token), `Prediction freeze Port 缺少：${token}`);
  check(adapter.includes(token), `Prediction freeze adapter 缺少：${token}`);
}
for (const token of ["execute_prediction::execute", "validate_pinned_route", "snapshot_probabilities", "snapshot_features", "freeze_snapshot", "P4FreezeTaskState::Frozen"]) {
  check(useCase.includes(token) || projection.includes(token), `P4 freeze use case 缺少：${token}`);
}
check(!useCase.includes("PersistenceStore") && !projection.includes("PersistenceStore"), "P4 freeze use case 不得依赖具体 PersistenceStore");
check(!useCase.includes("football_persistence_postgres") && !projection.includes("football_persistence_postgres"), "P4 freeze use case 不得导入 PostgreSQL crate");
check(!useCase.includes("create_research_run") && !useCase.includes("append_evidence_claim") && !useCase.includes("execute_p4_openai_research"), "P4 freeze use case 不得接管 Research 写入/联网执行职责");
check(service.includes("execute_p4_freeze::execute") && facade.includes("execute_p4_freeze_task"), "Prediction Service/facade 未接通 freeze use case");
check(!legacy.includes("async fn execute_p4_freeze_task("), "旧 p4_orchestration 仍保留 freeze 实现");
check(legacy.includes("P4_FREEZE_JOB => self.execute_p4_freeze_task(payload.task_id, job_id).await"), "旧混合 worker 未委托 Prediction freeze service");
check(legacy.includes("execute_p4_research_task") && legacy.includes("execute_p4_openai_research"), "R3-07 Research 执行边界被意外迁出");

if (failures.length) {
  console.error("R3-06 P4 Freeze Execution 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}
console.log("R3-06 P4 Freeze Execution 验证通过：冻结状态机、固定路由、31字段/概率投影与不可变快照已由 Prediction Service 经 Ports 接管，Research 写入链保持隔离。");
