import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { isVersionAtLeast } from "./version.mjs";
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outerRoot = resolve(root, "..");
const failures = [];
const assert = (condition, message) => { if (!condition) failures.push(message); };
const text = (path) => readFileSync(join(root, path), "utf8").replace(/\r\n?/g, "\n");
const json = (path) => JSON.parse(text(path));
const hash = (path) => createHash("sha256").update(text(path), "utf8").digest("hex");

const contractPath = "contracts/entity-relationship-contract.json";
const contract = json(contractPath);
const packageJson = json("package.json");
const tauri = json("src-tauri/tauri.conf.json");
const cargo = text("Cargo.toml");
const cargoLock = text("Cargo.lock");
const migration = text("crates/persistence-postgres/migrations/0021_entity_relationships.sql");
const domain = [
  "crates/domain/src/coach/catalog.rs",
  "crates/domain/src/coach/name.rs",
  "crates/domain/src/coach/membership.rs",
  "crates/domain/src/team/membership.rs",
  "crates/domain/src/shared/entity_reference.rs",
  "crates/domain/src/shared/entity_match.rs",
  "crates/domain/src/shared/bulk_archive.rs",
].map(text).join("\n");
const persistence = text("crates/persistence-postgres/src/entity_catalog.rs");
const teamPersistence = text("crates/persistence-postgres/src/team_catalog.rs");
const playerPersistence = text("crates/persistence-postgres/src/player_catalog.rs");
const application = [
  text("crates/application/src/services/players/facade.rs"),
  text("crates/application/src/services/players/service.rs"),
].join("\n");
const commands = text("src-tauri/src/commands/catalog.rs");
const registry = text("src-tauri/src/bootstrap/command_registry.rs");
const client = text("src/api/client.ts");
const main = text("src/main.ts");
const teamsPage = text("src/pages/teams.ts");
const playersPage = text("src/pages/players.ts");
const types = text("src/types.ts");
const readme = text("README.md");

assert(contract.contract_id === "football.entity-relationship-contract.v1", "阶段2实体关系契约ID错误");
assert(contract.contract_version === "1.0.0", "阶段2实体关系契约版本错误");
assert(contract.baseline_source_version === "0.14.0", "阶段2基线必须是0.14.0");
assert(contract.release_version === "0.15.0", "阶段2发布版本必须是0.15.0");
assert(contract.stage === "H_PRE_STAGE_2", "阶段2不得提前进入接入点H");
assert(isVersionAtLeast(packageJson.version, contract.release_version), "当前项目版本早于阶段2实体关系版本");
assert(tauri.version === packageJson.version, "Tauri当前版本未同步");
assert(cargo.includes(`version = "${packageJson.version}"`), "Cargo workspace版本未同步");
assert(cargoLock.includes(`name = "football-application"\nversion = "${packageJson.version}"`), "Cargo.lock本地包版本未同步");
assert(readme.includes(`当前版本 **${packageJson.version}**`), "根README当前版本未同步");
assert(readme.includes("## 0.15.0 变更记录"), "根README缺少0.15.0变更记录");

for (const artifact of contract.artifacts) assert(existsSync(join(root, artifact)), `阶段2制品不存在：${artifact}`);
const contractHash = hash(contractPath);
assert(migration.includes(`ENTITY_RELATIONSHIP_CONTRACT_SHA256 = ${contractHash}`), "0021迁移顶部契约哈希错误");
assert(migration.includes(`'${contractHash}'`), "0021迁移登记契约哈希错误");
for (const required of [
  "CREATE TABLE football.coaches",
  "CREATE TABLE football.coach_names",
  "CREATE TABLE football.team_coach_periods",
  "refresh_team_head_coach_projection",
  "team_coach_periods_projection_trigger",
  "'coach'",
]) assert(migration.includes(required), `0021迁移缺少：${required}`);
assert(migration.includes("baseline_source_version") || migration.includes("'0.14.0', '0.15.0'"), "0021迁移未登记阶段2版本边界");

for (const command of contract.commands) {
  assert(commands.includes(`fn ${command}`), `Tauri命令缺少${command}`);
  assert(registry.includes(`commands::${command}`), `Tauri注册表缺少${command}`);
  assert(client.includes(`"${command}"`), `前端客户端缺少${command}`);
}
for (const model of [
  "pub struct CoachDraft",
  "pub struct CoachNameRecord",
  "pub struct TeamCoachPeriodRecord",
  "pub struct TeamPlayerPeriodRecord",
  "pub struct EntityReferenceRecord",
  "pub struct EntityMatchResult",
  "pub struct EntityDeletionCheck",
  "pub struct BulkArchiveResult",
]) assert(domain.includes(model), `领域层缺少${model}`);

for (const method of [
  "pub async fn create_coach",
  "pub async fn list_coaches",
  "pub async fn add_team_coach_period",
  "pub async fn list_entity_references",
  "pub async fn resolve_entity_reference",
  "pub async fn check_entity_deletion",
  "pub async fn bulk_archive_entities",
]) {
  assert(persistence.includes(method), `持久化层缺少${method}`);
  assert(application.includes(method), `应用层缺少${method}`);
}
assert(persistence.includes("稳定实体 ID 精确匹配"), "统一匹配缺少稳定ID优先级");
assert(persistence.includes("受信数据源外部 ID 精确匹配"), "统一匹配缺少外部ID优先级");
assert(persistence.includes("status: \"ambiguous\""), "统一匹配缺少歧义阻断结果");
assert(persistence.includes("can_permanently_delete: total == 0"), "永久删除未强制执行引用统计");
assert(persistence.includes("manual_bulk_archive"), "批量归档缺少审计来源");
for (const relation of ["player_availability", "substitutions", "dynamic_tag_opponents"]) assert(persistence.includes(`(\"${relation}\"`), `球队或球员引用检查缺少${relation}`);
assert(teamPersistence.includes("list_team_player_periods") && teamPersistence.includes("list_team_coach_periods"), "球队详情未加载完整球员与教练履历");
assert(teamPersistence.includes("head_coach=football.team_profiles.head_coach") && teamPersistence.includes(".bind(None::<&str>)"), "球队档案写入仍可能覆盖教练任期投影");
assert(teamPersistence.includes("check_entity_deletion(\"team\""), "球队永久删除未接入统一引用检查");
assert(!teamPersistence.includes("DELETE FROM football.player_team_periods WHERE team_id=$1"), "球队永久删除仍会主动清理球员履历");
assert(!teamPersistence.includes("UPDATE football.player_availability SET team_id=NULL WHERE team_id=$1"), "球队永久删除仍会改写历史可用性记录");
assert(playerPersistence.includes("check_entity_deletion(\"player\""), "球员永久删除未接入统一引用检查");
assert(!playerPersistence.includes("DELETE FROM football.player_team_periods WHERE player_id=$1"), "球员永久删除仍会主动清理历史球队履历");

assert(types.includes("interface CoachListItem") && types.includes("interface TeamCoachPeriodRecord"), "前端缺少教练和任期类型");
assert(types.includes("interface PlayerTeamPeriodRecord"), "前端球员详情未类型化完整球队履历");
assert(teamsPage.includes("教练与任期历史"), "球队中心缺少教练任期历史");
assert(teamsPage.includes("完整球员效力履历"), "球队中心缺少完整球员履历");
assert(teamsPage.includes("由当前教练任期自动生成"), "主教练字段未标记为兼容投影");
assert(teamsPage.includes("bulk-archive-teams"), "球队中心缺少批量归档");
assert(playersPage.includes("history-list\">${teamHistory}"), "球员中心缺少完整球队履历显示");
assert(playersPage.includes("bulk-archive-players"), "球员中心缺少批量归档");
assert(main.includes("api.checkEntityDeletion(\"player\""), "球员永久删除前未执行引用检查");
assert(main.includes("api.bulkArchiveEntities"), "前端未接通批量归档服务");
assert(main.includes("addTeamCoachPeriod"), "前端未接通教练任期保存");
assert(main.includes("head_coach: selectedTeam?.profile?.head_coach ?? null"), "球队档案保存可能覆盖主教练投影");

assert(contract.coach_model.history_preserved === true, "契约未锁定教练历史保留");
assert(contract.coach_model.current_head_coach_is_projection === true, "契约未锁定主教练投影");
assert(contract.entity_matching.ambiguous_match_blocks_automatic_binding === true, "契约未锁定歧义阻断");
assert(contract.lifecycle.referenced_entities_must_archive === true, "契约未锁定引用实体只能归档");
assert(contract.lifecycle.historical_rows_are_not_cascade_deleted === true, "契约未锁定历史关系禁止级联删除");

if (failures.length) {
  console.error("阶段2实体关系验证失败：");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}
console.log("阶段2实体关系验证通过：教练历史、统一匹配、引用检查、批量归档和双向履历均已锁定。");
