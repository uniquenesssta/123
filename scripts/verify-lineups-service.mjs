import { readLineupWorkflowSources } from "./features/lineupSources.mjs";
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
// R11-05 transport locations.
const lineupsTransportRoot = "src-tauri/src/commands/lineups";
const lineupsTransportGroups = {
  "": [
    "create_lineup",
    "create_lineup_pair",
    "list_lineups",
    "read_lineup",
    "remove_lineup_history",
    "read_match_lineup_chain",
    "list_team_match_lineups"
  ],
  "formations": [
    "list_formations",
    "save_formation_usage_distribution",
    "list_formation_usage_distributions",
    "resolve_formation_distribution"
  ],
  "matches": [
    "create_match",
    "delete_match"
  ],
  "presets": [
    "save_team_lineup_preset",
    "list_team_lineup_presets",
    "preview_team_lineup_preset_application",
    "duplicate_team_lineup_preset",
    "archive_team_lineup_preset",
    "delete_team_lineup_preset"
  ]
};
const lineupsTransportFiles = {
  "create_lineup": "src-tauri/src/commands/lineups/create_lineup.rs",
  "create_lineup_pair": "src-tauri/src/commands/lineups/create_lineup_pair.rs",
  "list_lineups": "src-tauri/src/commands/lineups/list_lineups.rs",
  "read_lineup": "src-tauri/src/commands/lineups/read_lineup.rs",
  "remove_lineup_history": "src-tauri/src/commands/lineups/remove_lineup_history.rs",
  "read_match_lineup_chain": "src-tauri/src/commands/lineups/read_match_lineup_chain.rs",
  "list_team_match_lineups": "src-tauri/src/commands/lineups/list_team_match_lineups.rs",
  "list_formations": "src-tauri/src/commands/lineups/formations/list_formations.rs",
  "save_formation_usage_distribution": "src-tauri/src/commands/lineups/formations/save_formation_usage_distribution.rs",
  "list_formation_usage_distributions": "src-tauri/src/commands/lineups/formations/list_formation_usage_distributions.rs",
  "resolve_formation_distribution": "src-tauri/src/commands/lineups/formations/resolve_formation_distribution.rs",
  "create_match": "src-tauri/src/commands/lineups/matches/create_match.rs",
  "delete_match": "src-tauri/src/commands/lineups/matches/delete_match.rs",
  "save_team_lineup_preset": "src-tauri/src/commands/lineups/presets/save_team_lineup_preset.rs",
  "list_team_lineup_presets": "src-tauri/src/commands/lineups/presets/list_team_lineup_presets.rs",
  "preview_team_lineup_preset_application": "src-tauri/src/commands/lineups/presets/preview_team_lineup_preset_application.rs",
  "duplicate_team_lineup_preset": "src-tauri/src/commands/lineups/presets/duplicate_team_lineup_preset.rs",
  "archive_team_lineup_preset": "src-tauri/src/commands/lineups/presets/archive_team_lineup_preset.rs",
  "delete_team_lineup_preset": "src-tauri/src/commands/lineups/presets/delete_team_lineup_preset.rs"
};
const facade=read("crates/application/src/services/lineups/facade.rs"); const service=read("crates/application/src/services/lineups/service.rs"); const app=read("crates/application/src/service/application_service.rs"); const composition=read("crates/application/src/composition/application_composition.rs"); const adapter=read("crates/application/src/composition/adapters/lineups.rs"); const registry=read("crates/application/src/composition/port_registry.rs"); const library=read("crates/application/src/lib.rs"); const tauri=Object.values(lineupsTransportFiles).map(read).join("\n"); const packageJson=JSON.parse(read("package.json")); const frontend=read("scripts/verify-frontend.mjs"); const persistenceMatch=read("crates/persistence-postgres/src/adapters/matches/catalog/read.rs");
check(app.includes("lineups: LineupService"),"ApplicationService 缺少 Lineup Service"); check(composition.includes("LineupService::new()"),"组合根未构造 LineupService"); check(!library.includes("mod player_catalog;"),"lib.rs 仍挂载旧 player_catalog 模块");
for(const implName of ["impl FormationPort for PersistenceStore","impl MatchCatalogPort for PersistenceStore","impl LineupPort for PersistenceStore","impl LineupPresetPort for PersistenceStore"]) check(adapter.includes(implName),`Lineups 组合适配器缺少：${implName}`); check(!registry.includes("impl FormationPort for PersistenceStore")&&!registry.includes("impl LineupPort for PersistenceStore"),"R3-05 具体适配实现堆叠到 port_registry.rs");
for(const method of methods) { check(facade.includes(`pub async fn ${method}`),`Lineups facade 缺少公共兼容方法：${method}`); check(service.includes(`fn ${method}`),`LineupService 缺少职责：${method}`); check(tauri.includes(`.${method}(`),`Tauri Lineups 公共调用链变化：${method}`); }
const serviceFiles=[...rustFiles("crates/application/src/services/lineups"),...rustFiles("crates/application/src/use_cases/lineups")];
for(const path of serviceFiles){const source=read(path); for(const token of ["football_persistence_postgres","PostgresStore","sqlx::","PgPool","PersistenceStore"]) check(!source.includes(token),`${path} 泄漏具体持久化实现：${token}`);}
for(const token of ["PersistenceStore::read_match(self, match_id)","PersistenceStore::read_match_lineup_chain_at(self, match_id, snapshot_type, reference_time)"]) check(adapter.includes(token),`组合适配器缺少既有内部读取能力：${token}`);
check(persistenceMatch.includes("pub async fn read_match("),"MatchCatalogPort 读取能力未通过合法 persistence crate 公共边界暴露");
check(packageJson.scripts?.["verify:lineups-service"]==="node scripts/verify-lineups-service.mjs","package.json 未登记 R3-05 专项门禁"); check(packageJson.scripts?.["verify:architecture"]?.includes("verify-lineups-service.mjs"),"verify:architecture 未接入 R3-05 门禁"); check(frontend.includes('"verify-lineups-service.mjs"'),"verify:frontend 未接入 R3-05 门禁");
// R7-07：创建、双方审计及版本锁必须由唯一事务 owner 持有。
const pairBase = "crates/persistence-postgres/src/adapters/lineups/pair_transaction";
for (const file of ["mod.rs", "validation.rs", "write.rs"]) check(existsSync(join(root, pairBase, file)), `缺少 R7-07 owner：${file}`);
const pair = read(`${pairBase}/mod.rs`);
const lineupWrite = read(`${pairBase}/write.rs`);
const lineupValidation = read(`${pairBase}/validation.rs`);
const oldCatalog = read("crates/persistence-postgres/src/player_catalog.rs");
const workbook = ["preview","conflict","commit","context","read","write","identity","validation","values","mod"].map((name) => read(`crates/persistence-postgres/src/adapters/workbooks/match_lineup/${name}.rs`)).join("\n");
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
// R7-08：时点规则、模型门禁、读历史与历史变更各自唯一持有职责。
const lineupBase = "crates/persistence-postgres/src/adapters/lineups";
const chainBase = `${lineupBase}/chain`;
const historyBase = `${lineupBase}/history`;
for (const file of ["chain/mod.rs", "chain/window.rs", "chain/validation.rs", "history/mod.rs", "history/read.rs", "history/mapping.rs", "history/removal.rs"]) check(existsSync(join(root, lineupBase, file)), `缺少 R7-08 owner：${file}`);
const chainOwner = read(`${chainBase}/mod.rs`);
const windowOwner = read(`${chainBase}/window.rs`);
const validationOwner = read(`${chainBase}/validation.rs`);
const historyRead = read(`${historyBase}/read.rs`);
const historyMapping = read(`${historyBase}/mapping.rs`);
const historyRemoval = read(`${historyBase}/removal.rs`);
check(!existsSync(join(root, "crates/persistence-postgres/src/lineup_chain.rs")), "旧 lineup_chain.rs 不得保留实现或转发壳");
for (const [file, source] of persistenceFiles.map((file) => [file, read(file)])) check(!source.includes("crate::lineup_chain::") && !source.includes("super::lineup_chain::"), `${file} 仍依赖旧 chain 路径`);
for (const [owner, methods] of [[`${chainBase}/mod.rs`, ["preferred_lineup_id", "read_match_lineup_chain", "read_match_lineup_chain_at"]], [`${historyBase}/read.rs`, ["list_lineups", "read_lineup", "list_team_match_lineups"]], [`${historyBase}/removal.rs`, ["remove_lineup_history"]], [`${historyBase}/mapping.rs`, ["lineup_record_from_row", "lineup_player_from_row"]], [`${chainBase}/window.rs`, ["normalize_lineup_snapshot_type", "lineup_snapshot_window", "lineup_snapshot_window_at"]], [`${chainBase}/validation.rs`, ["refresh_lineup_validation_in_tx"]]]) {
  for (const method of methods) {
    const declarations = persistenceFiles.filter((file) => new RegExp(`(?:pub(?:\\(crate\\))?\\s+)?(?:async\\s+)?fn\\s+${method}\\s*\\(`).test(read(file)));
    check(declarations.length === 1 && declarations[0] === owner, `${method} 必须只有一个职责 owner`);
  }
}
check(!/fn (?:list_lineups|read_lineup|remove_lineup_history|lineup_player_from_row|lineup_record_from_row)\s*\(/.test(oldCatalog) && oldCatalog.includes("pub async fn player_catalog_reference_data("), "旧 catalog 必须仅保留引用数据聚合，不残留阵容职责");
for (const token of ["lineup.status='active'", "lineup.history_hidden_at IS NULL", "lineup.model_eligible", "lineup.lineup_type IN ('confirmed','expected')", "lineup.captured_at <= $3", "lineup.captured_at >= $4", "ORDER BY lineup.captured_at DESC", "WHEN 'confirmed' THEN 2", "lineup.created_at DESC, lineup.id DESC", "LIMIT 1"]) check(chainOwner.includes(token), `preferred lineup 规则缺少：${token}`);
check(windowOwner.includes("reference_time.min(kickoff_time - Duration::seconds(1))") && windowOwner.includes("if cutoff_time < start_time") && windowOwner.includes('"T-90m" => Err'), "窗口边界及旧时点拒绝必须保持");
check(validationOwner.includes('lineup_type == "actual"') && validationOwner.includes('lineup_type != "actual"') && validationOwner.includes("starter_count != 11") && validationOwner.includes("formation_id.is_none()") && !validationOwner.includes(".begin()") && !validationOwner.includes(".commit()"), "门禁校验必须复用写入事务并保持 actual/11 首发/阵型隔离");
check(historyRead.includes("ORDER BY lineup.captured_at DESC, lineup.id DESC") && historyRead.includes("ORDER BY fixture.kickoff_time DESC, lineup.captured_at DESC, lineup.id DESC") && (historyRead.match(/limit.clamp\(1, 200\)/g) ?? []).length === 2 && (historyRead.match(/lineup.history_hidden_at IS NULL/g) ?? []).length === 2, "历史列表隐藏过滤、稳定排序与两处 limit clamp 必须保持");
check(historyRead.includes("lineup.captured_at::date") && historyMapping.includes("availability_status") && historyMapping.includes("role_source_position_code") && historyMapping.includes("未知阵容类型"), "历史时点角色来源与严格 mapping 必须保持");
const historyLock = historyRemoval.indexOf("pair_transaction::lock_match_in_tx");
const historyVersionLock = historyRemoval.indexOf("FOR UPDATE");
const historyReferences = historyRemoval.indexOf("let referenced: bool");
const historyRestore = historyRemoval.indexOf("let restored_lineup_id");
const historyAudit = historyRemoval.indexOf('"lineup_history_removed"');
check(historyRemoval.includes("SET TRANSACTION ISOLATION LEVEL READ COMMITTED") && historyLock >= 0 && historyVersionLock > historyLock && historyReferences > historyVersionLock && historyRestore > historyReferences && historyAudit > historyRestore && historyRemoval.indexOf("tx.commit().await?") > historyAudit, "历史删除必须先锁比赛、重读锁版本，再检查引用/恢复/审计并单事务提交");
check((historyRemoval.match(/self.pool.begin\(\)/g) ?? []).length === 1 && (historyRemoval.match(/tx.commit\(\)/g) ?? []).length === 1 && !historyRemoval.includes("fetch_one(&self.pool)") && !historyRemoval.includes("fetch_optional(&self.pool)"), "历史删除/恢复不能在事务外检查或另开事务");
for (const token of ["feature.match_player_contributions", "feature.snapshots", "model.runs", "supersedes_lineup_id = $1", "history_hidden_at = now()", "status = 'superseded'", "history_hidden_at IS NULL", "ORDER BY captured_at DESC, created_at DESC, id DESC", "lineup_history_removed"]) check(historyRemoval.includes(token), `历史引用/恢复保护缺少：${token}`);
for (const method of ["list_lineups", "read_lineup", "remove_lineup_history", "read_match_lineup_chain", "read_match_lineup_chain_at", "list_team_match_lineups"]) check(adapter.includes(`PersistenceStore::${method}(self,`), `LineupPort 未显式分派：${method}`);
for (const token of ["历史删除与创建都必须在版本写入前等待同一父锁", "截止时点包含等时记录", "同时间 confirmed 优先于 expected", "时间较新的 expected 优先于较旧 confirmed", "窗口起点包含等时记录", "窗口前一微秒不能选择", "隐藏历史按 ID 保留明细", "等时间历史以 UUID 降序稳定排列", "list_max.len(), 200", "team_max.len(), 200"]) check(pairIntegration.includes(token), `已有 chain/history 测试缺少：${token}`);
// R7-09：预设事务与只读预检分离，保留正式提交和主客侧草稿边界。
const presetBase = "crates/persistence-postgres/src/adapters/lineups/presets";
for (const file of ["mod.rs", "validation.rs", "write.rs", "read.rs", "preview.rs"]) check(existsSync(join(root, presetBase, file)), `缺少 R7-09 owner：${file}`);
check(!existsSync(join(root, "crates/persistence-postgres/src/team_lineup_presets.rs")) && !read("crates/persistence-postgres/src/lib.rs").includes("mod team_lineup_presets;"), "旧预设实现或转发壳不能残留");
check(read("crates/persistence-postgres/src/adapters/lineups/mod.rs").includes("pub(crate) mod presets;"), "lineups 命名空间未挂载唯一预设职责");
const presetWrite = read(`${presetBase}/write.rs`);
const presetRead = read(`${presetBase}/read.rs`);
const presetPreview = read(`${presetBase}/preview.rs`);
const presetValidation = read(`${presetBase}/validation.rs`);
for (const [owner, names] of [["write.rs", ["save_team_lineup_preset", "archive_team_lineup_preset", "delete_team_lineup_preset", "duplicate_team_lineup_preset"]], ["read.rs", ["list_team_lineup_presets", "read_team_lineup_preset", "parse_availability"]], ["preview.rs", ["preview_team_lineup_preset_application", "assess_application"]], ["validation.rs", ["validate_preset", "verify_membership_in_tx"]]]) {
  for (const name of names) {
    const declarations = persistenceFiles.filter((file) => (name !== "parse_availability" || file.startsWith(presetBase + "/")) && new RegExp(`(?:pub(?:\\(super\\))?\\s+)?(?:async\\s+)?fn\\s+${name}\\s*\\(`).test(read(file)));
    check(declarations.length === 1 && declarations[0] === `${presetBase}/${owner}`, `${name} 必须只有一个预设 owner`);
  }
}
const previewProduction = presetPreview.split("#[cfg(test)]")[0];
for (const source of [previewProduction, presetRead.split("#[cfg(test)]")[0]]) check(!/sqlx::query(?:_scalar|_as)?\s*\([\s\S]*?(?:INSERT INTO|UPDATE football|DELETE FROM)/.test(source) && !source.includes("write_audit_event") && !source.includes(".begin()") && !source.includes(".commit()") && !source.includes("create_lineup"), "预设读取/预检必须只读，不能提交阵容或审计");
check(previewProduction.includes("self.read_team_lineup_preset(preset_id).await?") && previewProduction.includes("Ok(assess_application(preset))"), "预检必须读取唯一预设事实再纯评估");
for (const token of ['preset.status != "active"', "preset.starter_count != 11", 'member.player_status != "active"', "member.current_team_id != Some(preset.team_id)", "AvailabilityStatus::Injured", "AvailabilityStatus::Suspended", "AvailabilityStatus::Unavailable", "AvailabilityStatus::Doubtful", "blockers.sort()", "blockers.dedup()", "warnings.sort()", "warnings.dedup()", "can_apply: blockers.is_empty()"]) check(previewProduction.includes(token), `预检门禁/提示缺少：${token}`);
const savePreset = presetWrite.slice(presetWrite.indexOf("pub async fn save_team_lineup_preset("), presetWrite.indexOf("pub async fn archive_team_lineup_preset("));
const archivePreset = presetWrite.slice(presetWrite.indexOf("pub async fn archive_team_lineup_preset("), presetWrite.indexOf("pub async fn delete_team_lineup_preset("));
const deletePreset = presetWrite.slice(presetWrite.indexOf("pub async fn delete_team_lineup_preset("), presetWrite.indexOf("pub async fn duplicate_team_lineup_preset("));
for (const [label, body, audit] of [["保存", savePreset, "team_lineup_preset_created"], ["归档", archivePreset, "team_lineup_preset_archived"], ["删除", deletePreset, "team_lineup_preset_deleted"]]) check((body.match(/self.pool.begin\(\)/g) ?? []).length === 1 && (body.match(/tx.commit\(\)/g) ?? []).length === 1 && body.indexOf(audit) >= 0 && body.indexOf(audit) < body.indexOf("tx.commit().await?"), `预设${label}必须在唯一事务共同提交审计`);
check(savePreset.indexOf("validate_preset(draft)?") < savePreset.indexOf("self.pool.begin()") && savePreset.indexOf("verify_membership_in_tx(") < savePreset.indexOf("UPDATE football.team_lineup_presets") && savePreset.includes("FOR UPDATE") && savePreset.includes("existing_team_id != draft.team_id") && savePreset.includes('status != "active"'), "保存必须先校验，再检查成员/归属和活动状态，锁住原预设版本");
check(savePreset.includes("let role_as_of = Utc::now().date_naive();") && savePreset.includes("metadata_with_role_resolution") && savePreset.includes("resolve_default_tactical_role_in_tx"), "保存角色继承/来源与共同审计日期缺失");
for (const token of ["starter_count != 11", "unique_players.len() != draft.members.len()", "(0.0..=1.0).contains(&probability)", "period.team_id = $2", "period.valid_from <= current_date", "period.valid_to >= current_date"]) {
  check(presetValidation.includes(token), `预设校验缺少：${token}`);
}
check(presetRead.includes("ORDER BY is_default DESC, status, updated_at DESC, lower(name), id") && presetRead.includes("LIMIT 200") && presetRead.includes("period.team_id=preset.team_id") && presetRead.includes("role_source_position_code") && presetRead.includes("position.valid_from <= current_date"), "预设列表上限/稳定顺序/成员和角色读取语义缺失");
for (const method of ["save_team_lineup_preset", "list_team_lineup_presets", "preview_team_lineup_preset_application", "duplicate_team_lineup_preset", "archive_team_lineup_preset", "delete_team_lineup_preset"]) check(adapter.includes(`PersistenceStore::${method}(self,`), `LineupPresetPort 未显式分派：${method}`);
check(presetValidation.includes("fetch_one(&mut **tx)") && !presetValidation.includes("self.pool") && !presetValidation.includes(".begin()") && !presetValidation.includes(".commit()"), "成员校验不得离开保存事务");
const mainSource = readLineupWorkflowSources(root);
const presetSource=read("src/features/lineups/presetApplication.ts");
const applyPreset=presetSource.slice(presetSource.indexOf("async function applyLineupPreset("));
check(applyPreset.includes("api.previewTeamLineupPresetApplication(presetId)") && applyPreset.includes("if (!preview.can_apply)") && applyPreset.includes("preview.preset.team_id !== current.team_id") && applyPreset.includes("[side]: {") && applyPreset.includes("...current,"), "套用必须复检并校验所选主客侧，只替换这一侧草稿");
check(!/api\.(?:createLineup|createLineupPair|saveTeamLineupPreset)/.test(applyPreset) && mainSource.includes("api.createLineupPair("), "预设套用不能隐式写正式阵容，必须保留显式双方提交");
for (const token of ["合法预设可以预览", "重复预览保留原预设及角色来源", "预览不得写正式阵容、球员或审计", "预览不得修改预设版本或时间", "客队成员不得保存到主队预设", "过期成员阻止套用", "归档预设阻止套用", "删除活动与归档预设级联清理成员"]) check(pairIntegration.includes(token), `既有数据库用例缺少：${token}`);
// R11-05 transport contract BEGIN
{
  const transportRoot = lineupsTransportRoot;
  const transportGroups = lineupsTransportGroups;
  const transportFiles = lineupsTransportFiles;
  const transportSignatures = {
  "create_lineup": "pub async fn create_lineup(\n    state: State<'_, AppState>,\n    draft: LineupDraft,\n) -> Result<LineupRecord, String>",
  "create_lineup_pair": "pub async fn create_lineup_pair(\n    state: State<'_, AppState>,\n    draft: LineupPairDraft,\n) -> Result<LineupPairRecord, String>",
  "list_lineups": "pub async fn list_lineups(\n    state: State<'_, AppState>,\n    match_id: Option<String>,\n    limit: u32,\n) -> Result<Vec<LineupRecord>, String>",
  "read_lineup": "pub async fn read_lineup(\n    state: State<'_, AppState>,\n    lineup_id: String,\n) -> Result<LineupRecord, String>",
  "remove_lineup_history": "pub async fn remove_lineup_history(\n    state: State<'_, AppState>,\n    lineup_id: String,\n    reason: Option<String>,\n) -> Result<LineupHistoryRemovalResult, String>",
  "read_match_lineup_chain": "pub async fn read_match_lineup_chain(\n    state: State<'_, AppState>,\n    match_id: String,\n    snapshot_type: String,\n) -> Result<MatchLineupChain, String>",
  "list_team_match_lineups": "pub async fn list_team_match_lineups(\n    state: State<'_, AppState>,\n    team_id: String,\n    limit: u32,\n) -> Result<Vec<TeamMatchLineupHistoryItem>, String>",
  "list_formations": "pub async fn list_formations(\n    state: State<'_, AppState>,\n    active_only: bool,\n) -> Result<Vec<FormationRecord>, String>",
  "save_formation_usage_distribution": "pub async fn save_formation_usage_distribution(\n    state: State<'_, AppState>,\n    draft: FormationUsageDistributionDraft,\n) -> Result<FormationUsageDistributionRecord, String>",
  "list_formation_usage_distributions": "pub async fn list_formation_usage_distributions(\n    state: State<'_, AppState>,\n    query: FormationUsageListQuery,\n) -> Result<Vec<FormationUsageDistributionRecord>, String>",
  "resolve_formation_distribution": "pub async fn resolve_formation_distribution(\n    state: State<'_, AppState>,\n    query: FormationDistributionQuery,\n) -> Result<football_domain::ResolvedFormationDistribution, String>",
  "create_match": "pub async fn create_match(\n    state: State<'_, AppState>,\n    draft: MatchDraft,\n) -> Result<MatchRecord, String>",
  "delete_match": "pub async fn delete_match(state: State<'_, AppState>, match_id: String) -> Result<(), String>",
  "save_team_lineup_preset": "pub async fn save_team_lineup_preset(\n    state: State<'_, AppState>,\n    draft: TeamLineupPresetDraft,\n) -> Result<TeamLineupPresetRecord, String>",
  "list_team_lineup_presets": "pub async fn list_team_lineup_presets(\n    state: State<'_, AppState>,\n    team_id: String,\n    include_archived: bool,\n) -> Result<Vec<TeamLineupPresetRecord>, String>",
  "preview_team_lineup_preset_application": "pub async fn preview_team_lineup_preset_application(\n    state: State<'_, AppState>,\n    preset_id: String,\n) -> Result<TeamLineupPresetApplicationPreview, String>",
  "duplicate_team_lineup_preset": "pub async fn duplicate_team_lineup_preset(\n    state: State<'_, AppState>,\n    preset_id: String,\n    name: String,\n) -> Result<TeamLineupPresetRecord, String>",
  "archive_team_lineup_preset": "pub async fn archive_team_lineup_preset(\n    state: State<'_, AppState>,\n    preset_id: String,\n) -> Result<TeamLineupPresetRecord, String>",
  "delete_team_lineup_preset": "pub async fn delete_team_lineup_preset(\n    state: State<'_, AppState>,\n    preset_id: String,\n) -> Result<(), String>"
};
  const transportValidation = {
  "list_lineups": "let match_id = match_id\n        .map(|value| parse_uuid(&value, \"比赛 ID\"))\n        .transpose()?;",
  "read_lineup": "let lineup_id = parse_uuid(&lineup_id, \"阵容 ID\")?;",
  "delete_match": "let match_id = parse_uuid(&match_id, \"比赛 ID\")?;",
  "list_team_lineup_presets": "let team_id = parse_uuid(&team_id, \"球队 ID\")?;",
  "preview_team_lineup_preset_application": "let preset_id = parse_uuid(&preset_id, \"阵容预设 ID\")?;",
  "duplicate_team_lineup_preset": "let preset_id = parse_uuid(&preset_id, \"阵容预设 ID\")?;",
  "archive_team_lineup_preset": "let preset_id = parse_uuid(&preset_id, \"阵容预设 ID\")?;",
  "delete_team_lineup_preset": "let preset_id = parse_uuid(&preset_id, \"阵容预设 ID\")?;"
};
  const transportCalls = {
  "create_lineup": "create_lineup(draft)",
  "create_lineup_pair": "create_lineup_pair(draft)",
  "list_lineups": "list_lineups(match_id, limit)",
  "read_lineup": "read_lineup(lineup_id)",
  "remove_lineup_history": "remove_lineup_history(parse_uuid(&lineup_id, \"阵容 ID\")?, reason)",
  "read_match_lineup_chain": "read_match_lineup_chain(parse_uuid(&match_id, \"比赛 ID\")?, snapshot_type)",
  "list_team_match_lineups": "list_team_match_lineups(parse_uuid(&team_id, \"球队 ID\")?, limit)",
  "list_formations": "list_formations(active_only)",
  "save_formation_usage_distribution": "save_formation_usage_distribution(draft)",
  "list_formation_usage_distributions": "list_formation_usage_distributions(query)",
  "resolve_formation_distribution": "resolve_formation_distribution(query)",
  "create_match": "create_match(draft)",
  "delete_match": "delete_match(match_id)",
  "save_team_lineup_preset": "save_team_lineup_preset(draft)",
  "list_team_lineup_presets": "list_team_lineup_presets(team_id, include_archived)",
  "preview_team_lineup_preset_application": "preview_team_lineup_preset_application(preset_id)",
  "duplicate_team_lineup_preset": "duplicate_team_lineup_preset(preset_id, name)",
  "archive_team_lineup_preset": "archive_team_lineup_preset(preset_id)",
  "delete_team_lineup_preset": "delete_team_lineup_preset(preset_id)"
};
  const transportContract = JSON.parse(read("architecture/command-contract.json"));
  const transportBoundaries = JSON.parse(read("architecture/module-boundaries.json"));
  const transportGroup = transportContract.command_groups.find(group => group.module === "lineups");
  const transportModules = read("src-tauri/src/commands.rs");
  const rustTokens = source => (source.match(/"(?:\\.|[^"\\])*"|[A-Za-z_][A-Za-z_0-9]*|[^\s]/g) ?? []).join("\0");
  const signatureTokens = source => rustTokens(source.replace(/,\s*\)/g, ")"));
  const moduleTokens = source => rustTokens(source.replace(/,\s*}/g, "}"));
  const exports = (module, names) => `pub use ${module}::{${names.flatMap(name => [`__cmd__${name}`, `__tauri_command_name_${name}`, name]).sort().join(",")}};`;
  check(transportModules.includes(`mod ${"lineups"};`) && transportModules.includes(`pub use ${"lineups"}::*;`), "传输根模块未登记唯一入口");
  check(transportGroup?.definition_file === `${transportRoot}/mod.rs` && transportGroup?.parameter_and_return_signature_file === `${transportRoot}/mod.rs`, "传输清单根出口漂移");
  check(transportBoundaries.tauri_commands?.groups?.["lineups"] === `${transportRoot}/mod.rs`, "传输边界owner漂移");
  const modules = Object.fromEntries([
    ...(transportGroups[""] ?? []).map(name => [name, [name]]),
    ...Object.entries(transportGroups).filter(([group]) => group !== ""),
  ]);
  const orderedModules = Object.keys(modules).sort();
  const expectedRoot = [...orderedModules.map(name => `mod ${name};`), ...orderedModules.map(name => exports(name, modules[name]))].join("\n");
  check(moduleTokens(read(`${transportRoot}/mod.rs`)) === moduleTokens(expectedRoot), "传输出口必须仅显式登记原命令与双宏");
  for (const [group, names] of Object.entries(transportGroups)) {
    if (!group) continue;
    const ordered = [...names].sort();
    const expected = [...ordered.map(name => `mod ${name};`), ...ordered.map(name => exports(name, [name]))].join("\n");
    check(moduleTokens(read(`${transportRoot}/${group}/mod.rs`)) === moduleTokens(expected), `传输子出口承载逻辑或漏宏/命令：${group}`);
  }
  for (const [name, file] of Object.entries(transportFiles)) {
    const source = read(file);
    const signature = source.match(new RegExp(`pub async fn ${name}\\([\\s\\S]*?\\{`))?.[0].slice(0, -1) ?? "";
    check(signatureTokens(signature) === signatureTokens(transportSignatures[name]), `传输公开签名漂移：${name}`);
    check(transportGroup?.command_definition_files?.[name] === file, `传输定义owner漂移：${name}`);
    check((source.match(/#\[tauri::command\]/g) ?? []).length === 1 && source.includes("bootstrap::AppState"), `传输未直接使用原State/唯一命令：${name}`);
    check(rustTokens(source).includes(rustTokens("." + transportCalls[name])), `传输未调用原Application能力与原参数/内联UUID：${name}`);
    check(source.includes(".map_err(|error| error.to_string())"), `传输错误投影漂移：${name}`);
    const fullBody = source.slice(source.indexOf("{", source.indexOf(`pub async fn ${name}`)) + 1, source.lastIndexOf("}"));
    const expectedBody = `${transportValidation[name] ?? ""} state.service.${transportCalls[name]}.await.map_err(|error| error.to_string())`;
    check(rustTokens(fullBody) === rustTokens(expectedBody), `薄传输不得新增业务门槛、文件处理或替代结果：${name}`);
    if (transportValidation[name]) {
      const body = source.slice(source.indexOf("{", source.indexOf(`pub async fn ${name}`)) + 1);
      check(rustTokens(body).startsWith(rustTokens(transportValidation[name])), `UUID/可选ID转换必须在原服务调用前失败返回：${name}`);
    }
    if (transportCalls[name].includes("parse_uuid") || transportValidation[name]?.includes("parse_uuid")) check(source.includes("commands::parse_uuid"), `传输不得复制UUID校验：${name}`);
    for (const forbidden of ["sqlx::", "football_persistence_postgres", "PostgresStore", "PgPool", "ApplicationService::new", "std::fs", "std::io", "tokio::spawn", "Mutex<", "static ", "macro_rules!", ".unwrap()", ".expect("]) {
      check(!source.includes(forbidden), `传输越过原业务/文件/生命周期边界：${name}/${forbidden}`);
    }
  }
  for (const retired of ["src-tauri/src/commands/catalog/formations.rs", "src-tauri/src/commands/catalog/lineup_presets.rs", "src-tauri/src/commands/catalog/lineups.rs", "src-tauri/src/commands/catalog/matches.rs"]) check(!existsSync(join(root, retired)), `旧传输owner仍存在：${retired}`);
}
// R11-05 transport contract END

if(failures.length) throw new Error(`Lineups Service 验证失败\n${failures.map((item)=>`- ${item}`).join("\n")}`);
console.log(`Lineups Service 验证通过：${serviceFiles.length} 个 Service/Use Case Rust 文件，19 个公开 Application 职责已切换 4 个既有 Ports，旧 player_catalog 所有者已退出。`);
