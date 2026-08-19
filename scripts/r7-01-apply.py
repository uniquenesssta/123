from __future__ import annotations

from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "crates/persistence-postgres/src"
BRANCH = "rewrite/r7-match-lineup-workbook-persistence"


def fail(message: str) -> None:
    raise SystemExit(f"R7-01 apply aborted: {message}")


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


if git("branch", "--show-current") != BRANCH:
    fail(f"expected branch {BRANCH}")
if git("status", "--porcelain"):
    fail("working tree must be clean before owner switch")

player_path = SRC / "player_catalog.rs"
exchange_path = SRC / "match_exchange.rs"
adapters_path = SRC / "adapters/mod.rs"
target = SRC / "adapters/matches/catalog"
if not player_path.exists() or not exchange_path.exists() or not adapters_path.exists():
    fail("required R7-01 source files are missing")
if target.exists():
    fail("adapters/matches/catalog already exists; refusing duplicate implementation")

player = player_path.read_text(encoding="utf-8")
exchange = exchange_path.read_text(encoding="utf-8")
adapters = adapters_path.read_text(encoding="utf-8")

# --- Extract exact existing Match Catalog behavior before removing old ownership. ---
method_create_start = player.index("    pub async fn create_match")
method_delete_start = player.index("    pub async fn delete_match", method_create_start)
method_list_start = player.index("    pub async fn list_upcoming_matches", method_delete_start)
method_lineup_start = player.index("    pub async fn create_lineup", method_list_start)

create_method = player[method_create_start:method_delete_start]
delete_method = player[method_delete_start:method_list_start]
list_methods = player[method_list_start:method_lineup_start]

scope_start = player.index("async fn resolve_match_scope_draft")
status_start = player.index("fn match_status", scope_start)
lineup_validation_start = player.index("struct ValidatedLineupDraft", status_start)
scope_block = player[scope_start:status_start]
status_block = player[status_start:lineup_validation_start]

match_mapper_start = player.index("fn match_record_from_row")
lineup_mapper_start = player.index("pub(crate) fn lineup_player_from_row", match_mapper_start)
match_mapper_block = player[match_mapper_start:lineup_mapper_start]

read_start = exchange.index("    pub async fn read_match_exchange")
read_end = exchange.index("    async fn match_player_references", read_start)
read_method = exchange[read_start:read_end].replace(
    "pub async fn read_match_exchange", "pub async fn read_match", 1
)

# --- Create responsibility-owned Match Catalog modules from the exact old logic. ---
target.mkdir(parents=True)
(target.parent / "mod.rs").write_text("pub(crate) mod catalog;\n", encoding="utf-8")
(target / "mod.rs").write_text(
    "mod create;\nmod delete;\nmod list;\nmod mapping;\nmod read;\nmod scope;\n",
    encoding="utf-8",
)
(target / "create.rs").write_text(
    "use super::{mapping::match_record_from_row, scope::{resolve_match_scope_draft, validate_match_scope}};\n"
    "use crate::{PersistenceError, PersistenceResult, PostgresStore};\n"
    "use football_domain::{MatchDraft, MatchRecord};\n"
    "use uuid::Uuid;\n\n"
    "impl PostgresStore {\n" + create_method + "}\n",
    encoding="utf-8",
)
(target / "delete.rs").write_text(
    "use crate::{PersistenceError, PersistenceResult, PostgresStore};\n"
    "use serde_json::json;\n"
    "use uuid::Uuid;\n\n"
    "impl PostgresStore {\n" + delete_method + "}\n",
    encoding="utf-8",
)
(target / "list.rs").write_text(
    "use super::mapping::match_record_from_row;\n"
    "use crate::{PersistenceResult, PostgresStore};\n"
    "use football_domain::MatchRecord;\n\n"
    "impl PostgresStore {\n" + list_methods + "}\n",
    encoding="utf-8",
)
(target / "scope.rs").write_text(
    "use crate::{PersistenceError, PersistenceResult};\n"
    "use chrono::{Datelike, NaiveDate};\n"
    "use football_domain::MatchDraft;\n"
    "use sqlx::Row;\n"
    "use uuid::Uuid;\n\n"
    + scope_block.replace(
        "async fn resolve_match_scope_draft", "pub(super) async fn resolve_match_scope_draft", 1
    ).replace(
        "async fn validate_match_scope", "pub(super) async fn validate_match_scope", 1
    ),
    encoding="utf-8",
)
(target / "mapping.rs").write_text(
    "use crate::{PersistenceError, PersistenceResult};\n"
    "use football_domain::{MatchRecord, MatchStatus};\n"
    "use sqlx::Row;\n\n"
    + status_block
    + match_mapper_block.replace(
        "fn match_record_from_row", "pub(super) fn match_record_from_row", 1
    )
    + """
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persisted_match_status_round_trips() {
        assert!(matches!(match_status("scheduled").unwrap(), MatchStatus::Scheduled));
        assert!(matches!(match_status("live").unwrap(), MatchStatus::Live));
        assert!(matches!(match_status("finished").unwrap(), MatchStatus::Finished));
        assert!(matches!(match_status("postponed").unwrap(), MatchStatus::Postponed));
        assert!(matches!(match_status("cancelled").unwrap(), MatchStatus::Cancelled));
        assert!(match_status("unknown").is_err());
    }
}
""",
    encoding="utf-8",
)
(target / "read.rs").write_text(
    "use super::mapping::match_record_from_row;\n"
    "use crate::{PersistenceResult, PostgresStore};\n"
    "use football_domain::MatchRecord;\n"
    "use uuid::Uuid;\n\n"
    "impl PostgresStore {\n" + read_method + "}\n",
    encoding="utf-8",
)

# --- Remove Match ownership from player_catalog.rs, leaving Lineup + reference aggregation only. ---
player = player[:method_create_start] + player[method_lineup_start:]
# Re-find helper offsets after method removal.
scope_start = player.index("async fn resolve_match_scope_draft")
lineup_validation_start = player.index("struct ValidatedLineupDraft", scope_start)
player = player[:scope_start] + player[lineup_validation_start:]
match_mapper_start = player.index("fn match_record_from_row")
lineup_mapper_start = player.index("pub(crate) fn lineup_player_from_row", match_mapper_start)
player = player[:match_mapper_start] + player[lineup_mapper_start:]
player = player.replace("use chrono::{Datelike, NaiveDate};\n", "", 1)
player = player.replace(
    "    LineupPlayerRecord, LineupRecord, LineupType, MatchDraft, MatchRecord, MatchStatus,\n    PlayerCatalogReferenceData,\n",
    "    LineupPlayerRecord, LineupRecord, LineupType, PlayerCatalogReferenceData,\n",
    1,
)
player = player.replace(
    "        assert_eq!(match_status(\"finished\").unwrap(), MatchStatus::Finished);\n",
    "",
    1,
)
player_path.write_text(player, encoding="utf-8")

# --- Make match_exchange a Workbook/Exchange caller of the new Match Catalog owner. ---
wrapper_start = exchange.index("    /// Reads one managed match through the persistence crate's public read API.")
wrapper_end = exchange.index("    pub async fn match_lineup_export_data", wrapper_start)
exchange = exchange[:wrapper_start] + exchange[wrapper_end:]
exchange = exchange.replace("self.read_match_exchange(", "self.read_match(")
read_start = exchange.index("    pub async fn read_match_exchange")
read_end = exchange.index("    async fn match_player_references", read_start)
exchange = exchange[:read_start] + exchange[read_end:]
status_mapper_start = exchange.index("fn match_status_from_str")
exchange = exchange[:status_mapper_start].rstrip() + "\n"
exchange = exchange.replace("    MatchLineupExportData, MatchLineupPlayerReference, MatchRecord, MatchStatus,\n", "    MatchLineupExportData, MatchLineupPlayerReference,\n", 1)
exchange_path.write_text(exchange, encoding="utf-8")

# Register the new R7 adapter namespace.
if "pub(crate) mod matches;" in adapters:
    fail("adapters/mod.rs already registers matches unexpectedly")
adapters = adapters.replace("pub(crate) mod competition;\n", "pub(crate) mod competition;\npub(crate) mod matches;\n", 1)
adapters_path.write_text(adapters, encoding="utf-8")

# Static ownership verifier: rejects duplicate/legacy Match Catalog owners.
verify = ROOT / "scripts/verify-r7-match-catalog.mjs"
verify.write_text(r'''import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const requireTrue = (value, message) => { if (!value) throw new Error(message); };

const base = "crates/persistence-postgres/src/adapters/matches/catalog";
const adapters = read("crates/persistence-postgres/src/adapters/mod.rs");
const playerCatalog = read("crates/persistence-postgres/src/player_catalog.rs");
const exchange = read("crates/persistence-postgres/src/match_exchange.rs");
const create = read(`${base}/create.rs`);
const remove = read(`${base}/delete.rs`);
const list = read(`${base}/list.rs`);
const readOwner = read(`${base}/read.rs`);
const scope = read(`${base}/scope.rs`);
const mapping = read(`${base}/mapping.rs`);

for (const file of ["mod.rs", "create.rs", "delete.rs", "list.rs", "read.rs", "scope.rs", "mapping.rs"]) {
  requireTrue(exists(`${base}/${file}`), `R7-01 缺少 Match Catalog owner 文件: ${file}`);
}
requireTrue(adapters.includes("pub(crate) mod matches;"), "adapters 未注册 matches owner");
requireTrue(create.includes("pub async fn create_match") && create.includes("ON CONFLICT (external_key) DO UPDATE"), "create_match 未迁入唯一 owner");
requireTrue(remove.includes("pub async fn delete_match") && remove.includes("research.runs") && remove.includes("platform.p4_freeze_tasks") && remove.includes("review.postmatch_settlements") && remove.includes("match_deleted"), "delete_match 保护/审计链不完整");
requireTrue(list.includes("pub async fn list_upcoming_matches") && list.includes("pub async fn list_managed_matches") && list.includes("ORDER BY fixture.kickoff_time") && list.includes("ORDER BY fixture.kickoff_time DESC"), "Match list owner/排序语义不完整");
requireTrue(readOwner.includes("pub async fn read_match") && readOwner.includes("WHERE match.id=$1"), "read_match 未迁入唯一 owner");
requireTrue(scope.includes("resolve_match_scope_draft") && scope.includes("validate_match_scope") && scope.includes("season_pattern") && scope.includes("competition_timezone"), "Match scope 解析/校验 owner 不完整");
requireTrue(mapping.includes("match_record_from_row") && mapping.includes("MatchStatus::Scheduled") && mapping.includes("MatchStatus::Cancelled"), "MatchRecord mapping owner 不完整");

for (const forbidden of ["pub async fn create_match", "pub async fn delete_match", "pub async fn list_upcoming_matches", "pub async fn list_managed_matches", "resolve_match_scope_draft", "validate_match_scope", "fn match_record_from_row"]) {
  requireTrue(!playerCatalog.includes(forbidden), `player_catalog.rs 仍持有 Match Catalog 职责: ${forbidden}`);
}
requireTrue(!exchange.includes("read_match_exchange"), "match_exchange.rs 仍持有重复 read_match_exchange owner");
requireTrue(!exchange.includes("fn match_record_from_row"), "match_exchange.rs 仍持有重复 MatchRecord mapper");
requireTrue(exchange.includes("self.read_match("), "match_exchange.rs 未切换到 Match Catalog read owner");
requireTrue(playerCatalog.includes("pub async fn create_lineup") && playerCatalog.includes("pub async fn create_lineup_pair"), "R7-01 越界迁移了 Lineup owner");

console.log("R7-01 Match Catalog ownership verification passed.");
''', encoding="utf-8")

# Focused real PostgreSQL contract for create/upsert/read/list/delete behavior.
contract = ROOT / "crates/persistence-postgres/tests/match_catalog_repository_contract.rs"
contract.write_text(r'''use chrono::{Duration, Utc};
use football_domain::{MatchDraft, MatchStatus, TeamDraft};
use football_persistence_postgres::{DatabaseOptions, PostgresStore};
use serde_json::json;
use uuid::Uuid;

const DATABASE_ENV: &str = "FOOTBALL_TEST_DATABASE_URL";

#[tokio::test]
#[ignore = "需要专用且可写的 PostgreSQL 测试数据库；设置 FOOTBALL_TEST_DATABASE_URL 后显式运行"]
async fn match_catalog_contract_is_preserved() {
    let connection_url = std::env::var(DATABASE_ENV).unwrap_or_else(|_| {
        panic!("运行 R7-01 契约测试前必须设置 {DATABASE_ENV}，并指向专用、可写的 PostgreSQL 测试数据库")
    });
    let store = PostgresStore::connect(&DatabaseOptions {
        connection_url,
        max_connections: 4,
        connect_timeout_seconds: 10,
    })
    .await
    .expect("连接 R7-01 PostgreSQL 测试数据库");
    store.migrate().await.expect("执行数据库迁移");

    let token = Uuid::new_v4().simple().to_string();
    let home = store
        .create_team(&TeamDraft {
            canonical_name: format!("R7 Home {token}"),
            country_code: Some("ZZ".to_string()),
            metadata: json!({"contract": "r7-01"}),
        })
        .await
        .expect("创建主队");
    let away = store
        .create_team(&TeamDraft {
            canonical_name: format!("R7 Away {token}"),
            country_code: Some("ZZ".to_string()),
            metadata: json!({"contract": "r7-01"}),
        })
        .await
        .expect("创建客队");

    let kickoff = Utc::now() + Duration::days(2);
    let created = store
        .create_match(&MatchDraft {
            external_key: String::new(),
            competition_id: None,
            season_id: None,
            stage_id: None,
            round_id: None,
            home_team_id: home.id,
            away_team_id: away.id,
            kickoff_time: kickoff,
            status: MatchStatus::Scheduled,
            venue: Some("  R7 Test Ground  ".to_string()),
            metadata: json!({"contract": "r7-01", "phase": "create"}),
        })
        .await
        .expect("创建比赛");
    assert!(created.external_key.starts_with("MATCH-"));
    assert_eq!(created.home_team_id, home.id);
    assert_eq!(created.away_team_id, away.id);
    assert_eq!(created.venue.as_deref(), Some("R7 Test Ground"));
    assert!(matches!(created.status, MatchStatus::Scheduled));

    let read = store.read_match(created.id).await.expect("读取单场比赛");
    assert_eq!(read.id, created.id);
    assert_eq!(read.external_key, created.external_key);

    let upcoming = store.list_upcoming_matches(250).await.expect("读取 upcoming matches");
    assert!(upcoming.iter().any(|item| item.id == created.id));
    let managed = store.list_managed_matches(500).await.expect("读取 managed matches");
    assert!(managed.iter().any(|item| item.id == created.id));

    let updated = store
        .create_match(&MatchDraft {
            external_key: created.external_key.clone(),
            competition_id: None,
            season_id: None,
            stage_id: None,
            round_id: None,
            home_team_id: home.id,
            away_team_id: away.id,
            kickoff_time: kickoff + Duration::hours(1),
            status: MatchStatus::Postponed,
            venue: Some(" Updated Ground ".to_string()),
            metadata: json!({"contract": "r7-01", "phase": "upsert"}),
        })
        .await
        .expect("按 external_key 更新比赛");
    assert_eq!(updated.id, created.id);
    assert!(matches!(updated.status, MatchStatus::Postponed));
    assert_eq!(updated.venue.as_deref(), Some("Updated Ground"));

    let bad_sides = store
        .create_match(&MatchDraft {
            external_key: format!("R7-BAD-{token}"),
            competition_id: None,
            season_id: None,
            stage_id: None,
            round_id: None,
            home_team_id: home.id,
            away_team_id: home.id,
            kickoff_time: kickoff,
            status: MatchStatus::Scheduled,
            venue: None,
            metadata: json!({}),
        })
        .await;
    assert!(bad_sides.is_err(), "同队作为主客队必须被拒绝");

    store.delete_match(created.id).await.expect("删除未受保护比赛");
    assert!(store.read_match(created.id).await.is_err());
    store.close().await;
}
''', encoding="utf-8")

# Mark the current node as VERIFYING; completion record is intentionally deferred until gates pass.
stage = ROOT / "docs/modular-rewrite/R07-match-lineup-workbook-persistence/README.md"
stage_text = stage.read_text(encoding="utf-8")
if "| R7-01 | Match Catalog | READY |" not in stage_text:
    fail("R7 stage index no longer has R7-01 READY")
stage_text = stage_text.replace("| R7-01 | Match Catalog | READY |", "| R7-01 | Match Catalog | VERIFYING |", 1)
stage_text += "\n## R7-01 当前验证状态\n\n- Match Catalog owner 已切换到 `adapters/matches/catalog/`；节点保持 `VERIFYING`，等待最小门禁、真实 PostgreSQL contract 与阶段回归。\n"
stage.write_text(stage_text, encoding="utf-8")

print("R7-01 Match Catalog owner switch prepared successfully.")
print("Next: cargo fmt --all, static ownership gate, Rust checks, focused PostgreSQL contract, then stage regression.")
