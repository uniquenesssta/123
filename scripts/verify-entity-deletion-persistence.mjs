import fs from "node:fs";
const read=(p)=>fs.readFileSync(new URL(`../${p}`,import.meta.url),"utf8").replace(/\r\n?/g,"\n");
const exists=(p)=>fs.existsSync(new URL(`../${p}`,import.meta.url));
const req=(v,m)=>{if(!v)throw new Error(m)};
const root="crates/persistence-postgres/src/adapters/catalog/deletion/";
for(const p of [
  "mod.rs","ids.rs","preflight/mod.rs","preflight/check.rs","preflight/labels.rs","preflight/references.rs",
  "archive/mod.rs","archive/bulk.rs","archive/write.rs","bulk_delete.rs","safe_delete.rs","delete_write.rs",
  "force_delete/mod.rs","force_delete/operation.rs","force_delete/targets.rs","force_delete/counts.rs","force_delete/execute.rs"
])req(exists(root+p),`R6-09 owner missing: ${p}`);
const player=read("crates/persistence-postgres/src/player_catalog.rs");
const lib=read("crates/persistence-postgres/src/lib.rs");
const check=read(root+"preflight/check.rs");
const refs=read(root+"preflight/references.rs");
const archive=read(root+"archive/bulk.rs")+read(root+"archive/write.rs");
const bulk=read(root+"bulk_delete.rs");
const safeDelete=read(root+"safe_delete.rs");
const deleteWrite=read(root+"delete_write.rs");
const forceOperation=read(root+"force_delete/operation.rs");
const forceTargets=read(root+"force_delete/targets.rs");
const forceCounts=read(root+"force_delete/counts.rs");
const forceExecute=read(root+"force_delete/execute.rs");
req(!exists("crates/persistence-postgres/src/entity_catalog.rs")&&!lib.includes("mod entity_catalog;"),"legacy entity catalog owner remains");
req(!exists("crates/persistence-postgres/src/team_catalog.rs")&&!lib.includes("mod team_catalog;"),"legacy team deletion owner remains");
req(!player.includes("pub async fn delete_player"),"legacy player delete owner remains");
req(check.includes("pub async fn check_entity_deletion")&&archive.includes("pub async fn bulk_archive_entities"),"AT1 owners missing");
for(const n of ["bulk_delete_players","bulk_delete_teams"])req((bulk.match(new RegExp(`pub async fn ${n}\\b`,"g"))||[]).length===1,`AT2 bulk owner invalid: ${n}`);
req((safeDelete.match(/pub async fn delete_player\b/g)||[]).length===1,"AT2 delete_player owner invalid");
for(const p of ["preflight/check.rs","archive/bulk.rs","bulk_delete.rs","safe_delete.rs","force_delete/operation.rs"]){const s=read(root+p);req(!s.includes("sqlx::")&&!s.includes("SELECT ")&&!s.includes("DELETE FROM ")&&!s.includes("CREATE TEMP TABLE"),`coordinator owns SQL: ${p}`)}
for(const r of ["matches","lineups","player_team_periods","team_coach_periods","player_availability","formation_usage","team_tactical_observations","team_ability_observations","substitutions","match_events","team_match_reviews","player_match_reviews","player_match_observations"])req(refs.includes(`"${r}"`),`reference count missing: ${r}`);
req(check.includes("can_permanently_delete: total == 0")&&check.includes("只允许归档"),"preflight semantics changed");
req(archive.includes("manual_bulk_archive")&&archive.includes("_archived"),"archive audit semantics changed");
for(const token of ["check_entity_deletion(\"team\"","check_entity_deletion(\"player\""])req(safeDelete.includes(token),`safe delete lost preflight: ${token}`);
for(const token of ["team_deleted","player_deleted","FOR UPDATE","DELETE FROM football.external_entity_ids","tx.commit().await?"])req(deleteWrite.includes(token),`permanent delete write contract missing: ${token}`);
for(const token of ["球队不存在","球员不存在"])req(deleteWrite.includes(token),`permanent delete missing-entity error changed: ${token}`);
for(const [name,kind] of [["write_player_delete","player"],["write_team_delete","team"]]) {
  const start=deleteWrite.indexOf(`async fn ${name}(`);
  const end=deleteWrite.indexOf("async fn ",start+10);
  const body=deleteWrite.slice(start,end<0?undefined:end);
  const isolation=body.indexOf("SET TRANSACTION ISOLATION LEVEL READ COMMITTED");
  const lock=body.indexOf("FOR UPDATE");
  const recheck=body.indexOf(`ensure_no_references(&mut tx, "${kind}"`);
  const mutation=body.indexOf("DELETE FROM football.external_entity_ids");
  req(isolation>=0 && lock>isolation && recheck>lock && mutation>recheck,`safe delete lock/recheck/mutation order invalid: ${name}`);
}
req(refs.includes("connection: &mut sqlx::PgConnection") && refs.includes(".fetch_one(&mut *connection)"),"reference counts cannot use caller transaction");
req(deleteWrite.includes("player_reference_counts(connection, entity_id).await?") && deleteWrite.includes("team_reference_counts(connection, entity_id).await?"),"transaction recheck omits full player/team references");
req(deleteWrite.includes("check_from_references(entity_type, entity_id") && check.includes("pub(crate) fn check_from_references("),"UI and delete lost shared reference decision");
req(deleteWrite.includes("if !check.can_permanently_delete") && deleteWrite.includes("InvalidState(check.reason)"),"transaction reference rejection missing");
for(const relation of ["team_lineup_presets","team_lineup_preset_members","team_season_memberships","dynamic_tag_opponents","ability_observations","ability_snapshots","dynamic_tags","match_contributions","ability_candidates"])req(refs.includes(`"${relation}"`),`protected history relation missing: ${relation}`);
const permanentContract=read("crates/persistence-postgres/tests/entity_permanent_delete_repository_contract.rs");
for(const token of ["safe_permanent_delete_contract_is_preserved","safe_delete_rechecks_concurrent_history_after_parent_lock","pg_blocking_pids","FOR UPDATE","uncommitted dynamic tag","uncommitted preset","commit_reference","rollback concurrent history","external binding retained","delete audit count","SELECT current_database()","cleanup safe delete fixture"])
  req(permanentContract.includes(token),`safe delete contract coverage missing: ${token}`);
for(const table of ["football.player_team_periods","football.team_coach_periods","football.player_availability","feature.formation_usage_observations","feature.team_tactical_observations","feature.team_ability_observations"])req(!deleteWrite.includes(`DELETE FROM ${table}`),`safe permanent delete became destructive: ${table}`);
req(!exists("crates/persistence-postgres/src/team_force_delete.rs")&&!lib.includes("mod team_force_delete;"),"legacy force-delete owner remains");
req(read(root+"mod.rs").includes("mod force_delete;"),"AT3 force-delete adapter module missing");
req(forceOperation.includes("pub async fn preview_force_delete_team")&&forceOperation.includes("pub async fn force_delete_team")&&forceOperation.includes("request.confirmation_text.trim() != label"),"AT3 force-delete coordinator contract missing");
req(forceTargets.includes("FOR UPDATE")&&forceTargets.includes("CREATE TEMP TABLE purge_matches")&&forceTargets.includes("pub(super) async fn temp_ids"),"AT3 target-set owner incomplete");
req(forceCounts.includes("UNION ALL SELECT 'match_events'")&&forceCounts.includes("team_lineup_preset_members"),"AT3 count owner incomplete");
req(forceExecute.includes("set_config('football.force_purge', 'on', true)")&&forceExecute.includes("DELETE FROM football.teams WHERE id=$1"),"AT3 execution owner incomplete");
console.log("R6-09 deletion ownership and R7-03 transactional history protection verified.");
