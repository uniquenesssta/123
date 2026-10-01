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

// R7-10：账本持有唯一 SQL 职责，工作簿业务流程持有唯一提交事务。
const ledgerRoot = "crates/persistence-postgres/src/adapters/workbooks/batch_ledger";
for (const file of ["mod.rs", "batch.rs", "rows.rs", "mapping.rs", "read.rs"]) check(fs.existsSync(path.join(root, ledgerRoot, file)), `缺少批次账本职责：${file}`);
const ledgerBatch = read(`${ledgerRoot}/batch.rs`);
const ledgerRows = read(`${ledgerRoot}/rows.rs`);
const ledgerMapping = read(`${ledgerRoot}/mapping.rs`);
const ledgerRead = read(`${ledgerRoot}/read.rs`);
const playerWorkbook = ["preview", "conflict", "commit", "export", "identity", "validation", "values"]
  .map((name) => read(name === "export" ? "crates/persistence-postgres/src/adapters/workbooks/monthly_player/read.rs" : `crates/persistence-postgres/src/adapters/workbooks/player_catalog/${name}.rs`)).join("\n");
const matchWorkbook = read("crates/persistence-postgres/src/match_exchange.rs");
const integration = read("crates/persistence-postgres/tests/postgres_integration.rs");
const persistencePaths = [];
const collectPersistence = (directory) => { for (const entry of fs.readdirSync(directory,{withFileTypes:true})) { const full=path.join(directory,entry.name); if(entry.isDirectory()) collectPersistence(full); else if(entry.name.endsWith(".rs")) persistencePaths.push(path.relative(root,full).replaceAll("\\","/")); } };
collectPersistence(path.join(root,"crates/persistence-postgres/src"));
for (const [owner,names] of [["read.rs",["read_spreadsheet_import_preview","read_match_lineup_import_preview","read_team_monthly_import_preview"]],["batch.rs",["lock_batch_in_tx","start_batch_in_tx","finish_batch_in_tx","create_player_batch_in_tx","create_match_batch_in_tx","require_pending"]],["rows.rs",["insert_import_row","count_preview_rows","lock_import_row_in_tx","commit_rows_in_tx","mark_imported_in_tx","refresh_pending_counts_in_tx","skip_import_row_in_tx","resolve_import_row_in_tx"]]]) {
  for (const name of names) {
    const owners=persistencePaths.filter((file)=>new RegExp(`(?:pub(?:\\(crate\\))?\\s+)?(?:async\\s+)?fn\\s+${name}\\s*\\(`).test(read(file)));
    check(owners.length===1 && owners[0]===`${ledgerRoot}/${owner}`,`${name} 必须只有一个批次账本 owner`);
  }
}
for (const [label,source] of [["球员",playerWorkbook],["比赛",matchWorkbook]]) {
  check(!source.includes("catalog.import_batches") && !source.includes("catalog.import_rows"), `${label}工作簿仍残留第二份账本 SQL`);
  check(!/fn (?:insert_row|insert_import_row|row_from_db|import_row_from_row|count_rows|count_preview_rows)\s*\(/.test(source),`${label}工作簿仍残留旧行账本实现`);
}
for (const [source,family,method,apply] of [[playerWorkbook,"Player","commit_spreadsheet_import","apply_import_row"],[matchWorkbook,"Match","commit_match_lineup_import","apply_match_exchange_row"]]) {
  const start=source.indexOf(`pub async fn ${method}(`); const next=source.indexOf("pub async fn ",start+20); const body=source.slice(start,next < 0?undefined:next);
  const lock=body.indexOf(`ledger::lock_batch_in_tx(&mut tx, batch_id, ImportFamily::${family})`);
  const pending=body.indexOf("ledger::require_pending(");
  const blockers=body.indexOf("ledger_rows::blocking_rows_in_tx(");
  const running=body.indexOf("ledger::start_batch_in_tx(");
  const applyOffset=body.indexOf(`${apply}(`);
  const imported=body.indexOf("ledger_rows::mark_imported_in_tx(");
  const finish=body.indexOf(`ledger::finish_batch_in_tx(&mut tx, &result, ImportFamily::${family})`);
  const commit=body.indexOf("tx.commit().await?");
  check((body.match(/self.pool.begin\(\)/g)??[]).length===1 && (body.match(/tx.commit\(\)/g)??[]).length===1 && lock>=0 && pending>lock && blockers>pending && running>blockers && applyOffset>running && imported>applyOffset && finish>imported && commit>finish, `${method} 必须锁批次、检查状态与冲突、应用业务、写行与计数审计后唯一提交`);
  check(body.includes("Ok(result)") && body.includes("error_count: 0"),`${method} 返回必须使用同一成功结果对象`);
}
for (const source of [ledgerBatch,ledgerRows]) check(!source.includes(".begin()") && !source.includes(".commit()") && !source.includes("self.pool") && !source.includes("crate::spreadsheet_exchange") && !source.includes("crate::match_exchange"),"账本步骤不得另开/提交事务或反向依赖工作簿业务");
check(ledgerBatch.includes("import_type=ANY($2) FOR UPDATE") && ledgerBatch.includes('vec!["player_catalog_xlsx", "player_monthly_xlsx"]') && ledgerBatch.includes('vec!["match_lineup_xlsx"]') && ledgerBatch.includes('vec!["team_monthly_xlsx"]'),"批次父锁必须隔离球员与比赛导入类型");
check(/if\s+status\s*==\s*"pending"\s*\{/.test(ledgerBatch) && ledgerRows.includes("status IN ('conflict','error')") && ledgerRows.includes("FOR UPDATE"),"批次状态/未解决冲突与行锁门禁缺失");
check(playerWorkbook.includes('status == "succeeded"') && playerWorkbook.includes('batch.try_get::<i64, _>("ended_previous_count")? as u64') && !matchWorkbook.includes('status == "succeeded"'),"球员成功重试读回与比赛重复提交拒绝语义必须保持");
const finisher=ledgerBatch.slice(ledgerBatch.indexOf("pub(crate) async fn finish_batch_in_tx"),ledgerBatch.indexOf("#[cfg(test)]"));
for (const field of ["inserted_count","updated_count","ended_previous_count","skipped_count","error_count"]) check(finisher.includes(`.bind(result.${field} as i64)`),`成功账本漏用结果计数：${field}`);
check(finisher.includes(".bind(result.finished_at)") && finisher.indexOf("execute(&mut **tx)")<finisher.indexOf("crate::write_audit_event(") && finisher.includes('"spreadsheet_import_committed"') && finisher.includes('"match_lineup_import_committed"') && finisher.includes('"ended_previous":result.ended_previous_count'),"完成计数、结束时间与原两类审计必须共用结果和调用方事务");
for (const name of ["skip_import_row_in_tx","resolve_import_row_in_tx"]) {
  const start=ledgerRows.indexOf(`pub(crate) async fn ${name}(`); const next=ledgerRows.indexOf("pub(crate)",start+20); const body=ledgerRows.slice(start,next<0?undefined:next);
  check(body.includes("execute(&mut **tx)") && body.includes("refresh_pending_counts_in_tx(tx, batch_id).await") && body.includes("AND batch_id="),`${name} 必须在行更新后同事务刷新批次计数并限定所属批次`);
}
check(ledgerRows.includes("skipped_count=(SELECT count(*)") && ledgerRows.includes("error_count=(SELECT count(*)") && ledgerRows.includes("WHERE id=$1 AND status='pending'"),"冲突处理后必须刷新 pending 的跳过与阻断计数");
check((matchWorkbook.match(/ledger::lock_batch_in_tx\(/g)??[]).length===3 && matchWorkbook.includes('row_status != "conflict"') && matchWorkbook.includes("该冲突行已被其他操作修改"),"比赛冲突两阶段校验必须重新取得父锁并检查行仍为 conflict");
check(ledgerRead.includes("ORDER BY row_number, sheet_name, id") && ledgerRead.includes("ORDER BY row_number,sheet_name,id") && !ledgerRead.includes(".begin()") && !ledgerRead.includes(".commit()"),"两类只读预览排序与不写入语义缺失");
for (const token of ["未知导入实体", "未知比赛导入实体", "未知导入动作", "未知导入状态", "未知导入模式", "ready_end_previous"]) check(ledgerMapping.includes(token),`严格 codec 契约缺少：${token}`);
for (const token of ["非 pending 球员批次不得提交", "未解决球员冲突不能启动业务写入", "球员冲突跳过与批次暂存计数共同提交", "球员成功返回与账本全部计数一致", "球员成功批次重试不得重复审计", "非 pending 比赛批次不得提交", "未解决比赛冲突不能启动业务写入", "比赛冲突跳过与批次暂存计数共同提交", "候选解决后的行状态与批次计数一致", "末行失败必须回滚替代阵容、前十个球员、账本与审计"]) check(integration.includes(token),`原数据库回归缺少：${token}`);
// R7-11：球员工作簿唯一职责与公开入口、事务及行身份契约。
const playerRoot = "crates/persistence-postgres/src/adapters/workbooks/player_catalog";
const playerPreview = read(`${playerRoot}/preview.rs`);
const playerConflict = read(`${playerRoot}/conflict.rs`);
const playerCommit = read(`${playerRoot}/commit.rs`);
const playerIdentity = read(`${playerRoot}/identity.rs`);
const playerValidation = read(`${playerRoot}/validation.rs`);
const playerValues = read(`${playerRoot}/values.rs`);
const playerExport = read("crates/persistence-postgres/src/adapters/workbooks/monthly_player/read.rs");
check(!fs.existsSync(path.join(root,"crates/persistence-postgres/src",["spreadsheet","exchange.rs"].join("_"))) && !read("crates/persistence-postgres/src/lib.rs").includes("mod spreadsheet_exchange;"),"旧球员工作簿 owner/注册必须删除");
check(read("crates/persistence-postgres/src/adapters/workbooks/mod.rs").includes("mod player_catalog;"),"球员工作簿新职责未注册");
for (const [file,names] of [["preview",["preview_spreadsheet_import","preview_spreadsheet_import_with_team_references","preview_spreadsheet_import_inner"]],["conflict",["resolve_spreadsheet_import_conflict"]],["commit",["commit_spreadsheet_import","apply_import_row"]],["export",["spreadsheet_export_data"]],["identity",["validate_external_id_resolution","decision_from_matches","resolve_player_reference","resolve_team_reference"]],["validation",["validate_spreadsheet_row","validate_child_fields"]],["values",["normalize_spreadsheet_payload","parse_spreadsheet_datetime","spreadsheet_clear_fields"]]]) {
  for (const name of names) {
    const owners=persistencePaths.filter((path)=>new RegExp(`fn\\s+${name}\\s*\\(`).test(read(path)));
    check(owners.length===1 && owners[0]===(file === "export" ? "crates/persistence-postgres/src/adapters/workbooks/monthly_player/read.rs" : `${playerRoot}/${file}.rs`),`${name} 球员职责必须只有一个 owner`);
  }
}
for (const name of ["spreadsheet_export_data","preview_spreadsheet_import","preview_spreadsheet_import_with_team_references","read_spreadsheet_import_preview","resolve_spreadsheet_import_conflict","commit_spreadsheet_import"]) check(new RegExp(`PersistenceStore::${name}\\(\\s*self`).test(spreadsheetAdapter),`球员 Port 必须显式分派唯一持久化入口：${name}`);
for (const source of [playerIdentity,playerValidation,playerValues,playerExport]) check(!source.includes(".begin()") && !source.includes(".commit()") && !source.includes(".execute("),"球员校验/身份/载荷/导出不得持有事务或写业务事实");
check(!playerPreview.includes("football.players") && !playerPreview.includes("football.player_team_periods") && playerPreview.includes("ledger_rows::insert_import_row(") && (playerPreview.match(/row_number:\s*raw.row_number/g)??[]).length===2 && (playerPreview.match(/sheet_name:\s*raw.sheet_name.clone\(\)/g)??[]).length===2,"球员预检仅暂存真实工作表/物理行和原载荷，不写球员事实");
const conflictTokens=["ledger::lock_batch_in_tx(","ledger::require_pending(","ledger_rows::lock_import_row_in_tx(",'row_status != "conflict"',"candidate.entity_id == selected","existing != selected","ledger_rows::resolve_import_row_in_tx(","spreadsheet_import_conflict_resolved","tx.commit().await?"];
let conflictOffset=-1;
for (const token of conflictTokens) {const next=playerConflict.indexOf(token);check(next>conflictOffset,`球员冲突事务顺序缺失：${token}`);conflictOffset=next;}
check((playerConflict.match(/self.pool.begin\(\)/g)??[]).length===1 && (playerConflict.match(/tx.commit\(\)/g)??[]).length===1,"球员冲突行、计数和审计必须共同提交唯一事务");
check(/write_external_entity_id\(\s*tx,[\s\S]*?\.await\?;/.test(playerCommit) && playerCommit.includes("struct ImportCommitContext") && !playerCommit.includes("ON CONFLICT (provider_id, entity_type, external_id) DO UPDATE"),"球员提交必须使用共享身份保护并保留同事务工作簿键解析");
check(playerIdentity.includes("existing_id != target_id") && (playerIdentity.match(/if existing_entity_id.is_some\(\)/g)??[]).length===2 && playerIdentity.includes("该外部 ID 已绑定到另一条数据库记录，禁止自动改绑"),"球员预检不得改绑既有 ID 或转绑工作簿/包内待新增实体");
for (const [source,banned] of [[playerValues,["super::identity","super::validation","super::commit","super::preview"]],[playerIdentity,["super::validation","super::commit","super::preview"]],[playerValidation,["super::commit","super::preview"]]]) for (const dependency of banned) check(!source.includes(dependency),`球员职责反向依赖：${dependency}`);
for (const token of ["同名","external_id_never_rebinds_existing_or_deferred_identity","external_id_same_target_and_new_target_keep_update_and_skip_semantics","child_deferred_reference_keeps_team_key_name_and_other_fields","canonical_action_preserves_child_add_and_explicit_operations"]) check(playerIdentity.includes(token),`球员身份/子记录内联边界回归缺少：${token}`);
for (const token of ["球员预览读回保持行 UUID、工作表、物理行与载荷","球员预览及重复读回不得写入球员、效力期或自动创建球队","球员只读预览不得产生提交审计"]) check(integration.includes(token),`现有球员数据库用例缺少：${token}`);
check(integration.includes("team_package_player_team_period_subrecords_are_distinct") && read("crates/persistence-postgres/tests/entity_matching_references_repository_contract.rs").includes("external_id_identity_and_import_atomicity_are_preserved"),"球员多子记录与外部 ID 整批回滚既有回归必须保留");
if (failures.length) {
  console.error("Exchange Service 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}
console.log("Exchange Service 验证通过：AT1 + AT2 共 24 个公共用例由唯一 ExchangeService 编排，Spreadsheet/Monthly Ports 与 PersistenceStore 适配完整，旧 owners 清零且兼容错误优先级保持。");
