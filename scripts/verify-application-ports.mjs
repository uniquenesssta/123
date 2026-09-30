import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import { listFiles } from "./architecture/lib/repository.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const inventoryPath = path.join(root, "architecture/application-port-inventory.json");
const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8"));
const portsPath = "crates/application/src/ports";
const portsRoot = path.join(root, portsPath);
const read = (file) => fs.readFileSync(path.join(root, file), "utf8").replace(/\r\n?/g, "\n");
const hash = (source) => createHash("sha256").update(source, "utf8").digest("hex");
const failures = [];
const bannedInfrastructure = [
  "football_persistence_postgres",
  "sqlx",
  "PostgresStore",
  "PgPool",
  "PgConnection",
  "PersistenceError",
];

if (inventory.schemaVersion !== "football.application-port-inventory.v1") {
  failures.push("Application Port 清单 schemaVersion 不受支持");
}
if (inventory.policy.universalRepositoryAllowed !== false) {
  failures.push("Application Port 策略必须禁止万能 Repository");
}

const portFiles = listFiles([portsPath], { extensions: [".rs"] });
const actualTraits = new Map();
const registeredTraits = new Set();
const domains = new Set();
const jsonBoundaries = new Map();
for (const boundary of inventory.policy.existingJsonBoundaries ?? []) {
  if (jsonBoundaries.has(boundary.owner)) failures.push(`JSON 兼容边界重复：${boundary.owner}`);
  jsonBoundaries.set(boundary.owner, boundary);
  if (!portFiles.includes(boundary.owner) || !boundary.reason || hash(read(boundary.owner)) !== boundary.sourceSha256) {
    failures.push(`既有 JSON 兼容边界已变化或登记失效：${boundary.owner}`);
  }
}

for (const domain of inventory.domains) {
  if (domains.has(domain.name)) failures.push(`Port 领域重复登记：${domain.name}`);
  domains.add(domain.name);
  const file = path.join(portsRoot, domain.name, "mod.rs");
  if (!fs.existsSync(file)) {
    failures.push(`缺少 Port 领域模块：${domain.name}/mod.rs`);
    continue;
  }
  for (const traitName of domain.traits) {
    const key = `${domain.name}/${traitName}`;
    if (registeredTraits.has(key)) failures.push(`Port trait 重复登记：${key}`);
    registeredTraits.add(key);
  }
}

for (const owner of portFiles) {
  const text = read(owner);
  const domainName = path.relative(portsRoot, path.join(root, owner)).split(path.sep)[0];
  if (owner !== `${portsPath}/mod.rs` && owner !== `${portsPath}/error.rs` && !domains.has(domainName)) {
    failures.push(`未登记 Port 领域或根文件：${owner}`);
  }
  for (const token of bannedInfrastructure) {
    if (new RegExp(`\\b${token}\\b`).test(text)) failures.push(`${owner} Port 泄漏基础设施符号：${token}`);
  }
  if (/\bserde_json\s*::\s*(?:Value\b|\{[^}]*\bValue\b)/s.test(text) && !jsonBoundaries.has(owner)) {
    failures.push(`${owner} Port 泄漏未登记的 JSON Value 边界`);
  }
  if (/\b(?:trait|struct|enum|type)\s+\w*Repository\b/.test(text)) {
    failures.push(`${owner} Port 出现万能 Repository 命名`);
  }
  if (/pub\s+use\s+[^;]+::\s*\*/.test(text)) {
    failures.push(`${owner} Port 禁止 glob re-export`);
  }
  for (const match of text.matchAll(/\bpub\s+(?:unsafe\s+)?trait\s+(\w+)\b/g)) {
    const key = `${domainName}/${match[1]}`;
    if (actualTraits.has(key)) failures.push(`Port trait 重复声明：${key}（${actualTraits.get(key)}、${owner}）`);
    actualTraits.set(key, owner);
  }
}

for (const [key, owner] of actualTraits) {
  if (!registeredTraits.has(key)) failures.push(`未登记公开 Port trait：${key}（${owner}）`);
}
for (const key of registeredTraits) {
  if (!actualTraits.has(key)) failures.push(`已登记 Port trait 缺少声明：${key}`);
}

const applicationFiles = listFiles(["crates/application/src"], { extensions: [".rs"] });
const concreteImports = applicationFiles.filter((file) => read(file).includes("football_persistence_postgres"));
const expectedImports = [...inventory.sourceScan.directConcretePersistenceImports].sort();
if (JSON.stringify(concreteImports) !== JSON.stringify(expectedImports)) {
  failures.push(
    `Application 具体 PostgreSQL 导入集合变化：期望 ${expectedImports.join(", ")}，实际 ${concreteImports.join(", ")}`,
  );
}

const rootModule = read(`${portsPath}/mod.rs`);
if (!rootModule.includes("pub use error::{PortError, PortErrorKind, PortResult};")) {
  failures.push("Port 根模块缺少统一 PortError/PortResult 出口");
}

if (failures.length > 0) {
  console.error("Application Ports 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}

const actualScan = {
  portSourceRoot: portsPath,
  portSourceFiles: portFiles,
  portSourceSha256: hash(JSON.stringify(portFiles.map((file) => [file, read(file)]))),
  publicPortTraitCount: actualTraits.size,
  applicationRustFileCount: applicationFiles.length,
  directConcretePersistenceImports: concreteImports,
};
if (process.argv.includes("--refresh-source-scan")) {
  inventory.sourceScan = actualScan;
  fs.writeFileSync(inventoryPath, `${JSON.stringify(inventory, null, 2)}\n`, "utf8");
} else if (JSON.stringify(inventory.sourceScan) !== JSON.stringify(actualScan)) {
  console.error("Application Ports sourceScan 漂移；声明/依赖检查通过后运行 node scripts/verify-application-ports.mjs --refresh-source-scan 并审查变更。");
  process.exit(1);
}

console.log(
  `Application Ports 验证通过：${inventory.domains.length} 个职责域、${actualTraits.size} 个公开 Port trait、${portFiles.length} 个递归 Port 文件；` +
  `${applicationFiles.length} 个 Application 文件中的具体 PostgreSQL 导入仍仅位于组合根。`,
);
