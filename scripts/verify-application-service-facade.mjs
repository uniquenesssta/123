import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8").replaceAll("\r\n", "\n");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
function rustFiles(dir) {
  const absolute = join(root, dir);
  const result = [];
  for (const entry of readdirSync(absolute, { withFileTypes: true })) {
    const child = join(absolute, entry.name);
    if (entry.isDirectory()) result.push(...rustFiles(relative(root, child)));
    else if (entry.name.endsWith(".rs")) result.push(relative(root, child).replaceAll("\\", "/"));
  }
  return result;
}

const rootService = read("crates/application/src/service/application_service.rs");
const databaseFacade = read("crates/application/src/services/database/facade.rs");
const bootstrapFacade = read("crates/application/src/services/database/bootstrap.rs");
const lifecycleConnect = read("crates/application/src/use_cases/application_facade/database_lifecycle/connect.rs");
const lifecycleReset = read("crates/application/src/use_cases/application_facade/database_lifecycle/reset.rs");
const applicationBootstrap = read("crates/application/src/use_cases/application_facade/bootstrap.rs");
const researchFacade = read("crates/application/src/services/research/facade.rs");
const researchService = read("crates/application/src/services/research/service.rs");
const predictionFacade = read("crates/application/src/services/prediction/facade.rs");
const predictionCompatibility = read("crates/application/src/services/prediction/compatibility.rs");
const p4Facade = read("crates/application/src/services/p4_orchestration/facade.rs");

check(!existsSync(join(root, "crates/application/src/p4_orchestration.rs")), "旧根级 p4_orchestration owner 仍存在");
check(!rootService.includes("async fn"), "ApplicationService 根对象仍承载异步业务方法");
check(!rootService.includes("AtomicBool") && !rootService.includes("RwLock"), "ApplicationService 根对象仍直接持有业务运行状态");
for (const token of ["football_persistence_postgres", "sqlx::", "reqwest::", "PostgresStore", "transition_store(", "execute_with_sink"]) {
  check(!rootService.includes(token), `ApplicationService 根对象泄漏具体实现：${token}`);
}

check(databaseFacade.includes("database_lifecycle::connect::execute(self, options).await"), "connect_database 未收敛为 lifecycle use case 委托");
check(databaseFacade.includes("database_lifecycle::reset::execute(self, options, confirmation).await"), "reset_database 未收敛为 lifecycle use case 委托");
for (const token of ["prepare_connection", "register_built_ins", "register_persistence_artifacts", "register_openai_research_artifacts", "start_job_worker", "reset_to_pristine", "if let Err", "match prepared.health"]) {
  check(!databaseFacade.includes(token), `Database facade 仍持有生命周期业务编排：${token}`);
}
check(lifecycleConnect.includes("prepare_connection") && lifecycleConnect.includes("start_job_worker") && lifecycleConnect.includes("p4_orchestration.start"), "连接生命周期未完整迁出 facade");
check(lifecycleReset.includes("reset_to_pristine") && lifecycleReset.includes("connect::execute"), "reset 生命周期未完整迁出 facade");
check(bootstrapFacade.includes("bootstrap::execute(self).await"), "bootstrap facade 未收敛为单一 use case 委托");
for (const token of ["transition_store(", "list_recent_runs(50)", "load_hierarchy", "load_catalog", "database_health_from_snapshot"]) {
  check(!bootstrapFacade.includes(token), `bootstrap facade 仍持有聚合业务：${token}`);
}
check(applicationBootstrap.includes("prediction_compatibility::list_recent_runs"), "bootstrap 未通过 Prediction compatibility 边界读取历史运行");

check(!researchFacade.includes("P4ManualConflictAccess"), "Research facade 仍组装人工冲突多 Port access");
check(!researchFacade.includes("&session, &session"), "Research facade 仍重复组装 OpenAI 多 Port access");
check(researchService.includes("execute_p4_openai_research_session") && researchService.includes("resolve_p4_conflict_session"), "ResearchService 缺少单 session 兼容委托边界");

check(!predictionFacade.includes("model_run_list_item_from_port"), "Prediction facade 仍直接转换 persistence compatibility DTO");
check(!predictionFacade.includes(".into_iter()"), "Prediction facade 仍持有 recent-run 映射流程");
check(predictionFacade.includes("compatibility::list_recent_runs"), "Prediction facade 未委托 compatibility 模块");
check(predictionCompatibility.includes("model_run_list_item_from_port") && predictionCompatibility.includes(".into_iter()"), "Prediction compatibility DTO 映射未迁入独立模块");

for (const token of ["claim_next_p4_job", "complete_p4_job", "fail_p4_job", "for ", "loop ", "match "]) {
  check(!p4Facade.includes(token), `P4 facade 仍持有 orchestration 业务：${token}`);
}

const applicationImplFiles = rustFiles("crates/application/src").filter((path) => read(path).includes("impl ApplicationService"));
for (const path of applicationImplFiles) {
  const source = read(path);
  check(path.includes("/services/") || path === "crates/application/src/service/application_service.rs", `ApplicationService impl 仍位于非 Service 兼容层：${path}`);
  for (const token of ["football_persistence_postgres", "sqlx::", "reqwest::", "PostgresStore"] ) {
    check(!source.includes(token), `ApplicationService facade 泄漏具体实现 ${token}: ${path}`);
  }
}

if (failures.length) {
  console.error("ApplicationService 兼容门面验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}
console.log(`ApplicationService 兼容门面验证通过：${applicationImplFiles.length} 个 impl 文件仅保留兼容入口/会话获取/单一委托，bootstrap、数据库生命周期、Research 多 Port 组装、Prediction DTO 映射与 P4 orchestration 均已迁出门面。`);
