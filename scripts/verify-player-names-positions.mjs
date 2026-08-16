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

const playersMod = read("crates/persistence-postgres/src/adapters/catalog/players/mod.rs");
const namesMod = read("crates/persistence-postgres/src/adapters/catalog/players/names/mod.rs");
const positionsMod = read("crates/persistence-postgres/src/adapters/catalog/players/positions/mod.rs");
const namesWrite = requireTokens("crates/persistence-postgres/src/adapters/catalog/players/names/add_player_name.rs", [
  "pub async fn add_player_name",
  "UPDATE football.player_names SET is_primary = false",
  "UPDATE football.players",
  "INSERT INTO football.player_names",
  "query_as::<_, PlayerNameRow>",
  "tx.commit().await?",
], "Player Names write owner");
const positionsWrite = requireTokens("crates/persistence-postgres/src/adapters/catalog/players/positions/assign_player_position.rs", [
  "pub async fn assign_player_position",
  "UPDATE football.player_positions SET is_primary = false",
  "INSERT INTO football.player_positions",
  "JOIN football.positions",
  "query_as::<_, PlayerPositionRow>",
  "tx.commit().await?",
], "Player Positions write owner");
const legacy = read("crates/persistence-postgres/src/player_catalog.rs");
const namesRead = read("crates/persistence-postgres/src/adapters/catalog/players/detail/names/read.rs");
const positionsRead = read("crates/persistence-postgres/src/adapters/catalog/players/detail/positions/read.rs");
const port = read("crates/application/src/ports/player/mod.rs");
const adapter = read("crates/application/src/composition/adapters/players.rs");

check(playersMod.includes("mod names;") && playersMod.includes("mod positions;"), "players 模块未注册 Names/Positions owner");
check(namesMod.includes("mod add_player_name;") && namesMod.includes("mod input_policy;") && namesMod.includes("mod mapper;") && namesMod.includes("mod row;"), "Names 目录职责拆分不完整");
check(positionsMod.includes("mod assign_player_position;") && positionsMod.includes("mod input_policy;") && positionsMod.includes("mod mapper;") && positionsMod.includes("mod row;"), "Positions 目录职责拆分不完整");
check(!namesMod.includes("sqlx::") && !namesMod.includes("impl PostgresStore"), "Names mod.rs 不得承载 SQL 或业务实现");
check(!positionsMod.includes("sqlx::") && !positionsMod.includes("impl PostgresStore"), "Positions mod.rs 不得承载 SQL 或业务实现");
check(namesWrite.includes("normalize_name(validated.name)"), "Player Names 未复用唯一名称规范化 owner");
check(positionsWrite.includes("validate_player_position_draft(draft)?"), "Player Positions 未经过独立输入策略");
check(!legacy.includes("pub async fn add_player_name"), "legacy player_catalog.rs 仍拥有 add_player_name");
check(!legacy.includes("pub async fn assign_player_position"), "legacy player_catalog.rs 仍拥有 assign_player_position");
check(!legacy.includes("fn player_name_from_row"), "legacy player_catalog.rs 仍拥有 PlayerName Row mapper");
check(!legacy.includes("fn player_position_from_row"), "legacy player_catalog.rs 仍拥有 PlayerPosition Row mapper");
check(namesRead.includes("players::names::{map_player_name, PlayerNameRow}"), "Player Detail Names read 未复用新 Names mapper/row owner");
check(positionsRead.includes("players::positions::{map_player_position, PlayerPositionRow}"), "Player Detail Positions read 未复用新 Positions mapper/row owner");
for (const obsolete of [
  "crates/persistence-postgres/src/adapters/catalog/players/detail/names/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/players/detail/names/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/players/detail/positions/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/players/detail/positions/row.rs",
]) check(!exists(obsolete), `旧 Detail 映射 owner 未删除：${obsolete}`);
check(port.includes("async fn add_player_name(&self, draft: &PlayerNameDraft) -> PortResult<PlayerNameRecord>;"), "PlayerCatalogPort add_player_name 公共契约变化");
check(port.includes("async fn assign_player_position("), "PlayerCatalogPort assign_player_position 公共契约变化");
check(adapter.includes("self.add_player_name(draft)") && adapter.includes("self.assign_player_position(draft)"), "Application persistence adapter 未保持既有调用语义");
check(exists("crates/persistence-postgres/tests/player_names_positions_repository_contract.rs"), "R6-04 PostgreSQL contract test 缺失");

if (failures.length) {
  console.error("R6-04 Player Names / Positions ownership verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R6-04 Player Names / Positions ownership verified: write owners, typed rows/mappers, detail reads and public ports remain uniquely bounded.");
