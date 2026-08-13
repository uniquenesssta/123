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

const serviceRoot = "crates/application/src/services/exchange";
const useCaseRoot = "crates/application/src/use_cases/exchange";
const adapterRoot = "crates/application/src/composition/adapters/exchange";
const legacyOwners = [
  ["crates/application/src", "exchange.rs"].join("/"),
  ["crates/application/src", "spreadsheet.rs"].join("/"),
];
const matchLineupUseCases = [
  "export_match_lineup_template",
  "export_match_lineup_data",
  "preview_match_lineup_import",
  "read_match_lineup_import_preview",
  "resolve_match_lineup_import_conflict",
  "commit_match_lineup_import",
  "export_ai_match_package",
  "preview_ai_match_package",
];
const spreadsheetUseCases = [
  "export_team_package_template",
  "export_team_package_preview_json",
  "preview_team_package_import",
  "commit_team_package_import",
  "export_player_catalog_template",
  "export_player_catalog_data",
  "preview_player_catalog_import",
  "read_player_catalog_import_preview",
  "resolve_player_catalog_import_conflict",
  "commit_player_catalog_import",
  "export_team_monthly_template",
  "export_team_monthly_data",
  "preview_team_monthly_import",
  "read_team_monthly_import_preview",
  "resolve_team_monthly_import_conflict",
  "commit_team_monthly_import",
];
const allUseCases = [...matchLineupUseCases, ...spreadsheetUseCases];

for (const legacyOwner of legacyOwners) {
  check(!fs.existsSync(path.join(root, legacyOwner)), `旧 Exchange Application owner 仍存在：${legacyOwner}`);
}
for (const file of [
  `${serviceRoot}/mod.rs`,
  `${serviceRoot}/service/mod.rs`,
  `${serviceRoot}/service/match_lineup.rs`,
  `${serviceRoot}/service/spreadsheet.rs`,
  `${serviceRoot}/facade/mod.rs`,
  `${serviceRoot}/facade/match_lineup.rs`,
  `${serviceRoot}/facade/spreadsheet.rs`,
  `${useCaseRoot}/mod.rs`,
  `${useCaseRoot}/file_validation/mod.rs`,
  `${useCaseRoot}/file_validation/spreadsheet.rs`,
  `${adapterRoot}/mod.rs`,
  `${adapterRoot}/match_lineup.rs`,
  `${adapterRoot}/spreadsheet.rs`,
]) {
  check(fs.existsSync(path.join(root, file)), `缺少 Exchange 模块文件：${file}`);
}
for (const useCase of allUseCases) {
  check(fs.existsSync(path.join(root, useCaseRoot, useCase, "mod.rs")), `Exchange 公共 Use Case 未独立成目录：${useCase}`);
}
for (const useCase of spreadsheetUseCases) {
  check(fs.existsSync(path.join(root, useCaseRoot, useCase, "use_case.rs")), `Spreadsheet Use Case 缺少唯一实现：${useCase}`);
}

const service = [read(`${serviceRoot}/service/match_lineup.rs`), read(`${serviceRoot}/service/spreadsheet.rs`)].join("\n");
const facade = [read(`${serviceRoot}/facade/match_lineup.rs`), read(`${serviceRoot}/facade/spreadsheet.rs`)].join("\n");
const port = read("crates/application/src/ports/exchange/mod.rs");
const matchLineupAdapter = read(`${adapterRoot}/match_lineup.rs`);
const spreadsheetAdapter = read(`${adapterRoot}/spreadsheet.rs`);
const library = read("crates/application/src/lib.rs");
const commands = read("src-tauri/src/commands/exchange.rs");
const composition = read("crates/application/src/composition/application_composition.rs");
const applicationService = read("crates/application/src/service/application_service.rs");

check(!library.includes("mod exchange;"), "lib.rs 仍登记旧 Exchange owner");
check(!library.includes("mod spreadsheet;"), "lib.rs 仍登记旧 Spreadsheet owner");
check(composition.includes("exchange: ExchangeService"), "ApplicationComposition 未聚合 ExchangeService");
check(composition.includes("exchange: ExchangeService::new()"), "组合根未构造唯一 ExchangeService");
check(applicationService.includes("pub(crate) exchange: ExchangeService"), "ApplicationService 未持有 ExchangeService");
check(applicationService.includes("exchange: parts.exchange"), "ApplicationService 未从组合根接收 ExchangeService");
for (const method of allUseCases) {
  check(service.includes(`fn ${method}`), `ExchangeService 缺少方法：${method}`);
  check(facade.includes(`pub async fn ${method}`), `ApplicationService 兼容入口缺少：${method}`);
  check(commands.includes(`pub async fn ${method}`), `Tauri Exchange 命令缺少：${method}`);
}
check((facade.match(/self\.exchange/g) ?? []).length >= allUseCases.length, "Exchange facade 未统一委托 ExchangeService");

for (const token of [
  "pub trait MatchLineupExchangePort",
  "pub trait SpreadsheetExchangePort",
  "pub trait MonthlyWorkbookPort",
  "async fn export_match_lineup(",
  "async fn ai_match_package_context(",
  "async fn reference_data(",
  "async fn export_data(",
  "async fn data_gaps(",
  "async fn preview_import_with_team_references(",
  "async fn read_import_preview(",
  "async fn resolve_conflict(",
  "async fn commit_import(",
]) {
  check(port.includes(token), `Exchange Port 缺少真实能力：${token}`);
}
check(matchLineupAdapter.includes("impl MatchLineupExchangePort for PersistenceStore"), "MatchLineupExchangePort 未由 PersistenceStore 适配");
for (const call of [
  "match_lineup_export_data",
  "preview_match_lineup_import",
  "read_match_lineup_import_preview",
  "resolve_match_lineup_import_conflict",
  "commit_match_lineup_import",
  "ai_match_package_context",
]) {
  check(matchLineupAdapter.includes(call), `Match Lineup adapter 缺少委托：${call}`);
}
check(spreadsheetAdapter.includes("impl SpreadsheetExchangePort for PersistenceStore"), "SpreadsheetExchangePort 未由 PersistenceStore 适配");
check(spreadsheetAdapter.includes("impl MonthlyWorkbookPort for PersistenceStore"), "MonthlyWorkbookPort 未由 PersistenceStore 适配");
for (const call of [
  "player_catalog_reference_data",
  "spreadsheet_export_data",
  "player_monthly_data_gaps",
  "preview_spreadsheet_import",
  "preview_spreadsheet_import_with_team_references",
  "read_spreadsheet_import_preview",
  "resolve_spreadsheet_import_conflict",
  "commit_spreadsheet_import",
  "team_monthly_workbook_data",
  "preview_team_monthly_import",
  "read_team_monthly_import_preview",
  "resolve_team_monthly_import_conflict",
  "commit_team_monthly_import",
]) {
  check(spreadsheetAdapter.includes(call), `Spreadsheet adapter 缺少委托：${call}`);
}

const banned = ["football_persistence_postgres", "sqlx::", "PostgresStore", "PersistenceStore"];
for (const relativePath of [
  `${serviceRoot}/service/match_lineup.rs`,
  `${serviceRoot}/service/spreadsheet.rs`,
  `${serviceRoot}/facade/match_lineup.rs`,
  `${serviceRoot}/facade/spreadsheet.rs`,
  ...matchLineupUseCases.map((name) => `${useCaseRoot}/${name}/mod.rs`),
  ...spreadsheetUseCases.map((name) => `${useCaseRoot}/${name}/use_case.rs`),
]) {
  const text = read(relativePath);
  for (const token of banned) check(!text.includes(token), `${relativePath} 泄漏基础设施符号：${token}`);
}

for (const [relativePath, first, second, label] of [
  [`${useCaseRoot}/export_match_lineup_template/mod.rs`, "validate_output", "session.await?", "比赛模板导出"],
  [`${useCaseRoot}/preview_match_lineup_import/mod.rs`, "read_match_lineup_workbook", "session.await?", "比赛导入预检"],
  [`${useCaseRoot}/export_team_package_template/use_case.rs`, "validate_xlsx_path", "session.await?", "完整资料包模板导出"],
  [`${useCaseRoot}/preview_team_package_import/use_case.rs`, "read_team_package_workbook", "session.await?", "完整资料包预检"],
  [`${useCaseRoot}/export_player_catalog_data/use_case.rs`, "validate_xlsx_path", "session.await?", "球员数据导出"],
  [`${useCaseRoot}/preview_player_catalog_import/use_case.rs`, "read_player_monthly_workbook", "session.await?", "球员导入预检"],
  [`${useCaseRoot}/export_team_monthly_data/use_case.rs`, "validate_xlsx_path", "session.await?", "球队月度数据导出"],
  [`${useCaseRoot}/preview_team_monthly_import/use_case.rs`, "read_team_monthly_workbook", "session.await?", "球队月度导入预检"],
  [`${useCaseRoot}/commit_team_package_import/use_case.rs`, "team_batch_id.is_none()", "session.await?", "完整资料包提交"],
]) {
  checkOrder(read(relativePath), first, second, `${label} 改变了既有错误优先级`);
}
const previewJson = read(`${useCaseRoot}/export_team_package_preview_json/use_case.rs`);
check(!previewJson.includes("session.await?"), "预检 JSON 导出错误地引入数据库依赖");
check(
  read(`${serviceRoot}/service/spreadsheet.rs`).includes("export_team_package_preview_json::execute(output_path, preview)"),
  "预检 JSON 导出不再保持无数据库会话行为",
);
for (const message of ["请选择 JSON 输出位置", "请选择 Excel 输出位置", "Excel 文件不存在：", "无法创建输出目录"]) {
  check(read(`${useCaseRoot}/file_validation/spreadsheet.rs`).includes(message), `Spreadsheet 文件错误语义丢失：${message}`);
}
const workbookRecognition = [
  read(`${useCaseRoot}/preview_player_catalog_import/use_case.rs`),
  read(`${useCaseRoot}/preview_team_package_import/use_case.rs`),
].join("\n");
for (const message of [
  "检测到 football.team-monthly.v1 球队月度工作包",
  "检测到 football.team-package.v1 球队完整资料包",
  "检测到球员工作包",
]) {
  check(workbookRecognition.includes(message), `Spreadsheet 工作包类型识别语义丢失：${message}`);
}

const stale = [];
const walk = (directory) => {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (entry.isFile() && /\.(?:rs|mjs|js|ts)$/.test(entry.name)) {
      const text = fs.readFileSync(full, "utf8");
      for (const legacyOwner of legacyOwners) {
        if (text.includes(legacyOwner)) stale.push(path.relative(root, full).replaceAll("\\", "/"));
      }
    }
  }
};
for (const scanRoot of ["crates/application/src", "scripts"]) walk(path.join(root, scanRoot));
check(stale.length === 0, `仍有源码/验证器硬编码旧 Exchange owner：${stale.join(", ")}`);

const packageDefinition = JSON.parse(read("package.json"));
const frontendVerifier = read("scripts/verify-frontend.mjs");
check(packageDefinition.scripts?.["verify:exchange-service"] === "node scripts/verify-exchange-service.mjs", "package.json 未登记 Exchange 专项门禁");
check(packageDefinition.scripts?.["verify:architecture"]?.includes("verify-exchange-service.mjs"), "verify:architecture 未接入 Exchange 门禁");
check(frontendVerifier.includes('"verify-exchange-service.mjs"'), "完整 frontend 未接入 Exchange 门禁");

if (failures.length) {
  console.error("Exchange Service 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}
console.log("Exchange Service 验证通过：AT1 + AT2 共 24 个公共用例由唯一 ExchangeService 编排，Spreadsheet/Monthly Ports 与 PersistenceStore 适配完整，旧 owners 清零且兼容错误优先级保持。");
