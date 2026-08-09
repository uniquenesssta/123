import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8").replaceAll("\r\n", "\n");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
function rustFiles(path) {
  const absolute = join(root, path);
  const result = [];
  for (const entry of readdirSync(absolute, { withFileTypes: true })) {
    const child = join(absolute, entry.name);
    if (entry.isDirectory()) result.push(...rustFiles(relative(root, child)));
    else if (entry.name.endsWith(".rs")) result.push(relative(root, child).replaceAll("\\", "/"));
  }
  return result;
}

const publicMethods = [
  "register_p4_schema_version",
  "register_p4_prompt_version",
  "register_p4_competition_profile_version",
  "create_p4_research_run",
  "record_p4_research_run_event",
  "append_p4_evidence_claim",
  "create_p4_evidence_conflict",
  "process_p4_research_evidence",
  "execute_p4_openai_research",
];
const required = [
  "crates/application/src/services/research/mod.rs",
  "crates/application/src/services/research/service.rs",
  "crates/application/src/services/research/facade.rs",
  "crates/application/src/use_cases/research/mod.rs",
  "crates/application/src/use_cases/research/artifact_catalog.rs",
  "crates/application/src/use_cases/research/ledger.rs",
  "crates/application/src/use_cases/research/fact_pipeline/mod.rs",
  "crates/application/src/use_cases/research/fact_pipeline/entity_resolution.rs",
  "crates/application/src/use_cases/research/fact_pipeline/time_audit.rs",
  "crates/application/src/use_cases/research/fact_pipeline/source_policy.rs",
  "crates/application/src/use_cases/research/fact_pipeline/evidence.rs",
  "crates/application/src/use_cases/research/fact_pipeline/conflict.rs",
  "crates/application/src/use_cases/research/fact_pipeline/routing.rs",
  "crates/application/src/use_cases/research/fact_pipeline/validation.rs",
  "crates/application/src/use_cases/research/fact_pipeline/types.rs",
  "crates/application/src/use_cases/research/openai_gateway/mod.rs",
  "crates/application/src/use_cases/research/openai_gateway/artifacts.rs",
  "crates/application/src/use_cases/research/openai_gateway/attempt_audit.rs",
  "crates/application/src/use_cases/research/openai_gateway/execution.rs",
  "crates/application/src/use_cases/research/openai_gateway/gateway.rs",
  "crates/application/src/use_cases/research/openai_gateway/references.rs",
  "crates/application/src/use_cases/research/openai_gateway/types.rs",
  "crates/application/src/use_cases/research/openai_gateway/validation.rs",
  "crates/application/src/use_cases/research/p4_worker/mod.rs",
  "crates/application/src/use_cases/research/p4_worker/context.rs",
  "crates/application/src/use_cases/research/p4_worker/execution.rs",
  "crates/application/src/use_cases/research/p4_worker/transitions.rs",
  "crates/application/src/composition/adapters/research.rs",
  "crates/application/src/ports/research/mod.rs",
];
for (const path of required) check(existsSync(join(root, path)), `缺少 R3-07 文件：${path}`);
check(!existsSync(join(root, "crates/application/src/p4_persistence.rs")), "旧 p4_persistence.rs 仍残留 Research owner");
check(!existsSync(join(root, "crates/application/src/fact_pipeline.rs")), "旧 fact_pipeline.rs 仍残留 Fact Pipeline owner 或空转发层");
check(!existsSync(join(root, "crates/application/src/openai_research.rs")), "旧 openai_research.rs 仍残留 OpenAI Research owner 或空转发层");

const facade = read("crates/application/src/services/research/facade.rs");
const service = read("crates/application/src/services/research/service.rs");
const ports = read("crates/application/src/ports/research/mod.rs");
const adapter = read("crates/application/src/composition/adapters/research.rs");
const databaseFacade = read("crates/application/src/services/database/facade.rs");
const openaiExecution = read("crates/application/src/use_cases/research/openai_gateway/execution.rs");
const openaiArtifacts = read("crates/application/src/use_cases/research/openai_gateway/artifacts.rs");
const openaiAudit = read("crates/application/src/use_cases/research/openai_gateway/attempt_audit.rs");
const pipeline = read("crates/application/src/use_cases/research/fact_pipeline/mod.rs");
const p4Worker = read("crates/application/src/use_cases/research/p4_worker/execution.rs");
const p4Transitions = read("crates/application/src/use_cases/research/p4_worker/transitions.rs");
const p4Orchestration = read("crates/application/src/p4_orchestration.rs");
const p4Workbench = read("crates/application/src/p4_workbench.rs");
const lib = read("crates/application/src/lib.rs");
const packageJson = JSON.parse(read("package.json"));
const frontend = read("scripts/verify-frontend.mjs");

for (const method of publicMethods) {
  check(facade.includes(`fn ${method}`), `Research facade 缺少公共兼容方法：${method}`);
  check(service.includes(`fn ${method}`), `ResearchService 缺少职责：${method}`);
}
check(ports.includes("trait ResearchArtifactPort"), "Research Ports 缺少 ResearchArtifactPort");
check(ports.includes("register_competition_profile"), "ResearchArtifactPort 缺少赛事配置版本写入能力");
check(ports.includes("PortResult<ResearchRunRecord>"), "Research run event 未保留公开返回记录契约");
check(ports.includes("trait ResearchEvidenceLedgerPort"), "Research Ports 缺少 ResearchEvidenceLedgerPort");
check(ports.includes("trait FactPipelinePort"), "Research Ports 缺少 FactPipelinePort");
check(ports.includes("struct SerializedConflictEventPayload"), "FactPipelinePort 缺少类型化冲突事件载荷边界");
check(!ports.includes("serde_json::Value"), "Research Ports 泄漏裸 serde_json::Value");
for (const capability of ["find_entity_candidates", "append_entity_resolution", "append_time_audit", "append_conflict_evaluation", "append_conflict_event", "append_evidence_route"]) {
  check(ports.includes(`fn ${capability}`), `FactPipelinePort 缺少能力：${capability}`);
}
check(adapter.includes("impl ResearchEvidenceLedgerPort for ActiveDatabase"), "Research adapter 缺少 Evidence Ledger 实现");
check(adapter.includes("impl FactPipelinePort for ActiveDatabase"), "Research adapter 缺少 Fact Pipeline 实现");
for (const token of [".fact_pipeline_context(research_run_id)", ".find_entity_candidates(", ".append_entity_resolution(draft)", ".append_time_audit(draft)", ".append_conflict_evaluation(draft)", ".append_conflict_event(", ".append_evidence_route(draft)"]) {
  check(adapter.includes(token), `Fact Pipeline adapter 未复用既有持久化能力：${token}`);
}
check(databaseFacade.includes(".register_persistence_artifacts(prepared.session())"), "数据库初始化未通过 ResearchService 注册内置 Research schema");
check(openaiArtifacts.includes("fact_pipeline::register_fact_pipeline_artifacts(port).await?"), "OpenAI artifact 初始化未继续注册 Fact Pipeline 来源策略");
check(service.includes("fn register_openai_research_artifacts"), "ResearchService 缺少 OpenAI artifact 初始化职责");
check(databaseFacade.replaceAll(/\s/g, "").includes("self.research.register_openai_research_artifacts(prepared.session()).await"), "数据库初始化未通过 ResearchService 注册 OpenAI Research artifacts");
check(ports.includes("trait ResearchGatewayAuditPort"), "Research Ports 缺少 ResearchGatewayAuditPort");
for (const capability of ["append_attempt", "attempt_number_offset", "usage_totals", "append_web_references"]) {
  check(ports.includes(`fn ${capability}`), `ResearchGatewayAuditPort 缺少能力：${capability}`);
}
check(adapter.includes("impl ResearchGatewayAuditPort for ActiveDatabase"), "Research adapter 缺少 Gateway Audit 实现");
for (const token of [".append_openai_attempt(draft)", ".openai_attempt_number_offset(research_run_id)", ".openai_usage_totals()", ".append_web_references(citations, sources)"]) {
  check(adapter.includes(token), `Research Gateway adapter 未复用既有持久化能力：${token}`);
}
check(openaiExecution.includes("execute_with_sink"), "OpenAI Research 执行未保留 GatewayAttemptSink 审计链");
check(openaiExecution.includes("fact_pipeline::process_p4_research_evidence"), "OpenAI Research 执行未衔接 Fact Pipeline");
check(openaiAudit.includes("impl GatewayAttemptSink for PortAttemptSink"), "OpenAI attempt audit 未通过 Port-backed sink");
check(!lib.includes("mod p4_persistence;"), "Application 根模块仍登记旧 p4_persistence owner");
check(!lib.includes("mod fact_pipeline;"), "Application 根模块仍登记旧 fact_pipeline owner");
check(lib.includes("use_cases::research::fact_pipeline::ProcessResearchEvidenceCommand"), "公共 ProcessResearchEvidenceCommand 未从新 owner 重导出");
check(lib.includes("use_cases::research::openai_gateway::OpenAiResearchCommand"), "公共 OpenAiResearchCommand 未从新 owner 重导出");
check(!lib.includes("mod openai_research;"), "Application 根模块仍登记旧 openai_research owner");
check(pipeline.includes("trait FactPipelineAccess"), "Fact Pipeline 缺少 Ports 组合访问边界");
check(existsSync(join(root, "crates/application/src/p4_orchestration.rs")), "跨 Prediction/Research 的 P4 dispatcher 被提前删除");
check(existsSync(join(root, "crates/application/src/p4_workbench.rs")), "人工冲突裁决被提前删除");
check(service.includes("fn execute_p4_research_task"), "ResearchService 缺少 P4 Research worker 执行职责");
check(service.includes("fn finalize_successful_research"), "ResearchService 缺少 Research 成功收口职责");
check(facade.includes("fn execute_p4_research_task"), "Application Research facade 缺少 P4 Research worker 委托");
check(facade.includes("fn finalize_p4_research_task"), "Application Research facade 缺少人工裁决复用的 Research 收口入口");
check(p4Worker.includes("openai_gateway::execute_p4_openai_research"), "P4 Research worker 未复用已迁移 OpenAI Gateway");
check(p4Worker.includes("PredictionWorkflowPort"), "P4 Research worker 未通过 PredictionWorkflowPort 管理冻结任务状态");
check(p4Worker.includes("ResearchArtifactPort"), "P4 Research worker 未通过 ResearchArtifactPort 管理 research run");
check(p4Transitions.includes("JobQueuePort"), "P4 Research worker 未通过 JobQueuePort 安排 freeze job");
check(p4Orchestration.includes("self.execute_p4_research_task(payload.task_id, job_id)"), "P4 dispatcher 未委托 ResearchService 执行 Research job");
for (const token of ["OpenAiResearchCommand", "ResearchRunDraft", "ResearchRunStatus", "research_dynamic_context", "fn finalize_successful_research", "fn block_partial_research", "fn transition_missed", "execute_p4_openai_research("]) {
  check(!p4Orchestration.includes(token), `旧 p4_orchestration.rs 仍持有 Research worker 业务逻辑：${token}`);
}
check(!p4Workbench.includes("p4_orchestration::finalize_successful_research"), "人工冲突裁决仍依赖旧 P4 orchestration Research helper");
check(p4Workbench.includes("service.finalize_p4_research_task(&recovered).await"), "人工冲突裁决未复用 ResearchService 成功收口职责");

const pipelineFiles = rustFiles("crates/application/src/use_cases/research/fact_pipeline");
check(pipelineFiles.length >= 9, `Fact Pipeline 拆分不足，当前仅 ${pipelineFiles.length} 个职责文件`);
const researchFiles = [
  ...rustFiles("crates/application/src/services/research"),
  ...rustFiles("crates/application/src/use_cases/research"),
];
for (const path of researchFiles) {
  const source = read(path);
  for (const token of ["football_persistence_postgres", "PostgresStore", "sqlx::", "PgPool", "PersistenceStore"]) {
    check(!source.includes(token), `${path} 泄漏具体持久化实现：${token}`);
  }
}
check(packageJson.scripts?.["verify:research-service"] === "node scripts/verify-research-service.mjs", "package.json 未登记 R3-07 专项门禁");
check(packageJson.scripts?.["verify:architecture"]?.includes("verify-research-service.mjs"), "verify:architecture 未接入 R3-07 门禁");
check(frontend.includes('"verify-research-service.mjs"'), "verify:frontend 未接入 R3-07 门禁");

if (failures.length) throw new Error(`Research Service 验证失败\n${failures.map((item) => `- ${item}`).join("\n")}`);
console.log(`Research Service AT4 验证通过：${researchFiles.length} 个 Service/Use Case Rust 文件；9 个公开 Research API 保持兼容，Fact Pipeline、OpenAI Gateway 与 P4 Research worker 均进入 ResearchService/Ports，根 p4_orchestration.rs 仅保留跨服务 dispatcher/worker loop，人工冲突裁决继续留给后续 Atomic Task。`);
