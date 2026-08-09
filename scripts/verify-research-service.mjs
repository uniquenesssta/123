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
  "crates/application/src/composition/adapters/research.rs",
  "crates/application/src/ports/research/mod.rs",
];
for (const path of required) check(existsSync(join(root, path)), `缺少 R3-07 文件：${path}`);
check(!existsSync(join(root, "crates/application/src/p4_persistence.rs")), "旧 p4_persistence.rs 仍残留 Research owner");
check(!existsSync(join(root, "crates/application/src/fact_pipeline.rs")), "旧 fact_pipeline.rs 仍残留 Fact Pipeline owner 或空转发层");

const facade = read("crates/application/src/services/research/facade.rs");
const service = read("crates/application/src/services/research/service.rs");
const ports = read("crates/application/src/ports/research/mod.rs");
const adapter = read("crates/application/src/composition/adapters/research.rs");
const databaseFacade = read("crates/application/src/services/database/facade.rs");
const openai = read("crates/application/src/openai_research.rs");
const pipeline = read("crates/application/src/use_cases/research/fact_pipeline/mod.rs");
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
check(openai.replaceAll(" ", "").replaceAll(String.fromCharCode(10), "").replaceAll(String.fromCharCode(13), "").replaceAll(String.fromCharCode(9), "").includes("self.research.register_fact_pipeline_artifacts(session).await?"), "OpenAI artifact 初始化未通过 ResearchService 注册 Fact Pipeline 来源策略");
check(openai.includes("fn execute_p4_openai_research"), "后续 OpenAI Research 执行职责被提前迁移或删除");
check(!lib.includes("mod p4_persistence;"), "Application 根模块仍登记旧 p4_persistence owner");
check(!lib.includes("mod fact_pipeline;"), "Application 根模块仍登记旧 fact_pipeline owner");
check(lib.includes("use_cases::research::fact_pipeline::ProcessResearchEvidenceCommand"), "公共 ProcessResearchEvidenceCommand 未从新 owner 重导出");
check(pipeline.includes("trait FactPipelineAccess"), "Fact Pipeline 缺少 Ports 组合访问边界");
for (const path of ["crates/application/src/openai_research.rs", "crates/application/src/p4_orchestration.rs", "crates/application/src/p4_workbench.rs"]) {
  check(existsSync(join(root, path)), `后续 R3-07 职责被提前删除：${path}`);
}

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
console.log(`Research Service AT2 验证通过：${researchFiles.length} 个 Service/Use Case Rust 文件，8 个公开 Research API 已进入 ResearchService/Ports；Fact Pipeline 已按职责拆分为 ${pipelineFiles.length} 个模块，旧 p4_persistence.rs / fact_pipeline.rs 均已删除。`);
