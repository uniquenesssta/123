import fs from "node:fs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const requireTrue = (condition, message) => { if (!condition) failures.push(message); };

const domain = read("crates/domain/src/team_package.rs") + (read("crates/domain/src/lib.rs") + read("crates/domain/src/lineup/kind.rs") + read("crates/domain/src/lineup/player.rs") + read("crates/domain/src/lineup/snapshot.rs") + read("crates/domain/src/lineup/preset.rs") + read("crates/domain/src/lineup/chain.rs") + read("crates/domain/src/match_record/status.rs") + read("crates/domain/src/match_record/catalog.rs"));
const io = read("crates/spreadsheet-io/src/team_package.rs") + read("crates/spreadsheet-io/src/lib.rs");
const application = read("crates/application/src/spreadsheet.rs");
const persistence = read("crates/persistence-postgres/src/monthly_workbooks.rs");
const playerPersistence = read("crates/persistence-postgres/src/spreadsheet_exchange.rs");
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
requireTrue(application.includes("team_package_coverage") && application.includes("p4_input_ready") && application.includes("readiness_score"), "应用层缺少P4输入就绪度检查");
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

if (failures.length) {
  console.error("球队完整资料包契约验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("球队完整资料包契约验证通过：统一导入、P4就绪检查、球队/球员双链分发和前端合并均已锁定。");
