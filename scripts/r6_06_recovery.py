from pathlib import Path

path = Path('scripts/verify-player-directory-detail.mjs')
text = path.read_text(encoding='utf-8')
old_retained = """for (const retained of [
  'delete_player',
  'add_player_ability_observation',
]) {
  if (!legacy.includes(`pub async fn ${retained}`)) {
    throw new Error(`Later R6 owner moved prematurely or disappeared: ${retained}`);
  }
}
"""
new_retained = """for (const retained of [
  'delete_player',
]) {
  if (!legacy.includes(`pub async fn ${retained}`)) {
    throw new Error(`Later R6 owner moved prematurely or disappeared: ${retained}`);
  }
}
"""
if old_retained not in text:
    raise SystemExit('R6-03 retained legacy block not found')
text = text.replace(old_retained, new_retained, 1)

old_dynamic = """const dynamicTags = read('crates/persistence-postgres/src/dynamic_tags.rs');
for (const retained of [
  'add_player_dynamic_tag',
  'read_player_dynamic_tag',
  'list_player_dynamic_tags',
]) {
  if (!dynamicTags.includes(`pub async fn ${retained}`)) {
    throw new Error(`R6-03 must preserve dynamic tag owner in dynamic_tags.rs: ${retained}`);
  }
}
"""
new_dynamic = """const r606AdvancedOwners = [
  ['crates/persistence-postgres/src/adapters/catalog/abilities/observations/add.rs', 'add_player_ability_observation'],
  ['crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/add.rs', 'add_player_dynamic_tag'],
  ['crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/read.rs', 'read_player_dynamic_tag'],
  ['crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/list.rs', 'list_player_dynamic_tags'],
];
for (const [relative, method] of r606AdvancedOwners) {
  if (!exists(relative) || !read(relative).includes(`pub async fn ${method}`)) {
    throw new Error(`R6-06 advanced owner missing for retained R6-03 boundary: ${method}`);
  }
  if (legacy.includes(`pub async fn ${method}`)) {
    throw new Error(`R6-06 advanced owner still duplicated in legacy player_catalog.rs: ${method}`);
  }
}
checkDynamicRootRemoval();

function checkDynamicRootRemoval() {
  if (exists('crates/persistence-postgres/src/dynamic_tags.rs')) {
    throw new Error('R6-06 advanced owner must remove legacy dynamic_tags.rs');
  }
}
"""
if old_dynamic not in text:
    raise SystemExit('R6-03 dynamic tag retained block not found')
text = text.replace(old_dynamic, new_dynamic, 1)
path.write_text(text, encoding='utf-8')
