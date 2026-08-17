import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const exists = (file) => fs.existsSync(path.join(root, file));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const required = [
  "crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/list.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/observations/add.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/observations/input_policy.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/observations/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/observations/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/list.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/add.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/input_policy.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/list.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/read.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/mapper.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/row.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/calculate.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/scoring.rs",
  "crates/persistence-postgres/tests/player_abilities_dynamic_tags_repository_contract.rs",
];
for (const file of required) check(exists(file), `R6-06 required file missing: ${file}`);
const legacy = read("crates/persistence-postgres/src/player_catalog.rs");
const lib = read("crates/persistence-postgres/src/lib.rs");
const catalog = read("crates/persistence-postgres/src/adapters/catalog/mod.rs");
const detailRead = read("crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/read.rs");
const port = read("crates/application/src/ports/player/mod.rs");
const adapter = read("crates/application/src/composition/adapters/players.rs");
check(!legacy.includes("pub async fn add_player_ability_observation"), "legacy player_catalog.rs 仍拥有 ability write");
check(!legacy.includes("pub async fn list_ability_dimensions"), "legacy player_catalog.rs 仍拥有 ability dimension query");
check(!legacy.includes("fn player_ability_observation_from_row"), "legacy player_catalog.rs 仍拥有 ability observation mapper");
check(!legacy.includes("fn ability_dimension_from_row"), "legacy player_catalog.rs 仍拥有 ability dimension mapper");
check(!exists("crates/persistence-postgres/src/dynamic_tags.rs"), "旧 dynamic_tags.rs 单文件 owner 尚未删除");
check(!lib.includes("mod dynamic_tags;"), "lib.rs 仍注册旧 dynamic_tags root owner");
check(catalog.includes("mod abilities;") && catalog.includes("mod dynamic_tags;"), "catalog 未注册 R6-06 owners");
check(detailRead.includes("catalog::abilities::observations"), "Player Detail ability read 未复用新 Row/mapper owner");
check(!exists("crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/mapper.rs"), "Detail ability duplicate mapper 未删除");
check(!exists("crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/row.rs"), "Detail ability duplicate row 未删除");
check(port.includes("async fn add_ability_observation(") && port.includes("async fn add_dynamic_tag(") && port.includes("async fn calculate_match_contribution("), "PlayerSignalPort 公共契约变化");
check(adapter.includes("self.add_player_ability_observation(draft)") && adapter.includes("self.add_player_dynamic_tag(draft)") && adapter.includes("self.calculate_player_match_contribution(request)"), "Application adapter 调用语义变化");
for (const modFile of [
  "crates/persistence-postgres/src/adapters/catalog/abilities/mod.rs",
  "crates/persistence-postgres/src/adapters/catalog/abilities/observations/mod.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/mod.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/mod.rs",
  "crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/mod.rs",
]) {
  const text = read(modFile);
  check(!text.includes("sqlx::") && !text.includes("impl PostgresStore"), `${modFile} 不得承载 SQL/业务实现`);
}
if (failures.length) {
  console.error("R6-06 Abilities / Dynamic Tags ownership verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R6-06 Abilities / Dynamic Tags ownership verified: dimensions, observations, tag definitions, tag writes/reads and contribution scoring have unique modular owners.");
