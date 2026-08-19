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
for(const token of ["球队不存在","球员不存在","球队已关联 {match_count} 场比赛，为保留历史赛果不能永久删除","球队已关联 {review_count} 条球队或球员赛后复盘，为保留历史记录不能永久删除"])req(deleteWrite.includes(token),`permanent delete error changed: ${token}`);
for(const table of ["football.player_team_periods","football.team_coach_periods","football.player_availability","feature.formation_usage_observations","feature.team_tactical_observations","feature.team_ability_observations"])req(!deleteWrite.includes(`DELETE FROM ${table}`),`safe permanent delete became destructive: ${table}`);
req(!exists("crates/persistence-postgres/src/team_force_delete.rs")&&!lib.includes("mod team_force_delete;"),"legacy force-delete owner remains");
req(read(root+"mod.rs").includes("mod force_delete;"),"AT3 force-delete adapter module missing");
req(forceOperation.includes("pub async fn preview_force_delete_team")&&forceOperation.includes("pub async fn force_delete_team")&&forceOperation.includes("request.confirmation_text.trim() != label"),"AT3 force-delete coordinator contract missing");
req(forceTargets.includes("FOR UPDATE")&&forceTargets.includes("CREATE TEMP TABLE purge_matches")&&forceTargets.includes("pub(super) async fn temp_ids"),"AT3 target-set owner incomplete");
req(forceCounts.includes("UNION ALL SELECT 'match_events'")&&forceCounts.includes("team_lineup_preset_members"),"AT3 count owner incomplete");
req(forceExecute.includes("set_config('football.force_purge', 'on', true)")&&forceExecute.includes("DELETE FROM football.teams WHERE id=$1"),"AT3 execution owner incomplete");
console.log("R6-09 AT1+AT2+AT3 deletion preflight/archive/safe-delete/force-delete ownership verified.");
