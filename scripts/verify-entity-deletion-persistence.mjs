import fs from "node:fs";
const read=(p)=>fs.readFileSync(new URL(`../${p}`,import.meta.url),"utf8").replace(/\r\n?/g,"\n");
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
