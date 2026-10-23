import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), "utf8");
const exists = (relativePath) => fs.existsSync(path.join(root, relativePath));
const failures = [];
const check = (condition, message) => {
  if (!condition) failures.push(message);
};

const legacyOwner = ["crates", "application", "src", "analytics.rs"].join("/");
const serviceRoot = "crates/application/src/services/analytics";
const useCaseRoot = "crates/application/src/use_cases/analytics";
const publicUseCaseFiles = [
  `${useCaseRoot}/calculation/overview.rs`,
  `${useCaseRoot}/decide_data_quality_finding/mod.rs`,
  `${useCaseRoot}/jobs/commands.rs`,
  `${useCaseRoot}/ai_package/exports.rs`,
  `${useCaseRoot}/ai_package/response.rs`,
  `${useCaseRoot}/ai_package/suggestions.rs`,
  `${useCaseRoot}/parameter_lifecycle/candidates.rs`,
  `${useCaseRoot}/parameter_lifecycle/readiness.rs`,
  `${useCaseRoot}/parameter_lifecycle/validation.rs`,
  `${useCaseRoot}/parameter_lifecycle/bindings.rs`,
];
const requiredFiles = [
  `${serviceRoot}/mod.rs`,
  `${serviceRoot}/service.rs`,
  `${serviceRoot}/facade.rs`,
  `${useCaseRoot}/mod.rs`,
  ...["mod.rs", "refresh.rs", "tests.rs"].map((file) => `${useCaseRoot}/calculation/${file}`),
  ...["mod.rs", "mapping.rs", "metrics.rs", "projection.rs", "read.rs", "repository.rs", "samples.rs", "tests.rs"].map((file) => `crates/persistence-postgres/src/adapters/analytics/calculation/${file}`),
  "crates/persistence-postgres/src/adapters/analytics/mod.rs",
  `${useCaseRoot}/jobs/mod.rs`,
  `${useCaseRoot}/jobs/worker.rs`,
  ...["commands.rs", "coordinator.rs", "execution.rs", "request.rs", "tests.rs"].map((file) => `${useCaseRoot}/jobs/${file}`),
  ...["mod.rs", "claim.rs", "commands.rs", "lifecycle.rs", "mapping.rs", "policy.rs", "read.rs", "tests.rs"].map((file) => `crates/persistence-postgres/src/adapters/jobs/${file}`),
  `${useCaseRoot}/ai_package/mod.rs`,
  `${useCaseRoot}/ai_package/tests.rs`,
  ...["mod.rs", "data.rs", "decisions.rs", "exports.rs", "imports.rs", "mapping.rs", "policy.rs", "suggestions.rs", "tests.rs"].map((file) => `crates/persistence-postgres/src/adapters/analytics/analysis_package/${file}`),
  `${useCaseRoot}/parameter_lifecycle/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/metrics.rs`,
  `${useCaseRoot}/parameter_lifecycle/tests.rs`,
  ...["mod.rs", "readiness.rs", "candidates.rs", "artifacts.rs", "replay.rs", "shadow.rs", "bindings.rs", "decisions.rs", "mapping.rs", "policy.rs", "tests.rs"].map((file) => `crates/persistence-postgres/src/parameter_lifecycle/${file}`),
  ...publicUseCaseFiles,
  "crates/application/src/ports/analytics/mod.rs",
  "crates/application/src/ports/analytics/contracts.rs",
  "crates/application/src/composition/adapters/analytics.rs",
  "crates/application/src/composition/adapters/jobs.rs",
];
for (const file of requiredFiles) {
  check(exists(file), `缺少 Analytics 模块文件：${file}`);
}
for (const file of [
  `${useCaseRoot}/analytics_overview/mod.rs`,
  `${useCaseRoot}/overview.rs`,
  `${useCaseRoot}/data_quality.rs`,
  `${useCaseRoot}/jobs.rs`,
  ...["enqueue_analysis_job", "list_background_jobs", "cancel_background_job", "retry_background_job"].map((entry) => `${useCaseRoot}/jobs/${entry}/mod.rs`),
  `${useCaseRoot}/ai_package.rs`,
  ...["export_ai_analysis_response_template", "export_ai_analysis_package", "preview_ai_analysis_response", "import_ai_analysis_response", "list_ai_analysis_suggestions", "decide_ai_analysis_suggestion"].map((entry) => `${useCaseRoot}/ai_package/${entry}/mod.rs`),
  ...["generate_parameter_tuning_candidate", "list_parameter_tuning_candidates", "decide_parameter_tuning_candidate", "parameter_lifecycle_readiness", "run_parameter_shadow_validation", "list_parameter_shadow_validations", "promote_parameter_candidate", "rollback_parameter_candidate", "list_parameter_promotion_decisions"].map((entry) => `${useCaseRoot}/parameter_lifecycle/${entry}/mod.rs`),
  "crates/persistence-postgres/src/parameter_lifecycle.rs",
]) {
  check(!exists(file), `Analytics 聚合 Use Case owner 未退出：${file}`);
}
check(!exists(legacyOwner), "旧 Analytics owner 仍然存在");

const library = read("crates/application/src/lib.rs");
const service = read(`${serviceRoot}/service.rs`);
const facade = read(`${serviceRoot}/facade.rs`);
const ports = read("crates/application/src/ports/analytics/mod.rs");
const contracts = read("crates/application/src/ports/analytics/contracts.rs");
const analyticsAdapter = read("crates/application/src/composition/adapters/analytics.rs");
const jobsAdapter = read("crates/application/src/composition/adapters/jobs.rs");
const composition = read("crates/application/src/composition/application_composition.rs");
const applicationService = read("crates/application/src/service/application_service.rs");
const worker = read(`${useCaseRoot}/jobs/worker.rs`);
const coordinator = read(`${useCaseRoot}/jobs/coordinator.rs`);
const execution = read(`${useCaseRoot}/jobs/execution.rs`);
const jobCommands = read(`${useCaseRoot}/jobs/commands.rs`);
const jobRequest = read(`${useCaseRoot}/jobs/request.rs`);
const parameterMetrics = read(`${useCaseRoot}/parameter_lifecycle/metrics.rs`);
const parameterShadow = read(`${useCaseRoot}/parameter_lifecycle/validation.rs`);
const packageDefinition = JSON.parse(read("package.json"));
const frontendVerifier = read("scripts/verify-frontend.mjs");

check(!library.includes("mod analytics;"), "lib.rs 仍注册旧顶层 Analytics owner");
check(service.includes("pub(crate) struct AnalyticsService"), "缺少 AnalyticsService");
check(facade.includes("impl ApplicationService"), "缺少 Analytics ApplicationService 兼容门面");
check(composition.includes("analytics: AnalyticsService"), "ApplicationComposition 未组合 AnalyticsService");
check(applicationService.includes("analytics: AnalyticsService"), "ApplicationService 未持有 AnalyticsService");

for (const traitName of ["AnalyticsPort", "JobQueuePort", "ParameterLifecyclePort"]) {
  const matches = ports.match(new RegExp(`\\bpub\\s+trait\\s+${traitName}\\b`, "g")) ?? [];
  check(matches.length === 1, `Analytics Port ${traitName} 数量应为 1，实际 ${matches.length}`);
}
check(analyticsAdapter.includes("impl AnalyticsPort for PersistenceStore"), "Analytics adapter 缺少 AnalyticsPort");
check(analyticsAdapter.includes("impl ParameterLifecyclePort for PersistenceStore"), "Analytics adapter 缺少 ParameterLifecyclePort");
check(!analyticsAdapter.includes("impl JobQueuePort for PersistenceStore"), "Analytics adapter 重复拥有 JobQueuePort");
check(jobsAdapter.includes("impl JobQueuePort for PersistenceStore"), "既有 jobs adapter 未实现 JobQueuePort");
check(!ports.includes("serde_json::Value"), "Analytics Port 暴露通用 JSON Value");
check(contracts.includes("pub struct ParameterDefinition"), "缺少参数定义专用契约");
check(contracts.includes("pub enum AnalyticsJobProgressPayload"), "缺少后台任务进度专用契约");
check(contracts.includes("pub enum AnalyticsJobResult"), "缺少后台任务结果专用契约");

const serviceAndUseCases = [
  service,
  worker,
  coordinator,
  execution,
  jobRequest,
  parameterMetrics,
  read(`${useCaseRoot}/calculation/refresh.rs`),
  ...publicUseCaseFiles.map(read),
].join("\n");
for (const token of ["sqlx::", "PostgresStore", "PersistenceStore", "football_persistence_postgres"]) {
  check(!serviceAndUseCases.includes(token), `Service/Use Case 泄漏基础设施符号：${token}`);
}
check(!service.includes("std::fs") && !service.includes("Path::new"), "AnalyticsService 直接承担文件 I/O");
check(read(`${useCaseRoot}/ai_package/exports.rs`).includes("write_analysis_package"), "AI Analysis Package 导出未归属独立 Use Case");
check(read(`${useCaseRoot}/ai_package/response.rs`).includes("read_analysis_response"), "AI Analysis Package 预检未归属独立 Use Case");
check(coordinator.includes("claim_next_by_types") && execution.includes("AnalyticsJobResult") && worker.includes("run_queue(&port).await"), "后台任务启动/领取结算/执行职责未切换唯一模块");
check(service.includes("start_job_worker") && service.includes("analytics::jobs::worker::spawn"), "AnalyticsService 未暴露后台 worker 启动边界");
check(parameterMetrics.includes("expected_calibration_error") && parameterMetrics.includes("lifecycle_metrics"), "Parameter Lifecycle 指标计算未独立拆分");
check(parameterShadow.includes("binding_unchanged") && parameterShadow.includes("finite_probabilities"), "Parameter shadow 安全门禁缺失");
check(publicUseCaseFiles.length === 10, `Analytics 公共入口应归属10个实际职责源（队列、资料包及参数生命周期按职责编组），实际 ${publicUseCaseFiles.length}`);


const calculationRoot = `${useCaseRoot}/calculation`;
const pgCalculation = "crates/persistence-postgres/src/adapters/analytics/calculation";
const calculationRefresh = read(`${calculationRoot}/refresh.rs`);
const calculationOverview = read(`${calculationRoot}/overview.rs`);
const repository = read(`${pgCalculation}/repository.rs`);
const samples = read(`${pgCalculation}/samples.rs`);
const metrics = read(`${pgCalculation}/metrics.rs`);
const projection = read(`${pgCalculation}/projection.rs`);
const pgRead = read(`${pgCalculation}/read.rs`);
const pgLegacy = read("crates/persistence-postgres/src/analytics.rs");
check(read("crates/persistence-postgres/src/adapters/analytics/analysis_package/data.rs").includes("use chrono::{DateTime, Utc};") &&  /use football_domain::\{[^}]*\bANALYTICS_CALCULATION_VERSION\b[^}]*\};/s.test(read("crates/persistence-postgres/src/adapters/analytics/analysis_package/exports.rs")), "资料包真实 owner 所需的时间/版本生产导入缺失");
const compact = (text) => text.replace(/\s+/g, "");
const occurrences = (text, token) => text.split(token).length - 1;
check(read(`${useCaseRoot}/mod.rs`).includes("pub(crate) mod calculation;") &&
  !read(`${useCaseRoot}/mod.rs`).includes("mod analytics_overview;"), "Calculation 未登记唯一 App owner");
check(service.includes("analytics::calculation::analytics_overview(port).await") &&
  !service.includes("analytics::analytics_overview"), "总览实际 Service 未切换 Calculation");
check(occurrences(execution, "refresh_analytics(port, &request).await?") === 2 &&
  !execution.includes("port.refresh("), "两条实际 worker 刷新必须经过 Calculation");
check(occurrences(calculationRefresh, "port.refresh(request).await?") === 1 &&
  occurrences(calculationOverview, "port.overview().await?") === 1, "Calculation 原 Port 单次边界改变");
for (const file of [`${calculationRoot}/mod.rs`, `${pgCalculation}/mod.rs`, "crates/persistence-postgres/src/adapters/analytics/mod.rs"]) {
  check(!/\bfn\b|\bimpl\b|\blet\b/.test(read(file)), `Calculation 出口承载业务：${file}`);
}
check(read("crates/persistence-postgres/src/adapters/mod.rs").includes("mod analytics;"), "PG Calculation 未接入原适配器树");
for (const owner of ["refresh_analytics", "analytics_overview", "load_evaluation_samples", "sample_group_ids",
  "insert_calibration_bucket", "insert_drift_finding", "evaluation_sample_from_row"]) {
  check(!new RegExp(`\\bfn\\s+${owner}\\b`).test(pgLegacy), `旧 PG analytics 重复计算职责：${owner}`);
}
check(occurrences(repository, "self.pool.begin().await?") === 1 &&
  occurrences(repository, "tx.commit().await?") === 1, "计算刷新必须保留原单事务");
const sequence = ["self.load_evaluation_samples(request).await?", "calculate_analytics(",
  "self.pool.begin().await?", "samples.chunks(500)", "analysis_windows(request, &samples, calculation.generated_at)",
  "serde_json::to_value(&calculation)?", "INSERT INTO analytics.analysis_snapshots", "for bucket in &calculation.calibration",
  "for finding in &calculation.drift", "sample_group_ids(&samples)", "for comparison in &calculation.comparisons",
  "write_audit_event(", "tx.commit().await?", "self.analytics_overview().await"];
let previous = -1;
for (const token of sequence) {
  const index = repository.indexOf(token);
  check(index > previous, `计算原事务/异常时点顺序改变：${token}`);
  previous = index;
}
check(repository.includes("ON CONFLICT (run_id, kickoff_time, calculation_version) DO UPDATE SET") &&
  repository.includes("ON CONFLICT (model_version_id, parameter_set_id, competition_id, window_start, window_end, calculation_version)") &&
  repository.includes("DO UPDATE SET sample_size=EXCLUDED.sample_size, metrics=EXCLUDED.metrics, calculated_at=EXCLUDED.calculated_at"), "原评估样本/模型指标冲突更新政策改变");
check(occurrences(metrics, ".execute(&mut **tx)") === 2 && !/\.begin\(|\.commit\(|&self\.pool/.test(metrics), "两个指标子记录必须借用原事务");
for (const token of ["SELECT DISTINCT ON (run.id)", "run.status='succeeded'", "review.status='finalized'",
  "COALESCE((review.prediction_evaluation->>'available')::boolean,false)=true",
  "fixture.kickoff_time >= $2", "fixture.kickoff_time <= $3", "ORDER BY run.id, review.created_at DESC, review.id DESC",
  ".bind(request.competition_id)", ".bind(request.window_start)", ".bind(request.window_end)",
  "rows.iter().map(evaluation_sample_from_row).collect()"])
  check(samples.includes(token), `计算样本原筛选/顺序/错误映射缺失：${token}`);
const normalizedProjection = compact(projection);
check(/pub\(super\)\s+struct\s+AnalysisWindows\s*\{/.test(projection) &&
  /fn\s+analysis_windows\([\s\S]*?\)\s*->\s*AnalysisWindows\s*\{/.test(projection), "计算窗口必须返回具名内部投影，避免复杂时间元组");
check(compact(repository).includes("letAnalysisWindows{window_start,window_end,metric_window_start,metric_window_end,}=analysis_windows(request,&samples,calculation.generated_at);"), "计算事务必须解构同一窗口投影的四个原字段");
for (const token of ["pub(super)window_start:Option<DateTime<Utc>>", "pub(super)window_end:Option<DateTime<Utc>>",
  "pub(super)metric_window_start:DateTime<Utc>", "pub(super)metric_window_end:DateTime<Utc>"])
  check(normalizedProjection.includes(token), `计算窗口具名字段类型改变：${token}`);
check(normalizedProjection.includes("AnalysisWindows{window_start,window_end,metric_window_start,metric_window_end,}"), "具名窗口字段必须保持原值映射");
check(!/#\s*\[\s*(?:allow|expect)\s*\([^\]]*type_complexity/.test(projection + repository), "计算窗口不得抑制 type_complexity");

for (const token of ["request.window_start.or_else(||samples.iter().map(|item|item.kickoff_time).min())",
  "request.window_end.or_else(||samples.iter().map(|item|item.kickoff_time).max())",
  "window_start.unwrap_or(generated_at)", "window_end.unwrap_or(generated_at)",
  "sample.model_key.clone()", "sample.model_version.clone()", "sample.parameter_version.clone()", "sample.snapshot_type.clone()",
  "(sample.model_version_id,sample.parameter_set_id)", ".collect()"])
  check(normalizedProjection.includes(token), `原窗口/分组投影规则缺失：${token}`);
check(!/\.sort|\.dedup|\.entry\(|\.or_insert/.test(projection), "计算投影不得改样本顺序或重复组覆盖政策");
check(pgRead.includes("SELECT summary FROM analytics.analysis_snapshots ORDER BY created_at DESC, id DESC LIMIT 1") &&
  compact(pgRead).includes(".map(serde_json::from_value).transpose()?;letquality=self.latest_data_quality_summary(100).await?;letquery_performance=self.latest_query_performance().await?;"), "总览解码与诊断读取先后改变");
for (const [dir, names] of [
  [calculationRoot, ["refresh_preserves_every_request_field_and_full_overview", "refresh_preserves_default_unbounded_request_without_extra_calls",
    "overview_keeps_snapshot_diagnostics_and_does_not_refresh", "refresh_propagates_all_port_errors_without_retry_or_followup_read", "overview_propagates_all_port_errors_without_refresh_or_retry"]],
  [pgCalculation, ["empty_samples_keep_null_snapshot_window_and_generated_metric_window", "inferred_windows_use_min_max_without_reordering_and_keep_nanoseconds",
    "explicit_windows_override_samples_without_repairing_inverted_bounds", "one_sided_windows_infer_only_missing_bound_and_fallback_only_empty_bound",
    "group_identity_uses_all_four_case_sensitive_strings", "duplicate_group_uses_last_actual_uuid_pair_in_original_input_order",
    "missing_snapshot_keeps_empty_calculation_and_actual_diagnostics", "saved_calculation_projects_every_metric_record_version_and_diagnostics"]],
]) {
  check(read(`${dir}/mod.rs`).includes("#[cfg(test)]\nmod tests;"), `计算边界测试未登记原 target：${dir}`);
  for (const name of names) check(read(`${dir}/tests.rs`).includes(`fn ${name}(`), `计算边界测试缺失：${name}`);
}


const pgJobs = "crates/persistence-postgres/src/adapters/jobs";
const legacyPgJobs = ["crates", "persistence-postgres", "src", "jobs.rs"].join("/");
const claim = read(`${pgJobs}/claim.rs`);
const lifecycle = read(`${pgJobs}/lifecycle.rs`);
const queueCommands = read(`${pgJobs}/commands.rs`);
const queueRead = read(`${pgJobs}/read.rs`);
const queuePolicy = read(`${pgJobs}/policy.rs`);
const queueMapping = read(`${pgJobs}/mapping.rs`);
check(!exists(legacyPgJobs) && !read("crates/persistence-postgres/src/lib.rs").includes("mod jobs;"), "旧根队列owner未退出");
check(read("crates/persistence-postgres/src/adapters/mod.rs").includes("mod jobs;"), "共享队列owner未登记原adapter树");
for (const file of [`${useCaseRoot}/jobs/mod.rs`, `${pgJobs}/mod.rs`])
  check(!/\bfn\b|\bimpl\b|\blet\b/.test(read(file)), `Jobs出口承载业务：${file}`);
for (const entry of ["enqueue_analysis_job", "list_background_jobs", "cancel_background_job", "retry_background_job"]) {
  check(occurrences(jobCommands, `fn ${entry}<`) === 1 && service.includes(`analytics::jobs::${entry}(`), `后台命令真实入口未切换：${entry}`);
}
check(occurrences(worker, "tokio::spawn(") === 1 && !worker.includes("claim_next_by_types") && !worker.includes("execute_job"), "worker必须只持有原异步启动边界");
check(!coordinator.includes("tokio::spawn") && !coordinator.includes("update_progress") && !coordinator.includes("run_data_quality_scan"), "领取/结算协调器混入任务执行职责");
check(occurrences(coordinator, "claim_next_by_types(") === 1 && !coordinator.includes("p4_horizon"), "分析协调器领取集合不得混入P4");
for (const token of ["\"refresh_analytics\"", "\"data_quality_scan\"", "\"query_performance_scan\"", "\"full_analysis_refresh\""])
  check(coordinator.includes(token), `分析协调器缺少原领取类型：${token}`);
for (const token of ["Ok(None) => break", "Err(_) => break", "let _ = port.complete(job.id, value).await;", "let _ = port.fail(job.id, &error.to_string()).await;"])
  check(coordinator.includes(token), `原领取退出/结算错误处理政策缺失：${token}`);
check(occurrences(execution, 'ApplicationError::Validation("任务已取消".to_string())') === 6, "四类任务六取消检查点改变");
for (const token of ["15.0", "20.0", "10.0", "55.0", "82.0", "AnalyticsJobProgressPayload::SampleSize(overview.sample_size)", "AnalyticsJobProgressPayload::OpenFindings(quality.open_total)"])
  check(execution.includes(token), `原任务进度/载荷缺失：${token}`);
check(jobRequest.includes("payload.is_null() || payload.as_object().is_some_and(|item| item.is_empty())") &&
  jobRequest.includes("AnalyticsRefreshRequest::default()") && jobRequest.includes("serde_json::from_value(payload.clone())?"), "原空/部分/错误请求解码政策改变");
const queueSequence = ["let job = port.enqueue(&draft).await?;", "worker::spawn(port.clone());", "Ok(job)"];
let point = -1;
for (const token of queueSequence) { const current = jobCommands.indexOf(token); check(current > point, `原排队成功后启动顺序改变：${token}`); point = current; }
check(occurrences(jobCommands, "worker::spawn(port.clone());") === 2 && jobCommands.includes("let job = port.retry(job_id).await?;"), "原排队/重试成功后启动次数改变");
for (const token of ["job_types.is_empty()", "status='queued'", "cancellation_requested=false", "attempts < max_attempts", "available_at <= now()", "job_type = ANY($1)",
  "ORDER BY priority DESC,available_at,created_at,id", "FOR UPDATE SKIP LOCKED", "LIMIT 1", "attempts=attempts+1", "'started',1,'任务已开始'"])
  check(claim.includes(token), `共享领取原筛选/锁/计数/事件缺失：${token}`);
check(occurrences(claim, "self.pool.begin().await?") === 1 && occurrences(claim, "tx.commit().await?") === 2 &&
  claim.lastIndexOf("tx.commit().await?") > claim.indexOf("INSERT INTO platform.job_events") &&
  claim.lastIndexOf("tx.commit().await?") < claim.indexOf("job_from_row(&claimed)?"), "领取空分支/提交后解码事务顺序改变");
check(occurrences(lifecycle, "self.pool.begin().await?") === 3 && occurrences(lifecycle, "progress.clamp(0.0, 99.0)") === 2 &&
  lifecycle.includes("return Ok(true);") && lifecycle.includes("status='running'"), "原progress缺行/夹紧/事务政策改变");
for (const token of ["CASE WHEN cancellation_requested THEN 'cancelled' ELSE 'succeeded' END", "CASE WHEN cancellation_requested THEN NULL ELSE $2 END", "Value::Null",
  "attempts,max_attempts,cancellation_requested FROM platform.jobs WHERE id=$1 FOR UPDATE", "interval '30 seconds'", "failure_disposition(attempts, max_attempts, cancellation_requested)",
  "WHERE status='running'", "应用重启后重新排队", "应用重启时已达到最大尝试次数"])
  check(lifecycle.includes(token), `原完成/失败/启动恢复政策缺失：${token}`);
check(compact(queuePolicy).includes('ifcancellation_requested{("cancelled",true)}elseifattempts<max_attempts{("queued",false)}else{("failed",true)}'), "失败取消优先/重试预算政策改变");
for (const token of ["p4_horizon_research", "p4_horizon_freeze", "refresh_analytics", "data_quality_scan", "query_performance_scan", "full_analysis_refresh",
  "不支持的任务类型：{job_type}", "未知任务状态：{other}"])
  check(queuePolicy.includes(token), `原共享任务类型/状态错误规则缺失：${token}`);
check(compact(queueMapping).includes('letstatus:String=row.try_get("status")?;letstatus=job_status(&status)?;') &&
  queueRead.includes("ORDER BY created_at DESC,id DESC LIMIT $1") && queueRead.includes("limit.clamp(1, 500)") &&
  queueCommands.includes("ON CONFLICT (idempotency_key) DO UPDATE SET idempotency_key=EXCLUDED.idempotency_key") && queueCommands.includes("draft.max_attempts.clamp(1, 20)"), "队列映射/读取/幂等入队边界改变");
for (const token of ["status IN ('queued','running')", "status IN ('failed','cancelled') AND attempts < max_attempts", "任务不可重试，可能仍在运行或已达到最大尝试次数", "self.read_job(job_id).await"])
  check(queueCommands.includes(token), `原人工取消/重试边界缺失：${token}`);
for (const [dir, names] of [
  [`${useCaseRoot}/jobs`, ["request_defaults_partial_fields_and_nanoseconds_remain_original", "malformed_refresh_payload_stops_before_progress_or_work", "full_refresh_preserves_progress_payloads_order_request_and_result", "standalone_jobs_preserve_initial_checkpoint_and_complete_typed_result", "cancellation_at_each_checkpoint_stops_all_later_side_effects", "work_and_progress_errors_propagate_without_later_stages", "unsupported_job_kind_is_rejected_before_any_port_call", "coordinator_claims_only_original_four_types_and_completes_each_job", "coordinator_records_execution_failure_and_ignores_settlement_errors_before_next_claim", "empty_or_failed_claim_ends_worker_without_execution_or_settlement", "commands_preserve_input_and_start_worker_only_after_enqueue_or_retry_success", "list_and_cancel_keep_original_limits_identity_and_returned_job", "all_command_port_errors_keep_kind_and_message_without_starting_worker"]],
  [pgJobs, ["supports_exact_original_analysis_and_p4_task_types", "decodes_all_five_original_job_states", "rejects_unknown_state_without_trimming_case_conversion_or_aliases", "cancellation_has_priority_over_retry_budget_and_finishes_failure", "retries_strictly_below_original_attempt_budget", "reaches_terminal_failure_at_or_above_budget_without_new_validation"]],
]) {
  check(read(`${dir}/mod.rs`).includes("#[cfg(test)]\nmod tests;"), `后台任务测试未登记原target：${dir}`);
  for (const name of names) check(read(`${dir}/tests.rs`).includes(`fn ${name}(`), `后台任务边界测试缺失：${name}`);
}


const aiRoot = `${useCaseRoot}/ai_package`;
const pgPackage = "crates/persistence-postgres/src/adapters/analytics/analysis_package";
const appExports = read(`${aiRoot}/exports.rs`);
const appResponse = read(`${aiRoot}/response.rs`);
const appSuggestions = read(`${aiRoot}/suggestions.rs`);
const packageData = read(`${pgPackage}/data.rs`);
const packageImport = read(`${pgPackage}/imports.rs`);
const packageDecision = read(`${pgPackage}/decisions.rs`);
const packagePolicy = read(`${pgPackage}/policy.rs`);
const packageSuggestions = read(`${pgPackage}/suggestions.rs`);
for (const file of [`${aiRoot}/mod.rs`, `${pgPackage}/mod.rs`])
  check(!/\bfn\b|\bimpl\b|\blet\b/.test(read(file)), `资料包出口承载业务：${file}`);
check(read("crates/persistence-postgres/src/adapters/analytics/mod.rs").includes("mod analysis_package;"), "PG资料包owner未登记原analytics适配器树");
for (const [entry, file] of [["export_ai_analysis_response_template", appExports], ["export_ai_analysis_package", appExports],
  ["preview_ai_analysis_response", appResponse], ["import_ai_analysis_response", appResponse],
  ["list_ai_analysis_suggestions", appSuggestions], ["decide_ai_analysis_suggestion", appSuggestions]]) {
  check(occurrences(file, `fn ${entry}`) === 1 && service.includes(`analytics::ai_package::${entry}(`), `资料包实际入口未切换：${entry}`);
}
let exportStep = -1;
for (const token of ["let data = port.build_ai_analysis_data().await?;", "write_analysis_package(Path::new(&output_path), &data)?", "port.record_ai_export(&summary).await?;", "Ok(summary)"]) {
  const current = appExports.indexOf(token); check(current > exportStep, `原数据→写包→元数据记录顺序改变：${token}`); exportStep = current;
}
check(appExports.includes("response_template_bytes(source_package_id)?") && appExports.includes("fs::write(&output_path, bytes)?"), "模板原版本/来源参数/文件错误边界改变");
check(occurrences(appResponse, "read_analysis_response(Path::new(&input_path))?") === 2 && appResponse.includes("port.import_ai_response(&input_path, &preview).await?"), "同一文件预览/导入原解析边界改变");
check(appSuggestions.includes("status.as_deref(), limit") && appSuggestions.includes("port.decide_ai_suggestion(&draft).await?"), "建议列表/人工决定原Port边界改变");
for (const owner of ["build_ai_analysis_data", "record_ai_export", "import_ai_response", "list_ai_suggestions", "decide_ai_suggestion", "database_summary_json", "player_review_summary_json", "team_review_summary_json", "ability_candidates_json", "schema_summary_json", "read_ai_suggestion", "ai_suggestion_from_row", "json_string", "json_number", "json_uuid"])
  check(!new RegExp(`\\bfn\\s+${owner}\\b`).test(pgLegacy), `根analytics重复资料包职责：${owner}`);
let dataStep = -1;
for (const token of ["self.analytics_overview().await?", "self.database_summary_json().await?", "self.player_review_summary_json().await?", "self.team_review_summary_json().await?", "self.ability_candidates_json().await?", "self.schema_summary_json().await?"]) {
  const current = packageData.indexOf(token); check(current > dataStep, `资料包六数据读取原顺序改变：${token}`); dataStep = current;
}
check(occurrences(packageImport, "self.pool.begin().await?") === 1 && occurrences(packageImport, "tx.commit().await?") === 1, "回包导入原单事务改变");
let importStep = -1;
for (const token of ["validate_import_preview(preview)?", "self.pool.begin().await?", "SELECT EXISTS(SELECT 1 FROM analytics.ai_package_imports WHERE response_id=$1)", "if already_imported", "if let Some(source_package_id)", "SELECT EXISTS(SELECT 1 FROM analytics.ai_package_exports WHERE package_id=$1)", "if !source_exists", "INSERT INTO analytics.ai_package_imports", "for suggestion in &preview.suggestions", "INSERT INTO analytics.ai_suggestions", "write_audit_event(", "tx.commit().await?", 'self.list_ai_suggestions(Some("pending"), 500).await']) {
  const current = packageImport.indexOf(token); check(current > importStep, `原回包阻断/身份/导入/提交后全pending回读顺序改变：${token}`); importStep = current;
}
for (const token of ["该 AI 回包已经导入，不能重复写入建议", "AI 回包引用的来源分析包不属于当前数据库", "suggestion_severity(&suggestion.severity)", "serde_json::to_value(&preview.warnings)?", "$9,'pending'", "ai_analysis_response_imported"])
  check(packageImport.includes(token), `原回包幂等/来源/待审/审计政策缺失：${token}`);
check(occurrences(packageDecision, "self.pool.begin().await?") === 1 && occurrences(packageDecision, "tx.commit().await?") === 1, "人工建议决定原单事务改变");
let decisionStep = -1;
for (const token of ["self.pool.begin().await?", "SELECT suggestion_type,payload,status FROM analytics.ai_suggestions WHERE id=$1 FOR UPDATE", "let status: String", "ensure_pending(&status)?", "let suggestion_type: String", "let payload: Value", "AiSuggestionDecision::Accept if suggestion_type == \"ability_update\"", "json_uuid(&payload", "json_string(&payload", "let identity_valid:", "if !identity_valid", "ability_values(&payload)?", "SELECT value FROM feature.player_current_abilities", "INSERT INTO review.ability_update_candidates", "UPDATE analytics.ai_suggestions SET status", "write_audit_event(", "tx.commit().await?", "self.read_ai_suggestion(draft.suggestion_id).await"]) {
  const current = packageDecision.indexOf(token); check(current > decisionStep, `人工建议原锁/状态/身份/候选/提交顺序改变：${token}`); decisionStep = current;
}
for (const token of ['AiSuggestionDecision::Reject => "rejected"', 'AiSuggestionDecision::Accept => "accepted"', 'unwrap_or("desktop-user")', "'ai-analysis-response-v1','pending'", "AI 能力建议引用了不存在的球员或能力维度", "ai_suggestion_decided"])
  check(packageDecision.includes(token), `原人工建议/待审候选政策缺失：${token}`);
check(!/INSERT INTO feature\.player_ability_observations|INSERT INTO model\.|promote_parameter|promote_candidate/.test(packageDecision), "AI建议不得直接写正式能力观察或自动晋升参数");
for (const token of ['!preview.blocking_errors.is_empty()', 'preview.blocking_errors.join("；")', 'if status != "pending"', 'json_number(payload, "proposed_value")?.clamp(0.0, 100.0)', '.unwrap_or(0.5)', '.clamp(0.0, 1.0)', '.clamp(0, i32::MAX as i64) as i32'])
  check(packagePolicy.includes(token), `原导入/待审/能力payload政策缺失：${token}`);
check(compact(packagePolicy).includes('ifseverity.trim().is_empty(){"info"}else{severity}') &&
  compact(packagePolicy).includes('Ok(AbilitySuggestionValues{proposed_value,confidence,sample_size,})'), "严重级别原样回传/具名能力值投影改变");
check(packageSuggestions.includes("ORDER BY created_at DESC,id DESC LIMIT $2") && packageSuggestions.includes("limit.clamp(1, 1000)") && packageSuggestions.includes(".bind(status)"), "建议读取原筛选/上限/顺序改变");
for (const [dir, names] of [
  [aiRoot, ["template_writes_real_response_zip_with_optional_source_and_valid_hash", "import_forwards_same_file_preview_warnings_and_unfiltered_port_result", "missing_or_invalid_zip_fails_before_import_port", "export_writes_package_then_records_exact_summary_with_repeatable_content_hash", "data_read_errors_preserve_kind_and_stop_before_creating_file", "file_write_failures_keep_original_error_boundary_and_skip_export_record", "metadata_errors_leave_original_written_package_and_propagate_all_port_kinds", "import_port_errors_keep_kind_without_extra_writes_or_retry", "suggestion_list_preserves_optional_filter_limits_and_complete_records", "explicit_accept_and_reject_preserve_manual_identity_note_and_port_record", "suggestion_control_errors_preserve_all_port_kinds_without_other_actions"]],
  [pgPackage, ["blocking_preview_errors_keep_original_order_and_separator", "warnings_and_empty_suggestions_do_not_become_blocking_errors", "severity_defaults_only_for_blank_and_otherwise_keeps_original_bytes", "manual_decision_requires_exact_pending_status", "payload_string_values_remain_untrimmed_and_wrong_types_keep_error", "payload_uuid_preserves_valid_nil_and_original_missing_or_invalid_errors", "ability_values_keep_original_clamps_defaults_and_integer_type_policy", "ability_proposed_value_must_be_numeric_before_optional_defaults"]],
]) {
  check(read(`${dir}/mod.rs`).includes("#[cfg(test)]\nmod tests;"), `资料包测试未登记原target：${dir}`);
  for (const name of names) check(read(`${dir}/tests.rs`).includes(`fn ${name}(`), `资料包边界测试缺失：${name}`);
}

const publicMethods = [
  "analytics_overview",
  "enqueue_analysis_job",
  "list_background_jobs",
  "cancel_background_job",
  "retry_background_job",
  "decide_data_quality_finding",
  "export_ai_analysis_response_template",
  "export_ai_analysis_package",
  "preview_ai_analysis_response",
  "import_ai_analysis_response",
  "list_ai_analysis_suggestions",
  "decide_ai_analysis_suggestion",
  "generate_parameter_tuning_candidate",
  "list_parameter_tuning_candidates",
  "decide_parameter_tuning_candidate",
  "parameter_lifecycle_readiness",
  "run_parameter_shadow_validation",
  "list_parameter_shadow_validations",
  "promote_parameter_candidate",
  "rollback_parameter_candidate",
  "list_parameter_promotion_decisions",
];
check(publicMethods.length === 21, "Analytics公共门面21入口数量改变");
for (const method of publicMethods) {
  check(facade.includes(`fn ${method}`), `Analytics 兼容门面缺少方法：${method}`);
}

const executableRoots = ["scripts", "crates/application/src", "src-tauri/src", "src"];
const staleReferences = [];
const executableExtensions = new Set([".mjs", ".js", ".ts", ".rs", ".json"]);
const walk = (relativeDir) => {
  const absolute = path.join(root, relativeDir);
  if (!fs.existsSync(absolute)) return;
  for (const entry of fs.readdirSync(absolute, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.name === "target") continue;
    const relativePath = path.join(relativeDir, entry.name).replaceAll("\\", "/");
    if (entry.isDirectory()) {
      walk(relativePath);
    } else if (executableExtensions.has(path.extname(entry.name))) {
      const text = read(relativePath);
      if (text.includes(legacyOwner)) staleReferences.push(relativePath);
    }
  }
};
for (const dir of executableRoots) walk(dir);
check(staleReferences.length === 0, `仍有可执行链引用旧 Analytics owner：${staleReferences.join(", ")}`);


const parameterRoot = `${useCaseRoot}/parameter_lifecycle`;
const pgParameterRoot = "crates/persistence-postgres/src/parameter_lifecycle";
const parameterCandidates = read(`${parameterRoot}/candidates.rs`);
const parameterReadiness = read(`${parameterRoot}/readiness.rs`);
const parameterBindings = read(`${parameterRoot}/bindings.rs`);
const pgParameterCandidates = read(`${pgParameterRoot}/candidates.rs`);
const pgParameterReadiness = read(`${pgParameterRoot}/readiness.rs`);
const pgParameterArtifacts = read(`${pgParameterRoot}/artifacts.rs`);
const pgParameterReplay = read(`${pgParameterRoot}/replay.rs`);
const pgParameterShadow = read(`${pgParameterRoot}/shadow.rs`);
const pgParameterBindings = read(`${pgParameterRoot}/bindings.rs`);
const pgParameterDecisions = read(`${pgParameterRoot}/decisions.rs`);
const pgParameterPolicy = read(`${pgParameterRoot}/policy.rs`);
const pgParameterMapping = read(`${pgParameterRoot}/mapping.rs`);
for (const file of [`${parameterRoot}/mod.rs`, `${pgParameterRoot}/mod.rs`]) {
  check(!/\bfn\b|\bimpl\b|\blet\b/.test(read(file)), `参数生命周期出口承载业务：${file}`);
  check(read(file).includes("#[cfg(test)]\nmod tests;"), `参数生命周期未登记原单测：${file}`);
}
const parameterEntries = ["generate_parameter_tuning_candidate", "list_parameter_tuning_candidates", "decide_parameter_tuning_candidate", "parameter_lifecycle_readiness", "run_parameter_shadow_validation", "list_parameter_shadow_validations", "promote_parameter_candidate", "rollback_parameter_candidate", "list_parameter_promotion_decisions"];
const parameterApp = [parameterCandidates, parameterReadiness, parameterShadow, parameterBindings].join("\n");
for (const entry of parameterEntries) {
  check(occurrences(parameterApp, `fn ${entry}`) === 1 && service.includes(`parameter_lifecycle::${entry}(`) && !service.includes(`parameter_lifecycle::${entry}::execute`), `参数实际入口或唯一 owner 未切换：${entry}`);
}
for (const entry of ["save_parameter_tuning_candidate", "list_parameter_tuning_candidates", "decide_parameter_tuning_candidate", "read_parameter_tuning_candidate", "parameter_tuning_candidate_from_row"]) {
  check(!new RegExp(String.raw`\bfn\s+${entry}\b`).test(pgLegacy), `PG analytics 仍承载参数职责：${entry}`);
}
check(read("crates/persistence-postgres/src/lib.rs").includes("mod parameter_lifecycle;") && read("architecture/module-boundaries.json").includes(`${pgParameterRoot}/mod.rs`), "参数目录未保持原根模块登记与直接 owner");
check(parameterCandidates.includes("公开仓库未捆绑模型提供器，不能生成提供器私有参数候选") && !parameterCandidates.includes("save_tuning_candidate"), "公开生成不得替换为自动参数生成");
check(compact(parameterReadiness).includes('matches!(snapshot_type,"T-N"|"T-24h"|"T-6h"|"T-1h")') && parameterReadiness.includes("request.minimum_sample_size < 20"), "原就绪 horizon 或最小样本规则改变");
const shadowOrder = ["port.read_tuning_candidate(request.candidate_id).await?", "candidate.status.as_str()", "required_candidate_uuid(candidate.competition_id", "port.readiness(&readiness_request).await?", "if !readiness.ready_for_shadow_validation", "return Ok(port.save_shadow_validation(&record).await?)", ".load_replay_fixtures(", "fixtures.sort_by_key(|item| (item.kickoff_time, item.run_id))", "validate_baseline_probabilities(&fixtures)?", "fixtures.len() < minimum_sample_size(&candidate)", "lifecycle_split(&fixtures)?", ".read_parameter_set_definition(candidate_parameter_set_id)", ".validate_parameters(candidate_parameters.as_value())", "for fixture in holdout", "let binding_unchanged", "validation_key(&candidate, holdout, \"holdout\")", "Ok(port.save_shadow_validation(&record).await?)"];
let shadowPosition = -1;
for (const token of shadowOrder) { const next = parameterShadow.indexOf(token, shadowPosition + 1); check(next > shadowPosition, `原影子验证调用/错误顺序改变：${token}`); shadowPosition = next; }
check(parameterShadow.includes('"automatic_promotion": false') && parameterShadow.includes('"provider_state": "NOT_BUNDLED"') && !parameterShadow.includes("port.promote("), "影子验证不得自动修改正式绑定");
check(parameterBindings.indexOf("request.decision_note.trim().len() < 8") < parameterBindings.indexOf("port.read_tuning_candidate") && parameterBindings.includes("readiness.active_model_version_id != candidate.baseline_model_version_id") && parameterBindings.includes("readiness.active_parameter_set_id != candidate.baseline_parameter_set_id"), "人工晋升原说明/基线门禁改变");
check(occurrences(parameterBindings, "request.decision_note.trim().len() < 8") === 2 && occurrences(parameterBindings, "port.promote(&request).await?") === 1 && occurrences(parameterBindings, "port.rollback(&request).await?") === 1, "人工晋升/回滚请求不能新增透传重试或自动操作");
check(pgParameterReadiness.includes("contract_key = 'p4-postmatch-settlement'") && pgParameterReadiness.includes("stage = 'H'") && pgParameterReadiness.includes("status = 'locked'") && pgParameterReadiness.includes("settlement.horizon = $3") && pgParameterReadiness.includes("sample.parameter_set_id = settlement.parameter_set_id"), "原H与精确样本分区核验缺失");
check(pgParameterReplay.includes("SELECT DISTINCT ON (run.id)") && pgParameterReplay.includes("run.parameter_set_id = settlement.parameter_set_id") && pgParameterReplay.includes("review.prediction_evaluation->>'available'"), "原回放事实与参数身份过滤缺失");
check(pgParameterCandidates.includes("limit.clamp(1, 500)") && pgParameterCandidates.includes("WHERE id=$1 AND status='pending'") && pgParameterCandidates.includes("let status = tuning_decision_status(draft.decision);"), "参数候选原读取或显式人工审核改变");
check(compact(pgParameterPolicy).includes('ParameterTuningDecision::AcceptForBacktest=>"accepted_for_backtest"') && compact(pgParameterPolicy).includes('ParameterTuningDecision::Reject=>"rejected"') && compact(pgParameterPolicy).includes('"passed"=>"shadow_passed"') && compact(pgParameterPolicy).includes('"failed"=>"shadow_failed"') && compact(pgParameterPolicy).includes('_=>"blocked_by_h"'), "参数原状态转换规则改变");
check(pgParameterPolicy.includes('if status != "shadow_passed"') && pgParameterPolicy.includes('if status != "promoted"') && pgParameterPolicy.includes("u64::try_from(value)"), "参数人工门禁或负数拒绝规则改变");
check(pgParameterMapping.includes('u64::try_from(sample_size)') && pgParameterMapping.includes('row.try_get("sample_count")?'), "参数样本映射不得默默夹紧或丢弃错误");
check(occurrences(pgParameterArtifacts, "self.pool.begin().await?") === 1 && occurrences(pgParameterArtifacts, "tx.commit().await?") === 1 && pgParameterArtifacts.indexOf("INSERT INTO model.versions") >= 0 && pgParameterArtifacts.indexOf("INSERT INTO model.versions") < pgParameterArtifacts.indexOf("INSERT INTO model.parameter_sets") && pgParameterArtifacts.indexOf("INSERT INTO model.parameter_sets") < pgParameterArtifacts.indexOf("INSERT INTO analytics.parameter_tuning_candidates"), "候选制品原单事务或三表顺序改变");
check(pgParameterShadow.includes("ON CONFLICT (candidate_id, validation_key) DO NOTHING") && occurrences(pgParameterShadow, "self.pool.begin().await?") === 1 && occurrences(pgParameterShadow, "tx.commit().await?") === 1 && pgParameterShadow.includes("let candidate_status = shadow_candidate_status(&record.status);"), "影子报告原幂等键/不可覆盖或事务改变");
for (const token of ["ensure_promotable(&candidate.status)?", "ensure_rollback_allowed(&candidate.status)?", "FOR UPDATE OF binding", "AND model_version_id = $4", "AND parameter_set_id = $5", "WHERE id=$1 AND status='shadow_passed'", "WHERE id=$1 AND status='promoted'", "updated.rows_affected() != 1", "promoted.rows_affected() != 1", "rolled_back.rows_affected() != 1", "拒绝覆盖式晋升", "拒绝覆盖式回滚"]) check(pgParameterBindings.includes(token), `原人工绑定并发/状态/CAS保护缺失：${token}`);
check(occurrences(pgParameterBindings, "self.pool.begin().await?") === 2 && occurrences(pgParameterBindings, "tx.commit().await?") === 2 && occurrences(pgParameterBindings, "write_audit_event(") === 2 && !/UPDATE\s+analytics\.parameter_(?:promotion_decisions|binding_changes)/i.test(pgParameterBindings), "晋升/回滚必须各保留原独立事务和不可变账本");
check(pgParameterDecisions.includes("ORDER BY created_at DESC, id DESC") && pgParameterDecisions.includes("pub(super) async fn read_parameter_promotion_decision"), "原决策账本读取或跨兄弟可见性缺失");
check(occurrences(read(`${parameterRoot}/tests.rs`), "#[tokio::test]") === 15 && occurrences(read(`${pgParameterRoot}/tests.rs`), "#[test]") === 6, "参数新增原单测边界缺失");

check(
  packageDefinition.scripts?.["verify:analytics-service"] === "node scripts/verify-analytics-service.mjs",
  "package.json 未登记 Analytics Service 验证入口",
);
check(
  packageDefinition.scripts?.["verify:architecture"]?.includes("verify-analytics-service.mjs"),
  "Architecture 聚合门禁未接入 Analytics Service",
);
check(frontendVerifier.includes('"verify-analytics-service.mjs"'), "前端聚合验证未接入 Analytics Service");

// R11-07 transport contract BEGIN
{
  const read = relative => fs.readFileSync(path.join(root, relative), "utf8").replaceAll("\r\n", "\n");
  const check = (ok, message) => { if (!ok) failures.push(message); };
  const domains = {
  "analytics": {
    "": [
      "analytics_overview",
      "decide_data_quality_finding"
    ],
    "jobs": [
      "enqueue_analysis_job",
      "list_background_jobs",
      "cancel_background_job",
      "retry_background_job"
    ],
    "ai_package": [
      "export_ai_analysis_package",
      "preview_ai_analysis_response",
      "import_ai_analysis_response",
      "list_ai_analysis_suggestions",
      "decide_ai_analysis_suggestion",
      "export_ai_analysis_response_template"
    ],
    "parameter_lifecycle": [
      "generate_parameter_tuning_candidate",
      "list_parameter_tuning_candidates",
      "decide_parameter_tuning_candidate",
      "parameter_lifecycle_readiness",
      "run_parameter_shadow_validation",
      "list_parameter_shadow_validations",
      "promote_parameter_candidate",
      "rollback_parameter_candidate",
      "list_parameter_promotion_decisions"
    ]
  }
};
  const files = {
  "analytics_overview": "src-tauri/src/commands/analytics/analytics_overview.rs",
  "decide_data_quality_finding": "src-tauri/src/commands/analytics/decide_data_quality_finding.rs",
  "enqueue_analysis_job": "src-tauri/src/commands/analytics/jobs/enqueue_analysis_job.rs",
  "list_background_jobs": "src-tauri/src/commands/analytics/jobs/list_background_jobs.rs",
  "cancel_background_job": "src-tauri/src/commands/analytics/jobs/cancel_background_job.rs",
  "retry_background_job": "src-tauri/src/commands/analytics/jobs/retry_background_job.rs",
  "export_ai_analysis_package": "src-tauri/src/commands/analytics/ai_package/export_ai_analysis_package.rs",
  "preview_ai_analysis_response": "src-tauri/src/commands/analytics/ai_package/preview_ai_analysis_response.rs",
  "import_ai_analysis_response": "src-tauri/src/commands/analytics/ai_package/import_ai_analysis_response.rs",
  "list_ai_analysis_suggestions": "src-tauri/src/commands/analytics/ai_package/list_ai_analysis_suggestions.rs",
  "decide_ai_analysis_suggestion": "src-tauri/src/commands/analytics/ai_package/decide_ai_analysis_suggestion.rs",
  "export_ai_analysis_response_template": "src-tauri/src/commands/analytics/ai_package/export_ai_analysis_response_template.rs",
  "generate_parameter_tuning_candidate": "src-tauri/src/commands/analytics/parameter_lifecycle/generate_parameter_tuning_candidate.rs",
  "list_parameter_tuning_candidates": "src-tauri/src/commands/analytics/parameter_lifecycle/list_parameter_tuning_candidates.rs",
  "decide_parameter_tuning_candidate": "src-tauri/src/commands/analytics/parameter_lifecycle/decide_parameter_tuning_candidate.rs",
  "parameter_lifecycle_readiness": "src-tauri/src/commands/analytics/parameter_lifecycle/parameter_lifecycle_readiness.rs",
  "run_parameter_shadow_validation": "src-tauri/src/commands/analytics/parameter_lifecycle/run_parameter_shadow_validation.rs",
  "list_parameter_shadow_validations": "src-tauri/src/commands/analytics/parameter_lifecycle/list_parameter_shadow_validations.rs",
  "promote_parameter_candidate": "src-tauri/src/commands/analytics/parameter_lifecycle/promote_parameter_candidate.rs",
  "rollback_parameter_candidate": "src-tauri/src/commands/analytics/parameter_lifecycle/rollback_parameter_candidate.rs",
  "list_parameter_promotion_decisions": "src-tauri/src/commands/analytics/parameter_lifecycle/list_parameter_promotion_decisions.rs"
};
  const signatures = {
  "analytics_overview": "pub async fn analytics_overview(\n    state: State<'_, AppState>,\n) -> Result<football_domain::AnalyticsOverview, String>",
  "decide_data_quality_finding": "pub async fn decide_data_quality_finding(\n    state: State<'_, AppState>,\n    draft: football_domain::DataQualityDecisionDraft,\n) -> Result<football_domain::DataQualityFinding, String>",
  "enqueue_analysis_job": "pub async fn enqueue_analysis_job(\n    state: State<'_, AppState>,\n    draft: football_domain::EnqueueJobDraft,\n) -> Result<football_domain::BackgroundJob, String>",
  "list_background_jobs": "pub async fn list_background_jobs(\n    state: State<'_, AppState>,\n    limit: u32,\n) -> Result<Vec<football_domain::BackgroundJob>, String>",
  "cancel_background_job": "pub async fn cancel_background_job(\n    state: State<'_, AppState>,\n    job_id: String,\n) -> Result<football_domain::BackgroundJob, String>",
  "retry_background_job": "pub async fn retry_background_job(\n    state: State<'_, AppState>,\n    job_id: String,\n) -> Result<football_domain::BackgroundJob, String>",
  "export_ai_analysis_package": "pub async fn export_ai_analysis_package(\n    state: State<'_, AppState>,\n    output_path: String,\n) -> Result<football_domain::AiAnalysisPackageSummary, String>",
  "preview_ai_analysis_response": "pub fn preview_ai_analysis_response(\n    state: State<'_, AppState>,\n    input_path: String,\n) -> Result<football_domain::AiAnalysisResponsePreview, String>",
  "import_ai_analysis_response": "pub async fn import_ai_analysis_response(\n    state: State<'_, AppState>,\n    input_path: String,\n) -> Result<Vec<football_domain::AiAnalysisSuggestionRecord>, String>",
  "list_ai_analysis_suggestions": "pub async fn list_ai_analysis_suggestions(\n    state: State<'_, AppState>,\n    status: Option<String>,\n    limit: u32,\n) -> Result<Vec<football_domain::AiAnalysisSuggestionRecord>, String>",
  "decide_ai_analysis_suggestion": "pub async fn decide_ai_analysis_suggestion(\n    state: State<'_, AppState>,\n    draft: football_domain::AiSuggestionDecisionDraft,\n) -> Result<football_domain::AiAnalysisSuggestionRecord, String>",
  "export_ai_analysis_response_template": "pub fn export_ai_analysis_response_template(\n    state: State<'_, AppState>,\n    output_path: String,\n    source_package_id: Option<String>,\n) -> Result<String, String>",
  "generate_parameter_tuning_candidate": "pub async fn generate_parameter_tuning_candidate(\n    state: State<'_, AppState>,\n    draft: football_domain::ParameterTuningDraft,\n) -> Result<football_domain::ParameterTuningCandidateRecord, String>",
  "list_parameter_tuning_candidates": "pub async fn list_parameter_tuning_candidates(\n    state: State<'_, AppState>,\n    limit: u32,\n) -> Result<Vec<football_domain::ParameterTuningCandidateRecord>, String>",
  "decide_parameter_tuning_candidate": "pub async fn decide_parameter_tuning_candidate(\n    state: State<'_, AppState>,\n    draft: football_domain::ParameterTuningDecisionDraft,\n) -> Result<football_domain::ParameterTuningCandidateRecord, String>",
  "parameter_lifecycle_readiness": "pub async fn parameter_lifecycle_readiness(\n    state: State<'_, AppState>,\n    request: football_domain::ParameterLifecycleReadinessRequest,\n) -> Result<football_domain::ParameterLifecycleReadiness, String>",
  "run_parameter_shadow_validation": "pub async fn run_parameter_shadow_validation(\n    state: State<'_, AppState>,\n    request: football_domain::ParameterShadowValidationRequest,\n) -> Result<football_domain::ParameterShadowValidationRecord, String>",
  "list_parameter_shadow_validations": "pub async fn list_parameter_shadow_validations(\n    state: State<'_, AppState>,\n    candidate_id: String,\n) -> Result<Vec<football_domain::ParameterShadowValidationRecord>, String>",
  "promote_parameter_candidate": "pub async fn promote_parameter_candidate(\n    state: State<'_, AppState>,\n    request: football_domain::ParameterPromotionRequest,\n) -> Result<football_domain::ParameterPromotionDecisionRecord, String>",
  "rollback_parameter_candidate": "pub async fn rollback_parameter_candidate(\n    state: State<'_, AppState>,\n    request: football_domain::ParameterRollbackRequest,\n) -> Result<football_domain::ParameterPromotionDecisionRecord, String>",
  "list_parameter_promotion_decisions": "pub async fn list_parameter_promotion_decisions(\n    state: State<'_, AppState>,\n    candidate_id: String,\n) -> Result<Vec<football_domain::ParameterPromotionDecisionRecord>, String>"
};
  const validations = {
  "analytics_overview": "",
  "decide_data_quality_finding": "",
  "enqueue_analysis_job": "",
  "list_background_jobs": "let jobs =",
  "cancel_background_job": "let job_id = parse_uuid(&job_id, \"任务 ID\")?;",
  "retry_background_job": "let job_id = parse_uuid(&job_id, \"任务 ID\")?;",
  "export_ai_analysis_package": "",
  "preview_ai_analysis_response": "",
  "import_ai_analysis_response": "",
  "list_ai_analysis_suggestions": "",
  "decide_ai_analysis_suggestion": "",
  "export_ai_analysis_response_template": "let source_package_id = source_package_id\n        .map(|value| parse_uuid(&value, \"来源分析包 ID\"))\n        .transpose()?;",
  "generate_parameter_tuning_candidate": "",
  "list_parameter_tuning_candidates": "",
  "decide_parameter_tuning_candidate": "",
  "parameter_lifecycle_readiness": "",
  "run_parameter_shadow_validation": "",
  "list_parameter_shadow_validations": "",
  "promote_parameter_candidate": "",
  "rollback_parameter_candidate": "",
  "list_parameter_promotion_decisions": ""
};
  const calls = {
  "analytics_overview": "analytics_overview()",
  "decide_data_quality_finding": "decide_data_quality_finding(draft)",
  "enqueue_analysis_job": "enqueue_analysis_job(draft)",
  "list_background_jobs": "list_background_jobs(limit)",
  "cancel_background_job": "cancel_background_job(job_id)",
  "retry_background_job": "retry_background_job(job_id)",
  "export_ai_analysis_package": "export_ai_analysis_package(output_path)",
  "preview_ai_analysis_response": "preview_ai_analysis_response(input_path)",
  "import_ai_analysis_response": "import_ai_analysis_response(input_path)",
  "list_ai_analysis_suggestions": "list_ai_analysis_suggestions(status, limit)",
  "decide_ai_analysis_suggestion": "decide_ai_analysis_suggestion(draft)",
  "export_ai_analysis_response_template": "export_ai_analysis_response_template(output_path, source_package_id)",
  "generate_parameter_tuning_candidate": "generate_parameter_tuning_candidate(draft)",
  "list_parameter_tuning_candidates": "list_parameter_tuning_candidates(limit)",
  "decide_parameter_tuning_candidate": "decide_parameter_tuning_candidate(draft)",
  "parameter_lifecycle_readiness": "parameter_lifecycle_readiness(request)",
  "run_parameter_shadow_validation": "run_parameter_shadow_validation(request)",
  "list_parameter_shadow_validations": "list_parameter_shadow_validations(parse_uuid(&candidate_id, \"候选 ID\")?)",
  "promote_parameter_candidate": "promote_parameter_candidate(request)",
  "rollback_parameter_candidate": "rollback_parameter_candidate(request)",
  "list_parameter_promotion_decisions": "list_parameter_promotion_decisions(parse_uuid(&candidate_id, \"候选 ID\")?)"
};
  const isAsync = {
  "analytics_overview": true,
  "decide_data_quality_finding": true,
  "enqueue_analysis_job": true,
  "list_background_jobs": true,
  "cancel_background_job": true,
  "retry_background_job": true,
  "export_ai_analysis_package": true,
  "preview_ai_analysis_response": false,
  "import_ai_analysis_response": true,
  "list_ai_analysis_suggestions": true,
  "decide_ai_analysis_suggestion": true,
  "export_ai_analysis_response_template": false,
  "generate_parameter_tuning_candidate": true,
  "list_parameter_tuning_candidates": true,
  "decide_parameter_tuning_candidate": true,
  "parameter_lifecycle_readiness": true,
  "run_parameter_shadow_validation": true,
  "list_parameter_shadow_validations": true,
  "promote_parameter_candidate": true,
  "rollback_parameter_candidate": true,
  "list_parameter_promotion_decisions": true
};
  const contract = JSON.parse(read("architecture/command-contract.json"));
  const boundaries = JSON.parse(read("architecture/module-boundaries.json")).tauri_commands;
  const tokens = source => (source.match(/"(?:\\.|[^"\\])*"|[A-Za-z_][A-Za-z_0-9]*|[^\s]/g) ?? []).join("\0");
  const signatureTokens = source => tokens(source.replace(/,\s*\)/g, ")"));
  const moduleTokens = source => tokens(source.replace(/,\s*}/g, "}"));
  const exports = (module, names) => `pub use ${module}::{${names.flatMap(name => [`__cmd__${name}`, `__tauri_command_name_${name}`, name]).sort().join(",")}};`;
  const commandModules = read("src-tauri/src/commands.rs");
  for (const [domain, groups] of Object.entries(domains)) {
    const commandRoot = `src-tauri/src/commands/${domain}`;
    const group = contract.command_groups.find(item => item.module === domain);
    const rootModules = Object.fromEntries([...groups[""].map(name => [name, [name]]), ...Object.entries(groups).filter(([name]) => name !== "")]);
    const rootNames = Object.keys(rootModules).sort();
    check(moduleTokens(read(`${commandRoot}/mod.rs`)) === moduleTokens([...rootNames.map(name => `mod ${name};`), ...rootNames.map(name => exports(name, rootModules[name]))].join("\n")), `R11-07 根出口必须只显式登记原命令与双宏：${domain}`);
    for (const [name, commands] of Object.entries(groups)) {
      if (!name) continue;
      const ordered = [...commands].sort();
      check(moduleTokens(read(`${commandRoot}/${name}/mod.rs`)) === moduleTokens([...ordered.map(command => `mod ${command};`), ...ordered.map(command => exports(command, [command]))].join("\n")), `R11-07 子出口夹带逻辑或缺失命令/宏：${domain}/${name}`);
    }
    check(group?.definition_file === `${commandRoot}/mod.rs` && group?.parameter_and_return_signature_file === `${commandRoot}/mod.rs` && boundaries.groups[domain] === `${commandRoot}/mod.rs`, `R11-07 正式清单必须接通唯一目录：${domain}`);
    check(group?.commands.length === Object.values(groups).flat().length, `R11-07 原命令数量漂移：${domain}`);
    check(commandModules.includes(`mod ${domain};`) && commandModules.includes(`pub use ${domain}::*;`), `R11-07 原根登记丢失：${domain}`);
    check(!fs.existsSync(path.join(root, `${commandRoot}.rs`)), `R11-07 旧混合owner仍存在：${domain}`);
    for (const name of Object.values(groups).flat()) {
      const file = files[name];
      const source = read(file);
      const signature = source.match(new RegExp(`pub (?:async )?fn ${name}\\([\\s\\S]*?\\{`))?.[0].slice(0, -1) ?? "";
      check(signatureTokens(signature) === signatureTokens(signatures[name]), `R11-07 原参数/返回/async签名漂移：${name}`);
      check(group?.command_definition_files?.[name] === file, `R11-07 定义owner漂移：${name}`);
      check((source.match(/#\[tauri::command\]/g) ?? []).length === 1 && source.includes("bootstrap::AppState"), `R11-07 必须借用原State且每文件唯一命令：${name}`);
      const body = source.slice(source.indexOf("{", source.indexOf(`fn ${name}`)) + 1, source.lastIndexOf("}"));
      const expected = name === "list_background_jobs"
        ? "let jobs = state.service.list_background_jobs(limit).await.map_err(|error| error.to_string())?; record_failed_background_jobs(&state.issue_log, &jobs); Ok(jobs)"
        : `${validations[name]} state.service.${calls[name]}${isAsync[name] ? ".await" : ""}.map_err(|error| error.to_string())`;
      check(tokens(body) === tokens(expected), `R11-07 薄传输必须保留原UUID/参数/顺序/错误/日志投影：${name}`);
      if (expected.includes("parse_uuid")) check(source.includes("commands::parse_uuid"), `R11-07 UUID校验必须沿原唯一helper：${name}`);
      if (name === "list_background_jobs") check(source.includes("use crate::commands::record_failed_background_jobs;"), "R11-07 后台任务必须沿原共享失败投影owner，不能新增日志实现或丢失问题投影");
      for (const forbidden of ["sqlx::", "football_persistence_postgres", "PostgresStore", "PgPool", "std::fs", "std::io", "tokio::spawn", "ApplicationService::new", "Mutex<", "static ", "macro_rules!", ".unwrap()", ".expect("]) check(!source.includes(forbidden), `R11-07 命令越过原业务/状态/文件/worker边界：${name}/${forbidden}`);
    }
  }
}
// R11-07 transport contract END

if (failures.length) {
  throw new Error(`Analytics Service 验证失败\n${failures.map((item) => `- ${item}`).join("\n")}`);
}

console.log("Analytics Service 验证通过：旧 owner 已删除，21 个公共 Use Case 入口保持兼容，Calculation 及原事务拥有唯一 owner，Jobs启动/领取结算/执行与共享PG队列各唯一，AI Package/Parameter Lifecycle 支持模块边界明确，三个 Port 分属现有组合适配 owner且公共门面保持兼容。");
