import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8").replaceAll("\r\n", "\n");
const failures = [];
const check = (condition, message) => {
  if (!condition) failures.push(message);
};

function rustFiles(path) {
  const absolute = join(root, path);
  const result = [];
  for (const entry of readdirSync(absolute, { withFileTypes: true })) {
    const child = join(absolute, entry.name);
    if (entry.isDirectory()) result.push(...rustFiles(relative(root, child)));
    else if (entry.name.endsWith(".rs")) result.push(relative(root, child).replaceAll("\\", "/"));
  }
  return result;
}

const requiredFiles = [
  "crates/application/src/services/competition/mod.rs",
  "crates/application/src/services/competition/service.rs",
  "crates/application/src/services/competition/facade.rs",
  "crates/application/src/services/rules/mod.rs",
  "crates/application/src/services/rules/service.rs",
  "crates/application/src/services/rules/facade.rs",
  "crates/application/src/use_cases/competition/create_competition/mod.rs",
  "crates/application/src/use_cases/competition/delete_competition/mod.rs",
  "crates/application/src/use_cases/competition/create_season/mod.rs",
  "crates/application/src/use_cases/competition/create_stage/mod.rs",
  "crates/application/src/use_cases/competition/create_round/mod.rs",
  "crates/application/src/use_cases/competition/load_hierarchy/mod.rs",
  "crates/application/src/use_cases/rules/register_package/mod.rs",
  "crates/application/src/use_cases/rules/register_built_ins/mod.rs",
  "crates/application/src/use_cases/rules/create_binding/mod.rs",
  "crates/application/src/use_cases/rules/load_catalog/mod.rs",
  "crates/application/src/use_cases/rules/package_factory/mod.rs",
  "crates/application/src/use_cases/rules/package_validation/mod.rs",
];
for (const path of requiredFiles) check(existsSync(join(root, path)), `缺少 R3-03 文件：${path}`);

for (const legacy of [
  "crates/application/src/competition.rs",
  "crates/application/src/rule_packages.rs",
]) {
  check(!existsSync(join(root, legacy)), `R3-03 旧职责文件仍存在：${legacy}`);
}

const library = read("crates/application/src/lib.rs");
const applicationService = read("crates/application/src/service/application_service.rs");
const composition = read("crates/application/src/composition/application_composition.rs");
const registry = read("crates/application/src/composition/port_registry.rs");
const bootstrap = read("crates/application/src/services/database/bootstrap.rs");
const databaseFacade = read("crates/application/src/services/database/facade.rs");
const tauriCompetition = read("src-tauri/src/commands/competition.rs");
const packageDefinition = JSON.parse(read("package.json"));
const frontendVerifier = read("scripts/verify-frontend.mjs");

check(!library.includes("mod competition;"), "lib.rs 仍声明旧 competition 根模块");
check(!library.includes("mod rule_packages;"), "lib.rs 仍声明旧 rule_packages 根模块");
check(
  library.includes("pub use use_cases::rules::package_factory::default_rule_package_template;"),
  "默认规则包模板公共出口未保持兼容",
);
for (const field of ["competition: CompetitionService", "rules: RulesService"]) {
  check(applicationService.includes(field), `ApplicationService 缺少服务字段：${field}`);
}
for (const constructor of ["CompetitionService::new()", "RulesService::new()"]) {
  check(composition.includes(constructor), `组合根未构造：${constructor}`);
}
for (const implementation of [
  "impl CompetitionHierarchyPort for ActiveDatabase",
  "impl RulePackagePort for ActiveDatabase",
  "impl RuleRoutingPort for ActiveDatabase",
]) {
  check(registry.includes(implementation), `组合根适配器缺少：${implementation}`);
}

for (const forbidden of [
  "store.list_competitions()",
  "store.list_seasons()",
  "store.list_stages()",
  "store.list_rounds()",
  "store.list_rule_packages()",
  "store.list_competition_bindings()",
]) {
  check(!bootstrap.includes(forbidden), `bootstrap 仍绕过 Competition/Rules Service：${forbidden}`);
}
check(bootstrap.includes("self.competition.load_hierarchy(&active)"), "bootstrap 未委托 Competition Service");
check(bootstrap.includes("self.rules.load_catalog(&active)"), "bootstrap 未委托 Rules Service");
check(
  databaseFacade.includes("self.rules") && databaseFacade.includes("register_built_ins"),
  "数据库初始化未通过 Rules Service 注册内置规则包",
);
check(
  !databaseFacade.includes("store.register_rule_package") &&
    !databaseFacade.includes("store.ensure_type_default_binding"),
  "数据库初始化仍直接执行规则包持久化",
);

const serviceAndUseCaseFiles = [
  ...rustFiles("crates/application/src/services/competition"),
  ...rustFiles("crates/application/src/services/rules"),
  ...rustFiles("crates/application/src/use_cases/competition"),
  ...rustFiles("crates/application/src/use_cases/rules"),
];
for (const path of serviceAndUseCaseFiles) {
  const source = read(path);
  for (const token of ["football_persistence_postgres", "PostgresStore", "sqlx::", "PgPool", "PersistenceStore"]) {
    check(!source.includes(token), `${path} 泄漏具体持久化实现：${token}`);
  }
}

for (const command of [
  ".create_competition(draft)",
  ".delete_competition(competition_id)",
  ".create_season(draft)",
  ".create_stage(draft)",
  ".create_round(draft)",
  ".register_rule_package(draft)",
  ".create_competition_binding(draft)",
]) {
  check(tauriCompetition.includes(command), `Tauri Competition 公共调用链发生变化：${command}`);
}

check(
  packageDefinition.scripts?.["verify:competition-rules-service"] ===
    "node scripts/verify-competition-rules-service.mjs",
  "package.json 未登记 Competition/Rules Service 专项门禁",
);
check(
  packageDefinition.scripts?.["verify:architecture"]?.includes("verify-competition-rules-service.mjs"),
  "verify:architecture 未接入 Competition/Rules Service 专项门禁",
);
check(
  frontendVerifier.includes('"verify-competition-rules-service.mjs"'),
  "verify:frontend 未接入 Competition/Rules Service 专项门禁",
);

if (failures.length) {
  throw new Error(`Competition/Rules Service 验证失败\n${failures.map((item) => `- ${item}`).join("\n")}`);
}

console.log(
  `Competition/Rules Service 验证通过：${serviceAndUseCaseFiles.length} 个 Service/Use Case Rust 文件，赛事层级、规则包、绑定与 bootstrap 已切换 Ports 边界。`,
);
