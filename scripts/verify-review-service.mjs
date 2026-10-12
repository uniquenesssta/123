import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const failures = [];
const legacyAnalyticsOwner = ["crates", "application", "src", "analytics.rs"].join("/");
const coreRequired = [
  "crates/application/src/services/review/mod.rs",
  "crates/application/src/services/review/service.rs",
  "crates/application/src/services/review/facade.rs",
  "crates/application/src/use_cases/review/mod.rs",
  "crates/application/src/use_cases/review/facts/mod.rs",
  "crates/application/src/use_cases/review/facts/coordinator.rs",
  "crates/application/src/use_cases/review/engine_adapter/mod.rs",
  "crates/application/src/use_cases/review/engine_adapter/draft.rs",
  "crates/application/src/use_cases/review/engine_adapter/package.rs",
  "crates/application/src/use_cases/review/engine_adapter/tests.rs",
  "crates/application/src/use_cases/review/test_support.rs",
  "crates/application/src/use_cases/review/list_reviewable_matches/mod.rs",
  "crates/application/src/use_cases/review/list_match_reviews/mod.rs",
  "crates/application/src/use_cases/review/read_match_review/mod.rs",
  "crates/application/src/use_cases/review/list_ability_candidates/mod.rs",
  "crates/application/src/use_cases/review/decide_ability_candidate/mod.rs",
  "crates/application/src/composition/adapters/review.rs",
];
const packageRequired = [
  "crates/application/src/ports/review/package.rs",
  "crates/application/src/use_cases/review/package/mod.rs",
  "crates/application/src/use_cases/review/package/export.rs",
  "crates/application/src/use_cases/review/package/preview.rs",
  "crates/application/src/use_cases/review/package/workflow/mod.rs",
  "crates/application/src/use_cases/review/package/workflow/coordinator.rs",
  "crates/application/src/use_cases/review/package/workflow/guards.rs",
  "crates/application/src/use_cases/review/package/workflow/confirmation.rs",
  "crates/application/src/use_cases/review/package/workflow/tests.rs",
  "crates/application/src/use_cases/review/package/shared.rs",
  "crates/application/src/use_cases/review/package/workbook.rs",
];
const required = [...coreRequired, ...packageRequired];
for (const relative of required) {
  if (!fs.existsSync(path.join(root, relative))) failures.push(`缺少 Review 模块：${relative}`);
}
for (const legacy of [
  "crates/application/src/review.rs",
  "crates/application/src/use_cases/review/generate_match_review/mod.rs",
  "crates/application/src/match_review_package.rs",
]) {
  if (fs.existsSync(path.join(root, legacy))) failures.push(`旧 Review owner 仍存在：${legacy}`);
}
const lib = fs.readFileSync(path.join(root, "crates/application/src/lib.rs"), "utf8");
if (/^mod review;$/m.test(lib)) failures.push("lib.rs 仍直接挂载旧 review.rs");
if (/^mod match_review_package;$/m.test(lib)) failures.push("lib.rs 仍直接挂载旧 match_review_package.rs");
const services = fs.readFileSync(path.join(root, "crates/application/src/services/mod.rs"), "utf8");
const useCases = fs.readFileSync(path.join(root, "crates/application/src/use_cases/mod.rs"), "utf8");
const reviewUseCases = fs.readFileSync(path.join(root, "crates/application/src/use_cases/review/mod.rs"), "utf8");
const adapters = fs.readFileSync(path.join(root, "crates/application/src/composition/adapters/mod.rs"), "utf8");
if (!services.includes("pub(crate) mod review;")) failures.push("services 未登记 review");
if (!useCases.includes("pub(crate) mod review;")) failures.push("use_cases 未登记 review");
if (!reviewUseCases.includes("pub(crate) mod package;")) failures.push("Review use_cases 未登记 package");
if (!adapters.includes("mod review;")) failures.push("composition adapters 未登记 review");

const facade = fs.readFileSync(path.join(root, "crates/application/src/services/review/facade.rs"), "utf8");
const service = fs.readFileSync(path.join(root, "crates/application/src/services/review/service.rs"), "utf8");
const port = fs.readFileSync(path.join(root, "crates/application/src/ports/review/mod.rs"), "utf8");
const packagePort = fs.readFileSync(path.join(root, "crates/application/src/ports/review/package.rs"), "utf8");
const adapter = fs.readFileSync(path.join(root, "crates/application/src/composition/adapters/review.rs"), "utf8");
const tauriReview = fs.readFileSync(path.join(root, "src-tauri/src/commands/review.rs"), "utf8");
const tauriPostmatch = fs.readFileSync(path.join(root, "src-tauri/src/commands/postmatch.rs"), "utf8");
const coreMethods = [
  "generate_match_review",
  "list_reviewable_matches",
  "list_match_reviews",
  "read_match_review",
  "list_ability_candidates",
  "decide_ability_candidate",
];
for (const method of coreMethods) {
  if (!facade.includes(`pub async fn ${method}`)) failures.push(`Review facade 缺少公共方法 ${method}`);
  if (!service.includes(`fn ${method}`)) failures.push(`ReviewService 缺少职责 ${method}`);
  if (!tauriReview.includes(`pub async fn ${method}`)) failures.push(`Tauri Review 公共命令缺少 ${method}`);
}
const packageMethods = [
  "export_match_review_package",
  "read_match_review_package_workflow",
  "preview_match_review_package",
  "confirm_match_review_package",
  "commit_match_review_package_facts",
  "generate_match_review_from_package",
  "commit_match_review_package",
];
for (const method of packageMethods) {
  if (!facade.includes(`pub async fn ${method}`)) failures.push(`Review package facade 缺少公共方法 ${method}`);
  if (!service.includes(`fn ${method}`)) failures.push(`ReviewService 缺少 package 职责 ${method}`);
  if (!tauriPostmatch.includes(`pub async fn ${method}`)) failures.push(`Tauri Postmatch 公共命令缺少 ${method}`);
}
if (!port.includes("pub trait MatchReviewPort")) failures.push("缺少 MatchReviewPort");
if (!port.includes("pub trait MatchReviewWorkflowPort")) failures.push("缺少 R3-01 冻结 MatchReviewWorkflowPort");
if (!packagePort.includes("pub trait MatchReviewPackageSourcePort")) failures.push("缺少 MatchReviewPackageSourcePort");
if (!packagePort.includes("pub trait MatchReviewPackageStatePort")) failures.push("缺少 MatchReviewPackageStatePort");
if (!packagePort.includes("pub trait MatchReviewPackageFactsPort")) failures.push("缺少 MatchReviewPackageFactsPort");
if (!adapter.includes("impl MatchReviewPort for PersistenceStore")) failures.push("PersistenceStore 未实现 MatchReviewPort");
if (!adapter.includes("impl MatchReviewPackageSourcePort for PersistenceStore")) failures.push("PersistenceStore 未实现 MatchReviewPackageSourcePort");
if (!adapter.includes("impl MatchReviewPackageStatePort for PersistenceStore")) failures.push("PersistenceStore 未实现 MatchReviewPackageStatePort");
if (!adapter.includes("impl MatchReviewPackageFactsPort for PersistenceStore")) failures.push("PersistenceStore 未实现 MatchReviewPackageFactsPort");
if (!port.includes("status: Option<AbilityCandidateStatus>")) failures.push("能力候选状态过滤未进入 Review Port");
if (!port.includes("match_review_id: Option<Uuid>")) failures.push("能力候选复盘过滤未进入 Review Port");
if (!facade.includes("self.database") || !facade.includes("active_session()")) failures.push("Review facade 未经 DatabaseService 活动会话取得端口实现");

for (const relative of required.filter((item) => item.includes("/services/review/") || item.includes("/use_cases/review/"))) {
  const text = fs.readFileSync(path.join(root, relative), "utf8");
  for (const token of ["football_persistence_postgres", "sqlx::", "PostgresStore", "PersistenceStore", ".transition_store()"] ) {
    if (text.includes(token)) failures.push(`${relative} 泄漏具体持久化符号：${token}`);
  }
}

const composition = fs.readFileSync(path.join(root, "crates/application/src/composition/application_composition.rs"), "utf8");
const appService = fs.readFileSync(path.join(root, "crates/application/src/service/application_service.rs"), "utf8");
if (!composition.includes("review: ReviewService")) failures.push("ApplicationComposition 未持有唯一 ReviewService");
if (!appService.includes("pub(crate) review: ReviewService")) failures.push("ApplicationService 未持有 ReviewService");
if (fs.existsSync(path.join(root, "crates/application/src/postmatch.rs"))) failures.push("AT3 后旧 postmatch.rs 不得恢复");
if (!fs.existsSync(path.join(root, "crates/application/src/services/postmatch/facade.rs"))) failures.push("AT3 Postmatch facade 未建立");
if (fs.existsSync(path.join(root, legacyAnalyticsOwner))) failures.push("AT4 后旧 analytics.rs 不得恢复");
if (!fs.existsSync(path.join(root, "crates/application/src/services/analytics/facade.rs"))) failures.push("AT4 Analytics facade 未建立");

const workflowMod = fs.readFileSync(path.join(root, "crates/application/src/use_cases/review/package/workflow/mod.rs"), "utf8");
if (!workflowMod.includes("pub(crate) use coordinator::") || /(?:async fn|sqlx::|impl )/.test(workflowMod)) failures.push("工作流入口必须只登记模块并导出唯一 coordinator");
for (const method of ["read_active_workflow", "confirm", "commit"]) {
  if (!service.includes(`package::workflow::${method}(`)) failures.push(`ReviewService 未切换唯一工作流入口 ${method}`);
}
if (!service.includes("facts::commit_facts(")) failures.push("ReviewService 未调用唯一 facts 用例");
if (!reviewUseCases.includes("pub(crate) mod facts;")) failures.push("Review use_cases 未登记 facts");
const factsMod = fs.readFileSync(path.join(root, "crates/application/src/use_cases/review/facts/mod.rs"), "utf8");
if (!factsMod.includes("pub(crate) use coordinator::commit_facts;") || /(?:async fn|sqlx::|impl )/.test(factsMod)) failures.push("facts 出口必须只登记并导出唯一 coordinator");
const workflowCoordinator = fs.readFileSync(path.join(root, "crates/application/src/use_cases/review/package/workflow/coordinator.rs"), "utf8");
if (/async fn commit_facts</.test(workflowCoordinator) || !workflowCoordinator.includes("facts::commit_facts")) failures.push("组合工作流未复用唯一事实用例或保留重复实现");
if (service.includes("package::lifecycle::")) failures.push("ReviewService 仍调用旧 lifecycle");
for (const relative of ["crates/persistence-postgres/src/adapters/review/mod.rs", "crates/persistence-postgres/src/adapters/review/package_workflow/mod.rs", "crates/persistence-postgres/src/adapters/review/package_workflow/repository.rs"]) {
  if (!fs.existsSync(path.join(root, relative))) failures.push(`缺少工作流适配器：${relative}`);
}
const pgAdapters = fs.readFileSync(path.join(root, "crates/persistence-postgres/src/adapters/mod.rs"), "utf8");
const pgLib = fs.readFileSync(path.join(root, "crates/persistence-postgres/src/lib.rs"), "utf8");
if (!pgAdapters.includes("mod review;") || /^mod match_review_package;$/m.test(pgLib)) failures.push("PG 工作流适配器登记未完成");

const engineRoot = "crates/application/src/use_cases/review/engine_adapter/";
const engineDraft = fs.readFileSync(path.join(root, `${engineRoot}draft.rs`), "utf8");
const enginePackage = fs.readFileSync(path.join(root, `${engineRoot}package.rs`), "utf8");
const engineTests = fs.readFileSync(path.join(root, `${engineRoot}tests.rs`), "utf8");
const pgEngineRoot = "crates/persistence-postgres/src/adapters/review/engine_adapter/";
const pgEngine = Object.fromEntries(["mod.rs", "repository.rs", "preparation.rs", "context.rs", "report.rs"].map((file) => [file, fs.readFileSync(path.join(root, `${pgEngineRoot}${file}`), "utf8")]));
const pgReview = fs.readFileSync(path.join(root, "crates/persistence-postgres/src/review.rs"), "utf8");
const pgFacts = fs.readFileSync(path.join(root, "crates/persistence-postgres/src/adapters/review/facts/repository.rs"), "utf8");
const ordered = (source, tokens) => {
  let offset = 0;
  return tokens.every((token) => {
    const index = source.indexOf(token, offset);
    if (index < 0) return false;
    offset = index + token.length;
    return true;
  });
};
if (!reviewUseCases.includes("pub(crate) mod engine_adapter;") || reviewUseCases.includes("mod generate_match_review;")) failures.push("Review未切换唯一engine_adapter");
for (const method of ["generate_match_review", "generate_review"]) if (!service.includes(`engine_adapter::${method}(`)) failures.push(`ReviewService未调用引擎适配 ${method}`);
if (workflowCoordinator.includes("async fn generate_review<") || !workflowCoordinator.includes("engine_adapter::generate_review")) failures.push("组合工作流未复用唯一资料包复盘生成或保留重复owner");
if (!ordered(engineDraft, ["draft.match_id != draft.result.match_id", "draft.review_version.as_mut()", "version.trim().to_string()", "draft.review_version = None", "draft.notes.as_mut()", "notes.trim().to_string()", "port.generate_match_review(&draft).await?"])) failures.push("普通复盘ID前检/版本备注归一化/引擎调用契约漂移");
if (!ordered(enginePackage, ["port.read_workflow(", ".require_action(MatchReviewPackageWorkflowAction::GenerateReview)", "port.read_package_preview(", "apply_confirmation_metadata(", "port.generate_match_review(&preview.review).await?", ".mark_review_created("])) failures.push("资料包复盘动作/确认/生成成功后推进状态的顺序漂移");
if ((enginePackage.match(/\.mark_review_created\(/g) ?? []).length !== 1 || enginePackage.indexOf("port.generate_match_review(&preview.review).await?") > enginePackage.indexOf(".mark_review_created(")) failures.push("资料包生成成功后才可推进状态一次，不得提前或重复写入");
if (enginePackage.includes("draft.result.match_id") || enginePackage.includes("draft.notes.as_mut()")) failures.push("不得把普通复盘前检/归一化新增到原资料包路径");
for (const name of ["generate_match_review", "prepare_review_in_tx", "persist_calculation_in_tx", "load_review_players_in_tx", "calculate_recent_starter_overlap", "load_prediction_context_in_tx"]) if (new RegExp(`(?:async )?fn ${name}\\(`).test(pgReview)) failures.push(`旧PG review仍持有引擎职责 ${name}`);
if (!pgFacts.includes("engine_adapter::prepare_review_in_tx")) failures.push("事实提交没有复用迁移后的唯一完整准备");
if (!ordered(pgEngine["repository.rs"], ["validate_review_draft(draft)?", "self.pool.begin().await?", "prepare_review_in_tx(&mut tx, draft).await?", "calculate_review(&preparation)", "persist_calculation_in_tx(&mut tx, draft, &preparation, &calculation).await?", "write_audit_event(", "tx.commit().await?", "self.read_match_review(review_id).await"])) failures.push("原引擎事务校验/准备/计算/报告/审计/提交/回读顺序漂移");
if ((pgEngine["repository.rs"].match(/self\.pool\.begin\(\)\.await\?/g) ?? []).length !== 1 || (pgEngine["repository.rs"].match(/tx\.commit\(\)\.await\?/g) ?? []).length !== 1 || pgEngine["repository.rs"].indexOf("write_audit_event(") > pgEngine["repository.rs"].indexOf("tx.commit().await?")) failures.push("引擎必须只开一个原事务并在审计后提交一次");
for (const file of ["preparation.rs", "context.rs", "report.rs"]) if (/\.(?:begin|commit|rollback)\(/.test(pgEngine[file])) failures.push(`${file}必须借用原事务，不能提前完成或新增事务`);
if (!pgEngine["repository.rs"].includes("use football_review_engine::calculate_review;")) failures.push("唯一适配器没有直接调用原纯计算引擎");
if (!ordered(pgEngine["preparation.rs"], ["write_facts_in_tx(tx, draft).await?", "load_review_players_in_tx", "calculate_recent_starter_overlap", "load_prediction_context_in_tx", "Ok(ReviewPreparationData"])) failures.push("事实及原球员/球队/预测计算输入准备顺序漂移");
for (const token of ["snapshot.effective_contribution", "profile.average_value, 50", "match_review.match_id <> observation.match_id", "fixture.kickoff_time < $3", "LIMIT 10", "status='succeeded'", "completed_at DESC NULLS LAST", "指定的模型运行不存在、未成功或不属于该场比赛", "actual_scoreline_probability"]) if (!pgEngine["context.rs"].includes(token)) failures.push(`原计算上下文缺少 ${token}`);
if (!ordered(pgEngine["report.rs"], ["WITH superseded AS", "INSERT INTO review.ability_update_decisions", "UPDATE review.match_reviews SET status='superseded'", "let review_id = Uuid::new_v4()", "INSERT INTO review.match_reviews", "INSERT INTO review.player_match_reviews", "INSERT INTO review.ability_update_candidates", "INSERT INTO review.team_match_reviews", "INSERT INTO feature.player_dynamic_tags", "Ok(review_id)"])) failures.push("报告/候选/动态标签不可变快照与原版本替代顺序漂移");
for (const token of ["serde_json::to_value(&preparation.result)?", "serde_json::to_value(&preparation.substitutions)?", "calculation.conclusions.clone()", "'pending',$10,$11", "Duration::days(14)", "&calculation.calculation_version"]) if (!pgEngine["report.rs"].includes(token)) failures.push(`报告载荷/候选人工门禁缺少 ${token}`);
for (const name of ["generate_review_preserves_order_confirmation_and_returned_ids", "generate_review_stops_at_failed_dependency_or_domain_guard", "draft_mismatched_result_stops_before_engine_port", "draft_blank_version_and_notes_preserve_optional_contract", "draft_normalization_preserves_fact_payload_and_source_context", "draft_generation_returns_the_complete_port_detail", "draft_generation_propagates_port_failure_without_workflow_writes"]) if (!engineTests.includes(`fn ${name}(`)) failures.push(`缺少原Application target引擎适配边界测试 ${name}`);

const candidateRoot = "crates/persistence-postgres/src/adapters/review/ability_candidate_transaction/";
const candidate = Object.fromEntries(["mod.rs", "read.rs", "mapping.rs", "validation.rs", "repository.rs", "tests.rs"].map((file) => [file, fs.readFileSync(path.join(root, `${candidateRoot}${file}`), "utf8")]));
const pgReviewMod = fs.readFileSync(path.join(root, "crates/persistence-postgres/src/adapters/review/mod.rs"), "utf8");
if (!pgReviewMod.includes("mod ability_candidate_transaction;")) failures.push("PG Review未登记唯一候选事务owner");
if (/(?:async fn|sqlx::|impl )/.test(candidate["mod.rs"])) failures.push("候选出口仅登记职责，不能承载查询或事务");
for (const name of ["list_ability_candidates", "decide_ability_candidate", "read_ability_candidate", "ability_candidate_from_row"]) if (new RegExp(`(?:async )?fn ${name}\\(`).test(pgReview)) failures.push(`旧PG review仍持有候选职责 ${name}`);
if (!candidate["repository.rs"].includes("pub async fn decide_ability_candidate(") || !adapter.includes(".decide_ability_candidate(")) failures.push("原Review Port未接入候选公开入口");
if (!ordered(candidate["repository.rs"], ["self.pool.begin().await?", "FOR UPDATE OF candidate", "row.try_get(\"status\")?", "require_pending_candidate(&status)?", "decision_actor(draft.decided_by.as_deref())", "match draft.decision", "AbilityCandidateDecision::Accept", "FROM feature.player_current_abilities", "require_current_projection(current_value, current_projection)?", "Uuid::new_v4()", "INSERT INTO feature.player_ability_observations", "AbilityCandidateDecision::Reject => (\"rejected\", \"rejected\", None)", "UPDATE review.ability_update_candidates", "INSERT INTO review.ability_update_decisions", "write_audit_event(", "tx.commit().await?", "read_ability_candidate(&self.pool, draft.candidate_id).await"])) failures.push("候选锁/人工分支/过期门禁/共同写入审计/提交后回读顺序漂移");
if ((candidate["repository.rs"].match(/self\.pool\.begin\(\)\.await\?/g) ?? []).length !== 1 || (candidate["repository.rs"].match(/tx\.commit\(\)\.await\?/g) ?? []).length !== 1 || candidate["repository.rs"].indexOf("write_audit_event(") > candidate["repository.rs"].indexOf("tx.commit().await?")) failures.push("候选决策必须只有一个原事务且审计后只提交一次");
for (const file of ["read.rs", "mapping.rs", "validation.rs"]) if (/\.(?:begin|commit|rollback|execute)\(/.test(candidate[file])) failures.push(`${file}不得写入或另开候选事务`);
for (const token of ["if status != \"pending\"", "能力更新候选已经处理，当前状态：{status}", ".map(str::trim)", ".filter(|value| !value.is_empty())", ".unwrap_or(\"local_user\")", "(expected - actual).abs() <= 0.01", "(None, None) => true", "_ => false", "if !candidate_is_current", "球员当前能力已发生变化，该候选已经过期，请重新生成复盘"]) if (!candidate["validation.rs"].includes(token)) failures.push(`原候选状态/人工身份/过期规则缺少 ${token}`);
for (const token of ["AbilityCandidateDecision::Accept =>", "(\"accepted\", \"accepted\", Some(observation_id))", "observed_at.unwrap_or_else(Utc::now)", "format!(\"{calculation_version}:accepted\")", "draft.decision_note.as_deref()", "\"candidate_id\": draft.candidate_id", "\"evidence\": evidence", "\"decision_note\": draft.decision_note", "\"decided_by\": decided_by", "\"ability_update_candidate_decided\""]) if (!candidate["repository.rs"].includes(token)) failures.push(`原候选观察/决定/审计载荷缺少 ${token}`);
const acceptStart = candidate["repository.rs"].indexOf("AbilityCandidateDecision::Accept =>");
const rejectStart = candidate["repository.rs"].indexOf("AbilityCandidateDecision::Reject =>");
if (candidate["repository.rs"].indexOf("require_current_projection(", acceptStart) > rejectStart || candidate["repository.rs"].indexOf("INSERT INTO feature.player_ability_observations", acceptStart) > rejectStart || candidate["repository.rs"].slice(rejectStart).includes("INSERT INTO feature.player_ability_observations")) failures.push("拒绝不能写能力观察，过期门禁必须只在原接受分支先于观察写入");
for (const token of ["pub async fn list_ability_candidates(", "pub(super) async fn read_ability_candidate(", "status.map(AbilityCandidateStatus::as_str)", "limit.clamp(1, 2000)", "candidate.status = $1", "candidate.match_review_id = $2", "candidate.confidence DESC, candidate.created_at DESC", "能力更新候选不存在", "ability_candidate_from_row"]) if (!candidate["read.rs"].includes(token)) failures.push(`原候选只读查询缺少 ${token}`);
for (const token of ["status: candidate_status_from_db(&status)?", "\"pending\" => Ok(AbilityCandidateStatus::Pending)", "\"accepted\" => Ok(AbilityCandidateStatus::Accepted)", "\"rejected\" => Ok(AbilityCandidateStatus::Rejected)", "\"superseded\" => Ok(AbilityCandidateStatus::Superseded)", "未知候选状态：{other}"]) if (!candidate["mapping.rs"].includes(token)) failures.push(`原候选行映射缺少 ${token}`);
if (!pgReview.includes(".list_ability_candidates(None, 1000, Some(review_id))")) failures.push("复盘详情未复用唯一候选读取入口");
for (const name of ["pending_guard_rejects_every_decided_status", "pending_guard_preserves_exact_status_and_error_text", "decision_actor_defaults_only_for_missing_or_blank_identity", "decision_actor_trims_unicode_without_changing_identity", "current_projection_accepts_missing_pair_and_inclusive_tolerance", "current_projection_rejects_stale_missing_and_nonfinite_values", "candidate_status_mapping_preserves_four_domain_states", "candidate_status_mapping_rejects_unknown_without_normalization"]) if (!candidate["tests.rs"].includes(`fn ${name}(`)) failures.push(`缺少原PG unit target候选边界测试 ${name}`);

if (failures.length) {
  console.error("Review Service 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}
console.log("Review Service 验证通过：AT1 Review Core 与 AT2 Match Review Package 已进入 Service/Use Case/Port 边界；旧 review.rs / match_review_package.rs 已退出，公共 Tauri 命令保持原名。");
