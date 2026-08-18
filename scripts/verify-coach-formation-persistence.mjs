import fs from "node:fs";
const read = (p) => fs.readFileSync(new URL(`../${p}`, import.meta.url), "utf8").replace(/\r\n?/g, "\n");
const exists = (p) => fs.existsSync(new URL(`../${p}`, import.meta.url));
const req = (ok, msg) => { if (!ok) throw new Error(msg); };
const entity = read("crates/persistence-postgres/src/entity_catalog.rs");
const lib = read("crates/persistence-postgres/src/lib.rs");
const catalog = read("crates/persistence-postgres/src/adapters/catalog/mod.rs");
const coachRoot = "crates/persistence-postgres/src/adapters/catalog/coaches/";
const formationRoot = "crates/persistence-postgres/src/adapters/catalog/formations/";
const files = [];
const walk = (base, rel="") => { for (const e of fs.readdirSync(new URL(`../${base}${rel}`, import.meta.url), {withFileTypes:true})) e.isDirectory() ? walk(base, `${rel}${e.name}/`) : files.push(`${base}${rel}${e.name}`); };
walk(coachRoot); walk(formationRoot);
const all = files.filter(p => p.endsWith(".rs")).map(read).join("\n");
req(catalog.includes("mod coaches;") && catalog.includes("mod formations;"), "R6-07 catalog modules missing");
req(!exists("crates/persistence-postgres/src/formation_catalog.rs") && !lib.includes("mod formation_catalog;"), "legacy formation owner remains");
for (const name of ["create_coach","list_coaches","read_coach","add_coach_name","add_team_coach_period"]) {
  req((all.match(new RegExp(`pub async fn ${name}\\b`, "g")) || []).length === 1, `coach owner count invalid: ${name}`);
  req(!entity.includes(`pub async fn ${name}`), `coach legacy owner remains: ${name}`);
}
for (const name of ["list_formations","save_formation_usage_distribution","list_formation_usage_distributions","resolve_formation_distribution"]) req((all.match(new RegExp(`pub async fn ${name}\\b`, "g")) || []).length === 1, `formation owner count invalid: ${name}`);
req(!entity.includes("pub async fn list_entity_references") && !entity.includes("pub async fn resolve_entity_reference") && entity.includes("pub async fn bulk_archive_entities"), "R6-08 ownership switch incomplete or R6-09 ownership moved early");
req(!read(`${coachRoot}mod.rs`).includes("sqlx::") && !read(`${formationRoot}mod.rs`).includes("sqlx::"), "module export file owns SQL");
for (const required of [
  `${formationRoot}resolution/read.rs`,
  `${formationRoot}usage/preparation.rs`,
  `${formationRoot}usage/window/read.rs`,
  `${formationRoot}usage/write.rs`,
]) req(exists(required), `R6-07 responsibility owner missing: ${required}`);
const usage = files.filter(p => p.includes("/formations/usage/")).map(read).join("\n");
const resolution = read(`${formationRoot}resolution/resolve.rs`);
const window = read(`${formationRoot}usage/window.rs`);
const save = read(`${formationRoot}usage/save.rs`);
req(!resolution.includes("sqlx::") && !window.includes("sqlx::") && !save.includes("sqlx::"), "formation coordinator still owns SQL");
req(usage.includes("alpha * prior") && usage.includes("UNKNOWN_FORMATION_ID"), "formation smoothing/unknown fallback missing");
req(!usage.includes("DELETE FROM feature.formation_usage_observations"), "formation history became destructive");
for (const level of ["actual_lineup","confirmed_lineup","team_coach","team","coach","competition_default","system_default","unknown"]) req(resolution.includes(level), `resolution level missing: ${level}`);
const coachPort = read("crates/application/src/ports/player/mod.rs");
const formationPort = read("crates/application/src/ports/lineup/mod.rs");
req(coachPort.includes("trait CoachCatalogPort") && formationPort.includes("trait FormationPort"), "public ports changed");
console.log("R6-07 Coaches / Formation Usage persistence ownership verified.");
