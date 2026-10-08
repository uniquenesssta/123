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
  "resolve_p4_conflict",
];
const required = [
  "crates/application/src/services/research/mod.rs",
  "crates/application/src/services/research/service.rs",
  "crates/application/src/services/research/facade.rs",
  "crates/application/src/use_cases/research/mod.rs",
  "crates/application/src/use_cases/research/artifact_catalog.rs",
  "crates/application/src/use_cases/research/ledger.rs",
  "crates/application/src/use_cases/research/fact_pipeline/mod.rs",
  "crates/application/src/use_cases/research/fact_pipeline/command.rs",
  "crates/application/src/use_cases/research/fact_pipeline/prepare.rs",
  "crates/application/src/use_cases/research/fact_pipeline/process.rs",
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
  "crates/application/src/use_cases/research/p4_manual_conflict/mod.rs",
  "crates/application/src/use_cases/research/p4_manual_conflict/decision.rs",
  "crates/application/src/use_cases/research/p4_manual_conflict/reconciliation.rs",
  "crates/application/src/composition/adapters/research.rs",
  "crates/application/src/ports/research/mod.rs",
];
const postgresPipelineRoot = "crates/persistence-postgres/src/adapters/p4/fact_pipeline";
const postgresPipelineOwners = {
  source_policy: ["register_source_policy_version"],
  context: ["fact_pipeline_context"],
  candidates: ["find_entity_candidates"],
  entity_resolution: ["append_entity_resolution"],
  time_audit: ["append_time_audit"],
  conflicts: ["append_conflict_evaluation", "append_conflict_event"],
  routes: ["append_evidence_route"],
};
required.push(...["mod", "fingerprint", ...Object.keys(postgresPipelineOwners)].map((owner) => `${postgresPipelineRoot}/${owner}.rs`));
for (const path of required) check(existsSync(join(root, path)), `缺少 Research / Fact Pipeline 职责文件：${path}`);
check(!existsSync(join(root, "crates/application/src/p4_persistence.rs")), "旧 p4_persistence.rs 仍残留 Research owner");
check(!existsSync(join(root, "crates/application/src/fact_pipeline.rs")), "旧 fact_pipeline.rs 仍残留 Fact Pipeline owner 或空转发层");
check(!existsSync(join(root, "crates/application/src/openai_research.rs")), "旧 openai_research.rs 仍残留 OpenAI Research owner 或空转发层");
check(!existsSync(join(root, "crates/application/src/p4_workbench.rs")), "旧 p4_workbench.rs 仍残留人工冲突裁决 owner 或空转发层");

const facade = read("crates/application/src/services/research/facade.rs");
const service = read("crates/application/src/services/research/service.rs");
const ports = read("crates/application/src/ports/research/mod.rs");
const adapter = read("crates/application/src/composition/adapters/research.rs");
const databaseFacade = read("crates/application/src/services/database/facade.rs");
const lifecycleInitialize = read("crates/application/src/use_cases/application_facade/database_lifecycle/initialize.rs");
const openaiExecution = read("crates/application/src/use_cases/research/openai_gateway/execution.rs");
const openaiArtifacts = read("crates/application/src/use_cases/research/openai_gateway/artifacts.rs");
const openaiAudit = read("crates/application/src/use_cases/research/openai_gateway/attempt_audit.rs");
const pipelineRoot = "crates/application/src/use_cases/research/fact_pipeline";
const pipeline = read(`${pipelineRoot}/mod.rs`);
const pipelineProcess = read(`${pipelineRoot}/process.rs`);
const pipelinePrepare = read(`${pipelineRoot}/prepare.rs`);
const p4Worker = read("crates/application/src/use_cases/research/p4_worker/execution.rs");
const p4Transitions = read("crates/application/src/use_cases/research/p4_worker/transitions.rs");
const p4Manual = read("crates/application/src/use_cases/research/p4_manual_conflict/mod.rs");
const p4Decision = read("crates/application/src/use_cases/research/p4_manual_conflict/decision.rs");
const p4Reconciliation = read("crates/application/src/use_cases/research/p4_manual_conflict/reconciliation.rs");
const p4Orchestration = read("crates/application/src/use_cases/p4_orchestration/process_next.rs");
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
check(adapter.includes("impl ResearchEvidenceLedgerPort for PersistenceStore"), "Research adapter 缺少 Evidence Ledger 实现");
check(adapter.includes("impl FactPipelinePort for PersistenceStore"), "Research adapter 缺少 Fact Pipeline 实现");
for (const token of [".fact_pipeline_context(research_run_id)", ".find_entity_candidates(", ".append_entity_resolution(draft)", ".append_time_audit(draft)", ".append_conflict_evaluation(draft)", ".append_conflict_event(", ".append_evidence_route(draft)"]) {
  check(adapter.includes(token), `Fact Pipeline adapter 未复用既有持久化能力：${token}`);
}
check(lifecycleInitialize.includes(".register_persistence_artifacts(session)"), "数据库初始化未通过 ResearchService 注册内置 Research schema");
check(openaiArtifacts.includes("fact_pipeline::register_fact_pipeline_artifacts(port).await?"), "OpenAI artifact 初始化未继续注册 Fact Pipeline 来源策略");
check(service.includes("fn register_openai_research_artifacts"), "ResearchService 缺少 OpenAI artifact 初始化职责");
check(lifecycleInitialize.replaceAll(/\s/g, "").includes("application.research.register_openai_research_artifacts(session).await"), "数据库初始化未通过 ResearchService 注册 OpenAI Research artifacts");
check(ports.includes("trait ResearchGatewayAuditPort"), "Research Ports 缺少 ResearchGatewayAuditPort");
check(ports.includes("trait ResearchManualConflictPort"), "Research Ports 缺少 ResearchManualConflictPort");
for (const capability of ["append_manual_route_override", "route_readiness"]) {
  check(ports.includes(`fn ${capability}`), `ResearchManualConflictPort 缺少能力：${capability}`);
}
for (const capability of ["append_attempt", "attempt_number_offset", "usage_totals", "append_web_references"]) {
  check(ports.includes(`fn ${capability}`), `ResearchGatewayAuditPort 缺少能力：${capability}`);
}
check(adapter.includes("impl ResearchGatewayAuditPort for PersistenceStore"), "Research adapter 缺少 Gateway Audit 实现");
check(adapter.includes("impl ResearchManualConflictPort for PersistenceStore"), "Research adapter 缺少 Manual Conflict 实现");
for (const token of [".append_p4_manual_route_override(draft)", ".p4_route_readiness(task_id)"]) {
  check(adapter.includes(token), `Manual Conflict adapter 未复用既有持久化能力：${token}`);
}
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
check(pipelineProcess.includes("trait FactPipelineAccess"), "Fact Pipeline 缺少 Ports 组合访问边界");
check(!existsSync(join(root, "crates/application/src/p4_orchestration.rs")), "旧跨域 P4 orchestration owner 仍残留");
check(!lib.includes("mod p4_workbench;"), "Application 根模块仍登记旧 p4_workbench owner");
check(service.includes("fn execute_p4_research_task"), "ResearchService 缺少 P4 Research worker 执行职责");
check(!facade.includes("fn execute_p4_research_task"), "ApplicationService Research facade 仍保留仅供 orchestration 的私有 worker helper");
check(service.includes("fn resolve_p4_conflict"), "ResearchService 缺少人工冲突裁决职责");
check(facade.includes("pub async fn resolve_p4_conflict"), "Application Research facade 缺少公共人工冲突裁决入口");
check(facade.includes("ApplicationResult<P4TaskWorkspace>"), "resolve_p4_conflict 返回契约发生变化");
check(!service.includes("fn finalize_successful_research"), "AT5 后仍残留仅供旧 workbench 的 ResearchService finalizer 转发");
check(!facade.includes("fn finalize_p4_research_task"), "AT5 后仍残留仅供旧 workbench 的 facade finalizer 转发");
check(p4Transitions.includes("fn finalize_successful_research"), "P4 Research worker 丢失可复用的成功收口职责");
check(p4Worker.includes("openai_gateway::execute_p4_openai_research"), "P4 Research worker 未复用已迁移 OpenAI Gateway");
check(p4Worker.includes("PredictionWorkflowPort"), "P4 Research worker 未通过 PredictionWorkflowPort 管理冻结任务状态");
check(p4Worker.includes("ResearchArtifactPort"), "P4 Research worker 未通过 ResearchArtifactPort 管理 research run");
check(p4Transitions.includes("JobQueuePort"), "P4 Research worker 未通过 JobQueuePort 安排 freeze job");
check(p4Orchestration.includes(".execute_p4_research_task("), "P4 dispatcher 未委托 ResearchService 执行 Research job");
for (const token of ["OpenAiResearchCommand", "ResearchRunDraft", "ResearchRunStatus", "research_dynamic_context", "fn finalize_successful_research", "fn block_partial_research", "fn transition_missed", "execute_p4_openai_research("]) {
  check(!p4Orchestration.includes(token), `P4 orchestration use case 仍持有 Research worker 业务逻辑：${token}`);
}
check(p4Manual.includes("P4ManualConflictAccess"), "人工冲突裁决缺少 Ports 组合访问边界");
for (const token of ["PredictionWorkflowPort", "JobQueuePort", "ResearchArtifactPort", "ResearchManualConflictPort"]) {
  check(p4Manual.includes(token), `人工冲突裁决访问边界缺少：${token}`);
}
check(p4Decision.includes("Utc"), "人工冲突裁决未保留截止时间判断");
check(p4Decision.includes("p4-manual-conflict:{}:{}:{}"), "人工冲突裁决幂等键格式发生变化");
check(p4Decision.includes('actor: "local_user"'), "人工冲突裁决 actor 语义发生变化");
check(p4Decision.includes('"PROBABLE"') && p4Decision.includes('"NOT_FOUND"'), "人工冲突裁决 verification 语义发生变化");
check(p4Reconciliation.includes("for _ in 0..4"), "人工冲突裁决并发恢复重试次数发生变化");
check(p4Reconciliation.includes("manual-review-succeeded:{task_id}"), "人工裁决 Research run event 幂等键发生变化");
check(p4Reconciliation.includes("p4_worker::finalize_successful_research"), "人工冲突裁决未复用 AT4 Research 成功收口");

const pipelineFiles = rustFiles("crates/application/src/use_cases/research/fact_pipeline");
// R8-09 checks ownership and call order rather than accepting a file-count split.
function inOrder(source, tokens, label) {
  let previous = -1;
  for (const token of tokens) {
    const index = source.indexOf(token, previous + 1);
    check(index > previous, `${label} 缺少或改变步骤顺序：${token}`);
    if (index >= 0) previous = index;
  }
}
for (const [label, source] of [["Application Fact Pipeline", pipeline], ["Postgres Fact Pipeline", read(`${postgresPipelineRoot}/mod.rs`)]]) {
  check(!/\b(?:async\s+)?fn\s|\bstruct\s|\btrait\s|\bimpl\b|use\s+super::\*/.test(source), `${label} 目录出口持有业务实现或通配依赖`);
}
check(pipeline.includes("pub use command::ProcessResearchEvidenceCommand;") && pipeline.includes("pub(crate) use process::{process_p4_research_evidence, FactPipelineAccess};"), "Fact Pipeline 公共命令或内部入口出口发生漂移");
check(pipelineProcess.includes("FactPipelinePort + ResearchEvidenceLedgerPort"), "Fact Pipeline 编排绕过组合 Ports");
check(pipelineProcess.includes("BTreeMap<String, Vec<PreparedFact>>") && pipelineProcess.includes("groups.into_values()"), "事实分组不再保持原排序");
inOrder(pipelineProcess, ["validate_pipeline_command(&command)?", "port.context(command.research_run_id)", "validate_pipeline_context(&command, &context)?", "build_source_index(&command, &policy)?", "for fact in command.output.facts", "prepare_fact(", "for prepared_group in groups.into_values()", "process_fact_group(", "for missing in command.output.missing_fields", "process_missing_field("], "Fact Pipeline 编排");
inOrder(pipelinePrepare, [".find_entity_candidates(", "candidates.retain(", "decide_entity_resolution(", ".append_entity_resolution(", "audit_fact_time(", ".append_time_audit(", ".source_urls", "let group_key"], "事实输入准备");
check(pipelinePrepare.includes("candidate.relation.as_deref() == Some(side)"), "事实输入准备丢失主客队约束");
check(pipelinePrepare.includes('format!("entity:{}:{}", context.research_run_id, fact.fact_key)') && pipelinePrepare.includes('format!("time:{}:{}", context.research_run_id, fact.fact_key)'), "实体或时间幂等键漂移");
const pipelineValidation = read(`${pipelineRoot}/validation.rs`);
for (const identity of ["match_key", "data_cutoff_at", "schema_version"]) check(pipelineValidation.includes(`command.output.${identity} != context.${identity}`), `Fact Pipeline 上下文校验丢失 ${identity}`);
const pipelineTime = read(`${pipelineRoot}/time_audit.rs`);
inOrder(pipelineTime, ["retrieved_at > cutoff", '"NOT_FOUND"', "timezone", "RejectedMissingEvidenceTime", "RejectedFuture", "RejectedInvalidOrder"], "事实时间闸门");
const postgresSources = Object.fromEntries(Object.keys(postgresPipelineOwners).map((owner) => [owner, read(`${postgresPipelineRoot}/${owner}.rs`)]));
const allPostgresSources = Object.values(postgresSources).join("\n");
for (const [owner, methods] of Object.entries(postgresPipelineOwners)) {
  check(read(`${postgresPipelineRoot}/mod.rs`).includes(`mod ${owner};`), `Postgres Fact Pipeline 未登记 ${owner}`);
  for (const method of methods) {
    check(postgresSources[owner].includes(`pub async fn ${method}(`), `${method} 未由 ${owner} 唯一持有`);
    check((allPostgresSources.match(new RegExp(`pub async fn ${method}\\(`, "g")) ?? []).length === 1, `${method} 有重复持久化实现`);
  }
}
check(!existsSync(join(root, "crates/persistence-postgres/src/fact_pipeline_records.rs")), "旧 fact_pipeline_records owner 仍残留");
check(!read("crates/persistence-postgres/src/lib.rs").includes("mod fact_pipeline_records;"), "Postgres 根仍登记旧 Fact Pipeline");
check(read("crates/persistence-postgres/src/adapters/p4/mod.rs").includes("pub(crate) mod fact_pipeline;"), "Fact Pipeline 未进入既有 P4 adapter");
for (const owner of ["entity_resolution", "time_audit", "conflicts", "routes"]) {
  check(postgresSources[owner].includes("ON CONFLICT") && postgresSources[owner].includes("ensure_fingerprint("), `${owner} 丢失幂等写入或载荷冲突校验`);
}
const postgresPolicy = postgresSources.source_policy;
inOrder(postgresPolicy, ["validate_source_policy_definition(draft)?", "self.pool.begin()", "pg_advisory_xact_lock", "SELECT id, policy_key", "INSERT INTO research.source_policy_versions", "write_audit_event("], "来源策略事务");
check(postgresPolicy.lastIndexOf("tx.commit().await?") > postgresPolicy.indexOf("write_audit_event("), "来源策略审计未在同一事务提交");
check(postgresSources.candidates.includes("context.data_cutoff_at.date_naive()") && postgresSources.candidates.includes("existing.score >= candidate.score"), "实体候选丢失截止日期或等分保留规则");
inOrder(postgresSources.routes, ["evidence_ids.sort_unstable()", "evidence_ids.dedup()", "let fingerprint = sha256_json", '.bind(&evidence_ids)'], "证据路由指纹与落库");
check(read(`${postgresPipelineRoot}/fingerprint.rs`).includes("existing == expected"), "Fact Pipeline 幂等冲突放宽了精确指纹比较");
for (const path of [...pipelineFiles, ...rustFiles(postgresPipelineRoot)]) check(!read(path).includes("use super::*"), `${path} 仍存在跨职责通配导入`);
const pipelineTests = read(`${pipelineRoot}/tests.rs`);
for (const test of ["invalid_command_and_context_stop_before_ledger_writes", "pipeline_preserves_context_source_and_repeatable_idempotency_keys", "opposite_team_candidate_never_enters_home_route", "missing_source_preserves_prior_resolution_and_time_audit_only", "port_errors_keep_kind_message_and_stop_at_the_failed_step", "missing_field_retrieved_after_cutoff_is_stale_and_blocked", "official_conflict_resolution_writes_evaluation_event_then_selected_route"]) check(pipelineTests.includes(`async fn ${test}(`), `现有测试入口缺少事实流水线边界：${test}`);
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
console.log(`Research Service / R8-09 Fact Pipeline 验证通过：${researchFiles.length} 个 Service/Use Case Rust 文件；10 个公开 Research API 保持兼容，Fact Pipeline、OpenAI Gateway、P4 Research worker 与人工冲突裁决均进入 ResearchService/Ports，根 p4_orchestration.rs 仅保留跨服务 dispatcher/worker loop，旧 p4_workbench.rs 已删除。`);
