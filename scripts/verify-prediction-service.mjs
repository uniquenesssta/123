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
check(!existsSync(join(root,"crates/application/src/p4_orchestration.rs")),"旧 p4_orchestration.rs owner 仍残留"); check(!orchestrationUseCase.includes("async fn execute_p4_freeze_task("),"P4 orchestration use case 不得实现 freeze execution"); check(orchestrationUseCase.includes(".execute_p4_freeze_task("),"P4 orchestration 未委托 Prediction Service 的 freeze use case");
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
check(builderProduction.includes("inspect_match_prediction_readiness::execute(port, registry, command.clone()).await?"), "输入构建必须执行既有完整度评估");
check(builderProduction.includes("readiness.can_run_formal") && builderProduction.includes("readiness.can_run_shadow") && builderProduction.includes("if !allowed {") && builderProduction.includes("return Err(ApplicationError::Validation(format!("), "输入 I/O 前必须按模式拒绝不允许的推演");
const constructionSteps = ["if !allowed {", "let model_family = normalize_model_selection", ".prepare_match_input_at(", "verify_prepared_input_matches_readiness(&prepared, &readiness)?;", "attach_prediction_input_audit(&mut prepared.match_input, &readiness)?;", "Ok(PredictionCommand {"];
const positions = constructionSteps.map(token=>builderProduction.indexOf(token));
check(positions.every((position,index)=>position>=0 && (index===0 || position>positions[index-1])), "输入构建顺序必须为权限、家族、准备、指纹、审计、命令");
check(/&model_family,\s*readiness\.assessed_at,/.test(builderProduction), "输入准备必须复用评估时间及规范化家族");
for(const field of ["match_input: prepared.match_input", "snapshot_type: prepared.snapshot_type", "competition_id: prepared.match_record.competition_id", "season_id: prepared.match_record.season_id", "stage_id: prepared.match_record.stage_id", "competition_kind: prepared.competition_kind", "model_family: command.model_family", "explicit_rule_package_id: command.explicit_rule_package_id"]) check(builderProduction.includes(field), `输入构建丢失原命令字段：${field}`);
for(const token of ["Utc::now", "Uuid::new", "execute_internal(", "save_run(", ".predict(", "sqlx::", "prepare_match_input("]) check(!builderProduction.includes(token), `输入构建越界副作用或重取时钟：${token}`);
for(const test of ["audited_input_preserves_clock_family_route_and_manifest", "formal_and_shadow_permissions_block_input_io", "changed_input_manifest_blocks_construction", "runtime_identity_changes_preserve_assessed_manifest", "input_port_failure_and_missing_audit_do_not_return_command", "invalid_family_and_non_object_input_keep_existing_errors"]) check(builder.includes(`async fn ${test}`), `R8-01 缺少既有 target 内的行为测试：${test}`);
if(failures.length) throw new Error(`Prediction Service 验证失败\n${failures.map((item)=>`- ${item}`).join("\n")}`);
console.log(`Prediction Service 验证通过：${predictionFiles.length} 个 Service/Use Case Rust 文件，18 个公开 Application 职责已进入 Prediction Service/Ports 边界，P4 freeze execution 与 snapshot persistence 均不再由旧混合 owner 直接实现。`);
