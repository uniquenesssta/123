import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const read = (relative) => fs.readFileSync(path.join(root, relative), 'utf8');
const exists = (relative) => fs.existsSync(path.join(root, relative));

const required = [
  'crates/persistence-postgres/src/adapters/catalog/players/directory/create_player.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/directory/update_player.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/directory/list_players.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/directory/input_policy.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/directory/list_row.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/directory/list_mapper.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/detail/read_player.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/detail/names/read.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/detail/positions/read.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/detail/team_periods/read.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/detail/availability/read.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/profile/read.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/read.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/detail/external_ids/read.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/record/row.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/record/mapper.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/normalization.rs',
  'crates/persistence-postgres/src/adapters/catalog/players/value_mapping.rs',
  'crates/persistence-postgres/tests/player_directory_detail_repository_contract.rs',
];
for (const relative of required) {
  if (!exists(relative)) throw new Error(`R6-03 required file missing: ${relative}`);
}

const legacy = read('crates/persistence-postgres/src/player_catalog.rs');
for (const method of ['create_player', 'update_player', 'list_players', 'read_player']) {
  if (legacy.includes(`pub async fn ${method}`)) {
    throw new Error(`R6-03 legacy owner still exposes ${method}`);
  }
}
for (const retained of [
  'delete_player',
  'add_player_name',
  'assign_player_position',
  'add_player_team_period',
  'add_player_availability',
  'add_player_ability_observation',
]) {
  if (!legacy.includes(`pub async fn ${retained}`)) {
    throw new Error(`Later R6 owner moved prematurely or disappeared: ${retained}`);
  }
}

const dynamicTags = read('crates/persistence-postgres/src/dynamic_tags.rs');
for (const retained of [
  'add_player_dynamic_tag',
  'read_player_dynamic_tag',
  'list_player_dynamic_tags',
]) {
  if (!dynamicTags.includes(`pub async fn ${retained}`)) {
    throw new Error(`R6-03 must preserve dynamic tag owner in dynamic_tags.rs: ${retained}`);
  }
}

const create = read('crates/persistence-postgres/src/adapters/catalog/players/directory/create_player.rs');
const update = read('crates/persistence-postgres/src/adapters/catalog/players/directory/update_player.rs');
const list = read('crates/persistence-postgres/src/adapters/catalog/players/directory/list_players.rs');
const detail = read('crates/persistence-postgres/src/adapters/catalog/players/detail/read_player.rs');

for (const token of ['player_created', 'football.player_names', 'write_audit_event', 'tx.commit']) {
  if (!create.includes(token)) throw new Error(`create_player contract marker missing: ${token}`);
}
for (const token of ['player_updated', 'metadata = metadata || $9', 'UPDATE football.player_names SET is_primary = false', 'tx.commit']) {
  if (!update.includes(token)) throw new Error(`update_player contract marker missing: ${token}`);
}
for (const token of ['NameSearch::parse', 'push_name_search', 'player.normalized_name, player.id', '球员分页游标必须同时包含名称和 ID']) {
  if (!list.includes(token)) throw new Error(`list_players contract marker missing: ${token}`);
}
for (const token of ['read_names', 'read_positions', 'read_team_periods', 'read_availability', 'read_ability_profile', 'read_ability_observations', 'list_player_dynamic_tags', 'read_external_ids']) {
  if (!detail.includes(token)) throw new Error(`read_player aggregation marker missing: ${token}`);
}

const catalogMod = read('crates/persistence-postgres/src/adapters/catalog/mod.rs');
if (!catalogMod.includes('pub(crate) mod players;')) {
  throw new Error('catalog module does not register players owner');
}

console.log('R6-03 Player Directory/Detail ownership verification: PASS');
