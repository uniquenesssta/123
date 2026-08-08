import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const contract = JSON.parse(read("contracts/team-lineup-presets-contract.json"));
const migration = read("crates/persistence-postgres/migrations/0045_team_lineup_presets.sql");
const domain = (read("crates/domain/src/lib.rs") + read("crates/domain/src/lineup/kind.rs") + read("crates/domain/src/lineup/player.rs") + read("crates/domain/src/lineup/snapshot.rs") + read("crates/domain/src/lineup/preset.rs") + read("crates/domain/src/lineup/chain.rs") + read("crates/domain/src/match_record/status.rs") + read("crates/domain/src/match_record/catalog.rs"));
const persistence = read("crates/persistence-postgres/src/team_lineup_presets.rs");
const application = read("crates/application/src/services/lineups/facade.rs");
const commands = read("src-tauri/src/commands/catalog.rs");
const commandRegistry = read("src-tauri/src/bootstrap/command_registry.rs");
const api = read("src/api/client.ts");
const types = read("src/types.ts");
const main = read("src/main.ts");
const teams = read("src/pages/teams.ts");
const lineups = read("src/pages/lineups.ts");
const searchable = read("src/components/searchableSelect.ts");
const css = read("src/styles/components.css");
const entityCatalog = read("crates/persistence-postgres/src/entity_catalog.rs");
const forceDelete = read("crates/persistence-postgres/src/team_force_delete.rs");

check(contract.format_version === "football.team-lineup-presets.v1", "E2 契约版本错误");
for (const token of [
  "CREATE TABLE IF NOT EXISTS football.team_lineup_presets",
  "CREATE TABLE IF NOT EXISTS football.team_lineup_preset_members",
  "team_lineup_presets_active_name_uq",
  "team_lineup_presets_one_default_uq",
  "PRIMARY KEY (preset_id, player_id)",
]) check(migration.includes(token), `阵容预设迁移缺少：${token}`);

for (const token of [
  "pub struct TeamLineupPresetDraft",
  "pub struct TeamLineupPresetRecord",
  "pub struct TeamLineupPresetApplicationPreview",
]) check(domain.includes(token), `领域契约缺少：${token}`);
for (const token of [
  "save_team_lineup_preset",
  "list_team_lineup_presets",
  "preview_team_lineup_preset_application",
  "duplicate_team_lineup_preset",
  "archive_team_lineup_preset",
  "delete_team_lineup_preset",
  "team_lineup_preset_deleted",
  "starter_count != 11",
  "verify_membership_in_tx",
  "preset_requires_exactly_eleven_starters",
  "preset_rejects_duplicate_players",
]) check(persistence.includes(token), `持久化能力缺少：${token}`);

for (const token of ["save_team_lineup_preset", "list_team_lineup_presets", "preview_team_lineup_preset_application", "duplicate_team_lineup_preset", "archive_team_lineup_preset", "delete_team_lineup_preset"]) {
  check(application.includes(token), `应用服务缺少：${token}`);
  check(commands.includes(token), `Tauri 命令缺少：${token}`);
  check(commandRegistry.includes(`commands::${token}`), `命令注册缺少：${token}`);
}
for (const token of ["TeamLineupPresetDraft", "TeamLineupPresetRecord", "TeamLineupPresetApplicationPreview"]) {
  check(types.includes(token), `TypeScript 类型缺少：${token}`);
}
for (const token of ["saveTeamLineupPreset", "listTeamLineupPresets", "previewTeamLineupPresetApplication", "duplicateTeamLineupPreset", "archiveTeamLineupPreset", "deleteTeamLineupPreset"]) {
  check(api.includes(token), `前端 API 缺少：${token}`);
}
for (const token of [
  "openTeamLineupPresetEditor",
  "saveCurrentLineupAsPreset",
  "previewApplyLineupPreset",
  "applyLineupPreset",
  "本场阵容可继续独立调整",
  "openTeamLineupPresetManager",
  "requestDeleteTeamLineupPreset",
  "永久删除阵容预设",
]) check(main.includes(token), `前端工作流缺少：${token}`);
check(teams.includes("常用阵容预设") && teams.includes("open-team-lineup-preset-editor"), "球队完整档案缺少阵容预设管理入口");
check(teams.includes("open-team-lineup-preset-manager") && teams.includes("管理 / 删除预设"), "球队中心缺少显式预设管理与删除入口");
check(lineups.includes("应用已保存阵容") && lineups.includes("保存当前阵容"), "比赛双方阵容缺少预设套用或保存入口");
check(lineups.includes("open-lineup-preset-manager") && lineups.includes("管理预设"), "比赛双方阵容缺少预设管理入口");
check(lineups.includes("preview-apply-lineup-preset"), "比赛阵容没有应用前预检");
for (const token of [".lineup-preset-card", ".lineup-preset-quickbar", ".lineup-preset-manager", ".lineup-preset-manager-row", ".preset-member-list", ".lineup-preset-preview-members"]) {
  check(css.includes(token), `阵容预设 UI 缺少：${token}`);
}

check(searchable.includes("preserveDraft"), "可搜索选择器未保护活动查询");
check(searchable.includes("startFreshQuery"), "可搜索选择器未区分新搜索与恢复搜索");
check(searchable.includes('input.addEventListener("beforeinput"') && searchable.includes('event.inputType.startsWith("insert")'), "首次输入没有替换选中标签，仍可能产生回弹感");
check(!searchable.includes("input.select()"), "恢复搜索词后仍存在全选覆盖风险");
check(searchable.includes("input.setSelectionRange(caret, caret)"), "恢复搜索词没有将光标折叠到末尾");
check(entityCatalog.includes('("team_lineup_presets", "SELECT count(*)::bigint FROM football.team_lineup_presets WHERE team_id=$1")'), "球队安全删除没有识别阵容预设引用");
check(entityCatalog.includes('("team_lineup_preset_members", "SELECT count(*)::bigint FROM football.team_lineup_preset_members WHERE player_id=$1")'), "球员安全删除没有识别阵容预设成员引用");
check(forceDelete.includes("DELETE FROM football.team_lineup_preset_members") && forceDelete.includes("DELETE FROM football.team_lineup_presets"), "球队强制删除没有清理阵容预设引用");

if (failures.length) {
  console.error("阶段 E2 阵容预设验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("阶段 E2 阵容预设验证通过：球队与比赛管理入口、活动/归档预设永久删除、比赛保存/预检/套用、11 人门禁、独立比赛草稿与搜索防回弹均已锁定。");
