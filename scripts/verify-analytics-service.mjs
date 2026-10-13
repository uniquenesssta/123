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
  `${useCaseRoot}/jobs/enqueue_analysis_job/mod.rs`,
  `${useCaseRoot}/jobs/list_background_jobs/mod.rs`,
  `${useCaseRoot}/jobs/cancel_background_job/mod.rs`,
  `${useCaseRoot}/jobs/retry_background_job/mod.rs`,
  `${useCaseRoot}/ai_package/export_ai_analysis_response_template/mod.rs`,
  `${useCaseRoot}/ai_package/export_ai_analysis_package/mod.rs`,
  `${useCaseRoot}/ai_package/preview_ai_analysis_response/mod.rs`,
  `${useCaseRoot}/ai_package/import_ai_analysis_response/mod.rs`,
  `${useCaseRoot}/ai_package/list_ai_analysis_suggestions/mod.rs`,
  `${useCaseRoot}/ai_package/decide_ai_analysis_suggestion/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/generate_parameter_tuning_candidate/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/list_parameter_tuning_candidates/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/decide_parameter_tuning_candidate/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/parameter_lifecycle_readiness/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/run_parameter_shadow_validation/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/list_parameter_shadow_validations/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/promote_parameter_candidate/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/rollback_parameter_candidate/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/list_parameter_promotion_decisions/mod.rs`,
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
  `${useCaseRoot}/ai_package/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/mod.rs`,
  `${useCaseRoot}/parameter_lifecycle/metrics.rs`,
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
  `${useCaseRoot}/ai_package.rs`,
  `${useCaseRoot}/parameter_lifecycle/candidates.rs`,
  `${useCaseRoot}/parameter_lifecycle/readiness.rs`,
  `${useCaseRoot}/parameter_lifecycle/promotion.rs`,
  `${useCaseRoot}/parameter_lifecycle/shadow.rs`,
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
const parameterMetrics = read(`${useCaseRoot}/parameter_lifecycle/metrics.rs`);
const parameterShadow = read(`${useCaseRoot}/parameter_lifecycle/run_parameter_shadow_validation/mod.rs`);
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
  parameterMetrics,
  read(`${useCaseRoot}/calculation/refresh.rs`),
  ...publicUseCaseFiles.map(read),
].join("\n");
for (const token of ["sqlx::", "PostgresStore", "PersistenceStore", "football_persistence_postgres"]) {
  check(!serviceAndUseCases.includes(token), `Service/Use Case 泄漏基础设施符号：${token}`);
}
check(!service.includes("std::fs") && !service.includes("Path::new"), "AnalyticsService 直接承担文件 I/O");
check(read(`${useCaseRoot}/ai_package/export_ai_analysis_package/mod.rs`).includes("write_analysis_package"), "AI Analysis Package 导出未归属独立 Use Case");
check(read(`${useCaseRoot}/ai_package/preview_ai_analysis_response/mod.rs`).includes("read_analysis_response"), "AI Analysis Package 预检未归属独立 Use Case");
check(worker.includes("claim_next_by_types") && worker.includes("AnalyticsJobResult"), "后台任务 worker 未归属独立协调模块");
check(service.includes("start_job_worker") && service.includes("analytics::jobs::worker::spawn"), "AnalyticsService 未暴露后台 worker 启动边界");
check(parameterMetrics.includes("expected_calibration_error") && parameterMetrics.includes("lifecycle_metrics"), "Parameter Lifecycle 指标计算未独立拆分");
check(parameterShadow.includes("binding_unchanged") && parameterShadow.includes("finite_probabilities"), "Parameter shadow 安全门禁缺失");
check(publicUseCaseFiles.length === 21, `Analytics 公共 Use Case 应保持 21 个唯一入口，实际 ${publicUseCaseFiles.length}`);


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
check(pgLegacy.includes("use chrono::{DateTime, Utc};") && pgLegacy.includes("ANALYTICS_CALCULATION_VERSION,"), "保留 AI 职责所需的时间/版本生产导入缺失");
const compact = (text) => text.replace(/\s+/g, "");
const occurrences = (text, token) => text.split(token).length - 1;
check(read(`${useCaseRoot}/mod.rs`).includes("pub(crate) mod calculation;") &&
  !read(`${useCaseRoot}/mod.rs`).includes("mod analytics_overview;"), "Calculation 未登记唯一 App owner");
check(service.includes("analytics::calculation::analytics_overview(port).await") &&
  !service.includes("analytics::analytics_overview"), "总览实际 Service 未切换 Calculation");
check(occurrences(worker, "refresh_analytics(port, &request).await?") === 2 &&
  !worker.includes("port.refresh("), "两条实际 worker 刷新必须经过 Calculation");
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

check(
  packageDefinition.scripts?.["verify:analytics-service"] === "node scripts/verify-analytics-service.mjs",
  "package.json 未登记 Analytics Service 验证入口",
);
check(
  packageDefinition.scripts?.["verify:architecture"]?.includes("verify-analytics-service.mjs"),
  "Architecture 聚合门禁未接入 Analytics Service",
);
check(frontendVerifier.includes('"verify-analytics-service.mjs"'), "前端聚合验证未接入 Analytics Service");

if (failures.length) {
  throw new Error(`Analytics Service 验证失败\n${failures.map((item) => `- ${item}`).join("\n")}`);
}

console.log("Analytics Service 验证通过：旧 owner 已删除，21 个公共 Use Case 入口保持兼容，Calculation 及原事务拥有唯一 owner，Jobs/AI Package/Parameter Lifecycle 支持模块边界明确，三个 Port 分属现有组合适配 owner且公共门面保持兼容。");
