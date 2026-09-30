import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8").replaceAll("\r\n", "\n");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
function rustFiles(path) { const absolute = join(root, path); const result = []; for (const entry of readdirSync(absolute, {withFileTypes:true})) { const child=join(absolute,entry.name); if(entry.isDirectory()) result.push(...rustFiles(relative(root,child))); else if(entry.name.endsWith(".rs")) result.push(relative(root,child).replaceAll("\\","/")); } return result; }
const methods=["list_formations","save_formation_usage_distribution","list_formation_usage_distributions","resolve_formation_distribution","create_match","delete_match","save_team_lineup_preset","list_team_lineup_presets","preview_team_lineup_preset_application","duplicate_team_lineup_preset","archive_team_lineup_preset","delete_team_lineup_preset","create_lineup","create_lineup_pair","list_lineups","read_lineup","remove_lineup_history","read_match_lineup_chain","list_team_match_lineups"];
const required=["crates/application/src/services/lineups/mod.rs","crates/application/src/services/lineups/service.rs","crates/application/src/services/lineups/facade.rs","crates/application/src/composition/adapters/lineups.rs","crates/application/src/use_cases/lineups/mod.rs",...methods.map((name)=>`crates/application/src/use_cases/lineups/${name}/mod.rs`)];
for(const path of required) check(existsSync(join(root,path)),`缺少 R3-05 文件：${path}`);
check(!existsSync(join(root,"crates/application/src/player_catalog.rs")),"旧 player_catalog.rs 仍残留阵容职责或空转发层");
const facade=read("crates/application/src/services/lineups/facade.rs"); const service=read("crates/application/src/services/lineups/service.rs"); const app=read("crates/application/src/service/application_service.rs"); const composition=read("crates/application/src/composition/application_composition.rs"); const adapter=read("crates/application/src/composition/adapters/lineups.rs"); const registry=read("crates/application/src/composition/port_registry.rs"); const library=read("crates/application/src/lib.rs"); const tauri=read("src-tauri/src/commands/catalog.rs"); const packageJson=JSON.parse(read("package.json")); const frontend=read("scripts/verify-frontend.mjs"); const persistenceMatch=read("crates/persistence-postgres/src/adapters/matches/catalog/read.rs");
check(app.includes("lineups: LineupService"),"ApplicationService 缺少 Lineup Service"); check(composition.includes("LineupService::new()"),"组合根未构造 LineupService"); check(!library.includes("mod player_catalog;"),"lib.rs 仍挂载旧 player_catalog 模块");
for(const implName of ["impl FormationPort for PersistenceStore","impl MatchCatalogPort for PersistenceStore","impl LineupPort for PersistenceStore","impl LineupPresetPort for PersistenceStore"]) check(adapter.includes(implName),`Lineups 组合适配器缺少：${implName}`); check(!registry.includes("impl FormationPort for PersistenceStore")&&!registry.includes("impl LineupPort for PersistenceStore"),"R3-05 具体适配实现堆叠到 port_registry.rs");
for(const method of methods) { check(facade.includes(`pub async fn ${method}`),`Lineups facade 缺少公共兼容方法：${method}`); check(service.includes(`fn ${method}`),`LineupService 缺少职责：${method}`); check(tauri.includes(`.${method}(`),`Tauri Lineups 公共调用链变化：${method}`); }
const serviceFiles=[...rustFiles("crates/application/src/services/lineups"),...rustFiles("crates/application/src/use_cases/lineups")];
for(const path of serviceFiles){const source=read(path); for(const token of ["football_persistence_postgres","PostgresStore","sqlx::","PgPool","PersistenceStore"]) check(!source.includes(token),`${path} 泄漏具体持久化实现：${token}`);}
for(const token of ["PersistenceStore::read_match(self, match_id)","read_match_lineup_chain_at(match_id, snapshot_type, reference_time)"]) check(adapter.includes(token),`组合适配器缺少既有内部读取能力：${token}`);
check(persistenceMatch.includes("pub async fn read_match("),"MatchCatalogPort 读取能力未通过合法 persistence crate 公共边界暴露");
check(packageJson.scripts?.["verify:lineups-service"]==="node scripts/verify-lineups-service.mjs","package.json 未登记 R3-05 专项门禁"); check(packageJson.scripts?.["verify:architecture"]?.includes("verify-lineups-service.mjs"),"verify:architecture 未接入 R3-05 门禁"); check(frontend.includes('"verify-lineups-service.mjs"'),"verify:frontend 未接入 R3-05 门禁");
// R7-07：创建、双方审计及版本锁必须由唯一事务 owner 持有。
const pairBase = "crates/persistence-postgres/src/adapters/lineups/pair_transaction";
for (const file of ["mod.rs", "validation.rs", "write.rs"]) check(existsSync(join(root, pairBase, file)), `缺少 R7-07 owner：${file}`);
const pair = read(`${pairBase}/mod.rs`);
const lineupWrite = read(`${pairBase}/write.rs`);
const lineupValidation = read(`${pairBase}/validation.rs`);
const oldCatalog = read("crates/persistence-postgres/src/player_catalog.rs");
const workbook = read("crates/persistence-postgres/src/match_exchange.rs");
const persistenceFiles = rustFiles("crates/persistence-postgres/src");
for (const method of ["create_lineup", "create_lineup_pair"]) {
  const declarations = persistenceFiles.filter((file) => new RegExp(`pub\\s+async\\s+fn\\s+${method}\\s*\\(`).test(read(file)));
  check(declarations.length === 1 && declarations[0] === `${pairBase}/mod.rs`, `${method} 必须只有一个创建 owner`);
}
check(!oldCatalog.includes("fn validate_lineup_draft") && !oldCatalog.includes("fn insert_lineup_in_tx"), "旧目录残留阵容写入/校验双实现");
check(pair.includes("FROM football.matches WHERE id=$1 FOR UPDATE") && (pair.match(/SET TRANSACTION ISOLATION LEVEL READ COMMITTED/g) ?? []).length === 2, "阵容写入必须在 READ COMMITTED 下取得共同父锁");
const single = pair.slice(pair.indexOf("pub async fn create_lineup("), pair.indexOf("pub async fn create_lineup_pair("));
const both = pair.slice(pair.indexOf("pub async fn create_lineup_pair("), pair.indexOf("/// 阵容版本写入"));
for (const [label, body, writes] of [["单侧", single, 1], ["双方", both, 2]]) {
  const lock = body.indexOf("lock_match_in_tx(");
  const write = body.indexOf("insert_lineup_in_tx(");
  const commit = body.indexOf("tx.commit().await?");
  check(lock >= 0 && write > lock && commit > write && (body.match(/self.pool.begin\(\)/g) ?? []).length === 1 && (body.match(/insert_lineup_in_tx\(/g) ?? []).length === writes, `${label}创建必须先锁定，再在唯一事务写入/提交`);
}
check(both.includes('"lineup_pair_created"') && both.indexOf('"lineup_pair_created"') < both.indexOf("tx.commit().await?"), "双方审计必须在业务事务提交前");
for (const guard of ["draft.home.match_id != draft.away.match_id", "draft.home.team_id == draft.away.team_id", "draft.home.snapshot_type != draft.away.snapshot_type", "draft.home.lineup_type != draft.away.lineup_type", "draft.home.team_id != home_team_id || draft.away.team_id != away_team_id"]) check(both.includes(guard), `双方校验缺少：${guard}`);
check(lineupValidation.includes("starters != 11") && lineupValidation.includes("normalize_lineup_snapshot_type") && lineupWrite.includes("refresh_lineup_validation_in_tx(tx, lineup_id)") && lineupWrite.includes('"lineup_created"'), "阵容结构、窗口模型门禁和单侧审计链必须保持");
check(!lineupWrite.includes("self.pool") && !lineupWrite.includes(".commit()") && !lineupWrite.includes(".begin()") && lineupWrite.includes("execute(&mut **tx)"), "阵容写入步骤不得另开或提交事务");
const workbookLineup = workbook.slice(workbook.indexOf("SpreadsheetEntityType::Lineup => {", workbook.indexOf("async fn apply_match_exchange_row")), workbook.indexOf("SpreadsheetEntityType::LineupPlayer => {", workbook.indexOf("async fn apply_match_exchange_row")));
check(workbookLineup.includes("pair_transaction::lock_match_in_tx(tx, match_id)") && workbookLineup.indexOf("lock_match_in_tx") < workbookLineup.indexOf("let supersedes_lineup_id"), "工作簿修改版本链必须先取得同一父锁");
check(adapter.includes("PersistenceStore::create_lineup(self, draft)") && adapter.includes("PersistenceStore::create_lineup_pair(self, draft)"), "LineupPort 必须显式分派到既有公开创建入口");
const pairIntegration = read("crates/persistence-postgres/tests/postgres_integration.rs");
check(pairIntegration.includes("取消必须发生在主队写入后、客队明细外键等待期间") && pairIntegration.includes("pair、single 与 workbook 必须在写入前等待同一比赛锁") && pairIntegration.includes("pg_blocking_pids(pid)"), "已有 pair 测试缺少真实锁等待、取消和混合写入并发断言");
if(failures.length) throw new Error(`Lineups Service 验证失败\n${failures.map((item)=>`- ${item}`).join("\n")}`);
console.log(`Lineups Service 验证通过：${serviceFiles.length} 个 Service/Use Case Rust 文件，19 个公开 Application 职责已切换 4 个既有 Ports，旧 player_catalog 所有者已退出。`);
