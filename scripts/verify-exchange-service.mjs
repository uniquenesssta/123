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
const checkOrder = (text, first, second, message) => {
  const firstIndex = text.indexOf(first);
  const secondIndex = text.indexOf(second);
  check(firstIndex >= 0 && secondIndex >= 0 && firstIndex < secondIndex, message);
};

const legacyOwner = ["crates/application/src", "exchange.rs"].join("/");
const serviceRoot = "crates/application/src/services/exchange";
const useCaseRoot = "crates/application/src/use_cases/exchange";
const expectedUseCases = [
  "export_match_lineup_template",
  "export_match_lineup_data",
  "preview_match_lineup_import",
  "read_match_lineup_import_preview",
  "resolve_match_lineup_import_conflict",
  "commit_match_lineup_import",
  "export_ai_match_package",
  "preview_ai_match_package",
];

check(!fs.existsSync(path.join(root, legacyOwner)), "旧 Exchange Application owner 仍存在");
for (const file of [
  `${serviceRoot}/mod.rs`,
  `${serviceRoot}/service.rs`,
  `${serviceRoot}/facade.rs`,
  `${useCaseRoot}/mod.rs`,
  `${useCaseRoot}/file_validation/mod.rs`,
  "crates/application/src/composition/adapters/exchange.rs",
]) {
  check(fs.existsSync(path.join(root, file)), `缺少 Exchange 模块文件：${file}`);
}
for (const useCase of expectedUseCases) {
  check(
    fs.existsSync(path.join(root, useCaseRoot, useCase, "mod.rs")),
    `Exchange 公共 Use Case 未独立成目录：${useCase}`,
  );
}

const service = read(`${serviceRoot}/service.rs`);
const facade = read(`${serviceRoot}/facade.rs`);
const port = read("crates/application/src/ports/exchange/mod.rs");
const adapter = read("crates/application/src/composition/adapters/exchange.rs");
const servicesRoot = read("crates/application/src/services/mod.rs");
const useCasesRoot = read("crates/application/src/use_cases/mod.rs");
const composition = read("crates/application/src/composition/application_composition.rs");
const applicationService = read("crates/application/src/service/application_service.rs");
const library = read("crates/application/src/lib.rs");
const commands = read("src-tauri/src/commands/exchange.rs");
const packageDefinition = JSON.parse(read("package.json"));
const frontendVerifier = read("scripts/verify-frontend.mjs");

check(servicesRoot.includes("pub(crate) mod exchange;"), "Service 根模块未登记 Exchange");
check(useCasesRoot.includes("pub(crate) mod exchange;"), "Use Case 根模块未登记 Exchange");
check(!library.includes("mod exchange;"), "lib.rs 仍登记旧 Exchange owner");
check(composition.includes("exchange: ExchangeService"), "ApplicationComposition 未聚合 ExchangeService");
check(composition.includes("exchange: ExchangeService::new()"), "组合根未构造唯一 ExchangeService");
check(applicationService.includes("pub(crate) exchange: ExchangeService"), "ApplicationService 未持有 ExchangeService");
check(applicationService.includes("exchange: parts.exchange"), "ApplicationService 未从组合根接收 ExchangeService");

for (const method of expectedUseCases) {
  check(service.includes(`fn ${method}`), `ExchangeService 缺少方法：${method}`);
  check(facade.includes(`pub async fn ${method}`), `ApplicationService 兼容入口缺少：${method}`);
  check(commands.includes(`pub async fn ${method}`), `Tauri Exchange 命令缺少：${method}`);
}
check(
  (facade.match(/\.exchange\s*\n?\s*\./g) ?? []).length >= expectedUseCases.length,
  "Exchange facade 未将 8 个公共入口委托唯一 ExchangeService",
);

for (const token of [
  "async fn export_match_lineup(",
  "match_id: Option<Uuid>",
  "async fn preview_import(",
  "mode: SpreadsheetImportMode",
  "async fn read_import_preview(",
  "async fn resolve_import_conflict(",
  "async fn commit_import(",
  "async fn ai_match_package_context(",
]) {
  check(port.includes(token), `MatchLineupExchangePort 缺少真实能力：${token}`);
}
check(adapter.includes("impl MatchLineupExchangePort for ActiveDatabase"), "Exchange Port 未由 ActiveDatabase 适配");
for (const call of [
  "match_lineup_export_data",
  "preview_match_lineup_import",
  "read_match_lineup_import_preview",
  "resolve_match_lineup_import_conflict",
  "commit_match_lineup_import",
  "ai_match_package_context",
]) {
  check(adapter.includes(call), `Exchange adapter 缺少持久化委托：${call}`);
}

const banned = ["football_persistence_postgres", "sqlx::", "PostgresStore", "PersistenceStore"];
for (const relativePath of [
  `${serviceRoot}/service.rs`,
  `${serviceRoot}/facade.rs`,
  ...expectedUseCases.map((name) => `${useCaseRoot}/${name}/mod.rs`),
  `${useCaseRoot}/file_validation/mod.rs`,
]) {
  const text = read(relativePath);
  for (const token of banned) {
    check(!text.includes(token), `${relativePath} 泄漏基础设施符号：${token}`);
  }
}

const template = read(`${useCaseRoot}/export_match_lineup_template/mod.rs`);
const exportData = read(`${useCaseRoot}/export_match_lineup_data/mod.rs`);
const previewImport = read(`${useCaseRoot}/preview_match_lineup_import/mod.rs`);
const exportAi = read(`${useCaseRoot}/export_ai_match_package/mod.rs`);
const previewAi = read(`${useCaseRoot}/preview_ai_match_package/mod.rs`);
for (const [text, first, second, label] of [
  [template, "validate_output", "session.await?", "比赛模板导出"],
  [exportData, "validate_output", "session.await?", "比赛数据导出"],
  [previewImport, "read_match_lineup_workbook", "session.await?", "比赛导入预检"],
  [exportAi, "validate_output", "session.await?", "AI 分析包导出"],
  [previewAi, "read_match_lineup_workbook", "session.await?", "AI 分析包导入预检"],
]) {
  checkOrder(text, first, second, `${label} 改变了文件错误与数据库未连接错误的既有优先级`);
}

const fileValidation = read(`${useCaseRoot}/file_validation/mod.rs`);
for (const message of ["请选择输出位置", "无法创建输出目录", "文件不存在：", "文件必须使用 .{extension} 扩展名"]) {
  check(fileValidation.includes(message), `文件边界错误语义丢失：${message}`);
}

const scanRoots = ["crates/application/src", "scripts"];
const stale = [];
const walk = (directory) => {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (entry.isFile() && /\.(?:rs|mjs|js|ts)$/.test(entry.name)) {
      const text = fs.readFileSync(full, "utf8");
      if (text.includes(legacyOwner)) stale.push(path.relative(root, full).replaceAll("\\", "/"));
    }
  }
};
for (const scanRoot of scanRoots) walk(path.join(root, scanRoot));
check(stale.length === 0, `仍有可执行源码/验证器硬编码旧 Exchange owner：${stale.join(", ")}`);

check(
  packageDefinition.scripts?.["verify:exchange-service"] === "node scripts/verify-exchange-service.mjs",
  "package.json 未登记 Exchange Service 专项入口",
);
check(
  packageDefinition.scripts?.["verify:architecture"]?.includes("verify-exchange-service.mjs"),
  "verify:architecture 未接入 Exchange Service",
);
check(frontendVerifier.includes('"verify-exchange-service.mjs"'), "完整 frontend 未接入 Exchange Service 门禁");

if (failures.length) {
  console.error("Exchange Service 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}
console.log(
  "Exchange Service 验证通过：旧 owner 已删除，8 个公共用例独立目录化，MatchLineupExchangePort 与组合适配边界完整，兼容入口和文件错误优先级保持。",
);
