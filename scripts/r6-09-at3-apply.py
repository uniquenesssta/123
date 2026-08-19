from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LEGACY_PATH = "crates/persistence-postgres/src/team_force_delete.rs"
DELETION_ROOT = "crates/persistence-postgres/src/adapters/catalog/deletion"


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text.replace("\r\n", "\n"), encoding="utf-8", newline="\n")


def need(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    need(count == 1, f"{label}: expected one anchor, got {count}")
    return text.replace(old, new, 1)


def section(source: str, start_marker: str, end_marker: str | None) -> str:
    need(source.count(start_marker) == 1, f"source marker count invalid: {start_marker}")
    start = source.index(start_marker)
    if end_marker is None:
        return source[start:].strip() + "\n"
    need(source.count(end_marker) == 1, f"source marker count invalid: {end_marker}")
    end = source.index(end_marker)
    need(start < end, f"source marker order invalid: {start_marker} -> {end_marker}")
    return source[start:end].strip() + "\n"


legacy = read(LEGACY_PATH)
markers = [
    "impl PostgresStore {",
    "async fn lock_team(",
    "async fn prepare_force_delete_targets(",
    "async fn execute_force_delete(",
    "async fn force_delete_counts(",
    "async fn temp_ids(",
]
for marker in markers:
    need(legacy.count(marker) == 1, f"legacy force-delete owner marker count invalid: {marker}")

impl_block = section(legacy, markers[0], markers[1])
lock_fn = section(legacy, markers[1], markers[2])
prepare_fn = section(legacy, markers[2], markers[3])
execute_fn = section(legacy, markers[3], markers[4])
counts_fn = section(legacy, markers[4], markers[5])
temp_ids_fn = section(legacy, markers[5], None)

set_config_block = '''        sqlx::query_scalar::<_, String>("SELECT set_config('football.force_purge', 'on', true)")
            .fetch_one(&mut *tx)
            .await?;

        execute_force_delete(&mut tx, request.team_id).await?;'''
impl_block = replace_once(
    impl_block,
    set_config_block,
    "        execute_force_delete(&mut tx, request.team_id).await?;",
    "move transaction-local force-purge enablement to execution owner",
)
need("sqlx::" not in impl_block, "force-delete operation coordinator must be SQL-free")

operation = f'''use super::{{
    counts::force_delete_counts,
    execute::execute_force_delete,
    targets::{{lock_team, prepare_force_delete_targets, temp_ids}},
}};
use crate::{{PersistenceError, PersistenceResult, PostgresStore}};
use football_domain::{{TeamForceDeletePreview, TeamForceDeleteRequest, TeamForceDeleteResult}};
use serde_json::json;
use std::collections::BTreeMap;
use uuid::Uuid;

{impl_block}'''

lock_fn = lock_fn.replace("async fn lock_team(", "pub(super) async fn lock_team(", 1)
prepare_fn = prepare_fn.replace(
    "async fn prepare_force_delete_targets(",
    "pub(super) async fn prepare_force_delete_targets(",
    1,
)
temp_ids_fn = temp_ids_fn.replace("async fn temp_ids(", "pub(super) async fn temp_ids(", 1)
targets = f'''use crate::{{PersistenceError, PersistenceResult}};
use sqlx::{{Postgres, Transaction}};
use uuid::Uuid;

{lock_fn}\n{prepare_fn}\n{temp_ids_fn}'''

counts_fn = counts_fn.replace(
    "async fn force_delete_counts(",
    "pub(super) async fn force_delete_counts(",
    1,
)
counts = f'''use crate::PersistenceResult;
use football_domain::EntityReferenceCount;
use sqlx::{{Postgres, Row, Transaction}};
use uuid::Uuid;

{counts_fn}'''

execute_fn = execute_fn.replace(
    "async fn execute_force_delete(",
    "pub(super) async fn execute_force_delete(",
    1,
)
execute_open = ") -> PersistenceResult<()> {\n"
need(execute_fn.count(execute_open) == 1, "execute owner opening anchor mismatch")
execute_fn = execute_fn.replace(
    execute_open,
    execute_open
    + '''    sqlx::query_scalar::<_, String>("SELECT set_config('football.force_purge', 'on', true)")
        .fetch_one(&mut **tx)
        .await?;

''',
    1,
)
execute = f'''use crate::PersistenceResult;
use sqlx::{{Postgres, Transaction}};
use uuid::Uuid;

{execute_fn}'''

write(f"{DELETION_ROOT}/force_delete/mod.rs", "mod counts;\nmod execute;\nmod operation;\nmod targets;\n")
write(f"{DELETION_ROOT}/force_delete/operation.rs", operation)
write(f"{DELETION_ROOT}/force_delete/targets.rs", targets)
write(f"{DELETION_ROOT}/force_delete/counts.rs", counts)
write(f"{DELETION_ROOT}/force_delete/execute.rs", execute)

mod_path = f"{DELETION_ROOT}/mod.rs"
mod_text = read(mod_path)
need("mod force_delete;" not in mod_text, "AT3 force-delete module was already registered")
mod_text = replace_once(
    mod_text,
    "mod delete_write;\n",
    "mod delete_write;\nmod force_delete;\n",
    "deletion module registration",
)
write(mod_path, mod_text)

lib_path = "crates/persistence-postgres/src/lib.rs"
lib_text = replace_once(read(lib_path), "mod team_force_delete;\n", "", "legacy force-delete module")
write(lib_path, lib_text)
(ROOT / LEGACY_PATH).unlink()

verify_force_path = "scripts/verify-force-team-delete.mjs"
verify_force = read(verify_force_path)
verify_force = replace_once(
    verify_force,
    'const persistence = read("crates/persistence-postgres/src/team_force_delete.rs");\nconst persistenceLib = read("crates/persistence-postgres/src/lib.rs");',
    '''const forceDeleteRoot = "crates/persistence-postgres/src/adapters/catalog/deletion/force_delete/";
const forceDeleteOperation = read(forceDeleteRoot + "operation.rs");
const forceDeleteTargets = read(forceDeleteRoot + "targets.rs");
const forceDeleteCounts = read(forceDeleteRoot + "counts.rs");
const forceDeleteExecute = read(forceDeleteRoot + "execute.rs");
const persistence = [forceDeleteOperation, forceDeleteTargets, forceDeleteCounts, forceDeleteExecute].join("\\n");
const persistenceLib = read("crates/persistence-postgres/src/lib.rs");
const deletionMod = read("crates/persistence-postgres/src/adapters/catalog/deletion/mod.rs");''',
    "force-delete verifier persistence owner",
)
verify_force = replace_once(
    verify_force,
    'requireTrue(persistenceLib.includes("mod team_force_delete;"), "持久化层未注册球队强制清除模块");',
    '''requireTrue(!fs.existsSync(new URL("../crates/persistence-postgres/src/team_force_delete.rs", import.meta.url)), "旧球队强制清除单文件 owner 仍存在");
requireTrue(!persistenceLib.includes("mod team_force_delete;") && deletionMod.includes("mod force_delete;"), "球队强制清除未注册到 deletion adapter owner");
requireTrue(!forceDeleteOperation.includes("sqlx::") && !forceDeleteOperation.includes("SELECT ") && !forceDeleteOperation.includes("DELETE FROM ") && !forceDeleteOperation.includes("CREATE TEMP TABLE"), "球队强制清除 coordinator 仍直接持有 SQL");
requireTrue(forceDeleteTargets.includes("FOR UPDATE") && forceDeleteTargets.includes("CREATE TEMP TABLE purge_matches") && forceDeleteTargets.includes("pub(super) async fn temp_ids"), "强制清除 target-set owner 不完整");
requireTrue(forceDeleteCounts.includes("pub(super) async fn force_delete_counts") && forceDeleteCounts.includes("UNION ALL SELECT 'match_events'"), "强制清除 impact-count owner 不完整");
requireTrue(forceDeleteExecute.includes("set_config('football.force_purge', 'on', true)") && forceDeleteExecute.includes("DELETE FROM football.teams WHERE id=$1"), "强制清除 execution owner 不完整");''',
    "force-delete verifier module registration",
)
write(verify_force_path, verify_force)

verify_deletion_path = "scripts/verify-entity-deletion-persistence.mjs"
verify_deletion = read(verify_deletion_path)
verify_deletion = replace_once(
    verify_deletion,
    '  "archive/mod.rs","archive/bulk.rs","archive/write.rs","bulk_delete.rs","safe_delete.rs","delete_write.rs"',
    '  "archive/mod.rs","archive/bulk.rs","archive/write.rs","bulk_delete.rs","safe_delete.rs","delete_write.rs",\n  "force_delete/mod.rs","force_delete/operation.rs","force_delete/targets.rs","force_delete/counts.rs","force_delete/execute.rs"',
    "deletion owner inventory",
)
verify_deletion = replace_once(
    verify_deletion,
    'const deleteWrite=read(root+"delete_write.rs");',
    '''const deleteWrite=read(root+"delete_write.rs");
const forceOperation=read(root+"force_delete/operation.rs");
const forceTargets=read(root+"force_delete/targets.rs");
const forceCounts=read(root+"force_delete/counts.rs");
const forceExecute=read(root+"force_delete/execute.rs");''',
    "deletion verifier force-delete owners",
)
verify_deletion = replace_once(
    verify_deletion,
    'for(const p of ["preflight/check.rs","archive/bulk.rs","bulk_delete.rs","safe_delete.rs"]){const s=read(root+p);req(!s.includes("sqlx::")&&!s.includes("SELECT ")&&!s.includes("DELETE FROM "),`coordinator owns SQL: ${p}`)}',
    'for(const p of ["preflight/check.rs","archive/bulk.rs","bulk_delete.rs","safe_delete.rs","force_delete/operation.rs"]){const s=read(root+p);req(!s.includes("sqlx::")&&!s.includes("SELECT ")&&!s.includes("DELETE FROM ")&&!s.includes("CREATE TEMP TABLE"),`coordinator owns SQL: ${p}`)}',
    "SQL-free coordinator inventory",
)
verify_deletion = replace_once(
    verify_deletion,
    'req(read("crates/persistence-postgres/src/team_force_delete.rs").includes("pub async fn force_delete_team"),"AT3 force delete moved early");\nconsole.log("R6-09 AT1+AT2 deletion preflight/archive/safe-permanent-delete ownership verified.");',
    '''req(!exists("crates/persistence-postgres/src/team_force_delete.rs")&&!lib.includes("mod team_force_delete;"),"legacy force-delete owner remains");
req(read(root+"mod.rs").includes("mod force_delete;"),"AT3 force-delete adapter module missing");
req(forceOperation.includes("pub async fn preview_force_delete_team")&&forceOperation.includes("pub async fn force_delete_team")&&forceOperation.includes("request.confirmation_text.trim() != label"),"AT3 force-delete coordinator contract missing");
req(forceTargets.includes("FOR UPDATE")&&forceTargets.includes("CREATE TEMP TABLE purge_matches")&&forceTargets.includes("pub(super) async fn temp_ids"),"AT3 target-set owner incomplete");
req(forceCounts.includes("UNION ALL SELECT 'match_events'")&&forceCounts.includes("team_lineup_preset_members"),"AT3 count owner incomplete");
req(forceExecute.includes("set_config('football.force_purge', 'on', true)")&&forceExecute.includes("DELETE FROM football.teams WHERE id=$1"),"AT3 execution owner incomplete");
console.log("R6-09 AT1+AT2+AT3 deletion preflight/archive/safe-delete/force-delete ownership verified.");''',
    "AT3 advancement assertion",
)
write(verify_deletion_path, verify_deletion)

stale = []
legacy_reference = "crates/persistence-postgres/src/team_force_delete.rs"
for path in sorted((ROOT / "scripts").rglob("*.mjs")):
    source = path.read_text(encoding="utf-8")
    if legacy_reference in source:
        stale.append(path.relative_to(ROOT).as_posix())
need(not stale, "R6-09 AT3 stale force-delete verifier owner references remain:\n- " + "\n- ".join(stale))

contract_path = "crates/persistence-postgres/tests/team_force_delete_repository_contract.rs"
write(
    contract_path,
    r'''use chrono::Utc;
use football_domain::{
    DataProviderDraft, ExternalEntityIdDraft, PlayerDraft, PlayerStatus, PlayerTeamPeriodDraft,
    PreferredFoot, TeamDraft, TeamForceDeleteRequest,
};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn team_force_delete_preserves_confirmation_scope_and_cleanup_contract() {
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
    let provider = store
        .create_data_provider(&DataProviderDraft {
            code: format!("r6_09_force_{token}"),
            name: format!("R6-09 Force Provider {token}"),
            provider_type: "official".into(),
            base_url: Some("https://example.test/r6-09-force".into()),
            metadata: json!({"contract":"r6-09-at3"}),
        })
        .await
        .expect("provider");
    let team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-09 Force Team {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({"contract":"r6-09-at3"}),
        })
        .await
        .expect("team");
    let player = store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-09 Force Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".into()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(181),
            status: PlayerStatus::Active,
            metadata: json!({"contract":"r6-09-at3"}),
        })
        .await
        .expect("player");
    store
        .add_player_team_period(&PlayerTeamPeriodDraft {
            player_id: player.id,
            team_id: team.id,
            season_id: None,
            squad_number: Some(9),
            valid_from: Utc::now().date_naive(),
            valid_to: None,
            registration_status: "registered".into(),
            source_document_id: None,
        })
        .await
        .expect("team period");
    for (entity_type, entity_id, external_id) in [
        ("team", team.id, format!("FORCE-TEAM-{token}")),
        ("player", player.id, format!("FORCE-PLAYER-{token}")),
    ] {
        store
            .add_external_entity_id(&ExternalEntityIdDraft {
                provider_id: provider.id,
                entity_type: entity_type.into(),
                entity_id,
                external_id,
                metadata: json!({}),
            })
            .await
            .expect("external id");
    }

    let preview = store
        .preview_force_delete_team(team.id)
        .await
        .expect("force-delete preview");
    assert_eq!(preview.team_id, team.id);
    assert_eq!(preview.label, team.canonical_name);
    assert_eq!(preview.confirmation_text, team.canonical_name);
    assert!(preview.total_rows >= 3);
    assert!(preview.references.iter().any(|item| item.relation == "teams" && item.count == 1));
    assert!(preview.references.iter().any(|item| item.relation == "players" && item.count >= 1));
    assert!(preview.references.iter().any(|item| item.relation == "player_team_periods" && item.count >= 1));

    let wrong = store
        .force_delete_team(&TeamForceDeleteRequest {
            team_id: team.id,
            confirmation_text: format!("{}-wrong", team.canonical_name),
        })
        .await
        .expect_err("wrong confirmation must be rejected");
    assert!(wrong.to_string().contains("确认文字不匹配"));
    let still_present: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.player_team_periods WHERE team_id=$1 AND player_id=$2",
    )
    .bind(team.id)
    .bind(player.id)
    .fetch_one(&pool)
    .await
    .expect("period after rejected confirmation");
    assert_eq!(still_present, 1);

    let result = store
        .force_delete_team(&TeamForceDeleteRequest {
            team_id: team.id,
            confirmation_text: team.canonical_name.clone(),
        })
        .await
        .expect("force delete");
    assert_eq!(result.team_id, team.id);
    assert_eq!(result.label, team.canonical_name);
    assert!(result.deleted_player_ids.contains(&player.id));
    assert!(result.deleted_counts.get("players").copied().unwrap_or_default() >= 1);
    assert!(result.deleted_counts.get("player_team_periods").copied().unwrap_or_default() >= 1);

    let team_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM football.teams WHERE id=$1")
        .bind(team.id)
        .fetch_one(&pool)
        .await
        .expect("team count");
    let player_count: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM football.players WHERE id=$1")
        .bind(player.id)
        .fetch_one(&pool)
        .await
        .expect("player count");
    let period_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.player_team_periods WHERE team_id=$1 OR player_id=$2",
    )
    .bind(team.id)
    .bind(player.id)
    .fetch_one(&pool)
    .await
    .expect("period count");
    let external_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.external_entity_ids WHERE entity_id=$1 OR entity_id=$2",
    )
    .bind(team.id)
    .bind(player.id)
    .fetch_one(&pool)
    .await
    .expect("external id count");
    let tombstone_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='team_force_deleted' AND entity_type='team_purge' AND entity_id=$1",
    )
    .bind(team.id.to_string())
    .fetch_one(&pool)
    .await
    .expect("force-delete tombstone");
    let provider_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM catalog.data_providers WHERE id=$1",
    )
    .bind(provider.id)
    .fetch_one(&pool)
    .await
    .expect("provider count");

    assert_eq!(team_count, 0);
    assert_eq!(player_count, 0);
    assert_eq!(period_count, 0);
    assert_eq!(external_count, 0);
    assert_eq!(tombstone_count, 1);
    assert_eq!(provider_count, 1);

    pool.close().await;
    store.close().await;
}
''',
)

print("R6-09 AT3 transformation prepared: force-delete ownership split by operation/targets/counts/execute with public contract unchanged.")
