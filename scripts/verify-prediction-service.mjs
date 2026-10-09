import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8").replaceAll("\r\n", "\n");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
function rustFiles(path) { const absolute = join(root, path); const result = []; for (const entry of readdirSync(absolute, {withFileTypes:true})) { const child=join(absolute,entry.name); if(entry.isDirectory()) result.push(...rustFiles(relative(root,child))); else if(entry.name.endsWith(".rs")) result.push(relative(root,child).replaceAll("\\","/")); } return result; }
const publicMethods=["execute_prediction","inspect_match_prediction_readiness","execute_prediction_from_match","execute_shadow_prediction_from_match","preview_route","dry_run_default_fixture","list_recent_runs","hide_run_from_history","read_run","plan_p4_horizons","list_p4_freeze_tasks","read_p4_freeze_task","list_p4_freeze_task_events","p4_freeze_readiness","read_p4_match_workspace","read_p4_task_workspace","freeze_p4_prematch_snapshot","read_p4_prematch_snapshot"];
const required=["crates/application/src/services/prediction/mod.rs","crates/application/src/services/prediction/service.rs","crates/application/src/services/prediction/facade.rs","crates/application/src/composition/adapters/prediction.rs","crates/application/src/ports/prediction/mod.rs","crates/application/src/use_cases/prediction/mod.rs","crates/application/src/use_cases/prediction/p4_snapshot/mod.rs","crates/application/src/use_cases/prediction/execute_p4_freeze/mod.rs","crates/application/src/use_cases/prediction/execute_p4_freeze/snapshot_projection.rs"];
for(const path of required) check(existsSync(join(root,path)),`缺少 R3-06 文件：${path}`);
check(!existsSync(join(root,"crates/application/src/prediction.rs")),"旧 prediction.rs 仍残留 Prediction owner 或空转发层");
check(!existsSync(join(root,"crates/application/src/p4_persistence.rs")),"旧 p4_persistence.rs 在 R3-07 Artifact 迁移后重新出现");
const facade=read("crates/application/src/services/prediction/facade.rs"); const service=read("crates/application/src/services/prediction/service.rs"); const adapter=read("crates/application/src/composition/adapters/prediction.rs"); const ports=read("crates/application/src/ports/prediction/mod.rs"); const useCases=read("crates/application/src/use_cases/prediction/mod.rs"); const snapshotUseCase=read("crates/application/src/use_cases/prediction/p4_snapshot/mod.rs"); const orchestrationUseCase=read("crates/application/src/use_cases/p4_orchestration/process_next.rs"); const researchFacade=read("crates/application/src/services/research/facade.rs"); const packageJson=JSON.parse(read("package.json")); const frontend=read("scripts/verify-frontend.mjs");
for(const method of publicMethods){ check(facade.includes(`fn ${method}`),`Prediction facade 缺少公共兼容方法：${method}`); check(service.includes(`fn ${method}`),`PredictionService 缺少职责：${method}`); }
check(ports.includes("trait P4FreezeExecutionPort"),"Prediction Ports 缺少 P4FreezeExecutionPort"); check(ports.includes("async fn freeze_snapshot("),"P4FreezeExecutionPort 缺少 freeze_snapshot"); check(ports.includes("async fn read_snapshot("),"P4FreezeExecutionPort 缺少 read_snapshot");
check(adapter.includes("impl P4FreezeExecutionPort for PersistenceStore"),"Prediction 组合适配器缺少 P4FreezeExecutionPort"); check(adapter.includes(".freeze_prematch_snapshot(draft)"),"Prediction 组合适配器未复用既有快照写入"); check(adapter.includes(".read_prematch_snapshot(snapshot_id)"),"Prediction 组合适配器未复用既有快照读取");
check(useCases.includes("pub(crate) mod p4_snapshot;"),"Prediction Use Cases 未登记 P4 Snapshot 模块");
check(snapshotUseCase.includes("pub(crate) async fn freeze<"),"P4 Snapshot use case 缺少 freeze 内部职责");
check(snapshotUseCase.includes("port.freeze_snapshot(&draft).await?"),"P4 Snapshot freeze 未通过 P4FreezeExecutionPort");
check(snapshotUseCase.includes("pub(crate) async fn read<"),"P4 Snapshot use case 缺少 read 内部职责");
check(snapshotUseCase.includes("port.read_snapshot(snapshot_id).await?"),"P4 Snapshot read 未通过 P4FreezeExecutionPort");
check(!existsSync(join(root,"crates/application/src/p4_orchestration.rs")),"旧 p4_orchestration.rs owner 仍残留"); check(!orchestrationUseCase.includes("async fn execute_p4_freeze_task("),"P4 orchestration use case 不得实现 freeze execution"); check(read("crates/application/src/use_cases/p4_orchestration/dispatch.rs").includes(".execute_p4_freeze_task("),"P4 orchestration 未委托 Prediction Service 的 freeze use case");
check(!existsSync(join(root,"crates/application/src/p4_workbench.rs")),"AT5 后旧 p4_workbench.rs 仍残留跨 Prediction/Research 混合 owner"); check(researchFacade.includes("fn resolve_p4_conflict"),"R3-07 冲突写入职责未迁入 Research facade");
const predictionFiles=[...rustFiles("crates/application/src/services/prediction"),...rustFiles("crates/application/src/use_cases/prediction")];
for(const path of predictionFiles){const source=read(path); for(const token of ["football_persistence_postgres","PostgresStore","sqlx::","PgPool","PersistenceStore"]) check(!source.includes(token),`${path} 泄漏具体持久化实现：${token}`); for(const token of ["football_model_stub","model_p4","private_model"]) check(!source.includes(token),`${path} 绕过 model-api/registry 边界：${token}`);}
check(packageJson.scripts?.["verify:prediction-service"]==="node scripts/verify-prediction-service.mjs","package.json 未登记 R3-06 专项门禁"); check(packageJson.scripts?.["verify:architecture"]?.includes("verify-prediction-service.mjs"),"verify:architecture 未接入 R3-06 门禁"); check(frontend.includes('"verify-prediction-service.mjs"'),"verify:frontend 未接入 R3-06 门禁");
// R7-06：时间比较使用实际 SQLx/PostgreSQL 精度，载荷指纹仍保留原输入。
const records = read("crates/persistence-postgres/src/p4_records.rs");
check(records.includes("data_cutoff_at = $2::timestamptz AS cutoff_matches") && records.includes("$2::timestamptz < kickoff_time AS cutoff_before_kickoff"), "P4 快照引用时间必须在 PostgreSQL 精度下精确比较");
check(records.includes("published_at > $2::timestamptz") && records.includes("effective_at > $2::timestamptz"), "P4 证据截止必须使用相同数据库时间边界");
check(records.includes("RETURNING created_at, data_cutoff_time, frozen_at") && records.includes('data_cutoff_at: row.try_get("data_cutoff_time")?') && records.includes('frozen_at: row.try_get("frozen_at")?'), "首建快照必须返回实际落库时间");
check(records.includes("snapshot_fingerprint_preserves_submicrosecond_input_identity"), "纳秒输入身份不能通过时间容差放宽");
// R8-01：输入构建唯一 owner；沿用原 Port、时钟、权限和审计契约。
const builderPath = "crates/application/src/use_cases/prediction/build_input/mod.rs";
check(existsSync(join(root,builderPath)), "R8-01 缺少输入构建 owner");
const builder = read(builderPath);
const builderProduction = builder.split("#[cfg(test)]")[0];
const storedExecution = read("crates/application/src/use_cases/prediction/execute_prediction_from_match/mod.rs");
check(useCases.includes("pub(crate) mod build_input;"), "R8-01 输入构建模块未登记");
check(storedExecution.includes("super::build_input::execute(port, registry, command, persist_run).await?") && storedExecution.includes("execute_internal(port, registry, input, persist_run).await"), "正式/影子执行必须顺序复用输入构建及原执行器，保留模式");
for (const token of ["prepare_match_input", "inspect_match_prediction_readiness", "attach_prediction_input_audit", "verify_prepared_input_matches_readiness", "PredictionCommand {"]) check(!storedExecution.includes(token), `原执行 owner 残留输入构建：${token}`);
check(builderProduction.includes("readiness::execute(port, registry, command.clone()).await?"), "输入构建必须执行既有完整度评估");
check(builderProduction.includes("readiness.can_run_formal") && builderProduction.includes("readiness.can_run_shadow") && builderProduction.includes("if !allowed {") && builderProduction.includes("return Err(ApplicationError::Validation(format!("), "输入 I/O 前必须按模式拒绝不允许的推演");
const constructionSteps = ["if !allowed {", "let model_family = normalize_model_selection", ".prepare_match_input_at(", "verify_prepared_input_matches_readiness(&prepared, &readiness)?;", "attach_prediction_input_audit(&mut prepared.match_input, &readiness)?;", "Ok(PredictionCommand {"];
const positions = constructionSteps.map(token=>builderProduction.indexOf(token));
check(positions.every((position,index)=>position>=0 && (index===0 || position>positions[index-1])), "输入构建顺序必须为权限、家族、准备、指纹、审计、命令");
check(/&model_family,\s*readiness\.assessed_at,/.test(builderProduction), "输入准备必须复用评估时间及规范化家族");
for(const field of ["match_input: prepared.match_input", "snapshot_type: prepared.snapshot_type", "competition_id: prepared.match_record.competition_id", "season_id: prepared.match_record.season_id", "stage_id: prepared.match_record.stage_id", "competition_kind: prepared.competition_kind", "model_family: command.model_family", "explicit_rule_package_id: command.explicit_rule_package_id"]) check(builderProduction.includes(field), `输入构建丢失原命令字段：${field}`);
for(const token of ["Utc::now", "Uuid::new", "execute_internal(", "save_run(", ".predict(", "sqlx::", "prepare_match_input("]) check(!builderProduction.includes(token), `输入构建越界副作用或重取时钟：${token}`);
for(const test of ["audited_input_preserves_clock_family_route_and_manifest", "formal_and_shadow_permissions_block_input_io", "changed_input_manifest_blocks_construction", "runtime_identity_changes_preserve_assessed_manifest", "input_port_failure_and_missing_audit_do_not_return_command", "invalid_family_and_non_object_input_keep_existing_errors"]) check(builder.includes(`async fn ${test}`), `R8-01 缺少既有 target 内的行为测试：${test}`);
// R8-02：历史 SQL 单一读取 owner，原特征投影保留真实职责和原算法。
const historicalRoot = "crates/persistence-postgres/src/adapters/prediction/historical_features";
const historyReader = read(`${historicalRoot}/mod.rs`);
const historySql = read(`${historicalRoot}/read.rs`);
const featureProjection = read("crates/persistence-postgres/src/team_features.rs");
check(read("crates/persistence-postgres/src/adapters/mod.rs").includes("mod prediction;") && read("crates/persistence-postgres/src/adapters/prediction/mod.rs").includes("mod historical_features;"), "R8-02 历史读取 owner 必须登记在现有 adapters");
check(historyReader.includes("fn calculate_team_pre_match_features(") && historyReader.includes("read::team_history(&self.pool, fixture, team_id, is_home, data_cutoff_time).await?") && historyReader.includes("read::goal_baseline("), "原输入调用必须由唯一 reader 读取历史与基准");
check(featureProjection.includes("fn project_team_features(") && !featureProjection.includes("sqlx::") && !featureProjection.includes("impl PostgresStore") && !featureProjection.includes("async fn"), "team_features 必须是无 I/O 的原特征投影，不能留下历史 SQL 或空转发");
check(historyReader.includes("const MAX_HISTORY_MATCHES: usize = 12;") && historyReader.includes("const MIN_SCOPED_MATCHES: usize = 4;") && historySql.includes("const HISTORY_QUERY_LIMIT: i64 = 36;"), "历史候选/范围阈值/选中上限不得改变");
check(historyReader.indexOf("select_history_scope(&all_matches)") < historyReader.indexOf(".take(MAX_HISTORY_MATCHES)"), "历史范围必须先选择再截取 12 场");
for (const token of ["WHERE historical.id <> $1", "historical.kickoff_time < $2", "result.finalized_at <= $3", "result.created_at <= $3", "historical.home_team_id = $4 OR historical.away_team_id = $4", "ORDER BY historical.kickoff_time DESC, historical.id DESC", "LIMIT $5", "historical.kickoff_time < $1", "result.finalized_at <= $2", "result.created_at <= $2", "$3::uuid IS NULL OR historical.competition_id = $3"]) check(historySql.includes(token), `历史时间/身份/范围 SQL 丢失：${token}`);
check(/\.bind\(fixture\.id\)\s*\.bind\(fixture\.kickoff_time\)\s*\.bind\(data_cutoff_time\)\s*\.bind\(team_id\)\s*\.bind\(HISTORY_QUERY_LIMIT\)/.test(historySql), "历史查询绑定必须保留比赛、开球、cutoff、球队与 36 上限顺序");
check(/\.bind\(kickoff_time\)\s*\.bind\(data_cutoff_time\)\s*\.bind\(competition_id\)/.test(historySql), "进球基准绑定必须复用原开球、cutoff 和赛事");
check(historyReader.indexOf("if all_matches.is_empty()") < historyReader.indexOf("read::goal_baseline("), "无历史必须按原顺序直接返回中性，不读取额外基准");
const historicalOwnerCount = rustFiles("crates/persistence-postgres/src").map(read).join("\n").match(/\bfn calculate_team_pre_match_features\(/g)?.length ?? 0;
check(historicalOwnerCount === 1, "球队历史读取不得出现重复 owner");
check(historySql.includes("query_as::<_, HistoricalResultRow>") && historySql.includes("query_as::<_, GoalBaselineRow>") && historySql.includes("fn map_history(") && historySql.includes("fn map_baseline("), "历史查询必须收敛到 typed row 及唯一映射");
for(const token of ["Utc::now", "Uuid::new", "INSERT INTO", "UPDATE ", "DELETE FROM", ".predict(", "save_run("]) check(!historySql.split("#[cfg(test)]")[0].includes(token) && !historyReader.split("#[cfg(test)]")[0].includes(token), `历史 reader 越界时钟/写入：${token}`);
check(featureProjection.includes("pub(crate) const DEFAULT_GOAL_BASELINE: f64 = 1.15;") && featureProjection.includes("0.5_f64.powf(age_days / 90.0).max(0.05)") && featureProjection.includes("(rating_confidence * 0.65).clamp(0.0, 0.75)"), "原基准、衰减及同源置信度去重不得改变");
for(const test of ["history_scope_requires_four_samples_and_prefers_same_season", "history_scope_filters_before_twelve_match_limit_and_keeps_order"]) check(historyReader.includes(`fn ${test}`), `历史范围缺少 inline 行为测试：${test}`);
for(const test of ["typed_history_mapping_keeps_team_perspective_venue_and_scope", "typed_baseline_mapping_keeps_default_and_non_finite_policy"]) check(historySql.includes(`fn ${test}`), `历史 row 缺少 inline 行为测试：${test}`);
for(const test of ["historical_feature_projection_keeps_fixed_platform_scores_and_evidence", "empty_history_keeps_neutral_features_without_evidence", "ratio_curve_is_continuous_and_centered", "history_curve_rewards_better_results", "venue_curve_stays_bounded"]) check(featureProjection.includes(`fn ${test}`), `历史投影缺少既有/追加测试：${test}`);
const pgHistory = read("crates/persistence-postgres/tests/postgres_integration.rs");
check(pgHistory.includes("historical_snapshot_excludes_results_ingested_after_the_cutoff") && pgHistory.includes("for column in [\"finalized_at\", \"created_at\"]") && pgHistory.includes("for finalized_is_future in [true, false]") && pgHistory.includes("cutoff + Duration::microseconds(1)") && pgHistory.includes("cutoff - Duration::microseconds(1)"), "原 PG cutoff target 必须保留并增加精确数据库时间边界");
// R8-03：唯一只读审计工作流；原检查/分级语义与原目标内测试。
const readinessRoot = "crates/application/src/use_cases/prediction/readiness";
const readinessExport = read(`${readinessRoot}/mod.rs`);
const readinessFlow = read(`${readinessRoot}/workflow.rs`);
const readinessLineups = read(`${readinessRoot}/lineups.rs`);
const readinessQuality = read(`${readinessRoot}/input_quality.rs`);
const readinessReport = read(`${readinessRoot}/report.rs`);
const readinessTests = read(`${readinessRoot}/tests.rs`);
check(useCases.includes("pub(crate) mod readiness;") && readinessExport.includes("pub(crate) use workflow::execute;"), "R8-03 审计模块必须登记且出口只导出");
check(!readinessExport.includes("fn ") && !existsSync(join(root,"crates/application/src/use_cases/prediction/inspect_match_prediction_readiness")) && !existsSync(join(root,"crates/application/src/use_cases/prediction/shared/readiness_checks.rs")), "旧审计入口/检查不得保留空转发或重复 owner");
check(service.includes("readiness::execute(port, registry, command).await"), "公开审计须直接使用唯一 owner");
check((readinessFlow.match(/Utc::now\(\)/g) ?? []).length === 1 && readinessFlow.includes("let assessed_at = Utc::now();"), "审计只能取得一次评估时钟");
check(readinessFlow.includes(".read_match_chain_at(command.match_id, &command.snapshot_type, assessed_at)") && /model_selection\.family,\s*assessed_at,/.test(readinessFlow), "阵容窗口和准备输入必须复用同一审计时间");
const auditSteps = ["let assessed_at =", "normalize_model_selection", "ensure_model_selection_registered(registry", ".read_match(command.match_id)", ".read_match_chain_at(", ".resolve_competition_context(", ".resolve_route(", ".prepare_match_input_at(", "summarize(&checks, &shadow_reasons)", "let input_manifest =", "map(sha256_value).transpose()?", "Ok(MatchPredictionReadiness {"];
const auditPositions = auditSteps.map(token => readinessFlow.indexOf(token, readinessFlow.indexOf("pub(crate) async fn execute")));
check(auditPositions.every((position, index) => position >= 0 && (index === 0 || position > auditPositions[index-1])), "审计验证/读取/路由/输入/报告顺序漂移");
check(readinessFlow.includes(".is_some_and(|chain| chain.ready_for_model)") && (readinessFlow.match(/error.kind == PortErrorKind::InvalidState/g) ?? []).length === 2 && readinessFlow.includes("error.kind == PortErrorKind::NotFound") && (readinessFlow.match(/Err\(error\) => return Err\(error.into\(\)\)/g) ?? []).length === 3, "窗口/准备状态与缺失路由生成报告，其他 Port 错误必须原样停止");
for (const token of ["command.explicit_rule_package_id.is_none()", "validate_snapshot_type(&command.snapshot_type", "if !model.supports(&context)", "can_run_formal: level.can_run_formal()", "can_run_shadow: level.can_run_shadow()", "route_identity.as_ref()", "&match_record,", "&command.snapshot_type,"]) check(readinessFlow.includes(token), `审计丢失兼容规则：${token}`);
for (const [name, source] of [["workflow",readinessFlow], ["lineups",readinessLineups], ["quality",readinessQuality], ["report",readinessReport]]) {
  for (const token of ["save_run(", ".predict(", "enqueue", "Uuid::new", "prepare_match_input("]) check(!source.includes(token), `只读审计 ${name} 越界：${token}`);
  if (name !== "workflow") for (const token of ["async fn", "Utc::now", "port.", "store.", "sqlx::"]) check(!source.includes(token), `审计纯检查 ${name} 引入 I/O：${token}`);
}
check(readinessLineups.includes("let selected_id = chain.selected_lineup_id?;") && readinessLineups.includes(".find(|lineup| lineup.id == selected_id)") && !readinessLineups.includes(".first()") && readinessLineups.includes("goalkeeper_count != 1") && readinessLineups.includes('code.eq_ignore_ascii_case("GK")'), "阵容选择必须精确匹配且首发只能有一个门将");
check(readinessQuality.includes("home_history >= 5 && away_history >= 5") && readinessQuality.includes("home_history == 0 || away_history == 0") && readinessQuality.includes(".clamp(0.0, 1.0)") && readinessQuality.includes("quality_score >= 0.65") && readinessQuality.includes("quality_score >= 0.40") && readinessQuality.includes("quality_score < 0.40"), "历史/质量门槛与影子规则漂移");
check(readinessReport.includes("score: score.min(weight)") && /sum::<u16>\(\)\s*\.min\(100\) as u8/.test(readinessReport) && readinessReport.includes("if !warnings.contains(reason)"), "报告评分上限或影子原因去重漂移");
check(/if !blockers.is_empty\(\) \{\s*PredictionReadinessLevel::Blocked\s*\} else if !shadow_reasons.is_empty\(\) \{\s*PredictionReadinessLevel::ShadowOnly\s*\} else if !warnings.is_empty\(\) \{\s*PredictionReadinessLevel::ReadyWithWarnings\s*\} else \{\s*PredictionReadinessLevel::FormalReady/.test(readinessReport), "报告分级必须依次阻断、影子、警告、正式，无评分替代门禁");
for (const test of ["formal_report_preserves_one_clock_order_route_manifest_and_read_only_calls", "history_and_quality_thresholds_preserve_scores_and_mode_permissions", "lineup_selection_goalkeepers_starter_context_and_identity_keep_blocking_priority", "unavailable_window_input_and_missing_route_become_reports_without_retry", "port_unavailable_errors_stop_at_the_original_read_boundary", "route_snapshot_scope_and_model_support_failures_remain_blocked_reports", "invalid_family_and_unregistered_selection_reject_before_read_io", "report_classification_preserves_labels_reason_order_dedup_and_score_caps"]) check(readinessTests.includes(`fn ${test}`), `R8-03 缺少原目标内行为测试：${test}`);
// R8-04：确定性清单与审计协议唯一 owner，沿用原 JSON 字节/指纹。
const manifestRoot = "crates/application/src/use_cases/prediction/input_manifest";
const manifestExport = read(`${manifestRoot}/mod.rs`);
const canonicalManifest = read(`${manifestRoot}/canonical.rs`);
const inputAudit = read(`${manifestRoot}/audit.rs`);
const manifestTests = read(`${manifestRoot}/tests.rs`);
const predictionExecution = read("crates/application/src/use_cases/prediction/execute_prediction/mod.rs");
check(useCases.includes("pub(crate) mod input_manifest;") && manifestExport.includes("pub(crate) use audit::") && manifestExport.includes("pub(crate) use canonical::"), "R8-04 输入清单必须登记且显式导出");
check(!manifestExport.includes("fn ") && !existsSync(join(root,"crates/application/src/use_cases/prediction/shared/audit.rs")) && !read("crates/application/src/use_cases/prediction/shared/mod.rs").includes("mod audit;"), "旧 shared audit 必须删除，不保留双实现或空转发");
for (const [name, source] of [["builder",builder], ["readiness",readinessFlow], ["executor",predictionExecution]]) check(source.includes("input_manifest::{") && !source.includes("shared::audit"), `清单调用方 ${name} 必须使用唯一新 owner`);
check(canonicalManifest.includes("let mut canonical_input = input.clone();") && canonicalManifest.includes("strip_runtime_prediction_input_identity(&mut canonical_input);"), "清单必须在副本上排除运行身份，不能修改调用者输入");
for (const field of ["audit_version", "database_match_id", "match_key", "competition_id", "season_id", "stage_id", "home_team_id", "away_team_id", "kickoff_time", "snapshot_type", "route_identity", "model_input", "data_quality"]) check(canonicalManifest.includes(`"${field}":`), `清单丢失原字段：${field}`);
const excluded = [...canonicalManifest.matchAll(/(?:object|snapshot|source)\.remove\("([^"]+)"\)/g)].map(match=>match[1]);
check(JSON.stringify(excluded) === JSON.stringify(["feature_snapshot_id", "input_audit", "snapshot_id", "frozen_at", "accessed_at"]), "只排除原五个运行身份字段，禁止放宽事实指纹");
check(canonicalManifest.includes('object.get_mut("snapshot").and_then(Value::as_object_mut)') && canonicalManifest.includes('object.get_mut("sources").and_then(Value::as_array_mut)') && canonicalManifest.includes("source.as_object_mut()"), "运行身份排除必须限定原层级和 JSON 形态");
check(canonicalManifest.includes("let bytes = serde_json::to_vec(value)?;") && canonicalManifest.includes("Ok(hex::encode(Sha256::digest(bytes)))"), "必须保持原 JSON 字节、SHA256 和小写 hex，不引入另一个 canonicalization");
const verifyBody = inputAudit.slice(inputAudit.indexOf("pub(crate) fn verify_prepared_input_matches_readiness"), inputAudit.indexOf("pub(crate) fn attach_prediction_input_audit"));
check(verifyBody.includes("readiness.input_manifest_sha256.as_deref().ok_or_else") && verifyBody.includes("let actual_sha256 = sha256_value(&manifest)?;") && verifyBody.includes("if actual_sha256 != expected_sha256") && verifyBody.includes("route_identity.as_ref()"), "受检重建必须比较受审计哈希及原路由/质量事实");
check(verifyBody.indexOf("readiness.input_manifest_sha256") < verifyBody.indexOf("let manifest ="), "缺失受审计指纹的错误必须前置");
const attachment = inputAudit.slice(inputAudit.indexOf("pub(crate) fn attach_prediction_input_audit"), inputAudit.indexOf("pub(crate) fn prediction_input_audit_summary"));
check(attachment.indexOf("readiness.input_manifest.clone()") < attachment.indexOf("readiness.input_manifest_sha256.clone()") && attachment.indexOf("readiness.input_manifest_sha256.clone()") < attachment.indexOf(".as_object_mut()"), "审计附加必须保留缺失清单、缺失哈希、非对象输入错误优先级");
for (const field of ["audit_version", "assessed_at", "level", "score", "can_run_formal", "can_run_shadow", "blockers", "warnings", "checks", "manifest", "manifest_sha256"]) check(attachment.includes(`"${field}":`), `审计附加丢失原字段：${field}`);
check(inputAudit.includes("if calculated_manifest_sha256 != manifest_sha256") && inputAudit.includes('.unwrap_or("not_assessed")') && inputAudit.includes("u8::try_from(value).ok()") && inputAudit.includes("input_sha256: input_sha256.to_string()"), "审计摘要须复核清单并保持可选元数据和原输入哈希");
const executionOrder = ["let input_sha256 = sha256_value(&request.input)?;", "let input_audit = prediction_input_audit_summary(&request.input, &input_sha256)?;", "prediction_model_adapter::execute(model.as_ref(), &request)?", ".save_successful_run("];
const executionPositions = executionOrder.map(token=>predictionExecution.indexOf(token));
check(executionPositions.every((position,index)=>position>=0 && (index===0 || position>executionPositions[index-1])), "执行必须先复核审计再 predict/save，原输入哈希不得先排除运行身份");
for (const path of predictionFiles) {
  const source = read(path).split("#[cfg(test)]")[0];
  for (const fn of ["build_prediction_input_manifest", "verify_prepared_input_matches_readiness", "attach_prediction_input_audit", "prediction_input_audit_summary"]) if(source.includes(`fn ${fn}(`)) check(path.startsWith(manifestRoot+"/"), `清单/审计重复 owner：${path} ${fn}`);
}
for (const source of [canonicalManifest,inputAudit]) for (const token of ["async fn", "Utc::now", "Uuid::new", "port.", "store.", "sqlx::", ".predict(", "save_successful_run("]) check(!source.includes(token), `纯清单/审计不得增加 I/O 或重取身份：${token}`);
for (const test of ["input_manifest_ignores_runtime_snapshot_identity", "audit_summary_rejects_modified_manifest", "fixed_manifest_preserves_original_shape_serialization_and_fingerprint", "runtime_exclusion_is_limited_to_original_documented_locations", "semantic_input_quality_route_match_and_window_changes_alter_fingerprint", "non_object_input_and_null_absence_preserve_original_manifest_semantics", "prepared_verification_preserves_hash_requirement_runtime_exclusion_and_drift_errors", "audit_attachment_preserves_payload_and_failure_priority", "audit_summary_preserves_field_errors_fallbacks_and_untrimmed_identity", "invalid_audit_stops_execution_before_prediction_and_history_write"]) check(manifestTests.includes(`fn ${test}`), `R8-04 缺少原目标内保留/新增测试：${test}`);
check(manifestTests.includes("178afe68af4d0cb8ba9341a7f5f47ec3b89c4c2b9ceaafd0e6615db3a9a0fb87"), "固定公开平台清单指纹必须保持");
// R8-05：路由/请求唯一纯 owner；Port 编排留在既有用例，模型调用由 R8-06 adapter 承担。
const requestRoot = "crates/application/src/use_cases/prediction/route_model_request";
const requestExport = read(`${requestRoot}/mod.rs`);
const requestSelection = read(`${requestRoot}/selection.rs`);
const requestRoute = read(`${requestRoot}/route.rs`);
const requestAssembly = read(`${requestRoot}/request.rs`);
const requestTests = read(`${requestRoot}/tests.rs`);
const routePreview = read("crates/application/src/use_cases/prediction/preview_route/mod.rs");
const defaultDryRun = read("crates/application/src/use_cases/prediction/dry_run_default_fixture/mod.rs");
check(useCases.includes("pub(crate) mod route_model_request;") && !requestExport.includes("fn "), "R8-05 必须登记并只导出");
check(/#\[cfg\(test\)\]\s*pub\(crate\) use request::ensure_match_input_id;/.test(requestExport) && !requestExport.split("#[cfg(test)]")[0].includes("ensure_match_input_id"), "仅供单测的身份 helper 导出必须限定 cfg(test)，普通库构建不得留下 unused re-export");

check(!existsSync(join(root,"crates/application/src/use_cases/prediction/shared/routing.rs")) && !read("crates/application/src/use_cases/prediction/shared/mod.rs").includes("mod routing;"), "旧 routing 必须完整删除");
for (const [name, source] of [["builder",builder], ["readiness",readinessFlow], ["preview",routePreview], ["executor",predictionExecution]]) check(source.includes("route_model_request::") && !source.includes("shared::routing"), `路由调用方 ${name} 未使用唯一 owner`);
for (const path of predictionFiles) {
  const source = read(path).split("#[cfg(test)]")[0];
  for (const fn of ["normalize_model_selection", "ensure_model_selection_registered", "match_context_from_command", "ensure_match_input_id", "parse_kickoff", "required_string", "nested_required_string", "compact_key_part", "validate_snapshot_type", "route_identity_manifest", "verify_route_identity_matches_input_audit", "apply_route_context", "build_model_request", "default_fixture_request"]) if(source.includes(`fn ${fn}(`)) check(path.startsWith(requestRoot+"/"), `重复路由/请求 owner：${path} ${fn}`);
}
for (const source of [requestSelection, requestRoute, requestAssembly]) for (const token of ["async fn", "Utc::now", "Uuid::new", "port.", "store.", "sqlx::", ".predict(", "save_successful_run(", "resolve_route("]) check(!source.includes(token), `路由/请求纯 owner 越界：${token}`);
check(requestSelection.includes('value.trim().to_ascii_lowercase()') && requestSelection.includes('normalized.starts_with("p4_")') && requestSelection.includes('normalized.starts_with("p7_")') && requestSelection.includes('if let Some(model_id) = selection.exact_model_id.as_deref()'), "家族默认/精确注册政策不得改变");
check(requestAssembly.includes('DateTime::parse_from_rfc3339(raw)') && requestAssembly.includes('value.with_timezone(&Utc)') && requestAssembly.includes('kickoff_time.format("%Y%m%dT%H%MZ")') && requestAssembly.includes('.take(12)') && requestAssembly.includes('"TEAM".to_string()'), "原时间、模拟身份和队名压缩必须保持");
check(requestAssembly.includes('if command.explicit_rule_package_id.is_some()') && requestAssembly.includes('context.competition_kind = decision.competition_profile.competition_kind;') && requestAssembly.includes('"explicit_competition_kind_override"') && requestAssembly.includes('else if decision.competition_profile.competition_kind != scope.competition_kind'), "显式上下文覆盖与自动类型拒绝必须保持");
check(requestAssembly.includes('let match_input = ensure_match_input_id(command.match_input, &context.match_key)?;') && requestAssembly.includes('model_id: decision.model_id.clone()') && requestAssembly.includes('model_version: decision.model_version.clone()') && requestAssembly.includes('parameter_version: decision.parameter_version.clone()') && requestAssembly.includes('rule_package_version: Some(decision.package_version.clone())') && requestAssembly.includes('parameters: decision.parameters.clone()'), "请求必须使用实际路由身份/参数，不回退默认");
check(requestAssembly.includes('.is_some_and(|value| !value.trim().is_empty())') && requestAssembly.includes('if !has_match_id {'), "原有效输入身份必须保留，只有缺失/空/非字符串才补齐");
check(requestRoute.includes('snapshot_type.trim().is_empty()') && requestRoute.includes('.any(|item| item == snapshot_type)') && requestRoute.includes('if expected != &actual') && requestRoute.includes('return Ok(());'), "快照精确成员和可选受审计路由身份不得放宽");
for (const field of ["source", "binding_id", "rule_package_id", "rule_package_key", "rule_package_version", "model_id", "model_version_id", "model_version", "parameter_set_id", "parameter_version", "competition_profile_id"]) check(requestRoute.includes(`"${field}":`), `路由身份丢字段：${field}`);
const routedSteps = [".resolve_competition_context(", "match_context_from_command(&command", "ensure_model_selection_registered(registry", ".resolve_route(", "apply_route_context(&command", "validate_snapshot_type(&command", "verify_route_identity_matches_input_audit(&decision", "prediction_model_adapter::supported_model(", "let request = build_model_request(command, context, &decision)?;", "let input_sha256 =", "prediction_model_adapter::execute(model.as_ref(), &request)?", ".save_successful_run("];
const routedPositions = routedSteps.map(token => predictionExecution.indexOf(token, predictionExecution.indexOf("pub(crate) async fn execute_internal")));
check(routedPositions.every((position,index)=>position>=0 && (index===0 || position>routedPositions[index-1])), "执行路由/支持/组装/审计/执行/保存错误顺序漂移");
check(!predictionExecution.includes("ModelRequest {") && !predictionExecution.includes("ModelIdentity {") && !predictionExecution.includes("explicit_competition_kind_override"), "执行器不得保留已迁出的请求组装/上下文 owner");
const previewSteps = ["parse_kickoff(&command.kickoff_time)?", ".resolve_competition_context(", "normalize_model_selection(&command.model_family)?", "ensure_model_selection_registered(registry", ".resolve_route(", "command.explicit_rule_package_id.is_none()", "Ok(decision)"];
const previewPositions = previewSteps.map(token=>routePreview.indexOf(token, routePreview.indexOf("pub(crate) async fn execute")));
check(previewPositions.every((position,index)=>position>=0 && (index===0 || position>previewPositions[index-1])), "预览解析/读 scope/规范化/路由/类型检查顺序漂移");
for (const token of [".predict(", ".supports(", "save_successful_run(", "build_model_request(", "validate_snapshot_type("]) check(!routePreview.includes(token), `预览不能增加模型/快照/写入行为：${token}`);
const defaultLookup = defaultDryRun.indexOf('prediction_model_adapter::registered_model(registry, P4_MODEL_ID)?');
check(defaultLookup >= 0 && defaultDryRun.includes('let request = default_fixture_request()?;') && defaultLookup < defaultDryRun.indexOf('default_fixture_request()?') && !defaultDryRun.includes('ModelRequest {'), "默认 dry run 必须先注册查找，再从唯一 owner 构建请求");
check(requestAssembly.includes('competition_kind: CompetitionKind::Custom') && requestAssembly.includes('metadata: Value::Null') && requestAssembly.includes('rule_package_version: None') && requestAssembly.includes('snapshot_type: "T-1h".to_string()'), "默认公开请求必须保持原 Custom/Null/无规则身份与窗口");
for (const test of ["model_selection_supports_family_and_exact_ids", "exact_model_must_exist_in_registry", "selection_defaults_whitespace_case_and_errors_keep_existing_policy", "manual_context_preserves_utc_scope_names_and_input_identity", "context_failure_priority_keeps_input_fields_before_model_selection", "explicit_route_override_and_request_keep_routed_identity_and_parameters", "snapshot_and_audited_route_preserve_exact_membership_and_all_identity_fields", "default_fixture_request_keeps_public_shell_payload_and_no_route_identity", "route_preview_is_read_only_and_preserves_resolution_error_order", "preview_and_executor_preserve_explicit_kind_policy_before_later_checks"]) check(requestTests.includes(`fn ${test}`), `R8-05 缺少原 target 行为测试：${test}`);
// R8-06：唯一注册模型调用边界；不提前迁移 R8-07 保存，不改变 dry run 策略。
const modelAdapterPath = "crates/application/src/model_registry/prediction_model_adapter.rs";
const modelAdapter = read(modelAdapterPath).split("#[cfg(test)]")[0];
const modelAdapterTests = read("crates/application/src/model_registry/prediction_model_adapter/tests.rs");
check(read("crates/application/src/model_registry/mod.rs").includes("pub(crate) mod prediction_model_adapter;"), "R8-06 adapter 必须登记为内部模块");
check(!existsSync(join(root,"crates/application/src/use_cases/prediction/execute-model")), "R8-06 不得创建只有空转发的 execute-model 层");
check(modelAdapter.includes(".get(model_id)") && modelAdapter.includes("ApplicationError::ModelNotFound(model_id.to_string())"), "注册模型查找必须精确匹配且保留缺失错误");
const supportSteps = ["let model = registered_model(registry, model_id)?;", "if !model.supports(context)", "model.descriptor().display_name", "scope_kind.as_str()", "Ok(model)"];
const supportPositions = supportSteps.map(token=>modelAdapter.indexOf(token));
check(supportPositions.every((position,index)=>position>=0 && (index===0 || position>supportPositions[index-1])), "注册查找/实际上下文 supports/原 scope 提示的顺序与语义必须保持");
check(/prediction_model_adapter::supported_model\(\s*registry,\s*&decision\.model_id,\s*&context,\s*scope\.competition_kind,\s*\)\?/.test(predictionExecution), "执行支持检查必须传实际路由 model/context 和原 scope kind");
check((modelAdapter.match(/\.predict\(/g) ?? []).length === 1 && modelAdapter.includes(".predict(request)") && modelAdapter.includes(".map_err(|error| ApplicationError::Model(error.to_string()))"), "adapter 必须原样传借用请求并保留完整模型错误，不重试");
const timingSteps = ["let started = Instant::now();", "let output = predict(model, request)?;", "let duration_ms = elapsed_milliseconds(started.elapsed());", "Ok((output, duration_ms))"];
const timingPositions = timingSteps.map(token=>modelAdapter.indexOf(token));
check(timingPositions.every((position,index)=>position>=0 && (index===0 || position>timingPositions[index-1])) && (modelAdapter.match(/Instant::now\(\)/g) ?? []).length === 1 && modelAdapter.includes("elapsed.as_millis().min(i64::MAX as u128) as i64"), "耗时必须只统计原模型调用，截断毫秒并按 i64 上限饱和");
for (const source of [predictionExecution, defaultDryRun]) for (const token of [".predict(", ".supports(", "Instant::now", ".map_err(|error| ApplicationError::Model", ".validate(", ".validate_input(", ".validate_parameters("]) check(!source.includes(token), `调用方残留模型执行 owner：${token}`);
check(defaultDryRun.includes("prediction_model_adapter::predict(model.as_ref(), &request)") && !defaultDryRun.includes("supported_model(") && !defaultDryRun.includes("prediction_model_adapter::execute("), "默认 dry run 只能按原政策直接 predict，不增加支持检查或计时");
for (const token of ["async fn", "sqlx::", "PgPool", "PersistenceStore", "football_persistence_postgres", "football_model_stub", "model_p4", "private_model", "save_run(", "save_successful_run(", "build_model_request(", "sha256_value(", "prediction_input_audit_summary(", ".validate(", ".validate_input(", ".validate_parameters(", "ModelOutput {", "ModelRequest {", "Uuid::new", "Utc::now"]) check(!modelAdapter.includes(token), `模型 adapter 越界副作用/校验/载荷改写：${token}`);
check(predictionExecution.includes("let run_id = if persist_run {") && predictionExecution.includes(".save_successful_run(&decision, &request, &output, duration_ms)") && predictionExecution.includes("Uuid::nil()"), "必须保留正式保存/影子 nil 行为，Application 不绕过原 ModelRunPort");
for (const test of ["registered_lookup_keeps_exact_provider_identity_and_missing_error", "supported_lookup_uses_actual_context_and_original_scope_error", "timed_execution_preserves_complete_request_and_provider_output", "model_errors_keep_all_messages_without_retry_or_fallback", "elapsed_milliseconds_truncate_and_saturate_without_overflow", "default_dry_run_keeps_request_and_omits_supports_and_validation", "routed_failures_preserve_precedence_and_stop_history_writes"]) check(modelAdapterTests.includes(`fn ${test}`), `R8-06 缺少原 Application target 行为测试：${test}`);
// R8-07：运行、快照、明细与完成审计共同事务；历史隐藏不破坏运行身份。
const runsRoot = "crates/persistence-postgres/src/adapters/prediction/runs";
const runExports = read(`${runsRoot}/mod.rs`);
const runWrite = read(`${runsRoot}/write.rs`);
const runInput = read(`${runsRoot}/input.rs`);
const runDetails = read(`${runsRoot}/details.rs`);
const runRead = read(`${runsRoot}/read.rs`);
const runVisibility = read(`${runsRoot}/visibility.rs`);
check(!existsSync(join(root, "crates/persistence-postgres/src/model_runs.rs")) && !read("crates/persistence-postgres/src/lib.rs").includes("mod model_runs;"), "旧 model_runs 必须完整删除，禁止保留空转发");
check(!runExports.includes("fn ") && runExports.includes("pub use read::ModelRunListItem;") && read("crates/persistence-postgres/src/adapters/prediction/mod.rs").includes("mod runs;") && read("crates/persistence-postgres/src/lib.rs").includes("pub use adapters::ModelRunListItem;"), "运行持久化目录必须登记并保留显式公共 DTO 出口");
for (const name of ["save_successful_run", "list_recent_runs", "hide_run_from_history", "read_run"]) {
  const owners = rustFiles("crates/persistence-postgres/src").filter(path => read(path).includes(`pub async fn ${name}(`));
  check(owners.length === 1 && owners[0].startsWith(runsRoot + "/"), `运行公共方法 ${name} 必须只有一个新 owner`);
}
const runWriteOrder = ["let run_id = Uuid::new_v4();", "let input_hash = sha256_json(&request.input)?;", "prepared_run_input_audit(&request.input)?", "to_json_value(&output.summary)?", "optional_uuid(&request.input, \"database_match_id\")?", "prepared_feature_snapshot(&request.input, &request.snapshot_type)?", "feature_snapshot.is_some() && database_match_id.is_none()", "let mut tx = self.pool.begin().await?;", "INSERT INTO feature.snapshots", "INSERT INTO model.runs", "save_model_details(&mut tx, run_id, &output.payload).await?;", "write_audit_event(", "tx.commit().await?;", "Ok(run_id)"];
const runWritePositions = runWriteOrder.map(token => runWrite.indexOf(token));
check(runWritePositions.every((position,index) => position >= 0 && (index === 0 || position > runWritePositions[index-1])), "运行校验/快照/运行/明细/审计/共同提交顺序漂移");
check((runWrite.match(/self\.pool\.begin\(\)/g) ?? []).length === 1 && (runWrite.match(/tx\.commit\(\)/g) ?? []).length === 1 && !runWrite.includes(".execute(&self.pool)"), "运行保存必须使用唯一事务，不允许提前或额外提交");
for (const token of ["ON CONFLICT DO NOTHING", "ORDER BY CASE WHEN id = $1 THEN 0 ELSE 1 END", "source_kind IN ('legacy', 'runtime')", "feature_snapshot_id = Some(persisted_id);", "'runtime', 'none'", "'succeeded'", '"model_run_completed"', ".bind(&input_audit.manifest_sha256)", ".bind(&input_hash)", ".bind(&output.payload)", ".bind(&output.explanation)", ".execute(&mut *tx)"]) check(runWrite.includes(token), `运行快照复用/载荷/事务语义丢失：${token}`);
check(runDetails.includes("Transaction<'_, sqlx::Postgres>") && (runDetails.match(/\.execute\(&mut \*\*tx\)/g) ?? []).length === 2 && !runDetails.includes(".begin(") && !runDetails.includes(".commit("), "模型明细必须借用原事务，不能独立提交");
for (const token of ["INSERT INTO model.run_modules", "INSERT INTO model.run_scorelines", "module_key", ".split_once('_')", 'required_i64(item, "goals_a")?', 'required_i64(item, "goals_b")?', 'required_i64(item, "rank")?', "i16::try_from", 'required_f64(item, "probability")?', 'required_f64(item, "cumulative_probability")?', '.unwrap_or("未分类")', ".bind(details)", ".bind(item)"]) check(runDetails.includes(token), `模型明细原解析/原样载荷丢失：${token}`);
check(runInput.includes("if calculated != manifest_sha256") && runInput.includes("sha256_json(&manifest)?") && runInput.includes('"runtime-input-audit-v0"') && runInput.includes('"not_assessed"') && runInput.includes("i16::try_from(value)") && runInput.includes("!(0..=100).contains(&value)"), "运行审计必须保留原回退、分数与哈希复核");
check(runInput.includes("if snapshot_id != id") && runInput.includes("if snapshot_type != expected_snapshot_type") && runInput.includes("!quality_score.is_finite() || !(0.0..=1.0).contains(&quality_score)"), "特征快照身份、精确类型或质量边界漂移");
check(runRead.includes(".bind(limit.clamp(1, 500))") && runRead.includes("r.history_hidden_at IS NULL") && runRead.includes("WHERE r.status = 'succeeded'") && runRead.includes("ORDER BY r.created_at DESC, r.id DESC") && runRead.includes("ORDER BY scoreline.rank ASC, scoreline.probability DESC") && runRead.includes("read_model_run_identity(self, run_id).await?"), "运行历史上限、可见性、顺序、比分与共享身份读取必须保持");
check(runVisibility.includes("history_hidden_at = COALESCE(history_hidden_at, now())") && runVisibility.includes("WHERE id = $1 AND status = 'succeeded'") && runVisibility.includes(".map(str::trim)") && runVisibility.includes('.unwrap_or("用户从推演历史列表中删除")') && runVisibility.indexOf("write_audit_event(") < runVisibility.indexOf("tx.commit().await?;"), "隐藏历史必须保留首次时间、原提示/理由与共同事务审计");
for (const source of [runRead,runVisibility]) check(!source.includes("DELETE FROM") && !source.includes("SET input_payload"), "历史读取/隐藏不得删除运行或改写输入身份");
for (const test of ["prepared_input_audit_validates_manifest_hash", "legacy_audit_preserves_fallback_shape_hash_and_missing_values", "audited_input_preserves_trimmed_identity_levels_and_optional_score", "audit_field_errors_keep_priority_and_null_manifest_semantics", "feature_snapshot_keeps_original_identity_window_and_schema", "feature_snapshot_rejects_missing_metadata_and_changed_identity", "feature_snapshot_quality_keeps_closed_unit_interval_and_type_checks"]) check(runInput.includes(`fn ${test}`), `运行审计/快照缺少原目标内测试：${test}`);
check(runDetails.includes("fn model_detail_scalars_preserve_integer_numeric_and_missing_errors"), "模型明细缺少数值边界测试");
const runContract = read("crates/persistence-postgres/tests/model_run_identity_repository_contract.rs");
for (const token of ["save_successful_run", "list_recent_runs", "hide_run_from_history", "assert_eq!(snapshot_count, 1", "manifest_hash", "hidden_at", "invalid_rank", "history_hidden_reason", "model_run_completed", "model_run_history_hidden"]) check(runContract.includes(token), `既有 PG 运行保存/回滚/可见性契约缺少：${token}`);
check(/assert_eq!\(\s*counts,\s*\(0, 0, 0, 0, 0\)/.test(runContract), "PG 明细失败必须断言运行/快照/明细/审计全部回滚");
const applicationRunTests = read("crates/application/src/use_cases/prediction/tests.rs");
for (const name of ["formal_prediction_saves_normalized_input_and_routed_identity_once", "repeated_shadow_prediction_never_writes_model_history", "prediction_errors_stop_every_later_side_effect", "failed_run_save_is_propagated_without_success_or_automatic_retry"]) check(applicationRunTests.includes(`fn ${name}`), `正式/影子/失败保存原回归丢失：${name}`);
for (const kind of ["Unavailable", "NotFound", "Conflict", "InvalidState", "Serialization", "Infrastructure"]) check(applicationRunTests.includes(`PortErrorKind::${kind}`), `保存错误不得静默降级或重试：${kind}`);
// R8-08: evidence append/conflict ledgers own their original atomic writes.
const ledgerRoot = "crates/persistence-postgres/src/adapters/p4/evidence_ledger";
const ledgerExport = read(`${ledgerRoot}/mod.rs`);
const claimWriter = read(`${ledgerRoot}/claims.rs`);
const conflictWriter = read(`${ledgerRoot}/conflicts.rs`);
const ledgerInput = read(`${ledgerRoot}/input.rs`);
const ledgerReferences = read(`${ledgerRoot}/references.rs`);
const ledgerRow = read(`${ledgerRoot}/row.rs`);
const ledgerTests = read(`${ledgerRoot}/tests.rs`);
const p4Idempotency = read("crates/persistence-postgres/src/adapters/p4/idempotency.rs");
check(read("crates/persistence-postgres/src/adapters/mod.rs").includes("pub(crate) mod p4;") && read("crates/persistence-postgres/src/adapters/p4/mod.rs").includes("pub(crate) mod evidence_ledger;"), "R8-08 账本必须接入既有 adapter owner");
check(!ledgerExport.includes("fn ") && ledgerExport.includes("pub(crate) use row::parse_verification_state;") && records.includes("evidence_ledger::parse_verification_state"), "账本出口只登记/导出，共享验证状态投影唯一复用");
for (const name of ["append_evidence_claim", "create_evidence_conflict", "validate_evidence_claim", "evidence_claim_fingerprint", "validate_evidence_version_references", "evidence_claim_record_from_row", "parse_verification_state", "advisory_lock", "validate_idempotency_key", "ensure_idempotent_fingerprint"]) {
  check(!records.includes(`fn ${name}(`), `旧 p4_records 仍持有账本/共用幂等实现：${name}`);
  const sources=[records,claimWriter,conflictWriter,ledgerInput,ledgerReferences,ledgerRow,p4Idempotency].join("\n");
  check((sources.match(new RegExp(`\\bfn ${name}\\(`,"g"))??[]).length===1, `P4 账本/幂等职责必须唯一：${name}`);
}
for (const [label,source,key,preflight] of [["claim",claimWriter,"evidence","validate_evidence_claim(draft)?;"],["conflict",conflictWriter,"conflict","prepared_conflict(draft)?;"]]) {
  check(source.indexOf(preflight)<source.indexOf("let mut tx = self.pool.begin().await?;") && source.indexOf(preflight)>=0, `${label} 纯前检必须先于事务`);
  check(source.includes(`advisory_lock(&mut tx, &format!("${key}:{}", draft.`) && source.indexOf("advisory_lock(&mut tx")<source.indexOf(".fetch_optional(&mut *tx)"), `${label} 同键事务锁必须先于重试查找`);
  check((source.match(/self\.pool\.begin\(\)/g)??[]).length===1 && (source.match(/tx\.commit\(\)/g)??[]).length===2, `${label} 必须保留一个事务与原新建/重试两个出口`);
  check(source.indexOf("ensure_idempotent_fingerprint(")<source.indexOf("return Ok(record);") && source.lastIndexOf("write_audit_event(")<source.lastIndexOf("tx.commit().await?;"), `${label} 重试先核对载荷，新建审计先于提交`);
  check(!source.includes(".execute(&self.pool)") && !source.includes(".fetch_one(&self.pool)") && !source.includes("DELETE FROM") && !source.includes("UPDATE "), `${label} 不得逃逸事务或修改历史`);
}
check(p4Idempotency.includes("SELECT pg_advisory_xact_lock(hashtext($1)::bigint)") && p4Idempotency.includes(".execute(&mut **tx)") && p4Idempotency.includes("value.trim().is_empty() || value.len() > 240") && p4Idempotency.includes("if existing != expected"), "原P4幂等锁、字节键上限与精确指纹比较必须保持");
check(records.includes("idempotency::{advisory_lock, ensure_idempotent_fingerprint, validate_idempotency_key}"), "非账本 P4 owner 必须复用唯一幂等实现");
for (const token of ["if run.try_get::<Uuid, _>(\"match_id\")? != draft.match_id", "if run.try_get::<Uuid, _>(\"schema_version_id\")? != draft.schema_version_id", "validate_evidence_version_references(&mut tx, draft).await?", "evidence_claim_appended", '"claim_fingerprint": claim_fingerprint', "FROM research.evidence_claims", "WHERE idempotency_key = $1"]) check(claimWriter.includes(token), `证据声明必须保留身份/引用/审计：${token}`);
check(claimWriter.indexOf("validate_evidence_version_references(&mut tx")<claimWriter.indexOf("INSERT INTO research.evidence_claims"), "证据版本引用必须在首次写入前验证");
for(const token of ["registered_schema_version.as_deref() != Some(draft.schema_version.as_str())", "(None, None) => {}", "(Some(prompt_version_id), Some(prompt_version))", "registered_prompt_version.as_deref() != Some(prompt_version)", "证据Prompt版本ID与版本号必须同时提供或同时为空", 'conflict.try_get::<Uuid, _>("match_id")? != draft.match_id', 'conflict.try_get::<Option<Uuid>, _>("entity_id")? != draft.entity_id']) check(ledgerReferences.includes(token), `证据引用保持ID/版本与冲突身份：${token}`);
for(const token of ["draft.verification_state.requires_source()", "draft.source_url.as_deref().is_none_or(str::is_empty)", "draft.source_title.as_deref().is_none_or(str::is_empty)", "draft.source_domain.as_deref().is_none_or(str::is_empty)", "draft.retrieved_at < draft.observed_at", '"metadata": draft.metadata', '"published_at": draft.published_at', '"observed_at": draft.observed_at', '"retrieved_at": draft.retrieved_at', '"schema_version_id": draft.schema_version_id', '"content_sha256": content_sha256', "collect::<BTreeSet<_>>()", "if evidence_ids.len() < 2", '"trace_id": draft.trace_id']) check(ledgerInput.includes(token), `账本前检/指纹原语义丢失：${token}`);
for(const token of ["async fn", "sqlx::", "Utc::now", "Uuid::new", "PostgresStore"]) check(!ledgerInput.includes(token), `账本纯前检越界：${token}`);
for(const token of ["rows.len() != evidence_ids.len()", "|| entity_type != draft.entity_type", "|| entity_id != draft.entity_id", "|| field_key != draft.field_key", "INSERT INTO research.evidence_conflict_members", "INSERT INTO research.evidence_conflict_events", "'opened'", "evidence_conflict_opened"]) check(conflictWriter.includes(token), `冲突建组身份/明细/事件/审计原子链缺失：${token}`);
check(conflictWriter.indexOf("if rows.len() != evidence_ids.len()")<conflictWriter.indexOf("INSERT INTO research.evidence_conflicts") && conflictWriter.indexOf("INSERT INTO research.evidence_conflict_members")<conflictWriter.indexOf("INSERT INTO research.evidence_conflict_events") && conflictWriter.indexOf("INSERT INTO research.evidence_conflict_events")<conflictWriter.indexOf("write_audit_event("), "冲突必须先核对全组身份再写头/成员/opened/审计");
for(const state of ["CONFIRMED","PROBABLE","CONFLICT","NOT_FOUND","STALE","NOT_APPLICABLE"]) check(ledgerRow.includes(`"${state}" => Ok(EvidenceVerificationState::`), `证据Row状态映射缺少：${state}`);
for(const test of ["evidence_source_is_required_for_supported_facts", "source_states_and_complete_provenance_keep_original_policy", "claim_mandatory_fields_keep_error_priority_and_raw_values", "claim_retrieval_window_keeps_nanosecond_boundary", "claim_fingerprint_preserves_semantic_identity_and_retry_key_policy", "conflict_preparation_deduplicates_sorts_and_keeps_identity_policy", "conflict_preflight_keeps_byte_key_limit_and_distinct_member_minimum", "verification_row_parser_keeps_six_states_and_unknown_errors", "idempotent_retry_keeps_exact_fingerprint_and_original_error"]) check(ledgerTests.includes(`fn ${test}`), `账本缺少原target行为测试：${test}`);
for(const token of ["claim field {field} must survive unchanged", "claim fingerprint must reject metadata/raw nanosecond drift", "same-key first writers share the original transaction lock", "failed conflict leaves no header/member/event/audit", "evidence ledger must remain append-only", "concurrent_conflict", "tokio::join!", "source_fk", "prompt_pair", "conflict_identity"]) check(pgHistory.includes(token), `既有PG账本契约缺少：${token}`);

// R8-10: real planner/recovery/dispatcher/task transaction owners; no new verification entry.
const horizonRoot="crates/application/src/use_cases/prediction/plan_p4_horizons";
const planExports=read(`${horizonRoot}/mod.rs`),planPrepare=read(`${horizonRoot}/prepare.rs`),planSchedule=read(`${horizonRoot}/schedule.rs`),planProcess=read(`${horizonRoot}/process.rs`),planQueue=read(`${horizonRoot}/queue.rs`).split("#[cfg(test)]")[0];
const dispatch=read("crates/application/src/use_cases/p4_orchestration/dispatch.rs"),failure=read("crates/application/src/use_cases/p4_orchestration/failure.rs");
const taskRoot="crates/persistence-postgres/src/adapters/p4/horizon";
const taskExports=read(`${taskRoot}/mod.rs`),taskInput=read(`${taskRoot}/input.rs`).split("#[cfg(test)]")[0],taskWrite=read(`${taskRoot}/tasks.rs`),taskRead=read(`${taskRoot}/read.rs`),taskEvents=read(`${taskRoot}/events.rs`),taskRow=read(`${taskRoot}/row.rs`).split("#[cfg(test)]")[0];
const taskLegacy=read("crates/persistence-postgres/src/p4_orchestration.rs");
check(!planExports.includes("fn ") && planExports.includes("pub(crate) use process::execute;") && !taskExports.includes("fn "),"R8-10 目录只登记/导出，禁止保留编排或空转发");
for(const name of ["prepare","schedule","process","queue"]) check(planExports.includes(`mod ${name};`),`R8-10 缺少规划职责：${name}`);
check(read("crates/persistence-postgres/src/adapters/p4/mod.rs").includes("pub(crate) mod horizon;"),"R8-10 持久化职责未接入原 adapter");
const prepareOrder=[".planning_match_context(",".resolve_competition_context(",".resolve_route(","if !is_p4_model(","for horizon in P4Horizon::CANONICAL", "validate_requested_fact_keys(",".read_schema(RESEARCH_SCHEMA_KEY", ".read_schema(SNAPSHOT_SCHEMA_KEY"];
const preparePositions=prepareOrder.map(token=>planPrepare.indexOf(token, token==="validate_requested_fact_keys("?planPrepare.indexOf("pub(super) async fn"):0));
check(preparePositions.every((pos,index)=>pos>=0 && (index===0||pos>preparePositions[index-1])),"R8-10 context/显式路由/时点/29字段/两版Schema预检顺序漂移");
check(planPrepare.includes('preferred_model_family: Some("p4".to_string())') && planPrepare.includes("explicit_rule_package_id: Some(command.explicit_rule_package_id)"),"R8-10 必须保留显式P4规则选择");
check((planProcess.match(/Utc::now\(\)/g)??[]).length===1 && planProcess.indexOf("let now = Utc::now();")<planProcess.indexOf("for horizon in P4Horizon::CANONICAL") && planProcess.includes("Vec::with_capacity(P4Horizon::CANONICAL.len())"),"R8-10 必须单次捕获时钟、遍历原三个正式时点");
check(planProcess.indexOf("validate_existing_task_identity(")>=0 && planProcess.indexOf("validate_existing_task_identity(")<planProcess.indexOf("queue::resume(port, &existing, now)") && planProcess.includes("if draft.state == P4FreezeTaskState::Missed"),"R8-10 既有任务必须先复核身份，过期首建不得入队");
check(planQueue.includes("if task.state != P4FreezeTaskState::Planned") && planQueue.indexOf("if task.state != P4FreezeTaskState::Planned")<planQueue.indexOf("if task.data_cutoff_at <= now") && planQueue.indexOf("next_state: P4FreezeTaskState::Missed")<planQueue.indexOf(".enqueue("),"R8-10 重试只恢复PLANNED，截止相等必须MISSED且无队列写入");
for(const token of ['"p4-freeze:{}:{}:{}:{}:{}:{}"',"data_cutoff_at.timestamp()","Duration::minutes(P4_RESEARCH_LEAD_MINUTES)","Duration::minutes(P4_FREEZE_GRACE_MINUTES)","research_schema_version_id: plan.research_schema.id","snapshot_schema_version_id: plan.snapshot_schema.id"]) check(planSchedule.includes(token),`R8-10 时点身份/调度政策丢失：${token}`);
for(const token of ['"p4_horizon_research"','Some(format!("p4-research-job:{}", task.id))','payload: json!({"task_id": task.id})',"available_at: Some(task.research_due_at)","priority: horizon_priority(task.horizon)","max_attempts: 3", "expected_state: P4FreezeTaskState::Planned","next_state: P4FreezeTaskState::ResearchQueued","research_job_id: Some(research_job.id)"]) check(planQueue.includes(token),`R8-10 原队列幂等/载荷/状态绑定政策丢失：${token}`);
check(/claim_next_p4_job\(\s*port,\s*&\[dispatch::P4_RESEARCH_JOB, dispatch::P4_FREEZE_JOB\]/.test(orchestrationUseCase) && orchestrationUseCase.includes("settle_job(port, &job, result).await") && dispatch.includes("serde_json::from_value(payload.clone())?") && dispatch.includes(".execute_p4_research_task(") && dispatch.includes(".execute_p4_freeze_task("),"R8-10 原队列领取、载荷解码与跨服务分派必须各有唯一职责");
check(orchestrationUseCase.indexOf("failure::mark_terminal_failure(")<orchestrationUseCase.indexOf("P4OrchestrationQueuePort::fail_p4_job(") && failure.includes("if job.attempts < job.max_attempts") && failure.includes("task.state.is_terminal() || !task.state.can_transition_to(P4FreezeTaskState::Failed)"),"R8-10 必须保留次数耗尽/合法非终态/尽力标记及原失败优先级");
for(const name of ["p4_planning_match_context","read_schema_version_by_key","read_research_run","find_p4_freeze_task_by_idempotency","read_p4_freeze_task","list_p4_freeze_tasks","list_p4_freeze_task_events","create_p4_freeze_task","transition_p4_freeze_task"]) {
 const owners=rustFiles("crates/persistence-postgres/src").filter(path=>read(path).includes(`pub async fn ${name}(`));
 check(owners.length===1&&owners[0].startsWith(taskRoot+"/"),`R8-10 公共方法必须有唯一真实owner：${name}`);
 check(!taskLegacy.includes(`fn ${name}(`),`R8-10 旧根仍残留任务实现：${name}`);
}
check((taskLegacy.match(/\bfn /g)??[]).length===5 && ["p4_freeze_readiness","p4_route_readiness","p4_readiness","find_frozen_p4_snapshot_id","p4_routed_facts"].every(name=>taskLegacy.includes(`fn ${name}(`)),"R8-10 后续readiness/Freeze五项职责必须原位保留");
check(/use football_domain::\{[^}]*\bResearchRunStatus\b[^}]*\};/.test(taskLegacy) && taskLegacy.includes("ResearchRunStatus::Succeeded.as_str()"),"R8-10 保留的 readiness 必须导入其真实使用的 ResearchRunStatus，子模块导入不能替代根模块绑定");
const createWrite=taskWrite.slice(taskWrite.indexOf("pub async fn create_p4_freeze_task"),taskWrite.indexOf("pub async fn transition_p4_freeze_task"));
const transitionWrite=taskWrite.slice(taskWrite.indexOf("pub async fn transition_p4_freeze_task"),taskWrite.indexOf("async fn lock_key"));
check(createWrite.indexOf("prepare(draft)?")<createWrite.indexOf("self.pool.begin()") && createWrite.indexOf("lock_key(")<createWrite.indexOf("select_task_by_idempotency(") && createWrite.includes("if existing != task_fingerprint"),"R8-10 纯前检/同键事务锁/首次载荷复核顺序漂移");
for(const [label,source] of [["create",createWrite],["transition",transitionWrite]]) check((source.match(/self\.pool\.begin\(\)/g)??[]).length===1 && (source.match(/tx\.commit\(\)/g)??[]).length===2 && source.lastIndexOf("append_task_event(")<source.lastIndexOf("write_audit_event(") && source.lastIndexOf("write_audit_event(")<source.lastIndexOf("tx.commit().await?;") && !source.includes(".execute(&self.pool)"),`R8-10 ${label} 必须保持任务/事件/审计共同事务，重试与首建各一个出口`);
check(transitionWrite.includes("FOR UPDATE") && transitionWrite.indexOf("if current == transition.next_state")<transitionWrite.indexOf("if current != transition.expected_state") && transitionWrite.includes("!current.can_transition_to(transition.next_state)"),"R8-10 状态锁、同状态无写重试、预期冲突与合法迁移顺序漂移");
for(const token of ["research_run_id = COALESCE($4, research_run_id)","research_job_id = COALESCE($5, research_job_id)","freeze_job_id = COALESCE($6, freeze_job_id)","snapshot_id = COALESCE($7, snapshot_id)","blockers = CASE WHEN $3 = 'null'::jsonb THEN blockers ELSE $3 END"]) check(transitionWrite.includes(token),`R8-10 状态更新原空值语义丢失：${token}`);
check(taskEvents.includes("Transaction<'_, Postgres>") && taskEvents.includes("ON CONFLICT (task_id, idempotency_key) DO NOTHING") && taskEvents.includes("if existing != event_fingerprint") && !taskEvents.includes(".begin(") && !taskEvents.includes(".commit("),"R8-10 事件必须借用原事务并复核同键指纹");
check(taskInput.includes("if !draft.horizon.is_canonical()") && taskInput.includes("if draft.requested_fact_keys.is_empty()") && taskInput.includes("requested_fact_keys.sort();") && taskInput.includes("requested_fact_keys.dedup();") && taskInput.includes('"trace_id": draft.trace_id') && taskInput.includes('"data_cutoff_at": draft.data_cutoff_at') && !taskInput.includes("sqlx::"),"R8-10 原纯前检和纳秒载荷指纹不得弱化");
check(taskRead.includes("limit.clamp(1, 500)") && taskRead.includes("ORDER BY occurred_at, id") && taskRead.includes("WHEN 'T-90m' THEN 3") && taskRow.includes('"T-N" => Ok(P4Horizon::LegacyTN)') && taskRow.includes('"T-90m" => Ok(P4Horizon::T90m)'),"R8-10 读取上限/原排序/历史兼容投影必须保持");
const planTests=read("crates/application/src/services/p4_orchestration/tests.rs"),settleTests=read("crates/application/src/use_cases/p4_orchestration/tests.rs");
for(const test of ["planner_pins_three_formal_horizons_route_schemas_facts_and_queue_policy","planner_retry_preserves_progressed_and_terminal_tasks_without_writes","planner_rejects_route_capability_or_fact_subset_before_writes","planner_rejects_every_pinned_identity_drift_before_resume","planner_marks_elapsed_horizons_missed_without_enqueuing_them","planner_preserves_port_error_kind_and_stops_at_each_boundary","planner_recovers_creation_followed_by_enqueue_failure_with_same_task_identity","planner_recovers_binding_failure_using_original_idempotent_queue_job"]) check(planTests.includes(`fn ${test}`),`R8-10 原Application目标缺少规划回归：${test}`);
check(read(`${horizonRoot}/queue.rs`).includes("fn planned_resume_uses_inclusive_cutoff_and_preserves_other_states") && settleTests.includes("fn exhausted_dispatch_failure_marks_legal_task_then_fails_queue") && settleTests.includes("fn task_terminal_transition_is_best_effort_but_queue_failure_propagates"),"R8-10 原目标缺少截止/终态/失败恢复行为测试");
const horizonPg=read("crates/persistence-postgres/tests/postgres_integration.rs");
for(const token of ["let horizon_draft = P4FreezeTaskDraft", "create_p4_freeze_task(&concurrent_draft)", "transition_p4_freeze_task(&rejected)", "p4_freeze_task_created", "p4_freeze_task_transitioned", "UPDATE platform.p4_freeze_task_events SET reason='changed'", "horizon_task.data_cutoff_at"]) check(horizonPg.includes(token),`R8-10 既有PG夹具缺少任务事务/重试/不可变断言：${token}`);


// R8-11: existing read entry points, independent query owners and task/run scoped projections.
const workbenchRoot="crates/persistence-postgres/src/adapters/p4/workbench";
const workbenchExport=read(`${workbenchRoot}/mod.rs`),matchRead=read(`${workbenchRoot}/matches.rs`),workspaceRead=read(`${workbenchRoot}/tasks.rs`),researchRead=read(`${workbenchRoot}/research.rs`),evidenceRead=read(`${workbenchRoot}/evidence.rs`),conflictRead=read(`${workbenchRoot}/conflicts.rs`);
const workbenchLegacy=read("crates/persistence-postgres/src/p4_workbench.rs");
check(read("crates/persistence-postgres/src/adapters/p4/mod.rs").includes("pub(crate) mod workbench;") && !workbenchExport.includes("fn "),"R8-11 adapter必须接入，目录只显式登记职责");
for(const name of ["matches","tasks","research","evidence","conflicts"]) check(workbenchExport.includes(`mod ${name};`),`R8-11 缺少独立读取职责：${name}`);
for(const [name,file] of [["read_p4_match_workspace","matches"],["read_p4_task_workspace","tasks"]]) {
 const owners=rustFiles("crates/persistence-postgres/src").filter(path=>read(path).includes(`pub async fn ${name}(`));
 check(owners.length===1 && owners[0]===`${workbenchRoot}/${file}.rs` && !workbenchLegacy.includes(`fn ${name}(`),`R8-11 公共读取必须有唯一owner：${name}`);
}
check((workbenchLegacy.match(/\bfn /g)??[]).length===5 && ["append_p4_manual_route_override","validate_selected_conflict_evidence","append_conflict_event_in_tx","lock_override","manual_override_from_row"].every(name=>workbenchLegacy.includes(`fn ${name}(`)),"R8-11 必须保留五项原人工决策写入职责");
for(const type of ["P4ManualConflictDecisionKind","P4ManualRouteOverrideDraft","P4ManualRouteOverrideRecord"]) check(new RegExp(`use football_domain::\\{[^}]*\\b${type}\\b[^}]*\\};`).test(workbenchLegacy),`R8-11 保留writer缺少真实Domain导入：${type}`);
const readSources=[matchRead,workspaceRead,researchRead,evidenceRead,conflictRead];
for(const source of readSources) for(const token of [".begin(",".commit(",".execute(","INSERT INTO","UPDATE ","DELETE FROM","Uuid::new","Utc::now",".enqueue(",".transition_p4_freeze_task(","record_research_run_event(","append_p4_manual_route_override("]) check(!source.includes(token),`R8-11 读取职责不得写入、生成身份或重新执行工作流：${token}`);
check(!workspaceRead.includes("sqlx::") && !workspaceRead.includes("SELECT "),"R8-11 任务汇总只编排既有读取职责");
const workspaceOrder=[".read_p4_freeze_task(task_id)",".p4_freeze_readiness(task_id)",".list_p4_freeze_task_events(task_id)",".p4_routed_facts(task_id)","research::read(self, research_run_id)","evidence::read(self, research_run_id)","conflicts::read(self, research_run_id, task_id)",".read_prematch_snapshot(snapshot_id)","Ok(P4TaskWorkspace {"];
const workspacePositions=workspaceOrder.map(token=>workspaceRead.indexOf(token));
check(workspacePositions.every((pos,index)=>pos>=0 && (index===0 || pos>workspacePositions[index-1])),"R8-11 读取顺序/错误优先级必须保持");
check((workspaceRead.match(/if let Some\(research_run_id\) = task.research_run_id/g)??[]).length===3 && workspaceRead.includes("if let Some(snapshot_id) = task.snapshot_id") && (workspaceRead.match(/Vec::new\(\)/g)??[]).length===2 && (workspaceRead.match(/\n\s*None\n/g)??[]).length===2,"R8-11 无研究或快照时保持None/空集合，不做查询");
check(matchRead.includes("LEFT JOIN football.competitions") && matchRead.includes("WHERE fixture.id = $1") && matchRead.includes(".bind(match_id)") && matchRead.includes(".fetch_optional(&self.pool)") && matchRead.includes('PersistenceError::InvalidState("比赛不存在".to_string())') && matchRead.includes("self.list_p4_freeze_tasks(Some(match_id), 100).await?"),"R8-11 比赛精确身份/可空赛事/不存在错误/100任务上限必须保持");
check(researchRead.includes("WHERE id = $1") && researchRead.includes(".bind(research_run_id)") && researchRead.includes(".fetch_one(&store.pool)"),"R8-11 研究元数据按精确run读取，不添加回退");
for(const field of ["id","status","attempt_count","response_id","model_id","error_category","error_message","created_at","started_at","finished_at"]) check(researchRead.includes(`row.try_get("${field}")?`),`R8-11 研究元数据丢失：${field}`);
check(evidenceRead.includes("WHERE research_run_id = $1") && evidenceRead.includes(".bind(research_run_id)") && evidenceRead.includes("ORDER BY field_key, created_at, id"),"R8-11 证据必须按当前run过滤并保持三字段排序");
for(const field of ["id","field_key","entity_type","entity_id","value","verification_state","source_tier","source_url","source_title","source_domain","published_at","observed_at","effective_at","retrieved_at","timezone","conflict_group_id","created_at"]) check(evidenceRead.includes(`row.try_get("${field}")?`),`R8-11 原证据来源/时间/值投影丢失：${field}`);
for(const token of ["claim.research_run_id = $1","members ON cardinality(members.evidence_ids) > 0","COALESCE(latest_event.event_type, 'opened') AS conflict_status","evaluation.research_run_id = $1","manual_override.task_id = $2","manual_override.conflict_id = conflict.id","array_agg(member.evidence_id ORDER BY member.evidence_id)","ORDER BY event.occurred_at DESC, event.id DESC","ORDER BY evaluation.created_at DESC, evaluation.id DESC","ORDER BY manual_override.created_at DESC, manual_override.id DESC","ORDER BY conflict.field_key, conflict.created_at, conflict.id"]) check(conflictRead.includes(token),`R8-11 冲突run/task范围或最新事件/评估/人工裁决排序漂移：${token}`);
check(/\.bind\(research_run_id\)\s*\.bind\(task_id\)/.test(conflictRead) && /try_get::<Option<Vec<Uuid>>, _>\("selected_evidence_ids"\)\?\s*\.unwrap_or_default\(\)/.test(conflictRead),"R8-11 冲突run/task绑定顺序或NULL人工证据空数组语义丢失");
for(const [owner,dto,method,tests] of [["read_p4_match_workspace","P4MatchWorkspace","read_match_workspace",["match_workspace_preserves_identity_nullable_competition_and_task_order_without_writes","match_workspace_preserves_all_port_errors_without_retry_or_fallback"]],["read_p4_task_workspace","P4TaskWorkspace","read_task_workspace",["task_workspace_preserves_empty_and_populated_views_in_progressed_and_terminal_states","task_workspace_preserves_all_port_errors_without_partial_view_or_retry"]]]) {
 const source=read(`crates/application/src/use_cases/prediction/${owner}/mod.rs`),production=source.split("#[cfg(test)]")[0];
 check(production.includes(`Ok(port.${method}(`) && production.includes(`ApplicationResult<${dto}>`) && (production.match(/await/g)??[]).length===1 && !production.includes("sqlx::"),`R8-11 既有Application委托边界漂移：${owner}`);
 for(const name of tests) check(source.includes(`fn ${name}`),`R8-11 原Application目标缺少工作台回归：${name}`);
}
for(const token of ["workbench-null-competition", "workbench-foreign-conflict", "NULL manual evidence becomes the original empty array", "workspace evidence field {field} must survive unchanged", "workbench snapshot fixture", "workbench reads must not write task/job/research/evidence/snapshot/audit ledgers", "workbench-peer-manual", "peer_conflict.manual_decision_kind.is_none()", "frozen_view.snapshot.as_ref().unwrap()", "workspace-response", "ConflictEvaluationStatus::ManualRequired", "workbench manual fixture", "one immutable decision per (task, conflict)"]) check(horizonPg.includes(token),`R8-11 原PG Stage C缺少真实读取投影契约：${token}`);

if(failures.length) throw new Error(`Prediction Service 验证失败\n${failures.map((item)=>`- ${item}`).join("\n")}`);
console.log(`Prediction Service 验证通过：${predictionFiles.length} 个 Service/Use Case Rust 文件，18 个公开 Application 职责已进入 Prediction Service/Ports 边界，P4 freeze execution 与 snapshot persistence 均不再由旧混合 owner 直接实现。`);
