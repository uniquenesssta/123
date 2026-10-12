import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const assert = (condition, message) => {
  if (!condition) throw new Error(message);
};

const facade = read("crates/application/src/services/postmatch/facade.rs");
const service = read("crates/application/src/services/postmatch/service.rs");
const ports = read("crates/application/src/ports/postmatch/mod.rs");
const reviewState = read("crates/application/src/ports/review/package.rs");
const adapter = read("crates/application/src/composition/adapters/postmatch.rs");
const reviewAdapter = read("crates/application/src/composition/adapters/review.rs");
const settlementRoot = "crates/application/src/use_cases/postmatch/settlement/";
const settle = read(`${settlementRoot}coordinator.rs`);
const composition = read("crates/application/src/composition/application_composition.rs");
const applicationService = read("crates/application/src/service/application_service.rs");
const contract = JSON.parse(read("contracts/postmatch-settlement-contract.json"));
const useCaseFiles = [
  "list_evidence_scoring_items",
  "decide_evidence_scoring_item",
  "refresh_postmatch_monitoring",
  "postmatch_overview",
].map((name) => read(`crates/application/src/use_cases/postmatch/${name}/mod.rs`));
const settlementMod = read(`${settlementRoot}mod.rs`);
const settlementRead = read(`${settlementRoot}read.rs`);
const settlementTests = read(`${settlementRoot}tests.rs`);
const applicationLayer = [facade, service, settle, settlementRead, settlementTests, ...useCaseFiles].join("\n");

assert(!exists("crates/application/src/postmatch.rs"), "旧 Postmatch Application owner 仍存在");
assert(
  contract.artifacts.includes("crates/application/src/services/postmatch/facade.rs"),
  "Postmatch 契约未指向新 authoritative facade",
);
for (const command of contract.commands) {
  assert(facade.includes(`fn ${command}`), `Postmatch facade 缺少公共入口：${command}`);
  assert(service.includes(`fn ${command}`), `Postmatch service 缺少委托：${command}`);
}
assert(
  ports.includes("trait PostmatchSettlementPort") && ports.includes("limit: u32"),
  "Postmatch Settlement Port 未对齐真实 limit 契约",
);
assert(
  ports.includes("status: Option<&str>") && ports.includes("async fn overview(&self, limit: u32)"),
  "Postmatch Port 查询能力未对齐真实 API",
);
assert(
  adapter.includes("postmatch_settlement_readiness") &&
    adapter.includes("settle_postmatch_review") &&
    adapter.includes("list_postmatch_settlements") &&
    adapter.includes("list_evidence_scoring_items") &&
    adapter.includes("decide_evidence_scoring_item") &&
    adapter.includes("refresh_postmatch_monitoring") &&
    adapter.includes("postmatch_overview"),
  "Postmatch PostgreSQL adapter 委托不完整",
);
assert(
  reviewState.includes("read_workflow_by_review") && reviewState.includes("mark_settled"),
  "Match Review Package State Port 缺少 Postmatch 状态推进能力",
);
assert(
  reviewAdapter.includes("read_match_review_package_workflow_by_review") &&
    reviewAdapter.includes("mark_match_review_package_settled"),
  "Review state adapter 未复用既有资料包状态持久化",
);
assert(
  settle.includes("MatchReviewPackageWorkflowAction::SettleReview") &&
    settle.includes("require_action") &&
    settle.includes("mark_settled"),
  "Postmatch settle use case 未保留资料包状态机门禁",
);
assert(
  composition.includes("PostmatchService") &&
    applicationService.includes("postmatch: PostmatchService"),
  "Application composition 未聚合唯一 PostmatchService",
);
for (const forbidden of [
  "football_persistence_postgres",
  "sqlx::",
  "PostgresStore",
  "PersistenceStore",
  ".transition_store()",
]) {
  assert(
    !applicationLayer.includes(forbidden),
    `Postmatch Service/Use Case 泄漏基础设施依赖：${forbidden}`,
  );
}

const pgRoot = "crates/persistence-postgres/src/adapters/postmatch/settlement/";
const pg = Object.fromEntries(["mod", "context", "readiness", "read", "repository", "evidence_queue", "evaluation_sample", "validation", "tests"].map((name) => [name, read(`${pgRoot}${name}.rs`)]));
const oldPg = read("crates/persistence-postgres/src/postmatch.rs");
const ordered = (source, tokens) => {
  let offset = 0;
  return tokens.every((token) => { const index = source.indexOf(token, offset); if (index < 0) return false; offset = index + token.length; return true; });
};
for (const name of ["postmatch_settlement_readiness", "settle_postmatch_review", "list_postmatch_settlements"]) {
  assert(!exists(`crates/application/src/use_cases/postmatch/${name}/mod.rs`), `旧结算用例仍存在 ${name}`);
  assert(service.includes(`postmatch::settlement::${name}(`), `Service未调用唯一结算入口 ${name}`);
}
assert(read("crates/application/src/use_cases/postmatch/mod.rs").includes("pub(crate) mod settlement;"), "Application未登记settlement");
assert(settlementMod.includes("pub(crate) use coordinator::settle_postmatch_review;") && !/(?:async fn|impl |sqlx::)/.test(settlementMod), "结算出口必须只登记与导出");
assert(ordered(settle, ["port.read_workflow_by_review(draft.match_review_id).await?", "workflow.status != MatchReviewPackageWorkflowStatus::Settled", "require_action(MatchReviewPackageWorkflowAction::SettleReview)", "port.settle(&draft).await?", "port.mark_settled(settlement.match_review_id).await?", "Ok(settlement)"]), "结算状态门禁/幂等重试/成功后按返回review推进顺序漂移");
assert((settle.match(/\.mark_settled\(/g) ?? []).length === 1 && !settle.includes("port.readiness("), "结算不能提前/重复推进状态或插入原路径不存在的readiness查询");
for (const name of ["postmatch_settlement_readiness", "settle_postmatch_review", "list_postmatch_settlements", "load_settlement_context", "insert_evidence_scoring_items_in_tx", "insert_postmatch_evaluation_sample_in_tx", "read_postmatch_settlement", "readiness_from_context", "settlement_select_sql", "settlement_from_row", "calculate_timeliness_score", "validate_probabilities", "json_text", "json_f64", "required"]) assert(!new RegExp(`(?:async )?fn ${name}(?:<[^>]+>)?\\(`).test(oldPg), `旧PG结算owner仍保留 ${name}`);
assert(read("crates/persistence-postgres/src/adapters/mod.rs").includes("mod postmatch;") && read("crates/persistence-postgres/src/adapters/postmatch/mod.rs").includes("mod settlement;"), "PG结算模块未实际登记");
assert(ordered(pg.repository, ["self.load_settlement_context(draft.match_review_id).await?", "readiness_from_context(&context)", "if !readiness.ready", "if let Some(existing_id)", "self.read_postmatch_settlement(existing_id).await", "sha256_json(&SettlementFingerprint", "POSTMATCH_SETTLEMENT_VERSION", "self.pool.begin().await?", "ON CONFLICT (settlement_key) DO NOTHING", "let persisted_id = if let Some(id) = inserted", "self.insert_evidence_scoring_items_in_tx(", "self.insert_postmatch_evaluation_sample_in_tx(", "write_audit_event(", "existing_fingerprint != fingerprint", "tx.commit().await?", "self.read_postmatch_settlement(persisted_id).await"]), "原结算readiness/已有返回/指纹/幂等插入/共同证据样本审计/冲突/提交后回读顺序漂移");
assert((pg.repository.match(/self\.pool\.begin\(\)\.await\?/g) ?? []).length === 1 && (pg.repository.match(/tx\.commit\(\)\.await\?/g) ?? []).length === 1 && pg.repository.indexOf("write_audit_event(") < pg.repository.indexOf("tx.commit().await?"), "结算只能开启/提交一次原事务，审计必须先于提交");
for (const name of ["context", "readiness", "read", "evidence_queue", "evaluation_sample", "validation"]) assert(!/\.(?:begin|commit|rollback)\(/.test(pg[name]), `${name}不得另开或完成结算事务`);
for (const token of ["ON CONFLICT (settlement_key) DO NOTHING", "result_fingerprint", "相同赛后结算键对应的正式赛果或预测评价已经发生变化", "\"provider_state\": \"NOT_BUNDLED\"", "\"automatic_parameter_promotion\": false"]) assert(pg.repository.includes(token), `结算幂等/外部模型/人工参数边界缺少 ${token}`);
for (const token of ["review_status == \"finalized\"", "Some(\"succeeded\")", "snapshot_match_id == Some(context.match_id)", "snapshot_model_version_id == context.model_version_id", "snapshot_parameter_set_id == context.parameter_set_id", "snapshot_competition_profile_id == context.competition_profile_id", "Some(\"real\" | \"manual\")", "== Some(\"real\")", "FORMAL_HORIZONS.contains", "ready: blocked_reasons.is_empty()", "existing_settlement_id: context.existing_settlement_id"]) assert(pg.readiness.includes(token), `原readiness门禁缺少 ${token}`);
assert(ordered(pg.evidence_queue, ["retrieved_at > data_cutoff_at", "不能进入正式评分队列", "calculate_timeliness_score(", "sha256_json(", "INSERT INTO review.evidence_scoring_items", "ON CONFLICT (settlement_id, evidence_id, field_key) DO NOTHING"]), "证据冻结截止/队列身份/幂等原事务漂移");
assert(ordered(pg.evaluation_sample, ["最终复盘没有可用预测评价", "actual_outcome != expected_actual_outcome", "validate_probabilities(home_win, draw, away_win)?", "最终复盘评价指标或数据覆盖率超出合法范围", "INSERT INTO analytics.evaluation_samples", "ON CONFLICT (run_id, kickoff_time, calculation_version) DO NOTHING"]), "正式样本结果/概率/指标与幂等原事务漂移");
assert(/#\[test\]\s+fn timeliness_is_bounded_and_rewards_fresh_sources/.test(pg.evidence_queue) && !/#\[test\]\s+#\[test\]/.test(oldPg), "原timeliness unit必须迁移注册一次，不能丢失或重复属性");
for (const name of ["plain_settlement_preserves_draft_detail_and_returned_review_identity", "review_created_package_settles_then_marks_once", "settled_package_retries_original_idempotent_port_without_new_readiness", "blocked_package_states_stop_before_settlement_with_original_domain_reason", "workflow_lookup_failure_stops_all_settlement_effects", "settlement_and_mark_errors_preserve_original_side_effect_boundary", "readiness_and_list_preserve_input_parameters_and_complete_results", "readiness_and_list_propagate_port_errors_without_workflow_writes"]) assert(settlementTests.includes(`fn ${name}(`), `缺少原App target结算测试 ${name}`);
for (const name of ["readiness_preserves_formal_horizons_and_real_manual_snapshot_sources", "readiness_accumulates_original_eight_blockers_in_order", "readiness_rejects_each_snapshot_identity_axis_and_nonformal_scope", "existing_settlement_identity_does_not_override_current_readiness_guards"]) assert(pg.tests.includes(`fn ${name}(`), `缺少原PG target readiness测试 ${name}`);
console.log(
  "Postmatch Service 验证通过：7 个公共入口、2 个既有 Ports、资料包状态机与 PostgreSQL adapter 边界完整。",
);
