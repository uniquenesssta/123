import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const exists = (file) => fs.existsSync(path.join(root, file));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const requireTokens = (file, tokens, label = file) => {
  const content = read(file);
  for (const token of tokens) check(content.includes(token), `${label} 缺少：${token}`);
  return content;
};

const required = [
  "crates/persistence-postgres/src/adapters/catalog/players/team_periods/add_player_team_period.rs",
  "crates/persistence-postgres/src/adapters/catalog/players/team_periods/input_policy.rs",
  "crates/persistence-postgres/src/adapters/catalog/players/team_periods/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/players/team_periods/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/players/team_periods/mod.rs",
  "crates/persistence-postgres/src/adapters/catalog/availability/add_player_availability.rs",
  "crates/persistence-postgres/src/adapters/catalog/availability/input_policy.rs",
  "crates/persistence-postgres/src/adapters/catalog/availability/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/availability/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/availability/mod.rs",
  "crates/persistence-postgres/tests/player_team_periods_availability_repository_contract.rs",
];
for (const file of required) check(exists(file), `R6-05 required file missing: ${file}`);

const legacy = read("crates/persistence-postgres/src/player_catalog.rs");
const playersMod = read("crates/persistence-postgres/src/adapters/catalog/players/mod.rs");
const catalogMod = read("crates/persistence-postgres/src/adapters/catalog/mod.rs");
const teamPeriodsMod = read("crates/persistence-postgres/src/adapters/catalog/players/team_periods/mod.rs");
const availabilityMod = read("crates/persistence-postgres/src/adapters/catalog/availability/mod.rs");
const teamPeriodsWrite = requireTokens("crates/persistence-postgres/src/adapters/catalog/players/team_periods/add_player_team_period.rs", [
  "pub async fn add_player_team_period",
  "validate_player_team_period_draft(draft)?",
  "INSERT INTO football.player_team_periods",
  "query_as::<_, PlayerTeamPeriodRow>",
  "JOIN football.teams",
  "LEFT JOIN football.seasons",
], "Team Period write owner");
const availabilityWrite = requireTokens("crates/persistence-postgres/src/adapters/catalog/availability/add_player_availability.rs", [
  "pub async fn add_player_availability",
  "validate_player_availability_draft(draft)?",
  "INSERT INTO football.player_availability",
  "query_as::<_, PlayerAvailabilityRow>",
  "LEFT JOIN football.teams",
], "Availability write owner");
const teamPeriodsRead = read("crates/persistence-postgres/src/adapters/catalog/players/detail/team_periods/read.rs");
const availabilityRead = read("crates/persistence-postgres/src/adapters/catalog/players/detail/availability/read.rs");
const port = read("crates/application/src/ports/player/mod.rs");
const adapter = read("crates/application/src/composition/adapters/players.rs");

check(playersMod.includes("mod team_periods;"), "players 模块未注册 Team Periods owner");
check(catalogMod.includes("mod availability;"), "catalog 模块未注册 Availability owner");
check(teamPeriodsMod.includes("mod add_player_team_period;") && teamPeriodsMod.includes("mod input_policy;") && teamPeriodsMod.includes("mod mapper;") && teamPeriodsMod.includes("mod row;"), "Team Periods 目录职责拆分不完整");
check(availabilityMod.includes("mod add_player_availability;") && availabilityMod.includes("mod input_policy;") && availabilityMod.includes("mod mapper;") && availabilityMod.includes("mod row;"), "Availability 目录职责拆分不完整");
check(!teamPeriodsMod.includes("sqlx::") && !teamPeriodsMod.includes("impl PostgresStore"), "Team Periods mod.rs 不得承载 SQL 或业务实现");
check(!availabilityMod.includes("sqlx::") && !availabilityMod.includes("impl PostgresStore"), "Availability mod.rs 不得承载 SQL 或业务实现");
check(teamPeriodsWrite.includes("validated.registration_status"), "Team Period 写 owner 未复用独立输入策略");
check(availabilityWrite.includes("validated.reason"), "Availability 写 owner 未复用独立输入策略");
check(!legacy.includes("pub async fn add_player_team_period"), "legacy player_catalog.rs 仍拥有 add_player_team_period");
check(!legacy.includes("pub async fn add_player_availability"), "legacy player_catalog.rs 仍拥有 add_player_availability");
check(!legacy.includes("fn player_team_period_from_row"), "legacy player_catalog.rs 仍拥有 Team Period Row mapper");
check(!legacy.includes("fn player_availability_from_row"), "legacy player_catalog.rs 仍拥有 Availability Row mapper");
check(teamPeriodsRead.includes("players::team_periods::{map_player_team_period, PlayerTeamPeriodRow}"), "Player Detail Team Period read 未复用新 Row/mapper owner");
check(availabilityRead.includes("catalog::availability::{map_player_availability, PlayerAvailabilityRow}"), "Player Detail Availability read 未复用新 Row/mapper owner");
for (const obsolete of [
  "crates/persistence-postgres/src/adapters/catalog/players/detail/team_periods/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/players/detail/team_periods/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/players/detail/availability/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/players/detail/availability/row.rs",
]) check(!exists(obsolete), `旧 Detail 映射 owner 未删除：${obsolete}`);
check(port.includes("async fn add_player_team_period("), "PlayerCatalogPort add_player_team_period 公共契约变化");
check(port.includes("async fn add_availability("), "PlayerSignalPort add_availability 公共契约变化");
check(adapter.includes("self.add_player_team_period(draft)") && adapter.includes("self.add_player_availability(draft)"), "Application persistence adapter 未保持既有调用语义");
check(legacy.includes("pub async fn add_player_ability_observation"), "R6-06 Ability owner 被提前迁移或删除");
const dynamicTags = read("crates/persistence-postgres/src/dynamic_tags.rs");
check(dynamicTags.includes("pub async fn add_player_dynamic_tag"), "R6-06 Dynamic Tag owner 被提前迁移或删除");

if (failures.length) {
  console.error("R6-05 Team Periods / Availability ownership verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R6-05 Team Periods / Availability ownership verified: write owners, input policies, typed rows/mappers, detail reads and public ports remain uniquely bounded.");
