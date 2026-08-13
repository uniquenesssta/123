import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relativePath) =>
  fs.readFileSync(path.join(root, relativePath), "utf8").replace(/^\uFEFF/, "").replaceAll("\r\n", "\n");
const failures = [];
const check = (condition, message) => {
  if (!condition) failures.push(message);
};

const serviceRoot = "crates/application/src/services/release";
const useCaseRoot = "crates/application/src/use_cases/release";
const adapter = "crates/application/src/composition/adapters/release.rs";
const legacyOwner = ["crates/application/src", "release_acceptance.rs"].join("/");
const useCases = ["run_acceptance", "list_runs", "read_run"];
const publicMethods = [
  "run_release_acceptance",
  "list_release_acceptance_runs",
  "read_release_acceptance_run",
];

check(!fs.existsSync(path.join(root, legacyOwner)), `旧 Release owner 仍存在：${legacyOwner}`);
for (const file of [
  `${serviceRoot}/mod.rs`,
  `${serviceRoot}/service.rs`,
  `${serviceRoot}/facade.rs`,
  `${useCaseRoot}/mod.rs`,
  `${useCaseRoot}/run_acceptance/mod.rs`,
  `${useCaseRoot}/run_acceptance/validation.rs`,
  `${useCaseRoot}/run_acceptance/summary.rs`,
  `${useCaseRoot}/run_acceptance/report.rs`,
  `${useCaseRoot}/run_acceptance/checks/mod.rs`,
  `${useCaseRoot}/run_acceptance/checks/chain.rs`,
  `${useCaseRoot}/run_acceptance/checks/performance.rs`,
  `${useCaseRoot}/run_acceptance/checks/security.rs`,
  `${useCaseRoot}/run_acceptance/checks/cost.rs`,
  `${useCaseRoot}/run_acceptance/checks/release.rs`,
  `${useCaseRoot}/list_runs/mod.rs`,
  `${useCaseRoot}/read_run/mod.rs`,
  adapter,
]) {
  check(fs.existsSync(path.join(root, file)), `缺少 Release 模块文件：${file}`);
}
for (const useCase of useCases) {
  check(fs.existsSync(path.join(root, useCaseRoot, useCase, "mod.rs")), `Release Use Case 未独立成目录：${useCase}`);
}

const service = read(`${serviceRoot}/service.rs`);
const facade = read(`${serviceRoot}/facade.rs`);
const library = read("crates/application/src/lib.rs");
const composition = read("crates/application/src/composition/application_composition.rs");
const applicationService = read("crates/application/src/service/application_service.rs");
const ports = read("crates/application/src/ports/release/mod.rs");
const adapterText = read(adapter);
const commands = read("src-tauri/src/commands/release_acceptance.rs");

check(!library.includes("mod release_acceptance;"), "lib.rs 仍登记旧 Release owner");
check(composition.includes("release: ReleaseService"), "ApplicationComposition 未聚合 ReleaseService");
check(composition.includes("release: ReleaseService::new()"), "组合根未构造唯一 ReleaseService");
check(applicationService.includes("pub(crate) release: ReleaseService"), "ApplicationService 未持有 ReleaseService");
check(applicationService.includes("release: parts.release"), "ApplicationService 未从组合根接收 ReleaseService");
for (const method of useCases) check(service.includes(`fn ${method}`), `ReleaseService 缺少方法：${method}`);
for (const method of publicMethods) check(facade.includes(`pub async fn ${method}`), `ApplicationService 兼容入口缺少：${method}`);
check((facade.match(/self\.release/g) ?? []).length >= publicMethods.length, "Release facade 未统一委托 ReleaseService");

for (const token of [
  "pub trait ReleaseAcceptancePort",
  "async fn runtime_facts(",
  "performance_window_days: u32",
  "cost_window_days: u32",
  "async fn persist_run(&self, run: &ReleaseAcceptanceRun)",
  "async fn list_runs(&self, limit: u32)",
  "async fn read_run(&self, run_id: Uuid)",
]) check(ports.includes(token), `ReleaseAcceptancePort 缺少真实能力：${token}`);
check(adapterText.includes("impl ReleaseAcceptancePort for PersistenceStore"), "ReleaseAcceptancePort 未由 PersistenceStore 适配");
for (const call of [
  "release_acceptance_runtime_facts",
  "persist_release_acceptance_run",
  "list_release_acceptance_runs",
  "read_release_acceptance_run",
]) check(adapterText.includes(call), `Release adapter 缺少持久化委托：${call}`);

const banned = ["football_persistence_postgres", "sqlx::", "PostgresStore", "PersistenceStore"];
for (const relativePath of [
  `${serviceRoot}/service.rs`,
  `${serviceRoot}/facade.rs`,
  `${useCaseRoot}/run_acceptance/mod.rs`,
  `${useCaseRoot}/run_acceptance/validation.rs`,
  `${useCaseRoot}/run_acceptance/summary.rs`,
  `${useCaseRoot}/run_acceptance/report.rs`,
  `${useCaseRoot}/run_acceptance/checks/mod.rs`,
  `${useCaseRoot}/run_acceptance/checks/chain.rs`,
  `${useCaseRoot}/run_acceptance/checks/performance.rs`,
  `${useCaseRoot}/run_acceptance/checks/security.rs`,
  `${useCaseRoot}/run_acceptance/checks/cost.rs`,
  `${useCaseRoot}/run_acceptance/checks/release.rs`,
  `${useCaseRoot}/list_runs/mod.rs`,
  `${useCaseRoot}/read_run/mod.rs`,
]) {
  const text = read(relativePath);
  for (const token of banned) check(!text.includes(token), `${relativePath} 泄漏基础设施符号：${token}`);
}

const validation = read(`${useCaseRoot}/run_acceptance/validation.rs`);
for (const token of [
  "performance_window_days.clamp(1, 365)",
  "cost_window_days.clamp(1, 365)",
  "单日成本预算",
  "周期成本预算",
  "必须是大于或等于 0 的有限数值",
]) check(validation.includes(token), `Release 请求兼容语义丢失：${token}`);

const chain = read(`${useCaseRoot}/run_acceptance/checks/chain.rs`);
for (const token of [
  "integration_contracts_a_to_i",
  "external_model_provider_boundary",
  "database_migrations",
  "runtime_lifecycle_evidence",
  "public_model_boundary",
  "external_model_runtime",
  "facts.migration_count >= 27",
  "重新执行连续数据库迁移，禁止手工跳过历史迁移。",
  "不得使用合成样本替代真实统计",
]) check(chain.includes(token), `Release chain 门禁语义丢失：${token}`);

const performance = read(`${useCaseRoot}/run_acceptance/checks/performance.rs`);
for (const token of [
  "facts.database_latency_ms > 1_500",
  "facts.database_latency_ms > 500",
  "value > 5_000.0",
  "value > 2_000.0",
  "database_query_health",
]) check(performance.includes(token), `Release 性能门禁语义丢失：${token}`);

const security = read(`${useCaseRoot}/run_acceptance/checks/security.rs`);
for (const token of [
  "facts.immutable_trigger_count >= 8",
  "immutable_ledgers",
  "credential_boundary",
  "API Key 仅由 Rust/Windows 凭据管理器读取",
]) check(security.includes(token), `Release 安全门禁语义丢失：${token}`);

const cost = read(`${useCaseRoot}/run_acceptance/checks/cost.rs`);
for (const token of [
  "openai_cost_observability",
  "显式成本预算已经超限，发布验收被阻断。",
  "至少一个预算未设置。",
  "降低调用量或调整经批准的预算后重新验收。",
]) check(cost.includes(token), `Release 成本门禁语义丢失：${token}`);

const releaseCheck = read(`${useCaseRoot}/run_acceptance/checks/release.rs`);
for (const token of [
  'env!("CARGO_PKG_VERSION") == "0.23.0"',
  'stage == "J"',
  "release_artifact_contract",
  "停止打包；同步 package、Cargo、Tauri、迁移和 J 契约后重新构建。",
]) check(releaseCheck.includes(token), `Release 发布门禁语义丢失：${token}`);

const summary = read(`${useCaseRoot}/run_acceptance/summary.rs`);
for (const token of [
  "ReleaseAcceptancePerformanceSummary",
  "ReleaseAcceptanceCostSummary",
  "BTreeMap",
  "ReleaseAcceptanceStatus::Blocked",
  "ReleaseAcceptanceStatus::Warning",
]) check(summary.includes(token), `Release 汇总语义丢失：${token}`);
const report = read(`${useCaseRoot}/run_acceptance/report.rs`);
for (const token of [
  "Sha256::new()",
  "RELEASE_ACCEPTANCE_CONTRACT_VERSION",
  "RELEASE_ACCEPTANCE_FIXTURE_VERSION",
  '"category_summaries"',
  '"performance"',
  '"cost"',
  '"checks"',
]) check(report.includes(token), `Release 报告哈希字段丢失：${token}`);

for (const command of publicMethods) {
  check(commands.includes(`fn ${command}`), `Tauri Release 命令缺少：${command}`);
}

const stale = [];
const walk = (directory) => {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (entry.isFile() && /\.(?:rs|mjs|js|ts)$/.test(entry.name)) {
      const relative = path.relative(root, full).replaceAll("\\", "/");
      if (relative === "scripts/verify-release-service.mjs") continue;
      const text = fs.readFileSync(full, "utf8");
      if (text.includes(legacyOwner)) stale.push(relative);
    }
  }
};
for (const scanRoot of ["crates/application/src", "scripts"]) walk(path.join(root, scanRoot));
check(stale.length === 0, `仍有源码/验证器硬编码旧 Release owner：${stale.join(", ")}`);

const packageDefinition = JSON.parse(read("package.json"));
const frontendVerifier = read("scripts/verify-frontend.mjs");
check(packageDefinition.scripts?.["verify:release-service"] === "node scripts/verify-release-service.mjs", "package.json 未登记 Release 专项门禁");
check(packageDefinition.scripts?.["verify:architecture"]?.includes("verify-release-service.mjs"), "verify:architecture 未接入 Release 门禁");
check(frontendVerifier.includes('"verify-release-service.mjs"'), "完整 frontend 未接入 Release 门禁");

if (failures.length) {
  console.error("Release Service 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}
console.log("Release Service 验证通过：3 个公共 Release Acceptance 用例由唯一 ReleaseService 编排，检查/汇总/哈希职责拆分，ReleaseAcceptancePort 与 PersistenceStore 适配完整且旧 owner 清零。");
