from __future__ import annotations

import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE = "b508d1ff7808b8735694cf1e57a4d603f2e973e3"


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8", newline="\n")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected 1 exact match, found {count}")
    return text.replace(old, new, 1)


def apply_persistence_root() -> None:
    write(
        "crates/persistence-postgres/src/adapters/mod.rs",
        """mod register_adapters;\n\npub use register_adapters::register_adapters;\n""",
    )
    write(
        "crates/persistence-postgres/src/adapters/register_adapters.rs",
        """use crate::{DatabaseOptions, PersistenceResult, PostgresStore};\n\npub async fn register_adapters(options: &DatabaseOptions) -> PersistenceResult<PostgresStore> {\n    PostgresStore::connect(options).await\n}\n""",
    )

    lib_path = "crates/persistence-postgres/src/lib.rs"
    lib = read(lib_path)
    lib = replace_once(lib, "mod analytics;\n", "mod adapters;\nmod analytics;\n", "persistence adapters module")
    lib = replace_once(
        lib,
        "pub use error::{PersistenceError, PersistenceResult};\n",
        "pub use adapters::register_adapters;\npub use error::{PersistenceError, PersistenceResult};\n",
        "persistence adapters export",
    )
    write(lib_path, lib)

    store_path = "crates/persistence-postgres/src/store/postgres_store.rs"
    store = read(store_path)
    store = replace_once(
        store,
        "pub struct PostgresStore {\n    pub(crate) pool: PgPool,\n}",
        "pub struct PostgresStore {\n    pub(crate) pool: PgPool,\n    redacted_url: String,\n}",
        "PostgresStore connection metadata field",
    )
    store = replace_once(
        store,
        "    pub async fn connect(options: &DatabaseOptions) -> PersistenceResult<Self> {\n        Ok(Self {\n            pool: create_pool(options).await?,\n        })\n    }\n\n    pub async fn close(&self) {",
        "    pub async fn connect(options: &DatabaseOptions) -> PersistenceResult<Self> {\n        let redacted_url = options.redacted_url();\n        Ok(Self {\n            pool: create_pool(options).await?,\n            redacted_url,\n        })\n    }\n\n    pub fn redacted_url(&self) -> &str {\n        &self.redacted_url\n    }\n\n    pub async fn close(&self) {",
        "PostgresStore redacted URL accessor",
    )
    write(store_path, store)


def write_application_adapter_owners() -> None:
    write(
        "crates/application/src/composition/adapters/persistence_error.rs",
        """use super::super::port_registry::PersistenceError;\nuse crate::ports::{PortError, PortErrorKind};\n\npub(super) fn map_persistence_error(error: PersistenceError) -> PortError {\n    let kind = match &error {\n        PersistenceError::Serialization(_) => PortErrorKind::Serialization,\n        PersistenceError::InvalidState(_) => PortErrorKind::InvalidState,\n        PersistenceError::RouteNotFound => PortErrorKind::NotFound,\n        PersistenceError::Sqlx(_) | PersistenceError::Migration(_) => PortErrorKind::Infrastructure,\n    };\n    PortError::new(kind, error.to_string())\n}\n""",
    )
    write(
        "crates/application/src/composition/adapters/database.rs",
        """use super::map_persistence_error;\nuse super::super::port_registry::{DatabaseHealth, DatabaseStats, PersistenceStore};\nuse crate::ports::{\n    database::{\n        DatabaseHealthSnapshot, DatabaseLifecyclePort, DatabaseObservabilityPort,\n        DatabaseStatistics,\n    },\n    PortResult,\n};\nuse async_trait::async_trait;\n\npub(crate) fn database_health_from_snapshot(snapshot: DatabaseHealthSnapshot) -> DatabaseHealth {\n    DatabaseHealth {\n        connected: snapshot.connected,\n        database_name: snapshot.database_name,\n        server_version: snapshot.server_version,\n        migration_count: snapshot.migration_count,\n        database_size_bytes: snapshot.database_size_bytes,\n        checked_at: snapshot.checked_at,\n        latency_ms: snapshot.latency_ms,\n    }\n}\n\npub(crate) fn database_stats_from_statistics(statistics: DatabaseStatistics) -> DatabaseStats {\n    DatabaseStats {\n        competitions: statistics.competitions,\n        teams: statistics.teams,\n        players: statistics.players,\n        matches: statistics.matches,\n        model_runs: statistics.model_runs,\n        rule_packages: statistics.rule_packages,\n        route_bindings: statistics.route_bindings,\n        ability_observations: statistics.ability_observations,\n        pending_ability_updates: statistics.pending_ability_updates,\n        data_providers: statistics.data_providers,\n        availability_records: statistics.availability_records,\n        active_lineups: statistics.active_lineups,\n        large_counts_are_estimates: statistics.large_counts_are_estimates,\n    }\n}\n\n#[async_trait]\nimpl DatabaseLifecyclePort for PersistenceStore {\n    async fn migrate(&self) -> PortResult<()> {\n        PersistenceStore::migrate(self)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn recover_interrupted_work(&self) -> PortResult<()> {\n        PersistenceStore::recover_interrupted_jobs(self)\n            .await\n            .map_err(map_persistence_error)?;\n        PersistenceStore::recover_interrupted_api_workspace_operations(self)\n            .await\n            .map_err(map_persistence_error)?;\n        Ok(())\n    }\n\n    async fn reset_to_pristine(&self) -> PortResult<()> {\n        PersistenceStore::reset_to_pristine(self)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn close(&self) -> PortResult<()> {\n        PersistenceStore::close(self).await;\n        Ok(())\n    }\n}\n\n#[async_trait]\nimpl DatabaseObservabilityPort for PersistenceStore {\n    async fn health(&self) -> PortResult<DatabaseHealthSnapshot> {\n        let health = PersistenceStore::health(self)\n            .await\n            .map_err(map_persistence_error)?;\n        Ok(DatabaseHealthSnapshot {\n            connected: health.connected,\n            database_name: health.database_name,\n            server_version: health.server_version,\n            migration_count: health.migration_count,\n            database_size_bytes: health.database_size_bytes,\n            checked_at: health.checked_at,\n            latency_ms: health.latency_ms,\n        })\n    }\n\n    async fn statistics(&self) -> PortResult<DatabaseStatistics> {\n        let statistics = PersistenceStore::stats(self)\n            .await\n            .map_err(map_persistence_error)?;\n        Ok(DatabaseStatistics {\n            competitions: statistics.competitions,\n            teams: statistics.teams,\n            players: statistics.players,\n            matches: statistics.matches,\n            model_runs: statistics.model_runs,\n            rule_packages: statistics.rule_packages,\n            route_bindings: statistics.route_bindings,\n            ability_observations: statistics.ability_observations,\n            pending_ability_updates: statistics.pending_ability_updates,\n            data_providers: statistics.data_providers,\n            availability_records: statistics.availability_records,\n            active_lineups: statistics.active_lineups,\n            large_counts_are_estimates: statistics.large_counts_are_estimates,\n        })\n    }\n}\n""",
    )
    write(
        "crates/application/src/composition/adapters/competition.rs",
        """use super::map_persistence_error;\nuse super::super::port_registry::PersistenceStore;\nuse crate::ports::{competition::CompetitionHierarchyPort, PortResult};\nuse async_trait::async_trait;\nuse football_domain::{\n    CompetitionDraft, CompetitionRecord, RoundDraft, RoundRecord, SeasonDraft, SeasonRecord,\n    StageDraft, StageRecord,\n};\nuse uuid::Uuid;\n\n#[async_trait]\nimpl CompetitionHierarchyPort for PersistenceStore {\n    async fn create_competition(&self, draft: &CompetitionDraft) -> PortResult<CompetitionRecord> {\n        PersistenceStore::create_competition(self, draft)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn delete_competition(&self, competition_id: Uuid) -> PortResult<()> {\n        PersistenceStore::delete_competition(self, competition_id)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn list_competitions(&self) -> PortResult<Vec<CompetitionRecord>> {\n        PersistenceStore::list_competitions(self)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn create_season(&self, draft: &SeasonDraft) -> PortResult<SeasonRecord> {\n        PersistenceStore::create_season(self, draft)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn list_seasons(&self) -> PortResult<Vec<SeasonRecord>> {\n        PersistenceStore::list_seasons(self)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn create_stage(&self, draft: &StageDraft) -> PortResult<StageRecord> {\n        PersistenceStore::create_stage(self, draft)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn list_stages(&self) -> PortResult<Vec<StageRecord>> {\n        PersistenceStore::list_stages(self)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn create_round(&self, draft: &RoundDraft) -> PortResult<RoundRecord> {\n        PersistenceStore::create_round(self, draft)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn list_rounds(&self) -> PortResult<Vec<RoundRecord>> {\n        PersistenceStore::list_rounds(self)\n            .await\n            .map_err(map_persistence_error)\n    }\n}\n""",
    )
    write(
        "crates/application/src/composition/adapters/rules.rs",
        """use super::map_persistence_error;\nuse super::super::port_registry::PersistenceStore;\nuse crate::ports::{rules::{RulePackagePort, RuleRoutingPort}, PortResult};\nuse async_trait::async_trait;\nuse football_domain::{\n    CompetitionBindingDraft, CompetitionBindingSummary, CompetitionKind,\n    ResolvedCompetitionContext, RouteDecision, RouteRequest, RulePackageDraft, RulePackageSummary,\n};\nuse football_model_api::ModelDescriptor;\nuse uuid::Uuid;\n\n#[async_trait]\nimpl RulePackagePort for PersistenceStore {\n    async fn register_rule_package(\n        &self,\n        descriptor: &ModelDescriptor,\n        draft: &RulePackageDraft,\n    ) -> PortResult<RulePackageSummary> {\n        PersistenceStore::register_rule_package(self, descriptor, draft)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn list_rule_packages(&self) -> PortResult<Vec<RulePackageSummary>> {\n        PersistenceStore::list_rule_packages(self)\n            .await\n            .map_err(map_persistence_error)\n    }\n}\n\n#[async_trait]\nimpl RuleRoutingPort for PersistenceStore {\n    async fn create_competition_binding(\n        &self,\n        draft: &CompetitionBindingDraft,\n    ) -> PortResult<CompetitionBindingSummary> {\n        PersistenceStore::create_competition_binding(self, draft)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn list_competition_bindings(&self) -> PortResult<Vec<CompetitionBindingSummary>> {\n        PersistenceStore::list_competition_bindings(self)\n            .await\n            .map_err(map_persistence_error)\n    }\n\n    async fn ensure_type_default_binding(\n        &self,\n        rule_package_id: Uuid,\n        competition_kind: CompetitionKind,\n        priority: i32,\n        label: &str,\n    ) -> PortResult<()> {\n        PersistenceStore::ensure_type_default_binding(\n            self,\n            rule_package_id,\n            competition_kind,\n            priority,\n            label,\n        )\n        .await\n        .map(|_| ())\n        .map_err(map_persistence_error)\n    }\n\n    async fn resolve_competition_context(\n        &self,\n        competition_id: Option<Uuid>,\n        season_id: Option<Uuid>,\n        stage_id: Option<Uuid>,\n        competition_kind: CompetitionKind,\n    ) -> PortResult<ResolvedCompetitionContext> {\n        PersistenceStore::resolve_competition_context(\n            self,\n            competition_id,\n            season_id,\n            stage_id,\n            competition_kind,\n        )\n        .await\n        .map_err(map_persistence_error)\n    }\n\n    async fn resolve_route(&self, request: &RouteRequest) -> PortResult<RouteDecision> {\n        PersistenceStore::resolve_route(self, request)\n            .await\n            .map_err(map_persistence_error)\n    }\n}\n""",
    )


def switch_application_adapters() -> None:
    adapter_root = ROOT / "crates/application/src/composition/adapters"
    existing = [p for p in adapter_root.rglob("*.rs") if p.name not in {"database.rs", "competition.rs", "rules.rs", "persistence_error.rs"}]
    for path in existing:
        text = path.read_text(encoding="utf-8")
        if "ActiveDatabase" not in text and "transition_store" not in text:
            continue
        relative = path.relative_to(adapter_root)
        if len(relative.parts) == 1:
            text = text.replace(
                "use super::super::port_registry::{map_persistence_error, ",
                "use super::map_persistence_error;\nuse super::super::port_registry::{",
            )
        else:
            text = text.replace(
                "use super::super::super::port_registry::{map_persistence_error, ",
                "use super::super::map_persistence_error;\nuse super::super::super::port_registry::{",
            )
        text = text.replace("ActiveDatabase", "PersistenceStore")
        text = text.replace(
            "let store = self.transition_store();",
            "let store = PersistenceStore::clone(self);",
        )
        text = text.replace("self.transition_store()", "self")
        if "transition_store" in text or "ActiveDatabase" in text:
            raise RuntimeError(f"adapter transition owner remains in {relative}")
        path.write_text(text, encoding="utf-8", newline="\n")

    mod_path = "crates/application/src/composition/adapters/mod.rs"
    mod = read(mod_path)
    mod = replace_once(mod, "mod ai_workspace;\n", "mod ai_workspace;\nmod competition;\nmod database;\n", "application adapter domain modules")
    mod = replace_once(mod, "mod players;\n", "mod persistence_error;\nmod players;\n", "application adapter error module")
    mod = replace_once(mod, "mod research;\n", "mod research;\nmod rules;\n", "application rules adapter module")
    mod = replace_once(
        mod,
        "pub(crate) use prediction::model_run_list_item_from_port;\n",
        "pub(crate) use database::{database_health_from_snapshot, database_stats_from_statistics};\npub(super) use persistence_error::map_persistence_error;\npub(crate) use prediction::model_run_list_item_from_port;\n",
        "application adapter exports",
    )
    write(mod_path, mod)


def rewrite_port_registry() -> None:
    write(
        "crates/application/src/composition/port_registry.rs",
        """pub(crate) use football_persistence_postgres::{\n    DatabaseHealth, DatabaseOptions, DatabaseStats, ModelRunListItem, PersistenceError,\n    PostgresStore as PersistenceStore,\n};\nuse football_persistence_postgres::register_adapters;\n\nuse super::adapters::map_persistence_error;\nuse crate::ports::PortResult;\n\npub(crate) struct PortRegistry;\n\nimpl PortRegistry {\n    pub(crate) fn new() -> Self {\n        Self\n    }\n\n    pub(crate) async fn connect_database(\n        &self,\n        options: &DatabaseOptions,\n    ) -> PortResult<PersistenceStore> {\n        register_adapters(options).await.map_err(map_persistence_error)\n    }\n}\n""",
    )

    mod_path = "crates/application/src/composition/mod.rs"
    mod = read(mod_path)
    mod = replace_once(
        mod,
        "pub(crate) use port_registry::{\n    database_health_from_snapshot, database_stats_from_statistics, ActiveDatabase, DatabaseHealth,\n    DatabaseOptions, DatabaseStats, ModelRunListItem, PersistenceError, PortRegistry,\n};\n\npub(crate) use adapters::model_run_list_item_from_port;\n",
        "pub(crate) use adapters::{\n    database_health_from_snapshot, database_stats_from_statistics, model_run_list_item_from_port,\n};\npub(crate) use port_registry::{\n    DatabaseHealth, DatabaseOptions, DatabaseStats, ModelRunListItem, PersistenceError,\n    PersistenceStore, PortRegistry,\n};\n",
        "application composition exports",
    )
    write(mod_path, mod)


def switch_application_sessions() -> None:
    root = ROOT / "crates/application/src"
    for path in root.rglob("*.rs"):
        if "composition/adapters" in path.as_posix() or path.name == "port_registry.rs":
            continue
        text = path.read_text(encoding="utf-8")
        if "ActiveDatabase" not in text:
            continue
        text = text.replace("ActiveDatabase", "PersistenceStore")
        path.write_text(text, encoding="utf-8", newline="\n")


def write_verifier_and_package() -> None:
    verifier = r'''import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const walk = (dir) => fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
  const full = path.join(dir, entry.name);
  return entry.isDirectory() ? walk(full) : [full];
});
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const persistenceAdapters = "crates/persistence-postgres/src/adapters";
const adapterFiles = fs.readdirSync(path.join(root, persistenceAdapters)).filter((name) => name.endsWith(".rs")).sort();
check(JSON.stringify(adapterFiles) === JSON.stringify(["mod.rs", "register_adapters.rs"]), `Persistence adapters root must contain only mod.rs/register_adapters.rs, got ${adapterFiles.join(", ")}`);
const pmod = read(`${persistenceAdapters}/mod.rs`);
const registration = read(`${persistenceAdapters}/register_adapters.rs`);
const plib = read("crates/persistence-postgres/src/lib.rs");
const store = read("crates/persistence-postgres/src/store/postgres_store.rs");
check(pmod.includes("mod register_adapters;") && pmod.includes("pub use register_adapters::register_adapters;"), "adapters/mod.rs must only expose register_adapters");
check(registration.includes("PostgresStore::connect(options).await"), "register_adapters must delegate concrete connection creation to PostgresStore");
check(!registration.includes("sqlx::") && !registration.includes("football_domain"), "register_adapters must not own SQL or domain behavior");
check(plib.includes("mod adapters;") && plib.includes("pub use adapters::register_adapters;"), "persistence lib must expose the adapter registration entry");
check(store.includes("redacted_url: String") && store.includes("pub fn redacted_url(&self) -> &str") && store.includes("let redacted_url = options.redacted_url();"), "PostgresStore must own active connection redaction metadata after wrapper removal");

const appRoot = path.join(root, "crates/application/src");
const appFiles = walk(appRoot).filter((file) => file.endsWith(".rs"));
const concreteImports = appFiles.filter((file) => fs.readFileSync(file, "utf8").includes("football_persistence_postgres"));
check(concreteImports.length === 1 && concreteImports[0].replaceAll("\\", "/").endsWith("crates/application/src/composition/port_registry.rs"), `Concrete persistence import must remain composition-only; got ${concreteImports.map((f) => path.relative(root, f)).join(", ")}`);
const combinedApplication = appFiles.map((file) => fs.readFileSync(file, "utf8")).join("\n");
check(!combinedApplication.includes("ActiveDatabase"), "ActiveDatabase wrapper must be fully removed");
check(!combinedApplication.includes("transition_store"), "transition_store forwarding path must be fully removed");

const portRegistry = read("crates/application/src/composition/port_registry.rs");
check(portRegistry.includes("use football_persistence_postgres::register_adapters;"), "PortRegistry must consume persistence register_adapters entry");
check(portRegistry.includes("-> PortResult<PersistenceStore>"), "PortRegistry must return the concrete PostgresStore adapter session");
check(portRegistry.includes("register_adapters(options).await.map_err(map_persistence_error)"), "PortRegistry must use the unique adapter registration entry");
check(!portRegistry.includes("#[async_trait]") && !/impl\s+\w+Port\s+for/.test(portRegistry), "port_registry.rs must not own Port implementations");
check(!portRegistry.includes("PersistenceStore::connect"), "Application composition must not bypass register_adapters");

const adapterRoot = path.join(root, "crates/application/src/composition/adapters");
const applicationAdapterFiles = walk(adapterRoot).filter((file) => file.endsWith(".rs"));
const adapterCombined = applicationAdapterFiles.map((file) => fs.readFileSync(file, "utf8")).join("\n");
const implMatches = adapterCombined.match(/impl\s+(?:(?:crate::ports::prediction::)?[A-Za-z0-9_]+Port)\s+for\s+PersistenceStore/g) ?? [];
check(implMatches.length === 40, `Expected 40 PostgreSQL-backed Application Port impls targeting PersistenceStore, got ${implMatches.length}`);
check(!adapterCombined.includes("for ActiveDatabase"), "No adapter Port impl may target the removed ActiveDatabase wrapper");
check(!adapterCombined.includes("transition_store"), "No adapter may retain transition_store forwarding");
for (const required of ["database.rs", "competition.rs", "rules.rs", "persistence_error.rs"]) {
  check(fs.existsSync(path.join(adapterRoot, required)), `Missing extracted application adapter owner: ${required}`);
}

const packageJson = JSON.parse(read("package.json"));
check(typeof packageJson.scripts["verify:persistence-adapters"] === "string", "package.json must expose verify:persistence-adapters");
check(packageJson.scripts["verify:architecture"].includes("verify-persistence-adapters.mjs"), "verify:architecture must include R4-04 adapter gate");

if (failures.length) {
  console.error("R4-04 Port Adapter registration verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R4-04 Port Adapter registration verified: PostgresStore is the 40-Port concrete target, ActiveDatabase forwarding is removed, and register_adapters is the composition registration entry.");
'''
    write("scripts/verify-persistence-adapters.mjs", verifier)

    package_path = "package.json"
    package = read(package_path)
    package = replace_once(
        package,
        "&& node scripts/verify-persistence-mapping.mjs\",\n    \"verify:persistence-foundation\"",
        "&& node scripts/verify-persistence-mapping.mjs && node scripts/verify-persistence-adapters.mjs\",\n    \"verify:persistence-foundation\"",
        "architecture adapter gate",
    )
    package = replace_once(
        package,
        '    "verify:persistence-mapping": "node scripts/verify-persistence-mapping.mjs",\n',
        '    "verify:persistence-mapping": "node scripts/verify-persistence-mapping.mjs",\n    "verify:persistence-adapters": "node scripts/verify-persistence-adapters.mjs",\n',
        "adapter npm script",
    )
    write(package_path, package)


def apply() -> None:
    apply_persistence_root()
    write_application_adapter_owners()
    switch_application_adapters()
    rewrite_port_registry()
    switch_application_sessions()
    write_verifier_and_package()


def record() -> None:
    run_id = os.environ["GITHUB_RUN_ID"]
    audit_run = os.environ.get("R4_04_AUDIT_RUN", "31686092182")
    task = f'''# R4-04 Port Adapter 注册\n\n## 状态\n\n`VERIFYING`\n\n## 实施结果\n\n- 在 `crates/persistence-postgres/src/adapters/` 建立 `mod.rs` 与 `register_adapters.rs`，由 Persistence 暴露唯一 Application composition 注册入口；`PostgresStore::connect` 继续保留给现有低层兼容调用和 PostgreSQL integration tests。\n- `PostgresStore` 接管活动连接的 redacted URL 元数据，删除 Application `ActiveDatabase` wrapper 与 `transition_store()` 转发链。\n- 40 个 PostgreSQL-backed Application Port 实现目标全部从 `ActiveDatabase` 切换为 `PostgresStore`（Application crate 持有 trait，因此 impl 保持在 Application composition adapters，避免反向依赖和 crate cycle）。\n- `port_registry.rs` 收敛为 concrete persistence import + `register_adapters` 连接注册；原 Database / Competition / Rules Port 实现拆到 `composition/adapters/database.rs`、`competition.rs`、`rules.rs`，PersistenceError -> PortError 映射拆到 `persistence_error.rs`。\n- 新增 `verify:persistence-adapters` 并接入 `verify:architecture`，锁定唯一 concrete import owner、40-Port target、零 `ActiveDatabase`/`transition_store`、注册入口与模块边界。\n- 未修改 Cargo dependency graph、公共 Application Ports、Tauri command/DTO、SQL、0001–0046 migrations、配置格式、错误类别语义、模型保护资产或 R5 业务 Repository。\n\n## 审计依据\n\n- scope audit run `{audit_run}`：确认 Application 只有 `composition/port_registry.rs` 直接导入 `football_persistence_postgres`；40 个 DB-backed Port impl 全部以 `ActiveDatabase` 为旧目标；`PostgresStore::connect` 另有现有 integration test 调用；Persistence crate 不依赖 Application，直接反转依赖会形成任务书禁止的 cycle。\n\n## 验证\n\nWindows 2025 / Rust 1.88.0 / Node 22 hard gate run `{run_id}` 实际执行并通过：\n\n- `npm run verify:persistence-adapters`。\n- `npm run verify:persistence-foundation`。\n- `npm run verify:persistence-audit`。\n- `npm run verify:persistence-mapping`。\n- `node scripts/verify_database_baseline.mjs`。\n- `node scripts/verify_protected_assets.mjs`。\n- `cargo fmt --all -- --check`。\n- `cargo check --locked -p football-persistence-postgres -p football-application`。\n- `cargo test --locked -p football-persistence-postgres`。\n- `cargo test --locked -p football-application`。\n- `npm run setup`。\n- `npm run verify:architecture`。\n- `npm run verify:frontend`。\n- `cargo clippy --locked --workspace --all-targets -- -D warnings`。\n- `cargo test --locked --workspace`。\n- Cargo manifests / `Cargo.lock`、历史 migrations 对 R4-04 基线 `{BASE}` 零 diff。\n\n## 未执行与剩余风险\n\n- 18 个要求专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL ignored integration tests 未执行；未执行 destructive database reset。\n- 当前节点保持 `VERIFYING`，等待 clean Public Platform CI、PR 合并、合并后 stage CI 与 R4 阶段出口记录；不得提前进入 R5-01。\n\n## 回退点\n\n- R4-04 基线：`{BASE}`。\n- 实施分支：`agent/r4-04-port-adapter-registration`。\n'''
    write("docs/modular-rewrite/R04-persistence-foundation/R04-04-port-adapter-注册.md", task)

    stage_path = "docs/modular-rewrite/R04-persistence-foundation/README.md"
    stage = read(stage_path)
    old = "| R4-04 | Application Port Adapter 注册 | READY | — |"
    new = "| R4-04 | Application Port Adapter 注册 | VERIFYING | [`R04-04-port-adapter-注册.md`](./R04-04-port-adapter-注册.md) |"
    stage = replace_once(stage, old, new, "R4-04 stage row")
    if "## R4-04 实施中" not in stage:
        stage = stage.rstrip() + f'''\n\n## R4-04 实施中\n\n- 从 R4-03 最终 canonical HEAD `{BASE}` 独立建立 `agent/r4-04-port-adapter-registration`。\n- scope audit run `{audit_run}` 确认旧入口为 `ActiveDatabase` + `transition_store`，40 个 DB-backed Application Ports 由该 wrapper 转发到 PostgresStore；Persistence -> Application 反向依赖会造成 crate cycle，因此实现采用 Application-owned trait / PostgresStore concrete target 的 Rust 合法边界。\n- hard gate run `{run_id}` 已通过 R4-04 专项、R4-01/R4-02/R4-03 回归、数据库冻结、architecture/frontend 与 workspace Rust 回归。\n- R4-04 当前 `VERIFYING`；clean CI、合并及 R4 stage completion 完成前 R5-01 继续 `BLOCKED`。\n'''
    write(stage_path, stage)

    readme_path = "README.md"
    root = read(readme_path)
    if "- R4-04 Port Adapter 注册" not in root:
        lines = root.splitlines()
        anchor = next((i for i, line in enumerate(lines) if line.startswith("- R4-03 Row 映射基础规范已建立 ")), None)
        if anchor is None:
            raise RuntimeError("root README R4-03 anchor missing")
        lines.insert(
            anchor + 1,
            f"- R4-04 Port Adapter 注册已新增 Persistence `adapters/register_adapters.rs` 唯一 composition 注册入口，并将 40 个 PostgreSQL-backed Application Port 的 concrete target 从 `ActiveDatabase` 切换为 `PostgresStore`；`ActiveDatabase`/`transition_store` 转发层已删除，Database/Competition/Rules Port 与 persistence error mapping 已从 `port_registry.rs` 拆回具名 adapter owner。hard gate run `{run_id}` 已通过专项、数据库冻结、architecture/frontend、Application/Persistence 与 workspace Rust 回归；18 个专用 PostgreSQL integration tests 与 destructive reset 未执行。当前 `VERIFYING`，等待 clean CI/PR 合并及 R4 阶段出口收口。",
        )
        root = "\n".join(lines) + "\n"
    write(readme_path, root)


def main() -> None:
    if len(sys.argv) != 2 or sys.argv[1] not in {"apply", "record"}:
        raise SystemExit("usage: r4-04-apply.py apply|record")
    if sys.argv[1] == "apply":
        apply()
    else:
        record()


if __name__ == "__main__":
    main()
