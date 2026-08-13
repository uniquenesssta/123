import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), "utf8");
const exists = (relativePath) => fs.existsSync(path.join(root, relativePath));
const failures = [];
const check = (condition, message) => {
  if (!condition) failures.push(message);
};

const requiredFiles = [
  "crates/application/src/services/database/mod.rs",
  "crates/application/src/services/database/service.rs",
  "crates/application/src/services/database/facade.rs",
  "crates/application/src/services/database/bootstrap.rs",
  "crates/application/src/use_cases/database/connect/mod.rs",
  "crates/application/src/use_cases/database/migrate/mod.rs",
  "crates/application/src/use_cases/database/health/mod.rs",
  "crates/application/src/use_cases/database/statistics/mod.rs",
  "crates/application/src/use_cases/database/reset/mod.rs",
];
for (const file of requiredFiles) check(exists(file), `缺少 Database Service 文件：${file}`);
check(!exists("crates/application/src/database.rs"), "旧 crates/application/src/database.rs 尚未删除");

const service = read("crates/application/src/services/database/service.rs");
const facade = read("crates/application/src/services/database/facade.rs");
const bootstrap = read("crates/application/src/services/database/bootstrap.rs");
const applicationBootstrap = read("crates/application/src/use_cases/application_facade/bootstrap.rs");
const lifecycleConnect = read("crates/application/src/use_cases/application_facade/database_lifecycle/connect.rs");
const lifecycleInitialize = read("crates/application/src/use_cases/application_facade/database_lifecycle/initialize.rs");
const lifecycleReset = read("crates/application/src/use_cases/application_facade/database_lifecycle/reset.rs");
const connect = read("crates/application/src/use_cases/database/connect/mod.rs");
const migrate = read("crates/application/src/use_cases/database/migrate/mod.rs");
const health = read("crates/application/src/use_cases/database/health/mod.rs");
const statistics = read("crates/application/src/use_cases/database/statistics/mod.rs");
const reset = read("crates/application/src/use_cases/database/reset/mod.rs");
const ports = read("crates/application/src/ports/database/mod.rs");
const portRegistry = read("crates/application/src/composition/port_registry.rs");
const databaseAdapter = read("crates/application/src/composition/adapters/database.rs");
const applicationService = read("crates/application/src/service/application_service.rs");
const researchService = read("crates/application/src/services/research/service.rs");
const researchArtifactCatalog = read("crates/application/src/use_cases/research/artifact_catalog.rs");
const tauri = read("src-tauri/src/commands/database.rs");
const stateOwnership = JSON.parse(read("architecture/state-ownership.json"));
const pkg = JSON.parse(read("package.json"));

for (const [label, source] of [
  ["DatabaseService", service],
  ["database facade", facade],
  ["database bootstrap", bootstrap],
  ["connect use case", connect],
  ["migrate use case", migrate],
  ["health use case", health],
  ["statistics use case", statistics],
  ["reset use case", reset],
]) {
  for (const token of ["football_persistence_postgres", "sqlx::", "PostgresStore", "PgPool"]) {
    check(!source.includes(token), `${label} 泄漏具体 PostgreSQL 符号：${token}`);
  }
}

check(service.includes("pub(crate) struct DatabaseService"), "缺少 DatabaseService");
check(service.includes("pub(crate) session: RwLock<Option<DatabaseSession>>"), "活动数据库状态未归属 DatabaseService");
check(service.includes("prepare_connection"), "DatabaseService 缺少连接准备边界");
check(service.includes("preflight_reset"), "DatabaseService 缺少清空预检边界");
check(service.includes("reset_to_pristine"), "DatabaseService 缺少清空协调边界");
check(applicationService.includes("database: DatabaseService"), "ApplicationService 未聚合 DatabaseService");
check(!applicationService.includes("RwLock<Option<DatabaseSession>>"), "ApplicationService 仍直接持有活动数据库槽位");

check(connect.includes("super::migrate::execute(port).await?"), "connect use case 未委托 migrate use case");
check(connect.includes("recover_interrupted_work"), "connect use case 缺少中断任务恢复");
check(connect.includes("connection_preparation_migrates_before_recovery"), "connect use case 缺少 fake port 顺序测试");
check(migrate.includes("port.migrate().await"), "migrate use case 未通过 lifecycle port");
check(health.includes("port.health().await"), "health use case 未通过 observability port");
check(statistics.includes("port.statistics().await"), "statistics use case 未通过 observability port");
check(reset.includes("confirmation.trim() != health.database_name"), "reset use case 缺少数据库名称强确认");
check(reset.includes("保存的连接配置与当前数据库不一致，已拒绝清空"), "reset use case 缺少目标数据库一致性保护");
check(reset.includes("port.reset_to_pristine().await"), "reset use case 未通过 lifecycle port 执行清空");
check(reset.includes("reset_requires_the_same_database_before_destructive_work"), "reset use case 缺少 fake port 回归测试");

check(ports.includes("async fn reset_to_pristine(&self) -> PortResult<()>;"), "DatabaseLifecyclePort 缺少清空能力");
check(databaseAdapter.includes("impl DatabaseLifecyclePort for PersistenceStore"), "PostgreSQL adapter 未实现 DatabaseLifecyclePort");
check(databaseAdapter.includes("impl DatabaseObservabilityPort for PersistenceStore"), "PostgreSQL adapter 未实现 DatabaseObservabilityPort");
check(portRegistry.includes("PostgresStore as PersistenceStore"), "PostgreSQL 具体适配器入口缺失");
check(portRegistry.includes("pub(crate) type DatabaseSession = PersistenceStore;"), "Application DatabaseSession 边界别名缺失");
check(portRegistry.includes("use football_persistence_postgres::register_adapters;"), "PostgreSQL adapter 注册入口未收敛到 register_adapters");
check(!portRegistry.includes("impl DatabaseLifecyclePort") && !portRegistry.includes("impl DatabaseObservabilityPort"), "port_registry.rs 仍承载数据库 Port 实现");

check(!facade.includes("initialize_database_contents"), "Database facade 仍持有连接初始化业务编排");
check(lifecycleConnect.includes("initialize::execute(application, prepared.session()).await"), "连接成功前的内置内容初始化顺序未迁入 lifecycle use case");
check(
  lifecycleInitialize.includes(".rules") && lifecycleInitialize.includes("register_built_ins"),
  "内置规则包注册链未通过 Rules Service",
);
check(!facade.includes("register_built_in_rule_packages"), "Database facade 仍保留旧内置规则包注册实现");
check(
  lifecycleInitialize.includes(".research") &&
    lifecycleInitialize.includes(".register_persistence_artifacts(session)"),
  "P4 persistence artifact 注册链未通过 ResearchService",
);
check(
  researchService.includes("register_persistence_artifacts") &&
    researchService.includes("artifact_catalog::register_built_ins(port).await"),
  "ResearchService 未委托 Artifact Catalog 注册内置 P4 schema",
);
check(
  researchArtifactCatalog.includes("for draft in built_in_schema_versions()") &&
    researchArtifactCatalog.includes("port.register_schema(&draft).await?"),
  "内置 P4 schema 未通过 ResearchArtifactPort 注册",
);
check(lifecycleInitialize.includes("register_openai_research_artifacts"), "research artifact 注册链缺失");
check(applicationBootstrap.includes("list_recent_runs(&application.prediction, &active, 50)"), "bootstrap 最近运行读取语义发生变化");
check(bootstrap.includes("bootstrap::execute(self).await"), "Application bootstrap facade 未收敛为单一 use case 委托");
check(lifecycleReset.includes("connect::execute(application, options).await"), "reset 生命周期未复用 connect use case");

check(tauri.includes("preflight_database_reset"), "Tauri 清空命令未委托 Application 预检");
check(tauri.includes(".reset_database(options, confirmation)"), "Tauri 清空命令未委托 Application reset use case");
check(!tauri.includes("PostgresStore::connect"), "Tauri 清空命令仍直接连接 PostgreSQL");
check(!tauri.includes("reset_store.reset_to_pristine"), "Tauri 清空命令仍直接执行持久化清空");

const activeDatabaseState = stateOwnership.states?.find((state) => state.id === "application.active-database");
check(
  activeDatabaseState?.owner === "crates/application/src/services/database/service.rs::DatabaseService.session",
  "application.active-database 状态所有者未切换到 DatabaseService.session",
);
check(stateOwnership.last_updated_task === "R3-02", "状态所有权契约未登记 R3-02 更新");
check(pkg.scripts?.["verify:database-service"] === "node scripts/verify-database-service.mjs", "package.json 缺少 Database Service 验证入口");
check(pkg.scripts?.["verify:architecture"]?.includes("verify-database-service.mjs"), "verify:architecture 未接入 Database Service 门禁");

if (failures.length > 0) {
  console.error("Database Service 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}

console.log("Database Service 验证通过：连接、迁移、健康、统计、清空均已进入 Service/Use Case/Port 边界，活动 DatabaseSession 由 DatabaseService 单一持有，Lifecycle/Observability Port 实现已迁入具名 database adapter，Tauri 不直接执行 PostgreSQL 清空流程，内置 P4 artifact 初始化通过 ResearchService/ResearchArtifactPort 保持可验证链路。");
