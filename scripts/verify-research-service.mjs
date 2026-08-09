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
];
const required = [
  "crates/application/src/services/research/mod.rs",
  "crates/application/src/services/research/service.rs",
  "crates/application/src/services/research/facade.rs",
  "crates/application/src/use_cases/research/mod.rs",
  "crates/application/src/use_cases/research/artifact_catalog.rs",
  "crates/application/src/use_cases/research/ledger.rs",
  "crates/application/src/composition/adapters/research.rs",
  "crates/application/src/ports/research/mod.rs",
];
for (const path of required) check(existsSync(join(root, path)), `缺少 R3-07 文件：${path}`);
check(!existsSync(join(root, "crates/application/src/p4_persistence.rs")), "旧 p4_persistence.rs 仍残留 Research Artifact/Ledger owner");

const facade = read("crates/application/src/services/research/facade.rs");
const service = read("crates/application/src/services/research/service.rs");
const ports = read("crates/application/src/ports/research/mod.rs");
const adapter = read("crates/application/src/composition/adapters/research.rs");
const databaseFacade = read("crates/application/src/services/database/facade.rs");
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
check(adapter.includes("impl ResearchEvidenceLedgerPort for ActiveDatabase"), "Research 组合适配器缺少 Evidence Ledger 实现");
check(adapter.includes(".append_evidence_claim(draft)"), "Evidence claim 未复用既有持久化能力");
check(adapter.includes(".create_evidence_conflict(draft)"), "Evidence conflict 未复用既有持久化能力");
check(databaseFacade.includes(".register_persistence_artifacts(prepared.session())"), "数据库初始化未通过 ResearchService 注册内置 Research schema");
check(!databaseFacade.includes("register_p4_persistence_artifacts"), "数据库初始化仍引用旧 p4_persistence owner");
check(!lib.includes("mod p4_persistence;"), "Application 根模块仍登记旧 p4_persistence owner");
for (const path of ["crates/application/src/openai_research.rs", "crates/application/src/fact_pipeline.rs", "crates/application/src/p4_orchestration.rs", "crates/application/src/p4_workbench.rs"]) {
  check(existsSync(join(root, path)), `后续 R3-07 职责被提前删除：${path}`);
}

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
console.log(`Research Service AT1 验证通过：${researchFiles.length} 个 Service/Use Case Rust 文件，7 个公开 Artifact/Ledger API 已迁入 ResearchService/Ports，旧 p4_persistence.rs 已删除。`);
