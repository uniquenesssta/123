import fs from "node:fs";
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
const matchPrediction = read("crates/persistence-postgres/src/match_prediction.rs");
const lineupChain = read("crates/persistence-postgres/src/lineup_chain.rs");
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
requireTrue(!matchPrediction.includes("match_exchange::match_record_from_row"), "match_prediction.rs 仍依赖旧 Match Exchange mapper");
requireTrue(matchPrediction.includes("adapters::matches::catalog::match_record_from_row"), "match_prediction.rs 未切换到 Match Catalog mapper");
requireTrue(!lineupChain.includes("read_match_exchange("), "lineup_chain.rs 仍依赖旧 read_match_exchange");
requireTrue(lineupChain.includes("self.read_match("), "lineup_chain.rs 未切换到 Match Catalog read owner");
requireTrue(playerCatalog.includes("pub async fn create_lineup") && playerCatalog.includes("pub async fn create_lineup_pair"), "R7-01 越界迁移了 Lineup owner");

console.log("R7-01 Match Catalog ownership verification passed.");
