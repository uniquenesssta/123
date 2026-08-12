import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const files = {
  library: "crates/persistence-postgres/src/lib.rs",
  store: "crates/persistence-postgres/src/store/postgres_store.rs",
  pool: "crates/persistence-postgres/src/pool/create_pool.rs",
  options: "crates/persistence-postgres/src/pool/database_options.rs",
  error: "crates/persistence-postgres/src/error/persistence_error.rs",
  migrations: "crates/persistence-postgres/src/migrations/run_migrations.rs",
  compatibility: "crates/persistence-postgres/src/migrations/reconcile_known_migrations.rs",
  reset: "crates/persistence-postgres/src/migrations/reset_to_pristine.rs",
  runtime: "crates/persistence-postgres/src/migrations/runtime_schema.rs",
  health: "crates/persistence-postgres/src/health/read_health.rs",
  healthDto: "crates/persistence-postgres/src/health/database_health.rs",
  statistics: "crates/persistence-postgres/src/statistics/read_statistics.rs",
  statisticsDto: "crates/persistence-postgres/src/statistics/database_stats.rs",
};
for (const [label, relative] of Object.entries(files)) check(exists(relative), `R4-01 缺少 ${label} owner：${relative}`);
check(!exists("crates/persistence-postgres/src/connection.rs"), "旧 connection.rs 仍存在");
check(!exists("crates/persistence-postgres/src/migration_compatibility.rs"), "旧 migration_compatibility.rs 仍存在");

const sources = Object.fromEntries(Object.entries(files).map(([key, relative]) => [key, read(relative)]));
check(sources.library.includes("mod store;") && sources.library.includes("mod pool;") && sources.library.includes("mod error;") && sources.library.includes("mod migrations;") && sources.library.includes("mod health;") && sources.library.includes("mod statistics;"), "lib.rs 未显式注册 R4-01 职责模块");
for (const token of ["pub use error::{PersistenceError, PersistenceResult};", "pub use health::DatabaseHealth;", "pub use pool::DatabaseOptions;", "pub use statistics::DatabaseStats;", "pub use store::PostgresStore;"]) check(sources.library.includes(token), `公共兼容出口缺失：${token}`);

const allRust = fs.readdirSync(path.join(root, "crates/persistence-postgres/src"), { recursive: true, withFileTypes: true })
  .filter((entry) => entry.isFile() && entry.name.endsWith(".rs"))
  .map((entry) => fs.readFileSync(path.join(entry.parentPath ?? entry.path, entry.name), "utf8"));
check(allRust.filter((source) => /pub struct PostgresStore\s*\{/.test(source)).length === 1, "PostgresStore 必须只有一个定义 owner");
check(allRust.filter((source) => /pub enum PersistenceError\s*\{/.test(source)).length === 1, "PersistenceError 必须只有一个定义 owner");
check(allRust.filter((source) => source.includes("PgPoolOptions::new()")).length === 1, "PgPoolOptions 创建逻辑必须只有一个 owner");

check(sources.store.includes("pub(crate) pool: PgPool"), "PostgresStore.pool 未保持唯一 crate 内 pool owner");
check(sources.store.includes("create_pool(options).await?"), "PostgresStore::connect 未委托 pool 创建模块");
check(sources.pool.includes("PgPoolOptions::new()") && sources.pool.includes("max_connections(options.max_connections.max(1))") && sources.pool.includes("acquire_timeout"), "pool 创建契约发生变化");
check(sources.options.includes("#[serde(default = \"default_max_connections\")]") && sources.options.includes("#[serde(default = \"default_connect_timeout_seconds\")]") && sources.options.includes("pub fn redacted_url"), "DatabaseOptions 默认值或脱敏接口发生变化");
for (const variant of ["Sqlx(#[from] sqlx::Error)", "Migration(#[from] sqlx::migrate::MigrateError)", "Serialization(#[from] serde_json::Error)", "InvalidState(String)", "RouteNotFound"]) check(sources.error.includes(variant), `PersistenceError 兼容变体缺失：${variant}`);

const reconcile = sources.migrations.indexOf("reconcile_known_legacy_migrations(&self.pool).await?;");
const migrate = sources.migrations.indexOf("MIGRATOR.run(&self.pool).await?;");
check(reconcile >= 0 && migrate > reconcile, "历史迁移兼容必须先于 SQLx Migrator");
check(sources.migrations.includes("ensure_runtime_schema_compatibility(&self.pool).await?;") && sources.migrations.includes("SELECT feature.refresh_player_ability_projections()"), "migration runner 缺少 runtime compatibility 或能力投影刷新");
for (const schema of ["ai_workspace", "analytics", "audit", "catalog", "feature", "football", "model", "platform", "research", "review"]) check(sources.reset.includes(`\"${schema}\"`), `reset 缺少 schema：${schema}`);
check(sources.reset.includes("football-platform-destructive-reset") && sources.reset.includes("DROP TABLE IF EXISTS public._sqlx_migrations") && sources.reset.includes("self.migrate().await"), "destructive reset 契约发生变化");
check(sources.runtime.includes("football-platform-schema-compatibility") && sources.runtime.includes("ADD COLUMN IF NOT EXISTS created_at timestamptz") && sources.runtime.includes("ability_observations_player_cutoff_idx"), "runtime schema compatibility 契约发生变化");
check(sources.health.includes("pub async fn health") && sources.health.includes("current_database()") && sources.health.includes("数据库缺少当前客户端所需的能力观察写入时点字段"), "health 行为契约发生变化");
check(sources.statistics.includes("pub async fn stats") && sources.statistics.includes("pg_stat_user_tables") && sources.statistics.includes("large_counts_are_estimates: true"), "statistics 估算/精确计数策略发生变化");
check(sources.compatibility.includes("COMPATIBLE_MIGRATION_VERSIONS: [i64; 11] = [12, 13, 14, 15, 16, 17, 18, 25, 26, 27, 31]"), "历史迁移兼容白名单发生变化");

if (failures.length) {
  console.error("R4-01 Persistence Foundation 验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R4-01 Persistence Foundation 验证通过：Store/Error/Pool/Migrations/Health/Statistics 已形成唯一职责 owner，公共兼容出口与数据库生命周期语义保持。");
