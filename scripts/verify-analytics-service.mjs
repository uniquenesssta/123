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
  `${useCaseRoot}/analytics_overview/mod.rs`,
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
check(analyticsAdapter.includes("impl AnalyticsPort for ActiveDatabase"), "Analytics adapter 缺少 AnalyticsPort");
check(analyticsAdapter.includes("impl ParameterLifecyclePort for ActiveDatabase"), "Analytics adapter 缺少 ParameterLifecyclePort");
check(!analyticsAdapter.includes("impl JobQueuePort for ActiveDatabase"), "Analytics adapter 重复拥有 JobQueuePort");
check(jobsAdapter.includes("impl JobQueuePort for ActiveDatabase"), "既有 jobs adapter 未实现 JobQueuePort");
check(!ports.includes("serde_json::Value"), "Analytics Port 暴露通用 JSON Value");
check(contracts.includes("pub struct ParameterDefinition"), "缺少参数定义专用契约");
check(contracts.includes("pub enum AnalyticsJobProgressPayload"), "缺少后台任务进度专用契约");
check(contracts.includes("pub enum AnalyticsJobResult"), "缺少后台任务结果专用契约");

const serviceAndUseCases = [
  service,
  worker,
  parameterMetrics,
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
check(publicUseCaseFiles.length === 21, `Analytics 公共 Use Case 应为 21 个独立目录，实际 ${publicUseCaseFiles.length}`);

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

console.log("Analytics Service 验证通过：旧 owner 已删除，21 个公共 Use Case 均为独立目录，Jobs/AI Package/Parameter Lifecycle 支持模块边界明确，三个 Port 分属现有组合适配 owner且公共门面保持兼容。");
