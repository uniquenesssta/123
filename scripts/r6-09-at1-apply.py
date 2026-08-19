from pathlib import Path
import json
import textwrap

root = Path('.')


def read(path):
    return (root / path).read_text(encoding='utf-8')


def write(path, content):
    target = root / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content.replace('\r\n', '\n'), encoding='utf-8', newline='\n')


def need(condition, message):
    if not condition:
        raise SystemExit(message)


def once(text, old, new, message):
    need(text.count(old) == 1, f'{message}: {text.count(old)} anchors')
    return text.replace(old, new, 1)


def cut(text, start, end, message):
    i = text.find(start)
    need(i >= 0, f'{message}: start missing')
    j = text.find(end, i)
    need(j >= 0, f'{message}: end missing')
    return text[i:j]


entity_path = Path('crates/persistence-postgres/src/entity_catalog.rs')
entity = read(entity_path)
need('pub async fn check_entity_deletion' in entity, 'R6-09 AT1 source owner missing')
need('pub async fn bulk_archive_entities' in entity, 'R6-09 AT1 archive source owner missing')

write(Path('crates/persistence-postgres/tests/entity_deletion_repository_contract.rs'), r'''use chrono::{Duration, Utc};
use football_domain::{PlayerDraft, PlayerStatus, PlayerTeamPeriodDraft, PreferredFoot, TeamDraft};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn deletion_preflight_and_archive_contract_is_preserved() {
    let url = std::env::var("FOOTBALL_TEST_DATABASE_URL").expect("设置 FOOTBALL_TEST_DATABASE_URL");
    let store = PostgresStore::connect(&DatabaseOptions {
        connection_url: url.clone(),
        max_connections: 4,
        connect_timeout_seconds: 10,
    })
    .await
    .expect("connect");
    store.migrate().await.expect("migrate");
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .expect("pool");
    let token = Uuid::new_v4().simple().to_string();

    let free = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-09 Free {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({}),
        })
        .await
        .expect("free team");
    let free_check = store
        .check_entity_deletion("team", free.id)
        .await
        .expect("free check");
    assert!(free_check.exists && free_check.can_permanently_delete && !free_check.must_archive);
    assert_eq!(free_check.reason, "没有历史引用，可以永久删除");

    let team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-09 Archive {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({}),
        })
        .await
        .expect("team");
    let player = store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-09 Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".into()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(180),
            status: PlayerStatus::Active,
            metadata: json!({}),
        })
        .await
        .expect("player");
    let from = Utc::now().date_naive();
    store
        .add_player_team_period(&PlayerTeamPeriodDraft {
            player_id: player.id,
            team_id: team.id,
            season_id: None,
            squad_number: Some(9),
            valid_from: from,
            valid_to: Some(from + Duration::days(30)),
            registration_status: "registered".into(),
            source_document_id: None,
        })
        .await
        .expect("period");

    let check = store
        .check_entity_deletion("team", team.id)
        .await
        .expect("protected check");
    assert!(!check.can_permanently_delete && check.must_archive && check.reason.contains("只允许归档"));
    assert!(check
        .references
        .iter()
        .any(|item| item.relation == "player_team_periods"));

    let result = store
        .bulk_archive_entities("team", &[team.id, team.id])
        .await
        .expect("archive");
    assert_eq!(result.requested_count, 1);
    assert_eq!(result.archived_ids, vec![team.id]);

    let active: bool = sqlx::query_scalar("SELECT is_active FROM football.teams WHERE id=$1")
        .bind(team.id)
        .fetch_one(&pool)
        .await
        .expect("active");
    let periods: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.player_team_periods WHERE team_id=$1",
    )
    .bind(team.id)
    .fetch_one(&pool)
    .await
    .expect("periods");
    assert!(!active);
    assert_eq!(periods, 1);

    pool.close().await;
    store.close().await;
}
''')

list_method = cut(
    entity,
    '    pub(crate) async fn list_team_player_periods(',
    '\n\n    pub async fn check_entity_deletion(',
    'list period',
).rstrip()
mapper = cut(
    entity,
    'fn team_player_period_from_row(',
    '\n\nfn unique_ids(',
    'period mapper',
).rstrip()
write(
    entity_path,
    'use crate::{PersistenceResult, PostgresStore};\n'
    'use football_domain::TeamPlayerPeriodRecord;\n'
    'use sqlx::Row;\n'
    'use uuid::Uuid;\n\n'
    'impl PostgresStore {\n' + list_method + '\n}\n\n' + mapper + '\n',
)

deletion = Path('crates/persistence-postgres/src/adapters/catalog/deletion')

check = cut(
    entity,
    '    pub async fn check_entity_deletion(',
    '\n\n    async fn entity_label(',
    'check',
)
check = once(
    check,
    'let label = self.entity_label(entity_type, entity_id).await?;',
    'let label = entity_label(&self.pool, entity_type, entity_id).await?;',
    'label call',
)
for kind, name in [
    ('team', 'team_reference_counts'),
    ('player', 'player_reference_counts'),
    ('coach', 'coach_reference_counts'),
]:
    check = once(
        check,
        f'"{kind}" => self.{name}(entity_id).await?,',
        f'"{kind}" => {name}(&self.pool, entity_id).await?,',
        f'{kind} reference call',
    )
write(
    deletion / 'preflight/check.rs',
    'use super::{\n'
    '    labels::entity_label,\n'
    '    references::{coach_reference_counts, player_reference_counts, team_reference_counts},\n'
    '};\n'
    'use crate::{adapters::catalog::references::validate_entity_type, PersistenceResult, PostgresStore};\n'
    'use football_domain::EntityDeletionCheck;\n'
    'use uuid::Uuid;\n\n'
    'impl PostgresStore {\n' + check.rstrip() + '\n}\n',
)

label = textwrap.dedent(
    cut(
        entity,
        '    async fn entity_label(',
        '\n\n    async fn team_reference_counts(',
        'label',
    )
).rstrip() + '\n'
label = once(label, 'async fn entity_label(', 'pub(crate) async fn entity_label(', 'label vis')
label = once(label, '    &self,\n', '    pool: &sqlx::PgPool,\n', 'label pool')
label = label.replace('&self.pool', 'pool')
write(
    deletion / 'preflight/labels.rs',
    'use crate::PersistenceResult;\nuse uuid::Uuid;\n\n' + label,
)

reference_functions = []
for name, next_name, prefix in [
    ('team_reference_counts', 'player_reference_counts', ''),
    ('player_reference_counts', 'coach_reference_counts', ''),
    ('coach_reference_counts', 'bulk_archive_entities', 'pub '),
]:
    source = textwrap.dedent(
        cut(
            entity,
            f'    async fn {name}(',
            f'\n\n    {prefix}async fn {next_name}(',
            name,
        )
    ).rstrip() + '\n'
    source = once(source, f'async fn {name}(', f'pub(crate) async fn {name}(', f'{name} vis')
    source = once(source, '    &self,\n', '    pool: &sqlx::PgPool,\n', f'{name} pool')
    source = source.replace('&self.pool', 'pool')
    reference_functions.append(source.rstrip())
count_relations = cut(
    entity,
    'async fn count_relations(',
    '\n\nfn team_player_period_from_row(',
    'count relations',
).rstrip()
write(
    deletion / 'preflight/references.rs',
    'use crate::PersistenceResult;\n'
    'use football_domain::EntityReferenceCount;\n'
    'use uuid::Uuid;\n\n'
    + '\n\n'.join(reference_functions)
    + '\n\n'
    + count_relations
    + '\n',
)
write(deletion / 'preflight/mod.rs', 'mod check;\npub(crate) mod labels;\npub(crate) mod references;\n')

bulk = cut(
    entity,
    '    pub async fn bulk_archive_entities(',
    '\n\n    async fn archive_entity(',
    'archive bulk',
)
bulk = once(
    bulk,
    'let label = self\n                .entity_label(entity_type, *id)\n                .await?\n                .unwrap_or_else(|| id.to_string());',
    'let label = entity_label(&self.pool, entity_type, *id)\n                .await?\n                .unwrap_or_else(|| id.to_string());',
    'archive label',
)
bulk = once(
    bulk,
    'match self.archive_entity(entity_type, *id).await {',
    'match archive_entity(&self.pool, entity_type, *id).await {',
    'archive call',
)
write(
    deletion / 'archive/bulk.rs',
    'use super::super::{ids::unique_ids, preflight::labels::entity_label};\n'
    'use super::write::archive_entity;\n'
    'use crate::{adapters::catalog::references::validate_entity_type, PersistenceResult, PostgresStore};\n'
    'use football_domain::{BulkArchiveFailedItem, BulkArchiveResult};\n'
    'use uuid::Uuid;\n\n'
    'impl PostgresStore {\n' + bulk.rstrip() + '\n}\n',
)

archive_write = textwrap.dedent(
    cut(
        entity,
        '    async fn archive_entity(',
        '\n\n    async fn entity_label_in_tx(',
        'archive write',
    )
).rstrip() + '\n'
archive_write = once(
    archive_write,
    'async fn archive_entity(&self, entity_type: &str, id: Uuid) -> PersistenceResult<bool> {',
    'pub(crate) async fn archive_entity(pool: &sqlx::PgPool, entity_type: &str, id: Uuid) -> PersistenceResult<bool> {',
    'archive sig',
).replace('self.pool.begin()', 'pool.begin()')
archive_write = once(
    archive_write,
    '} else if self\n        .entity_label_in_tx(&mut tx, entity_type, id)\n        .await?',
    '} else if entity_label_in_tx(&mut tx, entity_type, id).await?',
    'archive label tx',
)

label_in_tx = textwrap.dedent(
    cut(
        entity,
        '    async fn entity_label_in_tx(',
        '\n}\n\nasync fn count_relations(',
        'archive tx label',
    )
).rstrip() + '\n'
label_in_tx = once(label_in_tx, '    &self,\n', '', 'archive self')
write(
    deletion / 'archive/write.rs',
    'use crate::{write_audit_event, PersistenceError, PersistenceResult};\n'
    'use serde_json::json;\n'
    'use sqlx::{Postgres, Transaction};\n'
    'use uuid::Uuid;\n\n'
    + archive_write.rstrip()
    + '\n\n'
    + label_in_tx,
)
write(deletion / 'archive/mod.rs', 'mod bulk;\nmod write;\n')

unique = entity[entity.index('fn unique_ids('):].rstrip() + '\n'
unique = once(unique, 'fn unique_ids(', 'pub(crate) fn unique_ids(', 'unique ids vis')
write(deletion / 'ids.rs', 'use uuid::Uuid;\n\n' + unique)
write(deletion / 'mod.rs', 'mod archive;\nmod ids;\nmod preflight;\n')

catalog_path = Path('crates/persistence-postgres/src/adapters/catalog/mod.rs')
catalog = read(catalog_path)
catalog = once(catalog, 'mod dynamic_tags;\n', 'mod dynamic_tags;\nmod deletion;\n', 'catalog')
write(catalog_path, catalog)

write(
    Path('scripts/verify-entity-deletion-persistence.mjs'),
    '''import fs from "node:fs";
const read=(p)=>fs.readFileSync(new URL(`../${p}`,import.meta.url),"utf8").replace(/\\r\\n?/g,"\\n");
const exists=(p)=>fs.existsSync(new URL(`../${p}`,import.meta.url));
const req=(v,m)=>{if(!v)throw new Error(m)};
const root="crates/persistence-postgres/src/adapters/catalog/deletion/";
for(const p of ["mod.rs","ids.rs","preflight/mod.rs","preflight/check.rs","preflight/labels.rs","preflight/references.rs","archive/mod.rs","archive/bulk.rs","archive/write.rs"])req(exists(root+p),`R6-09 AT1 owner missing: ${p}`);
const entity=read("crates/persistence-postgres/src/entity_catalog.rs");
const check=read(root+"preflight/check.rs");
const refs=read(root+"preflight/references.rs");
const archive=read(root+"archive/bulk.rs")+read(root+"archive/write.rs");
for(const n of ["check_entity_deletion","bulk_archive_entities","team_reference_counts","player_reference_counts","coach_reference_counts"])req(!entity.includes(n),`legacy entity owner remains: ${n}`);
req(check.includes("pub async fn check_entity_deletion")&&archive.includes("pub async fn bulk_archive_entities"),"AT1 public owners missing");
for(const p of ["preflight/check.rs","archive/bulk.rs"]){const s=read(root+p);req(!s.includes("sqlx::")&&!s.includes("SELECT ")&&!s.includes("UPDATE football"),`coordinator owns SQL: ${p}`)}
for(const r of ["matches","lineups","player_team_periods","team_coach_periods","player_availability","formation_usage","team_tactical_observations","team_ability_observations","substitutions","match_events","team_match_reviews","player_match_reviews","player_match_observations"])req(refs.includes(`"${r}"`),`reference count missing: ${r}`);
req(check.includes("can_permanently_delete: total == 0")&&check.includes("只允许归档"),"preflight semantics changed");
req(archive.includes("manual_bulk_archive")&&archive.includes("_archived"),"archive audit semantics changed");
req(read("crates/persistence-postgres/src/team_catalog.rs").includes("pub async fn bulk_delete_teams"),"AT2 permanent delete moved early");
req(read("crates/persistence-postgres/src/team_force_delete.rs").includes("pub async fn force_delete_team"),"AT3 force delete moved early");
console.log("R6-09 AT1 deletion preflight/archive ownership verified.");
''',
)

path = Path('scripts/verify-entity-deletion.mjs')
text = read(path)
text = once(
    text,
    'const entityPersistence = text("crates/persistence-postgres/src/entity_catalog.rs");',
    'const entityPersistence = [\n'
    '  text("crates/persistence-postgres/src/adapters/catalog/deletion/preflight/check.rs"),\n'
    '  text("crates/persistence-postgres/src/adapters/catalog/deletion/preflight/references.rs"),\n'
    '  text("crates/persistence-postgres/src/adapters/catalog/deletion/archive/bulk.rs"),\n'
    '  text("crates/persistence-postgres/src/adapters/catalog/deletion/archive/write.rs"),\n'
    '].join("\\n");',
    'entity deletion verifier',
)
write(path, text)

path = Path('scripts/verify-entity-relationships.mjs')
text = read(path)
anchor = '  text("crates/persistence-postgres/src/adapters/catalog/references/directory/list.rs"),\n'
replacement = anchor + (
    '  text("crates/persistence-postgres/src/adapters/catalog/deletion/preflight/check.rs"),\n'
    '  text("crates/persistence-postgres/src/adapters/catalog/deletion/preflight/references.rs"),\n'
    '  text("crates/persistence-postgres/src/adapters/catalog/deletion/archive/bulk.rs"),\n'
    '  text("crates/persistence-postgres/src/adapters/catalog/deletion/archive/write.rs"),\n'
)
text = once(text, anchor, replacement, 'entity relationships deletion owners')
write(path, text)

path = Path('scripts/verify-coach-formation-persistence.mjs')
text = read(path)
text = once(
    text,
    'req(!entity.includes("pub async fn list_entity_references") && !entity.includes("pub async fn resolve_entity_reference") && entity.includes("pub async fn bulk_archive_entities"), "R6-08 ownership switch incomplete or R6-09 ownership moved early");',
    'const deletionArchive = read("crates/persistence-postgres/src/adapters/catalog/deletion/archive/bulk.rs");\n'
    'req(!entity.includes("pub async fn list_entity_references") && !entity.includes("pub async fn resolve_entity_reference") && !entity.includes("pub async fn bulk_archive_entities") && deletionArchive.includes("pub async fn bulk_archive_entities"), "R6-08 retained ownership or R6-09 AT1 archive owner invalid");',
    'coach retained',
)
write(path, text)

path = Path('scripts/verify-entity-matching-references-persistence.mjs')
text = read(path)
old = (
    'req(entity.includes("pub async fn check_entity_deletion") && entity.includes("pub async fn bulk_archive_entities"), "R6-09 deletion/archive moved early");\n'
    'req(entity.includes("team_reference_counts") && entity.includes("player_reference_counts") && entity.includes("coach_reference_counts"), "R6-09 reference-count owner moved early");'
)
new = (
    'const deletionCheck = read("crates/persistence-postgres/src/adapters/catalog/deletion/preflight/check.rs");\n'
    'const deletionRefs = read("crates/persistence-postgres/src/adapters/catalog/deletion/preflight/references.rs");\n'
    'const deletionArchive = read("crates/persistence-postgres/src/adapters/catalog/deletion/archive/bulk.rs");\n'
    'req(!entity.includes("pub async fn check_entity_deletion") && !entity.includes("pub async fn bulk_archive_entities") && deletionCheck.includes("pub async fn check_entity_deletion") && deletionArchive.includes("pub async fn bulk_archive_entities"), "R6-09 AT1 deletion/archive ownership invalid");\n'
    'req(!entity.includes("team_reference_counts") && deletionRefs.includes("team_reference_counts") && deletionRefs.includes("player_reference_counts") && deletionRefs.includes("coach_reference_counts"), "R6-09 AT1 reference-count ownership invalid");'
)
text = once(text, old, new, 'R6-08 retained')
write(path, text)

package_path = Path('package.json')
package = json.loads(read(package_path))
package['scripts']['verify:entity-deletion-persistence'] = 'node scripts/verify-entity-deletion-persistence.mjs'
architecture = package['scripts']['verify:architecture']
gate = 'node scripts/verify-entity-deletion-persistence.mjs'
need(gate not in architecture, 'R6-09 architecture gate already present unexpectedly')
package['scripts']['verify:architecture'] = architecture + ' && ' + gate
write(package_path, json.dumps(package, ensure_ascii=False, indent=2) + '\n')
