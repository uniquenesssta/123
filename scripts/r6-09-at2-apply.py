from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text.replace("\r\n", "\n"), encoding="utf-8", newline="\n")


def need(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    need(count == 1, f"{label}: expected 1 anchor, got {count}")
    return text.replace(old, new, 1)


def remove_rust_function(text: str, signature: str, label: str) -> str:
    start = text.find(signature)
    need(start >= 0, f"{label}: signature missing")
    brace = text.find("{", start)
    need(brace >= 0, f"{label}: opening brace missing")
    depth = 0
    end = None
    for index in range(brace, len(text)):
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                end = index + 1
                break
    need(end is not None, f"{label}: function not closed")
    while end < len(text) and text[end] in "\r\n":
        end += 1
    return text[:start] + text[end:]


# Contract first.
write(
    "crates/persistence-postgres/tests/entity_permanent_delete_repository_contract.rs",
    r'''use chrono::Utc;
use football_domain::{
    DataProviderDraft, ExternalEntityIdDraft, PlayerDraft, PlayerStatus, PlayerTeamPeriodDraft,
    PreferredFoot, TeamDraft,
};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn safe_permanent_delete_contract_is_preserved() {
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
            code: format!("r6_09_delete_{token}"),
            name: format!("R6-09 Delete Provider {token}"),
            provider_type: "official".into(),
            base_url: Some("https://example.test/r6-09-delete".into()),
            metadata: json!({"contract":"r6-09-at2"}),
        })
        .await
        .expect("provider");

    let free_team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-09 Delete Team {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({"contract":"r6-09-at2"}),
        })
        .await
        .expect("free team");
    store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "team".into(),
            entity_id: free_team.id,
            external_id: format!("TEAM-{token}"),
            metadata: json!({}),
        })
        .await
        .expect("team external id");
    let team_result = store
        .bulk_delete_teams(&[free_team.id, free_team.id])
        .await
        .expect("bulk delete free team");
    assert_eq!(team_result.requested_count, 1);
    assert_eq!(team_result.deleted_ids, vec![free_team.id]);
    assert!(team_result.blocked.is_empty());
    let team_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.teams WHERE id=$1",
    )
    .bind(free_team.id)
    .fetch_one(&pool)
    .await
    .expect("team count");
    let team_external_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.external_entity_ids WHERE entity_type='team' AND entity_id=$1",
    )
    .bind(free_team.id)
    .fetch_one(&pool)
    .await
    .expect("team external count");
    let team_audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='team_deleted' AND entity_type='team' AND entity_id=$1",
    )
    .bind(free_team.id.to_string())
    .fetch_one(&pool)
    .await
    .expect("team audit count");
    assert_eq!(team_count, 0);
    assert_eq!(team_external_count, 0);
    assert_eq!(team_audit_count, 1);

    let free_player = store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-09 Delete Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".into()),
            preferred_foot: PreferredFoot::Right,
            height_cm: Some(180),
            status: PlayerStatus::Active,
            metadata: json!({"contract":"r6-09-at2"}),
        })
        .await
        .expect("free player");
    store
        .add_external_entity_id(&ExternalEntityIdDraft {
            provider_id: provider.id,
            entity_type: "player".into(),
            entity_id: free_player.id,
            external_id: format!("PLAYER-{token}"),
            metadata: json!({}),
        })
        .await
        .expect("player external id");
    let player_result = store
        .bulk_delete_players(&[free_player.id, free_player.id])
        .await
        .expect("bulk delete free player");
    assert_eq!(player_result.requested_count, 1);
    assert_eq!(player_result.deleted_ids, vec![free_player.id]);
    assert!(player_result.blocked.is_empty());
    let player_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.players WHERE id=$1",
    )
    .bind(free_player.id)
    .fetch_one(&pool)
    .await
    .expect("player count");
    let player_external_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.external_entity_ids WHERE entity_type='player' AND entity_id=$1",
    )
    .bind(free_player.id)
    .fetch_one(&pool)
    .await
    .expect("player external count");
    let player_audit_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM audit.events WHERE event_type='player_deleted' AND entity_type='player' AND entity_id=$1",
    )
    .bind(free_player.id.to_string())
    .fetch_one(&pool)
    .await
    .expect("player audit count");
    assert_eq!(player_count, 0);
    assert_eq!(player_external_count, 0);
    assert_eq!(player_audit_count, 1);

    let protected_team = store
        .create_team(&TeamDraft {
            canonical_name: format!("R6-09 Protected Team {token}"),
            country_code: Some("ZZ".into()),
            metadata: json!({}),
        })
        .await
        .expect("protected team");
    let protected_player = store
        .create_player(&PlayerDraft {
            canonical_name: format!("R6-09 Protected Player {token}"),
            date_of_birth: None,
            nationality_code: Some("ZZ".into()),
            preferred_foot: PreferredFoot::Left,
            height_cm: Some(178),
            status: PlayerStatus::Active,
            metadata: json!({}),
        })
        .await
        .expect("protected player");
    store
        .add_player_team_period(&PlayerTeamPeriodDraft {
            player_id: protected_player.id,
            team_id: protected_team.id,
            season_id: None,
            squad_number: Some(7),
            valid_from: Utc::now().date_naive(),
            valid_to: None,
            registration_status: "registered".into(),
            source_document_id: None,
        })
        .await
        .expect("protected period");

    let blocked_team = store
        .bulk_delete_teams(&[protected_team.id])
        .await
        .expect("protected team result");
    assert!(blocked_team.deleted_ids.is_empty());
    assert_eq!(blocked_team.blocked.len(), 1);
    assert!(blocked_team.blocked[0].reason.contains("只允许归档"));
    let blocked_player = store
        .bulk_delete_players(&[protected_player.id])
        .await
        .expect("protected player result");
    assert!(blocked_player.deleted_ids.is_empty());
    assert_eq!(blocked_player.blocked.len(), 1);
    assert!(blocked_player.blocked[0].reason.contains("只允许归档"));
    let period_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.player_team_periods WHERE team_id=$1 AND player_id=$2",
    )
    .bind(protected_team.id)
    .bind(protected_player.id)
    .fetch_one(&pool)
    .await
    .expect("protected period count");
    assert_eq!(period_count, 1);

    pool.close().await;
    store.close().await;
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/deletion/bulk_delete.rs",
    r'''use super::{
    ids::unique_ids, preflight::labels::entity_label, safe_delete::delete_team,
};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{BulkDeleteBlockedItem, BulkDeleteResult};
use uuid::Uuid;

impl PostgresStore {
    pub async fn bulk_delete_players(
        &self,
        player_ids: &[Uuid],
    ) -> PersistenceResult<BulkDeleteResult> {
        let ids = unique_ids(player_ids);
        let mut deleted_ids = Vec::new();
        let mut blocked = Vec::new();
        for player_id in &ids {
            let label = entity_label(&self.pool, "player", *player_id)
                .await?
                .unwrap_or_else(|| player_id.to_string());
            match self.delete_player(*player_id).await {
                Ok(()) => deleted_ids.push(*player_id),
                Err(error) => blocked.push(BulkDeleteBlockedItem {
                    id: *player_id,
                    label,
                    reason: error.to_string(),
                }),
            }
        }
        Ok(BulkDeleteResult {
            requested_count: ids.len() as u64,
            deleted_ids,
            blocked,
        })
    }

    pub async fn bulk_delete_teams(
        &self,
        team_ids: &[Uuid],
    ) -> PersistenceResult<BulkDeleteResult> {
        let ids = unique_ids(team_ids);
        let mut deleted_ids = Vec::new();
        let mut blocked = Vec::new();
        for team_id in &ids {
            match delete_team(self, *team_id).await {
                Ok(()) => deleted_ids.push(*team_id),
                Err(error) => {
                    let label = entity_label(&self.pool, "team", *team_id)
                        .await?
                        .unwrap_or_else(|| team_id.to_string());
                    blocked.push(BulkDeleteBlockedItem {
                        id: *team_id,
                        label,
                        reason: error.to_string(),
                    });
                }
            }
        }
        Ok(BulkDeleteResult {
            requested_count: ids.len() as u64,
            deleted_ids,
            blocked,
        })
    }
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/deletion/safe_delete.rs",
    r'''use super::delete_write::{write_player_delete, write_team_delete};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use uuid::Uuid;

impl PostgresStore {
    pub async fn delete_player(&self, player_id: Uuid) -> PersistenceResult<()> {
        let check = self.check_entity_deletion("player", player_id).await?;
        if !check.can_permanently_delete {
            return Err(PersistenceError::InvalidState(check.reason));
        }
        write_player_delete(&self.pool, player_id).await
    }
}

pub(crate) async fn delete_team(
    store: &PostgresStore,
    team_id: Uuid,
) -> PersistenceResult<()> {
    let check = store.check_entity_deletion("team", team_id).await?;
    if !check.can_permanently_delete {
        return Err(PersistenceError::InvalidState(check.reason));
    }
    write_team_delete(&store.pool, team_id).await
}
''',
)

write(
    "crates/persistence-postgres/src/adapters/catalog/deletion/delete_write.rs",
    r'''use crate::{write_audit_event, PersistenceError, PersistenceResult};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub(crate) async fn write_player_delete(
    pool: &PgPool,
    player_id: Uuid,
) -> PersistenceResult<()> {
    let mut tx = pool.begin().await?;
    let player_name: String = sqlx::query_scalar(
        "SELECT canonical_name FROM football.players WHERE id = $1 FOR UPDATE",
    )
    .bind(player_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("球员不存在".to_string()))?;

    sqlx::query(
        "DELETE FROM football.external_entity_ids WHERE entity_type = 'player' AND entity_id = $1",
    )
    .bind(player_id)
    .execute(&mut *tx)
    .await?;
    write_audit_event(
        &mut tx,
        "player_deleted",
        "player",
        player_id.to_string(),
        json!({"canonical_name": player_name, "reference_check": "passed"}),
    )
    .await?;
    sqlx::query("DELETE FROM football.players WHERE id = $1")
        .bind(player_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub(crate) async fn write_team_delete(
    pool: &PgPool,
    team_id: Uuid,
) -> PersistenceResult<()> {
    let mut tx = pool.begin().await?;
    let team_name = sqlx::query_scalar::<_, String>(
        "SELECT canonical_name FROM football.teams WHERE id = $1 FOR UPDATE",
    )
    .bind(team_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("球队不存在".to_string()))?;
    let match_count: i64 = sqlx::query_scalar(
        "SELECT count(*)::bigint FROM football.matches WHERE home_team_id=$1 OR away_team_id=$1",
    )
    .bind(team_id)
    .fetch_one(&mut *tx)
    .await?;
    if match_count > 0 {
        return Err(PersistenceError::InvalidState(format!(
            "球队已关联 {match_count} 场比赛，为保留历史赛果不能永久删除"
        )));
    }
    let review_count: i64 = sqlx::query_scalar(
        r#"
        SELECT
            (SELECT count(*)::bigint FROM review.team_match_reviews WHERE team_id=$1)
          + (SELECT count(*)::bigint FROM review.player_match_reviews WHERE team_id=$1)
        "#,
    )
    .bind(team_id)
    .fetch_one(&mut *tx)
    .await?;
    if review_count > 0 {
        return Err(PersistenceError::InvalidState(format!(
            "球队已关联 {review_count} 条球队或球员赛后复盘，为保留历史记录不能永久删除"
        )));
    }
    sqlx::query(
        "DELETE FROM football.external_entity_ids WHERE entity_type='team' AND entity_id=$1",
    )
    .bind(team_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM football.teams WHERE id=$1")
        .bind(team_id)
        .execute(&mut *tx)
        .await?;
    write_audit_event(
        &mut tx,
        "team_deleted",
        "team",
        team_id.to_string(),
        json!({"canonical_name": team_name}),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}
''',
)

mod_path = "crates/persistence-postgres/src/adapters/catalog/deletion/mod.rs"
mod_text = read(mod_path)
mod_text = once(
    mod_text,
    "mod archive;\nmod ids;\nmod preflight;\n",
    "mod archive;\nmod bulk_delete;\nmod delete_write;\nmod ids;\nmod preflight;\nmod safe_delete;\n",
    "deletion module",
)
write(mod_path, mod_text)

player_path = "crates/persistence-postgres/src/player_catalog.rs"
player_text = read(player_path)
player_text = remove_rust_function(
    player_text,
    "    pub async fn delete_player(&self, player_id: Uuid) -> PersistenceResult<()> {",
    "player delete owner",
)
write(player_path, player_text)

lib_path = "crates/persistence-postgres/src/lib.rs"
write(lib_path, once(read(lib_path), "mod team_catalog;\n", "", "team_catalog module"))
(ROOT / "crates/persistence-postgres/src/team_catalog.rs").unlink()

write(
    "scripts/verify-entity-deletion-persistence.mjs",
    r'''import fs from "node:fs";
const read=(p)=>fs.readFileSync(new URL(`../${p}`,import.meta.url),"utf8").replace(/\r\n?/g,"\n");
const exists=(p)=>fs.existsSync(new URL(`../${p}`,import.meta.url));
const req=(v,m)=>{if(!v)throw new Error(m)};
const root="crates/persistence-postgres/src/adapters/catalog/deletion/";
for(const p of [
  "mod.rs","ids.rs","preflight/mod.rs","preflight/check.rs","preflight/labels.rs","preflight/references.rs",
  "archive/mod.rs","archive/bulk.rs","archive/write.rs","bulk_delete.rs","safe_delete.rs","delete_write.rs"
])req(exists(root+p),`R6-09 owner missing: ${p}`);
const entity=read("crates/persistence-postgres/src/entity_catalog.rs");
const player=read("crates/persistence-postgres/src/player_catalog.rs");
const lib=read("crates/persistence-postgres/src/lib.rs");
const check=read(root+"preflight/check.rs");
const refs=read(root+"preflight/references.rs");
const archive=read(root+"archive/bulk.rs")+read(root+"archive/write.rs");
const bulk=read(root+"bulk_delete.rs");
const safeDelete=read(root+"safe_delete.rs");
const deleteWrite=read(root+"delete_write.rs");
for(const n of ["check_entity_deletion","bulk_archive_entities","team_reference_counts","player_reference_counts","coach_reference_counts"])req(!entity.includes(n),`legacy entity owner remains: ${n}`);
req(!exists("crates/persistence-postgres/src/team_catalog.rs")&&!lib.includes("mod team_catalog;"),"legacy team deletion owner remains");
req(!player.includes("pub async fn delete_player"),"legacy player delete owner remains");
req(check.includes("pub async fn check_entity_deletion")&&archive.includes("pub async fn bulk_archive_entities"),"AT1 owners missing");
for(const n of ["bulk_delete_players","bulk_delete_teams"])req((bulk.match(new RegExp(`pub async fn ${n}\\b`,"g"))||[]).length===1,`AT2 bulk owner invalid: ${n}`);
req((safeDelete.match(/pub async fn delete_player\b/g)||[]).length===1,"AT2 delete_player owner invalid");
for(const p of ["preflight/check.rs","archive/bulk.rs","bulk_delete.rs","safe_delete.rs"]){const s=read(root+p);req(!s.includes("sqlx::")&&!s.includes("SELECT ")&&!s.includes("DELETE FROM "),`coordinator owns SQL: ${p}`)}
for(const r of ["matches","lineups","player_team_periods","team_coach_periods","player_availability","formation_usage","team_tactical_observations","team_ability_observations","substitutions","match_events","team_match_reviews","player_match_reviews","player_match_observations"])req(refs.includes(`"${r}"`),`reference count missing: ${r}`);
req(check.includes("can_permanently_delete: total == 0")&&check.includes("只允许归档"),"preflight semantics changed");
req(archive.includes("manual_bulk_archive")&&archive.includes("_archived"),"archive audit semantics changed");
for(const token of ["check_entity_deletion(\"team\"","check_entity_deletion(\"player\""])req(safeDelete.includes(token),`safe delete lost preflight: ${token}`);
for(const token of ["team_deleted","player_deleted","FOR UPDATE","DELETE FROM football.external_entity_ids","tx.commit().await?"])req(deleteWrite.includes(token),`permanent delete write contract missing: ${token}`);
for(const token of ["球队不存在","球员不存在","球队已关联 {match_count} 场比赛，为保留历史赛果不能永久删除","球队已关联 {review_count} 条球队或球员赛后复盘，为保留历史记录不能永久删除"])req(deleteWrite.includes(token),`permanent delete error changed: ${token}`);
for(const table of ["football.player_team_periods","football.team_coach_periods","football.player_availability","feature.formation_usage_observations","feature.team_tactical_observations","feature.team_ability_observations"])req(!deleteWrite.includes(`DELETE FROM ${table}`),`safe permanent delete became destructive: ${table}`);
req(read("crates/persistence-postgres/src/team_force_delete.rs").includes("pub async fn force_delete_team"),"AT3 force delete moved early");
console.log("R6-09 AT1+AT2 deletion preflight/archive/safe-permanent-delete ownership verified.");
''',
)

path = "scripts/verify-entity-deletion.mjs"
text = read(path)
text = once(
    text,
    'const teamPersistence = text("crates/persistence-postgres/src/team_catalog.rs");',
    'const teamPersistence = [\n  text("crates/persistence-postgres/src/adapters/catalog/deletion/safe_delete.rs"),\n  text("crates/persistence-postgres/src/adapters/catalog/deletion/delete_write.rs"),\n].join("\\n");',
    "entity deletion verifier owner",
)
write(path, text)

path = "scripts/verify-team-player-management.mjs"
text = read(path)
text = once(
    text,
    'const persistenceCatalog = text("crates/persistence-postgres/src/team_catalog.rs");',
    'const deletionPersistence = [\n  text("crates/persistence-postgres/src/adapters/catalog/deletion/bulk_delete.rs"),\n  text("crates/persistence-postgres/src/adapters/catalog/deletion/safe_delete.rs"),\n  text("crates/persistence-postgres/src/adapters/catalog/deletion/delete_write.rs"),\n].join("\\n");\nconst persistenceCatalog = deletionPersistence;',
    "team-player management team deletion owner",
)
text = once(
    text,
    'const playerPersistence = text("crates/persistence-postgres/src/player_catalog.rs");',
    "const playerPersistence = deletionPersistence;",
    "team-player management player deletion owner",
)
write(path, text)

path = "scripts/verify-team-directory-detail.mjs"
text = read(path)
text = once(
    text,
    'const legacyTeam = read("crates/persistence-postgres/src/team_catalog.rs");',
    'const legacyTeam = exists("crates/persistence-postgres/src/team_catalog.rs") ? read("crates/persistence-postgres/src/team_catalog.rs") : "";\nconst r609Deletion = read("crates/persistence-postgres/src/adapters/catalog/deletion/bulk_delete.rs");',
    "team directory legacy owner",
)
text = once(
    text,
    'check(legacyTeam.includes("pub async fn bulk_delete_teams"), "R6-01/R6-02 must not migrate R6-09 deletion early");',
    'check(!exists("crates/persistence-postgres/src/team_catalog.rs") && r609Deletion.includes("pub async fn bulk_delete_teams"), "R6-09 permanent team deletion owner advancement missing");',
    "team directory R6-09 owner advancement",
)
text = text.replace(
    "and R6-02 owner advancement without early R6-09 migration.",
    "and R6-02/R6-09 owner advancement.",
)
write(path, text)

path = "scripts/verify-team-names-profiles.mjs"
text = read(path)
text = once(
    text,
    'const legacy = read("crates/persistence-postgres/src/team_catalog.rs");',
    'const legacy = exists("crates/persistence-postgres/src/team_catalog.rs") ? read("crates/persistence-postgres/src/team_catalog.rs") : "";\nconst r609Deletion = read("crates/persistence-postgres/src/adapters/catalog/deletion/bulk_delete.rs");',
    "team names legacy owner",
)
text = once(
    text,
    'check(legacy.includes("pub async fn bulk_delete_teams"), "R6-02 must not migrate R6-09 team deletion early");\ncheck(legacy.includes("pub async fn bulk_delete_players"), "R6-02 must not disturb existing bulk player deletion path");',
    'check(!exists("crates/persistence-postgres/src/team_catalog.rs"), "R6-09 must remove legacy team_catalog deletion owner");\ncheck(r609Deletion.includes("pub async fn bulk_delete_teams") && r609Deletion.includes("pub async fn bulk_delete_players"), "R6-09 bulk permanent deletion owner advancement missing");',
    "team names R6-09 owner advancement",
)
write(path, text)

path = "scripts/verify-player-directory-detail.mjs"
text = read(path)
text = once(
    text,
    "const legacy = read('crates/persistence-postgres/src/player_catalog.rs');",
    "const legacy = read('crates/persistence-postgres/src/player_catalog.rs');\nconst r609Deletion = read('crates/persistence-postgres/src/adapters/catalog/deletion/safe_delete.rs');",
    "player directory deletion owner",
)
old = """for (const retained of [
  'delete_player',
]) {
  if (!legacy.includes(`pub async fn ${retained}`)) {
    throw new Error(`Later R6 owner moved prematurely or disappeared: ${retained}`);
  }
}"""
new = """if (legacy.includes('pub async fn delete_player')) {
  throw new Error('R6-09 legacy player_catalog.rs still owns delete_player');
}
if (!r609Deletion.includes('pub async fn delete_player')) {
  throw new Error('R6-09 safe permanent deletion owner missing delete_player');
}"""
text = once(text, old, new, "player directory R6-09 owner advancement")
write(path, text)

path = "scripts/verify-entity-resource-center.mjs"
text = once(
    read(path),
    'const teamPersistence = read("crates/persistence-postgres/src/team_catalog.rs");\n',
    "",
    "entity resource unused legacy owner",
)
write(path, text)

# Stop before any build if another historical verifier still depends on the deleted owner.
stale = []
allowed_team_catalog = {
    "scripts/verify-team-directory-detail.mjs",
    "scripts/verify-team-names-profiles.mjs",
}
for path in sorted(ROOT.glob("scripts/**/*.mjs")):
    source = path.read_text(encoding="utf-8")
    relative = path.relative_to(ROOT).as_posix()
    if "crates/persistence-postgres/src/team_catalog.rs" in source and relative not in allowed_team_catalog:
        stale.append(f"{relative}: direct legacy team_catalog.rs read remains")
    if (
        "crates/persistence-postgres/src/player_catalog.rs" in source
        and any(token in source for token in ("delete_player", "player_deleted", "bulk_delete_players"))
        and "adapters/catalog/deletion/" not in source
    ):
        stale.append(f"{relative}: player deletion verifier remains bound only to player_catalog.rs")
need(not stale, "R6-09 AT2 stale verifier audit failed:\n- " + "\n- ".join(stale))
