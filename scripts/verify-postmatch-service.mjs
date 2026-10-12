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
  "refresh_postmatch_monitoring",
  "postmatch_overview",
].map((name) => read(`crates/application/src/use_cases/postmatch/${name}/mod.rs`));
const settlementMod = read(`${settlementRoot}mod.rs`);
const settlementRead = read(`${settlementRoot}read.rs`);
const settlementTests = read(`${settlementRoot}tests.rs`);
const evidenceRoot = "crates/application/src/use_cases/postmatch/evidence_scoring/";
const evidence = Object.fromEntries(["mod", "read", "decision", "tests"].map((name) => [name, read(`${evidenceRoot}${name}.rs`)]));
const applicationLayer = [facade, service, settle, settlementRead, settlementTests, ...Object.values(evidence), ...useCaseFiles].join("\n");

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
const pg = Object.fromEntries(["mod", "context", "readiness", "read", "repository", "evidence_queue", "evaluation_sample", "validation", "tests"].map((name) => [name, read(name === "evidence_queue" ? "crates/persistence-postgres/src/adapters/postmatch/evidence_scoring/queue.rs" : `${pgRoot}${name}.rs`)]));
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

const scoringRoot = "crates/persistence-postgres/src/adapters/postmatch/evidence_scoring/";
const scoring = Object.fromEntries(["mod", "read", "mapping", "repository", "policy", "queue", "tests"].map((name) => [name, read(`${scoringRoot}${name}.rs`)]));
for (const name of ["list_evidence_scoring_items", "decide_evidence_scoring_item"]) {
  assert(!exists(`crates/application/src/use_cases/postmatch/${name}/mod.rs`), `旧证据用例仍存在 ${name}`);
  assert(service.includes(`postmatch::evidence_scoring::${name}(`), `Service未切证据入口 ${name}`);
}
assert(read("crates/application/src/use_cases/postmatch/mod.rs").includes("pub(crate) mod evidence_scoring;") && read("crates/persistence-postgres/src/adapters/postmatch/mod.rs").includes("mod evidence_scoring;"), "App/PG未登记唯一evidence_scoring");
assert(evidence.mod.includes("pub(crate) use decision::decide_evidence_scoring_item;") && evidence.mod.includes("pub(crate) use read::list_evidence_scoring_items;") && !/(?:async fn|impl |sqlx::)/.test(evidence.mod), "证据App出口必须仅登记/导出实际入口");
assert(evidence.read.includes(".list_evidence_scoring_items(status.as_deref(), limit)") && evidence.decision.includes("port.decide_evidence_scoring_item(&draft).await?"), "证据用例必须保持原字段/参数透传");
assert(!/trim\(|clamp\(|verdict_scores\(/.test(evidence.read + evidence.decision), "App不能新加原PG验证/归一化或评分公式");
for (const name of ["list_evidence_scoring_items", "decide_evidence_scoring_item", "read_evidence_scoring_item", "evidence_item_from_row", "verdict_scores"]) assert(!new RegExp(`(?:async )?fn ${name}\\(`).test(oldPg), `旧PG保留证据职责 ${name}`);
assert(!exists(`${pgRoot}evidence_queue.rs`) && !read(`${pgRoot}mod.rs`).includes("mod evidence_queue;"), "旧结算目录不能保留重复证据队列owner");
assert(scoring.queue.includes("pub(in crate::adapters::postmatch) async fn insert_evidence_scoring_items_in_tx("), "队列必须仅在原postmatch事务模块间共享");
assert(ordered(scoring.repository, ["validate_evidence_decision_note(&draft.decision_note)?", "EXISTS (SELECT 1 FROM review.evidence_scoring_decisions", "证据评分项不存在", "item.try_get::<bool, _>(\"decided\")?", "历史决定不可覆盖", "verdict_scores(draft.verdict, &verification_state, &source_tier)", "sha256_json(", "\"decision_note\": draft.decision_note.trim()", "self.pool.begin().await?", "INSERT INTO review.evidence_scoring_decisions", "write_audit_event(", "tx.commit().await?", "self.read_evidence_scoring_item(draft.item_id).await"]), "原说明/已判定前检/评分指纹/共同决定审计/提交后回读顺序漂移");
assert((scoring.repository.match(/self\.pool\.begin\(\)\.await\?/g) ?? []).length === 1 && (scoring.repository.match(/tx\.commit\(\)\.await\?/g) ?? []).length === 1 && scoring.repository.indexOf("write_audit_event(") < scoring.repository.indexOf("tx.commit().await?"), "证据决定必须仅一个原事务且审计后提交一次");
assert(!/ON CONFLICT|UPDATE review\.evidence_scoring_decisions|DELETE FROM review\.evidence_scoring_decisions/.test(scoring.repository), "证据决定不可覆盖或自动重试改写旧记录");
for (const token of ["!matches!(value, \"pending\" | \"scored\" | \"not_verifiable\")", "if note.trim().len() < 8", "证据判定说明至少需要 8 个字符", "if verdict == EvidenceVerdict::NotVerifiable", "return (None, None)", "EvidenceVerdict::Partial => 0.5", "source_tier.to_ascii_lowercase().as_str()", "accuracy * 0.70 + verification_quality * 0.15 + source_quality * 0.15", "(Some(round6(accuracy)), Some(round6(reliability)))"]) assert(scoring.policy.includes(token), `原评分/说明字节门槛/状态规则缺少 ${token}`);
for (const token of [".bind(draft.decided_by.as_deref())", ".bind(draft.decision_note.trim())", "\"decided_by\": draft.decided_by.as_deref()", "\"evidence_scoring_decided\""]) assert(scoring.repository.includes(token), `原人工决定载荷缺少 ${token}`);
for (const token of ["validate_evidence_status(status)?", "limit.clamp(1, 1000)", "decision.id IS NULL", "decision.verdict = 'not_verifiable'", "item.created_at DESC, item.id DESC", "evidence_item_from_row"]) assert(scoring.read.includes(token), `原评分查询/状态派生缺少 ${token}`);
for (const name of ["read", "mapping", "policy", "queue"]) assert(!/\.(?:begin|commit|rollback)\(/.test(scoring[name]), `${name}不得另开或完成证据决定事务`);
assert(/#\[test\]\s+fn not_verifiable_does_not_fake_accuracy/.test(scoring.policy) && /#\[test\]\s+fn timeliness_is_bounded_and_rewards_fresh_sources/.test(scoring.queue), "原verdict与timeliness两unit都必须注册保留");
for (const name of ["list_none_filter_preserves_zero_limit_and_full_port_records", "list_preserves_exact_optional_status_and_unclamped_limit", "decision_preserves_all_four_verdicts_and_complete_returned_record", "decision_keeps_missing_or_blank_actor_and_note_validation_at_port_boundary", "list_propagates_all_port_error_kinds_without_other_calls", "decision_propagates_port_failure_without_retry_or_monitoring"]) assert(evidence.tests.includes(`fn ${name}(`), `缺少App原target证据测试 ${name}`);
for (const name of ["status_filter_accepts_none_and_original_three_states", "status_filter_rejects_unknown_case_and_whitespace", "decision_note_accepts_original_utf8_byte_boundary_after_trim", "decision_note_rejects_short_or_blank_with_original_error", "verdict_accuracy_preserves_three_manual_values", "reliability_preserves_verification_quality_and_case_policy", "reliability_preserves_source_aliases_case_and_untrimmed_fallback", "not_verifiable_has_no_scores_for_any_quality_context"]) assert(scoring.tests.includes(`fn ${name}(`), `缺少PG原target证据测试 ${name}`);
const providerRoot = "crates/persistence-postgres/src/adapters/postmatch/provider_score/";
const provider = Object.fromEntries(["mod", "aggregation", "read", "mapping", "repository", "tests"].map((name) => [name, read(`${providerRoot}${name}.rs`)]));
assert(read("crates/persistence-postgres/src/adapters/postmatch/mod.rs").includes("mod provider_score;"), "供应商评分模块未登记");
assert(!/(?:async fn|impl |sqlx::)/.test(provider.mod), "供应商评分出口必须只登记模块");
for (const name of ["refresh_provider_scores_for_partition", "list_provider_scores", "provider_score_from_row", "mean"]) assert(!new RegExp(`(?:async )?fn ${name}\\(`).test(oldPg), `旧PG供应商职责未移除 ${name}`);
assert(!oldPg.includes("struct ProviderAggregate"), "旧PG仍持有供应商聚合状态");
assert(ordered(oldPg, [".active_postmatch_partition(", "self.refresh_provider_scores_for_partition(&partition)", "self.refresh_drift_for_partition(", "self.postmatch_overview(100).await"]), "原监控刷新必须保持供应商/漂移/总览顺序");
assert(ordered(provider.repository, ["settlement.competition_id=$1", "settlement.competition_profile_id=$2", "settlement.model_version_id=$3", "settlement.parameter_set_id=$4", "settlement.horizon=$5", "item.provider_id IS NOT NULL", "ORDER BY item.provider_id, decision.id", ".bind(partition.competition_id)", ".bind(partition.competition_profile_id)", ".bind(partition.model_version_id)", ".bind(partition.parameter_set_id)", ".bind(&partition.horizon)", ".fetch_all(&self.pool)", "aggregate_provider_rows(rows)?", "for (provider_id, aggregate) in groups", "aggregate.scores()", "aggregate.snapshot_identity(provider_id, &partition.partition_key)?", "INSERT INTO analytics.provider_score_snapshots", "ON CONFLICT (snapshot_key) DO NOTHING", ".bind(aggregate.total() as i64)", ".bind(aggregate.correct as i64)", ".bind(aggregate.partial as i64)", ".bind(aggregate.incorrect as i64)", ".bind(aggregate.not_verifiable as i64)", ".bind(round6(accuracy_mean))", ".bind(round6(timeliness_mean))", ".bind(round6(reliability_mean))", ".bind(weighted_score)", ".bind(&decision_set_sha256)", ".bind(snapshot_key)", ".bind(POSTMATCH_MONITORING_VERSION)", ".execute(&self.pool)"]), "供应商原分区查询/逐组评分/不可变快照绑定顺序漂移");
assert(/ORDER BY item\.provider_id, decision\.id\s*\n\s*"#/.test(provider.repository), "供应商决定顺序必须保持原升序，不能增加DESC或排序字段");
for (const name of ["repository", "read", "mapping", "aggregation"]) assert(!/\.(?:begin|commit|rollback)\(|write_audit_event\(|(?:UPDATE|DELETE FROM) analytics\.provider_score_snapshots/.test(provider[name]), `供应商${name}不得新增事务、审计或覆盖快照`);
assert(ordered(provider.aggregation, ["let mut groups: BTreeMap<Uuid, ProviderAggregate>", "for row in rows", "groups.entry(provider_id).or_default()", "match verdict.as_str()", "\"correct\" => entry.correct += 1", "\"partial\" => entry.partial += 1", "\"incorrect\" => entry.incorrect += 1", "_ => entry.not_verifiable += 1", "row.try_get::<Option<f64>, _>(\"accuracy_score\")?", "entry.accuracy.push(value)", "row.try_get::<Option<f64>, _>(\"reliability_score\")?", "entry.reliability.push(value)", "entry.timeliness.push(row.try_get(\"timeliness_score\")?)", ".push(row.try_get(\"decision_fingerprint\")?)"]), "供应商原计数/可用分数分母/全部时效/指纹读取顺序漂移");
for (const token of ["self.correct + self.partial + self.incorrect + self.not_verifiable", "mean(&self.accuracy).unwrap_or(0.0)", "mean(&self.timeliness).unwrap_or(0.0)", "mean(&self.reliability).unwrap_or(0.0)", "if self.accuracy.is_empty()", "round6(accuracy_mean * 0.60 + timeliness_mean * 0.15 + reliability_mean * 0.25)", "sha256_json(&self.decision_fingerprints)?", "(!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)"]) assert(provider.aggregation.includes(token), `原供应商公式/指纹缺少 ${token}`);
assert(/if self\.accuracy\.is_empty\(\)\s*\{\s*0\.0\s*\}/.test(provider.aggregation), "无准确率样本不能获得加权分数");
assert(!/\.sort(?:_unstable)?\(|\.dedup\(|\.filter\(/.test(provider.aggregation), "不得重排/去重指纹或改变分数样本分母");
assert(ordered(provider.aggregation, ["sha256_json(&self.decision_fingerprints)?", "let snapshot_key = format!(", '"{}:{}:{}:{}"', "provider_id, partition_key, decision_set_sha256, POSTMATCH_MONITORING_VERSION", "Ok((decision_set_sha256, snapshot_key))"]), "供应商原快照键身份顺序漂移");
for (const token of ["SELECT DISTINCT ON (score.provider_id, score.scope_key)", "JOIN catalog.data_providers provider ON provider.id=score.provider_id", "ORDER BY score.provider_id, score.scope_key, score.generated_at DESC, score.id DESC", "limit.clamp(1, 500)", "rows.iter().map(provider_score_from_row).collect()"]) assert(provider.read.includes(token), `供应商原最新快照读取缺少 ${token}`);
for (const field of ["sample_size", "correct_count", "partial_count", "incorrect_count", "not_verifiable_count"]) assert(new RegExp(`${field}: non_negative_u64\\(\\s*row.try_get\\("${field}"\\)\\?`).test(provider.mapping), `供应商计数解码缺少原负数门禁 ${field}`);
for (const name of ["empty_decisions_create_no_provider_groups", "empty_aggregate_keeps_zero_counts_means_and_score", "unverifiable_only_keeps_timeliness_but_cannot_gain_weighted_accuracy", "mixed_decisions_use_separate_available_score_denominators", "weighted_score_uses_unrounded_means_before_original_six_digit_rounding", "missing_reliability_and_timeliness_keep_original_zero_defaults", "sample_count_uses_all_verdict_counts_without_score_vector_lengths", "snapshot_digest_preserves_decision_order_and_duplicate_fingerprints", "snapshot_key_is_repeatable_and_separates_provider_and_partition"]) assert(provider.tests.includes(`fn ${name}(`), `缺少原PG target供应商评分测试 ${name}`);
console.log(
  "Postmatch Service 验证通过：7 个公共入口、2 个既有 Ports、资料包状态机与 PostgreSQL adapter 边界完整。",
);
