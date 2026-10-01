import fs from "node:fs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const requireTrue = (condition, message) => { if (!condition) failures.push(message); };

const domain = read("crates/domain/src/exchange/team_package/export.rs") + read("crates/domain/src/exchange/team_package/import.rs") + (read("crates/domain/src/lib.rs") + read("crates/domain/src/lineup/kind.rs") + read("crates/domain/src/lineup/player.rs") + read("crates/domain/src/lineup/snapshot.rs") + read("crates/domain/src/lineup/preset.rs") + read("crates/domain/src/lineup/chain.rs") + read("crates/domain/src/match_record/status.rs") + read("crates/domain/src/match_record/catalog.rs"));
const io = read("crates/spreadsheet-io/src/team_package.rs") + read("crates/spreadsheet-io/src/lib.rs");
const application =
  read("crates/application/src/services/exchange/facade/spreadsheet.rs") +
  read("crates/application/src/use_cases/exchange/preview_team_package_import/use_case.rs") +
  read("crates/application/src/use_cases/exchange/preview_team_package_import/coverage.rs") +
  read("crates/application/src/use_cases/exchange/commit_team_package_import/policy.rs") +
  read("crates/application/src/use_cases/exchange/export_team_package_preview_json/use_case.rs");
const persistence = ["mod", "preview", "conflict", "commit", "write", "names", "identity", "formation", "validation", "values"].map((name) => read(`crates/persistence-postgres/src/adapters/workbooks/team_package/${name}.rs`)).join("\n") + read("crates/persistence-postgres/src/adapters/workbooks/identity/teams.rs");
const playerPersistence = ["preview", "conflict", "commit", "export", "identity", "validation", "values"]
  .map((name) => read(name === "export" ? "crates/persistence-postgres/src/adapters/workbooks/monthly_player/read.rs" : `crates/persistence-postgres/src/adapters/workbooks/player_catalog/${name}.rs`)).join("\n");
const commands = read("src-tauri/src/commands/exchange.rs");
const registry = read("src-tauri/src/bootstrap/command_registry.rs");
const client = read("src/api/client.ts");
const types = read("src/types.ts");
const teams = read("src/pages/teams.ts");
const players = read("src/pages/players.ts");
const shell = read("src/app/shell.ts");
const navigation = read("src/app/navigation.ts");
const main = read("src/main.ts");
const styles = read("src/styles/app.css");
const readme = fs.readFileSync(new URL("../README.md", import.meta.url), "utf8");

requireTrue(domain.includes('TEAM_PACKAGE_FORMAT: &str = "football.team-package.v1"'), "缺少球队完整资料包稳定格式版本");
requireTrue(domain.includes("TeamPackageCoverage") && domain.includes("TeamPackageImportPreview") && domain.includes("TeamPackageCommitResult"), "领域层缺少统一资料包预检或提交类型");
for (const sheet of ["说明与校验", "球队总览", "球队名称", "球员与评分", "球员名称", "教练与阵型", "字段字典"]) {
  requireTrue(io.includes(sheet), `资料包缺少工作表：${sheet}`);
}
for (const input of ["team_attack_rating", "ability_attack", "tag_match_readiness", "tag_realization_multiplier", "formation_familiarity"]) {
  requireTrue(io.includes(input), `资料包缺少P4输入字段：${input}`);
}
for (const input of ["club_team_key", "club_team_name", "club_country_code", "club_registration_status", "club_valid_from", "club_valid_to"]) {
  requireTrue(io.includes(input), `资料包缺少俱乐部关系字段：${input}`);
}
requireTrue(io.includes("emitted_club_teams") && io.includes('Value::String("club".into())') && io.includes("club_period"), "资料包未把俱乐部主体和球员俱乐部关系分发到统一导入链");
requireTrue(io.includes('club.insert("team_key"') && io.includes('("club_team_key", "team_key")'), "俱乐部临时键未贯通球队主体与球员关系");
requireTrue(io.includes("derived_valid_from") && io.includes("TeamCoachPeriod") && io.includes('Value::String("head_coach".into())'), "教练表未从观察/核验时间补齐球队任期关系");
requireTrue(io.includes("coaches.insert(coach_identity.clone())"), "教练去重键被移动后仍会再次借用，Rust编译所有权保护缺失");
requireTrue(io.includes("DataValidation") && io.includes("set_freeze_panes") && io.includes("write_group_row") && io.includes("key_label") && io.includes("machine_key_format"), "资料包模板缺少中文字段、稳定键或可用性设计");
requireTrue(io.includes("normalized_rating_text") && io.includes("10_000.0") && io.includes("value / 100.0"), "资料包未兼容参考评分表的0–10000评分制");
requireTrue(io.includes("read_team_package_workbook") && io.includes("cell_text_for_key") && io.includes("Data::DateTime"), "资料包读取器缺少统一解析或Excel日期兼容");
requireTrue(io.includes('"upsert" | "merge" | "add_or_update" | "insert_or_update"') && io.includes("SpreadsheetAction::Upsert"), "资料包读取器未兼容 upsert 及常见同义动作");
requireTrue(io.includes('留空等同 upsert') && io.includes('&["upsert", "add", "update", "clear", "skip"]'), "资料包模板未将 upsert 设为可直接使用的推荐动作");
requireTrue(persistence.includes("canonical_team_import_action") && persistence.includes("upsert 已自动转换为 update"), "球队链未在预检后把 upsert 规范为 add/update");
requireTrue(playerPersistence.includes("canonical_spreadsheet_import_action") && playerPersistence.includes("SpreadsheetAction::Upsert"), "球员链未在预检后把 upsert 规范为 add/update");
requireTrue(application.includes("preview_team_package_import") && application.includes("TEAM_MONTHLY_FORMAT") && application.includes("PLAYER_MONTHLY_FORMAT"), "应用层未把资料包分发到现有球队与球员链路");
requireTrue(application.includes("coverage::calculate") && application.includes("p4_input_ready") && application.includes("readiness_score"), "应用层缺少P4输入就绪度检查");
requireTrue(application.includes("commit_team_package_import") && application.includes("ensure_preview_committable"), "统一提交未复用现有预检门禁");
requireTrue(persistence.includes('"formation_familiarity"'), "阵型熟悉度未保留到持久化元数据");
for (const command of ["export_team_package_template", "export_team_package_preview_json", "preview_team_package_import", "commit_team_package_import"]) {
  requireTrue(commands.includes(`fn ${command}`), `Tauri命令缺失：${command}`);
  requireTrue(registry.includes(`commands::${command}`), `Tauri命令未注册：${command}`);
  requireTrue(client.includes(`"${command}"`), `前端API缺失：${command}`);
}
requireTrue(types.includes("TeamPackageImportPreview") && types.includes("TeamPackageCoverage"), "前端类型缺少资料包覆盖率");
requireTrue(navigation.includes('key: "resources"') && navigation.includes('page: "teams"') && navigation.includes('label: "球队中心"') && navigation.includes('page: "players"') && navigation.includes('label: "球员中心"'), "球队与球员未统一归入资源一级模块");
requireTrue(teams.includes("统一导入入口") && teams.includes("P4 输入就绪度") && teams.includes("preview-team-package-import"), "球队与人员前端缺少导入优先工作区");
requireTrue(players.includes("球队与人员") && players.includes("球员浏览与管理") && players.includes("core-player-workspace") && players.includes("球员工作包"), "球员目录未纳入统一资源中心");
requireTrue(main.includes("previewTeamPackageImport") && main.includes("resolveTeamPackageConflict") && main.includes("commitTeamPackageImport"), "主控制器缺少资料包预检、冲突和提交链");
requireTrue(application.includes("TEAM_PACKAGE_PREVIEW_EXPORT_FORMAT") && application.includes("serde_json::to_vec_pretty") && application.includes("exported_row_count"), "完整预检 JSON 导出未保留全部预检记录或稳定格式");
requireTrue(teams.includes("导出完整预检 JSON") && main.includes("exportTeamPackagePreviewJson") && client.includes("chooseJsonExportFile"), "完整预检页面缺少 JSON 导出入口");
requireTrue(styles.includes(".team-package-entry-grid") && styles.includes(".team-package-readiness"), "统一资料包前端缺少专用响应式样式");
requireTrue(readme.includes("球队完整资料包") && readme.includes("P4 输入就绪度"), "README未记录本次统一资料包增强");
requireTrue(readme.includes("1248") && readme.includes("俱乐部关系增量补录"), "README未记录世界杯球员俱乐部关系补录");


// R7-12：真实共享球队链的唯一职责与原双链恢复边界。
const teamRoot = "crates/persistence-postgres/src/adapters/workbooks/team_package";
const teamFiles = ["mod", "preview", "conflict", "commit", "write", "names", "identity", "formation", "validation", "values"];
const sources = Object.fromEntries(teamFiles.map((name) => [name, read(`${teamRoot}/${name}.rs`)]));
const ledgerRoot = "crates/persistence-postgres/src/adapters/workbooks/batch_ledger";
const ledger = Object.fromEntries(["batch", "rows", "read", "mapping"].map((name) => [name, read(`${ledgerRoot}/${name}.rs`)]));
const persistenceFiles = [];
const collectRust = (directory) => {
  for (const item of fs.readdirSync(new URL(`../${directory}/`, import.meta.url), {withFileTypes:true})) {
    const path = `${directory}/${item.name}`;
    if (item.isDirectory()) collectRust(path);
    else if (item.name.endsWith(".rs")) persistenceFiles.push(path);
  }
};
collectRust("crates/persistence-postgres/src");
for (const [owner,names] of [
  [`${teamRoot}/preview.rs`,["preview_team_monthly_import"]],
  [`${teamRoot}/conflict.rs`,["resolve_team_monthly_import_conflict"]],
  [`${teamRoot}/commit.rs`,["commit_team_monthly_import"]],
  [`${teamRoot}/write.rs`,["execute_team_monthly_row","apply_team_update","upsert_team_profile_from_values"]],
  [`${teamRoot}/names.rs`,["ensure_team_name_alias","preserve_current_team_canonical_alias"]],
  ["crates/persistence-postgres/src/adapters/workbooks/identity/teams.rs",["consolidate_duplicate_ready_add_team_rows","consolidate_duplicate_ready_add_team_rows_by_source"]],
  [`${teamRoot}/identity.rs`,["bind_batch_team_references"]],
  [`${ledgerRoot}/read.rs`,["read_team_monthly_import_preview"]],
  [`${ledgerRoot}/mapping.rs`,["team_import_row_from_db","team_parse_import_mode","team_parse_status","team_parse_action","team_parse_entity_type"]],
  [`${ledgerRoot}/batch.rs`,["find_team_batch","create_team_batch_in_tx"]],
  [`${ledgerRoot}/rows.rs`,["set_import_payload_in_tx","skip_duplicate_import_row_in_tx"]],
]) {
  for (const name of names) {
    const owners = persistenceFiles.filter((path) => new RegExp(`\\bfn\\s+${name}\\s*\\(`).test(read(path)));
    requireTrue(owners.length === 1 && owners[0] === owner, `${name} 必须只有一个实际 owner`);
  }
}
const adapter = read("crates/application/src/composition/adapters/exchange/spreadsheet.rs");
for (const method of ["preview_team_monthly_import","read_team_monthly_import_preview","resolve_team_monthly_import_conflict","commit_team_monthly_import"]) requireTrue(adapter.includes(`PersistenceStore::${method}(`), `${method} 的原 MonthlyWorkbookPort 必须显式分派`);
requireTrue(read("crates/persistence-postgres/src/adapters/workbooks/mod.rs").includes("mod team_package;"),"球队资料包唯一模块未登记");
requireTrue(!persistence.includes("catalog.import_batches") && !persistence.includes("catalog.import_rows"),"球队资料包业务职责不得保存第二份批次/行 SQL");
for (const source of [sources.validation,sources.preview,sources.names,sources.write,sources.identity,read("crates/persistence-postgres/src/adapters/workbooks/identity/teams.rs"),sources.formation,ledger.batch,ledger.rows]) {
  if (source !== sources.preview) requireTrue(!source.includes(".begin()") && !source.includes(".commit()"),"球队步骤必须复用调用方事务，不得另开或提交");
}
requireTrue(!/\b(?:INSERT INTO|UPDATE|DELETE FROM)\s+(?:football|feature)\./.test(sources.preview+sources.validation),"球队预检不得写业务事实");
requireTrue((sources.preview.match(/row_number: raw.row_number/g)??[]).length === 2 && (sources.preview.match(/sheet_name: raw.sheet_name.clone\(\)/g)??[]).length === 2,"球队预检必须保留工作表与物理行身份");
requireTrue(sources.preview.includes("ledger::find_team_batch(") && sources.preview.includes("ledger::create_team_batch_in_tx(") && sources.preview.includes("ledger_rows::insert_import_row("),"同源球队批次复用及预检暂存链缺失");
requireTrue(ledger.batch.includes('Self::Team => vec!["team_monthly_xlsx"]') && ledger.batch.includes("import_type=ANY($2) FOR UPDATE"),"球队批次父锁必须隔离其他导入类型");
const commit = sources.commit;
const ordered = ["self.pool.begin()","ledger::lock_batch_in_tx(",'batch_status != "pending"',"ledger_rows::blocking_rows_in_tx(","ledger::start_batch_in_tx(","ledger_rows::commit_rows_in_tx(","normalize_monthly_datetime_payload(","consolidate_duplicate_ready_add_team_rows(","consolidate_duplicate_ready_add_team_rows_by_source(","bind_batch_team_references(","execute_team_monthly_row(","ledger_rows::mark_imported_in_tx(","execute_formation_groups(","ledger::finish_batch_in_tx(","tx.commit().await?"];
const offsets = ordered.map((token) => commit.indexOf(token));
requireTrue(offsets.every((value,index) => value >= 0 && (index === 0 || value > offsets[index-1])) && (commit.match(/self\.pool\.begin\(\)/g)??[]).length === 1 && (commit.match(/tx\.commit\(\)/g)??[]).length === 1,"球队提交必须依次锁定、校验、规范化、合并、关联、写事实/行/结果审计后唯一提交");
requireTrue(commit.includes('batch_status == "succeeded"') && commit.includes('batch.try_get::<i64, _>("ended_previous_count")? as u64') && commit.includes("Ok(result)"),"球队成功重试必须返回原计数及原完成时间");
requireTrue(ledger.rows.includes("WHEN 'team' THEN 0 WHEN 'coach' THEN 1 WHEN 'team_name' THEN 2") && ledger.rows.includes("row_number,id FOR UPDATE"),"球队提交行锁及原依赖/物理行顺序缺失");
const conflict = sources.conflict;
requireTrue(conflict.indexOf("ledger::lock_batch_in_tx(") < conflict.indexOf("ledger_rows::lock_import_row_in_tx(") && conflict.includes('status != "pending"') && conflict.includes('!= "conflict"') && conflict.includes("candidate.entity_id == selected") && conflict.includes('object.remove("_conflict_prefix")'),"球队裁决必须父锁先行、状态/行/候选校验并移除临时冲突标记");
for (const method of ["skip_import_row_in_tx","resolve_import_row_in_tx"]) {
  requireTrue(conflict.includes(`ledger_rows::${method}(`),"球队裁决必须复用共同事务计数更新");
  const body = ledger.rows.slice(ledger.rows.indexOf(`async fn ${method}(`));
  const end = body.indexOf("pub(crate)",20);
  const fn = end < 0 ? body : body.slice(0,end);
  requireTrue(fn.includes("refresh_pending_counts_in_tx(tx, batch_id).await") && fn.includes("AND batch_id="), `${method} 必须限定批次并共同刷新 pending 计数`);
}
requireTrue(sources.write.indexOf("preserve_current_team_canonical_alias(tx, team_id") < sources.write.indexOf("UPDATE football.teams SET canonical_name=$2"),"主显示名覆盖前必须保留原正式名别名");
const packageCommit = read("crates/application/src/use_cases/exchange/commit_team_package_import/use_case.rs");
requireTrue(packageCommit.indexOf('ensure_preview_committable(&preview, "球员') < packageCommit.indexOf("MonthlyWorkbookPort::commit_import(") && packageCommit.indexOf("MonthlyWorkbookPort::commit_import(") < packageCommit.indexOf("SpreadsheetExchangePort::commit_import("),"资料包须先预检双链，再按球队/球员顺序提交");
requireTrue(packageCommit.includes("球队、教练与阵型链已经提交成功；可修复后直接重试同一完整资料包批次"),"原球队成功/球员失败恢复边界必须保留");
for (const field of ["inserted_count","updated_count","ended_previous_count","skipped_count","error_count"]) requireTrue(packageCommit.includes(`.map(|value| value.${field})`),`资料包汇总缺少原双链计数 ${field}`);
const pg = read("crates/persistence-postgres/tests/postgres_integration.rs");
for (const token of ["team_preview_again","team_facts_after_failure","team_batch_after_failure","team_audits_after_failure","localized_name","skipped_counts","selected_counts"]) requireTrue(pg.includes(token),`原月度 PG 夹具缺少球队资料包边界断言：${token}`);

if (failures.length) {
  console.error("球队完整资料包契约验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("球队完整资料包契约验证通过：统一导入、P4就绪检查、球队/球员双链分发和前端合并均已锁定。");
