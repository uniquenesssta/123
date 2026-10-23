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
  `${useCaseRoot}/queries.rs`,
  `${useCaseRoot}/run_acceptance/workflow.rs`,
  `${useCaseRoot}/run_acceptance/checks/catalog.rs`,
  `${useCaseRoot}/tests.rs`,
  adapter,
]) {
  check(fs.existsSync(path.join(root, file)), `缺少 Release 模块文件：${file}`);
}
for (const file of ["mod.rs", "runtime.rs", "ledger.rs", "read.rs", "mapping.rs", "tests.rs"]) {
  check(fs.existsSync(path.join(root, "crates/persistence-postgres/src/release_acceptance", file)), `缺少发布验收PG职责：${file}`);
}
for (const file of [`${useCaseRoot}/list_runs/mod.rs`, `${useCaseRoot}/read_run/mod.rs`, "crates/persistence-postgres/src/release_acceptance.rs"]) {
  check(!fs.existsSync(path.join(root, file)), `旧发布验收职责未退出：${file}`);
}

const service = read(`${serviceRoot}/service.rs`);
const facade = read(`${serviceRoot}/facade.rs`);
const library = read("crates/application/src/lib.rs");
const composition = read("crates/application/src/composition/application_composition.rs");
const applicationService = read("crates/application/src/service/application_service.rs");
const ports = read("crates/application/src/ports/release/mod.rs");
const adapterText = read(adapter);
const commands = ["run_release_acceptance", "list_release_acceptance_runs", "read_release_acceptance_run"].map(name => read(`src-tauri/src/commands/release/${name}.rs`)).join("\n");

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
  `${useCaseRoot}/queries.rs`,
  `${useCaseRoot}/run_acceptance/workflow.rs`,
  `${useCaseRoot}/run_acceptance/checks/catalog.rs`,
  `${useCaseRoot}/tests.rs`,
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


const workflow = read(`${useCaseRoot}/run_acceptance/workflow.rs`);
const queries = read(`${useCaseRoot}/queries.rs`);
const catalog = read(`${useCaseRoot}/run_acceptance/checks/catalog.rs`);
const pgRoot = "crates/persistence-postgres/src/release_acceptance";
const runtime = read(`${pgRoot}/runtime.rs`);
const ledger = read(`${pgRoot}/ledger.rs`);
const pgRead = read(`${pgRoot}/read.rs`);
const mapping = read(`${pgRoot}/mapping.rs`);
const compact = (text) => text.replace(/\s+/g, "");
const occurrences = (text, token) => text.split(token).length - 1;
const sensitivePolicy = read("scripts/verify-public-model-boundary.mjs").match(/const sensitiveTokens = \[([\s\S]*?)\];/);
check(Boolean(sensitivePolicy), "原冻结模型保护策略缺失");
const sensitiveTokens = JSON.parse(`[${(sensitivePolicy?.[1] ?? "").replace(/,\s*$/, "")}]`);
check(sensitiveTokens.every((token) => typeof token === "string"), "原模型敏感策略应为字符串名单");
for (const owner of [workflow, queries, catalog]) {
  for (const token of sensitiveTokens) check(!owner.toLowerCase().includes(token.toLowerCase()), `新发布实际owner出现原保护模型内容：${token}`);
}
for (const file of [`${useCaseRoot}/mod.rs`, `${useCaseRoot}/run_acceptance/mod.rs`, `${useCaseRoot}/run_acceptance/checks/mod.rs`, `${pgRoot}/mod.rs`]) {
  check(!/\bfn\b|\bimpl\b|\blet\b/.test(read(file)), `发布验收目录出口承载业务：${file}`);
}
check(read(`${useCaseRoot}/mod.rs`).includes("#[cfg(test)]\nmod tests;") && read(`${pgRoot}/mod.rs`).includes("#[cfg(test)]\nmod tests;"), "发布验收未登记原单测目标");
check(read(`${useCaseRoot}/run_acceptance/mod.rs`).includes("pub(crate) use workflow::run_release_acceptance;") && read(`${useCaseRoot}/run_acceptance/checks/mod.rs`).includes("pub(super) use catalog::{build, check};"), "发布验收目录未显式导出实际 owner");
check(catalog.includes("pub(in crate::use_cases::release::run_acceptance) fn"), "目录内检查构造可见性不能扩大到公开服务");
for (const [method,call,owner] of [
  ["run_release_acceptance", "release::run_acceptance::run_release_acceptance(session, request)", workflow],
  ["list_release_acceptance_runs", "release::list_release_acceptance_runs(session, limit)", queries],
  ["read_release_acceptance_run", "release::read_release_acceptance_run(session, run_id)", queries],
]) {
  check(occurrences(owner, `fn ${method}`) === 1 && service.includes(call), `发布验收实际调用或唯一owner未切换：${method}`);
}
check(!service.includes("::execute("), "ReleaseService 仍经过旧execute入口");
check(occurrences(workflow, "session.await?")===1 && occurrences(workflow, "port.persist_run(&run).await?")===1, "原发布会话获取和报告保存必须各一次");
const order = ["validation::normalize_request(request)?", "Utc::now()", "Uuid::new_v4()", "session.await?", ".runtime_facts(request.performance_window_days, request.cost_window_days)", "checks::build(run_id, &runtime, &request)", "summary::build(&checks, &runtime, &request)", "let completed_at = Utc::now()", "report::hash(", "let run = ReleaseAcceptanceRun", "port.persist_run(&run).await?", "Ok(run)"];
let previous = -1;
for (const token of order) { const index = workflow.indexOf(token, previous+1); check(index > previous, `发布原校验/会话/生成/保存顺序改变：${token}`); previous=index; }
check(!workflow.includes("port.read_run(") && !workflow.includes("if summary.overall_status"), "原发布执行不得新增回读或丢弃阻断报告");
const checkOrder = ["chain::runtime_checks", "chain::fixture_checks", "performance::checks", "security::checks", "cost::checks", "release::checks", "checks.iter_mut().enumerate()", "i32::try_from(index + 1)"];
previous=-1;for(const token of checkOrder){const index=catalog.indexOf(token);check(index>previous,`发布检查顺序/序号改变：${token}`);previous=index;}
check(catalog.includes("id: Uuid::new_v4()") && catalog.includes("duration_ms: duration_ms.max(0)"), "检查身份或原非负耗时投影改变");
check(queries.includes("Ok(session.await?.list_runs(limit).await?)") && queries.includes("Ok(session.await?.read_run(run_id).await?)") && !queries.includes("clamp(") && !queries.includes("report::hash"), "报告查询不得新增筛选/窗口校验/重算哈希");
check(read("crates/persistence-postgres/src/lib.rs").includes("mod release_acceptance;") && read("architecture/module-boundaries.json").includes(`${pgRoot}/mod.rs`), "PG发布验收必须保持原根模块身份");
const runtimeOrder = ["self.health().await?", "SELECT DISTINCT stage", "FROM pg_trigger", "let chain =", "let performance =", "let cost =", "Ok(ReleaseAcceptanceRuntimeFacts"];
previous=-1;for(const token of runtimeOrder){const index=runtime.indexOf(token);check(index>previous,`原运行事实读取顺序改变：${token}`);previous=index;}
check(runtime.includes("performance_window_days.clamp(1, 365)") && runtime.includes("cost_window_days.clamp(1, 365)") && runtime.includes("percentile_cont(0.95)") && runtime.includes("usage_day_utc = (SELECT MAX(usage_day_utc) FROM usage)"), "原窗口/耗时P95/最新用量日期规则改变");
check(occurrences(ledger, "self.pool.begin().await?")===1 && occurrences(ledger,"tx.commit().await?")===1, "不可变报告与全部检查必须保持原单事务");
const ledgerOrder = ["self.pool.begin().await?", "INSERT INTO platform.release_acceptance_runs", "for check in &run.checks", "insert_release_acceptance_check(&mut tx, check).await?", "write_audit_event(", "tx.commit().await?", "Ok(())"];
previous=-1;for(const token of ledgerOrder){const index=ledger.indexOf(token,previous+1);check(index>previous,`原报告/check/审计/提交顺序改变：${token}`);previous=index;}
check(ledger.includes("INSERT INTO platform.release_acceptance_checks") && ledger.includes('.bind(&run.report_sha256)') && ledger.includes('"acceptance_mode":"public_shell_and_runtime"') && ledger.includes('"provider_state":"NOT_BUNDLED"') && !/UPDATE\s+platform\.release_acceptance_|ON\s+CONFLICT/i.test(ledger), "报告写入不得变为覆盖/冲突成功，哈希与公开元数据必须保持");
check(pgRead.includes("limit.clamp(1, 200)") && pgRead.includes("ORDER BY completed_at DESC, id DESC") && pgRead.includes("WHERE run_id=$1 ORDER BY sequence_no") && pgRead.includes("发布验收记录不存在"), "原报告列表/缺失/检查排序规则改变");
check(pgRead.includes("run_from_row(&row, checks)") && occurrences(mapping,"report_count(row.try_get::<i32, _>")===6 && mapping.includes("u32::try_from(value).unwrap_or_default()"), "报告映射及六计数默认不能改变");
for(const token of ['"pass" => Ok(ReleaseAcceptanceStatus::Pass)', '"warning" => Ok(ReleaseAcceptanceStatus::Warning)', '"blocked" => Ok(ReleaseAcceptanceStatus::Blocked)', '未知发布验收状态：{other}']) check(mapping.includes(token),`原状态解析/错误改变：${token}`);
check(!mapping.includes("value.trim()"), "状态解析不得新增修剪");
check(occurrences(read(`${useCaseRoot}/tests.rs`),"#[tokio::test]")===15 && occurrences(read(`${pgRoot}/tests.rs`),"#[test]")===4, "发布验收新增原单测边界缺失");

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

// R11-09 transport contract BEGIN
{
  const commandRoot = "src-tauri/src/commands/release";
  const files = {
  "run_release_acceptance": "src-tauri/src/commands/release/run_release_acceptance.rs",
  "list_release_acceptance_runs": "src-tauri/src/commands/release/list_release_acceptance_runs.rs",
  "read_release_acceptance_run": "src-tauri/src/commands/release/read_release_acceptance_run.rs"
};
  const signatures = {
  "run_release_acceptance": "pub async fn run_release_acceptance(\n    state: State<'_, AppState>,\n    request: football_domain::ReleaseAcceptanceRequest,\n) -> Result<football_domain::ReleaseAcceptanceRun, String>",
  "list_release_acceptance_runs": "pub async fn list_release_acceptance_runs(\n    state: State<'_, AppState>,\n    limit: u32,\n) -> Result<Vec<football_domain::ReleaseAcceptanceRunSummary>, String>",
  "read_release_acceptance_run": "pub async fn read_release_acceptance_run(\n    state: State<'_, AppState>,\n    run_id: String,\n) -> Result<football_domain::ReleaseAcceptanceRun, String>"
};
  const bodies = {
  "run_release_acceptance": "state\n        .service\n        .run_release_acceptance(request)\n        .await\n        .map_err(|error| error.to_string())",
  "list_release_acceptance_runs": "state\n        .service\n        .list_release_acceptance_runs(limit)\n        .await\n        .map_err(|error| error.to_string())",
  "read_release_acceptance_run": "state\n        .service\n        .read_release_acceptance_run(parse_uuid(&run_id, \"验收运行 ID\")?)\n        .await\n        .map_err(|error| error.to_string())"
};
  const tokens = source => (source.match(/"(?:\\.|[^"\\])*"|[A-Za-z_][A-Za-z_0-9]*|[^\s]/g) ?? []).join("\0");
  const signatureTokens = source => tokens(source.replace(/,\s*\)/g, ")"));
  const moduleTokens = source => tokens(source.replace(/,\s*}/g, "}"));
  const names = Object.keys(files).sort();
  const contract = JSON.parse(read("architecture/command-contract.json"));
  const boundaries = JSON.parse(read("architecture/module-boundaries.json")).tauri_commands;
  const group = contract.command_groups.find(item => item.module === "release");
  const modules = read("src-tauri/src/commands.rs");
  check(!fs.existsSync(path.join(root, "src-tauri/src/commands/release_acceptance.rs")), "R11-09 旧发布命令owner仍存在");
  check(modules.includes("mod release;") && modules.includes("pub use release::*;") && !modules.includes("release_acceptance"), "R11-09 根入口未切换唯一release目录");
  check(group?.definition_file === `${commandRoot}/mod.rs` && group?.parameter_and_return_signature_file === `${commandRoot}/mod.rs` && boundaries.groups.release === `${commandRoot}/mod.rs` && !Object.hasOwn(boundaries.groups, "release_acceptance"), "R11-09 正式清单必须指向唯一release目录");
  check(JSON.stringify(group?.commands) === JSON.stringify(["run_release_acceptance", "list_release_acceptance_runs", "read_release_acceptance_run"]), "R11-09 原三命令组内顺序漂移");
  const exports = name => `pub use ${name}::{__cmd__${name},__tauri_command_name_${name},${name}};`;
  check(moduleTokens(read(`${commandRoot}/mod.rs`)) === moduleTokens([...names.map(name => `mod ${name};`), ...names.map(exports)].join("\n")), "R11-09 出口只显式登记三条命令与双宏，不承载业务或替代路由");
  for (const name of names) {
    const file = files[name];
    const source = read(file);
    const signature = source.match(new RegExp(`pub (?:async )?fn ${name}\\([\\s\\S]*?\\{`))?.[0].slice(0, -1) ?? "";
    const body = source.slice(source.indexOf("{", source.indexOf(`fn ${name}`)) + 1, source.lastIndexOf("}"));
    check(signatureTokens(signature) === signatureTokens(signatures[name]), `R11-09 原State/async/参数/返回签名漂移：${name}`);
    check(tokens(body) === tokens(bodies[name]), `R11-09 必须保留真实Service/原参数/UUID/await/错误转换：${name}`);
    check(group?.command_definition_files?.[name] === file, `R11-09 原命令实际定义owner漂移：${name}`);
    const imports = `use crate::bootstrap::AppState; ${name === "read_release_acceptance_run" ? "use crate::commands::parse_uuid;" : ""} use tauri::State;`;
    check(tokens(source.slice(0, source.indexOf("#[tauri::command]"))) === tokens(imports) && (source.match(/#\[tauri::command\]/g) ?? []).length === 1 && (source.match(/\bfn\b/g) ?? []).length === 1, `R11-09 命令必须只借原State与唯一UUID helper：${name}`);
    for (const token of ["sqlx::", "football_persistence_postgres", "PostgresStore", "PgPool", "std::fs", "tokio::spawn", "ApplicationService::new", "Mutex<", "static ", "macro_rules!", ".unwrap()", ".expect("]) check(!source.includes(token), `R11-09 命令越过原业务/状态/存储边界：${name}/${token}`);
  }
}
// R11-09 transport contract END

if (failures.length) {
  console.error("Release Service 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}
console.log("Release Service 验证通过：3 个公共 Release Acceptance 用例由唯一 ReleaseService 编排，检查/汇总/哈希职责拆分，ReleaseAcceptancePort 与 PersistenceStore 适配完整且旧 owner 清零。");
