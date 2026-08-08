import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const inventory = JSON.parse(
  fs.readFileSync(path.join(root, "architecture/application-port-inventory.json"), "utf8"),
);
const portsRoot = path.join(root, "crates/application/src/ports");
const failures = [];
const bannedInfrastructure = [
  "football_persistence_postgres",
  "sqlx::",
  "PostgresStore",
  "PgPool",
  "PersistenceError",
  "serde_json::Value",
];

if (inventory.schemaVersion !== "football.application-port-inventory.v1") {
  failures.push("Application Port 清单 schemaVersion 不受支持");
}
if (inventory.policy.universalRepositoryAllowed !== false) {
  failures.push("Application Port 策略必须禁止万能 Repository");
}

let traitCount = 0;
for (const domain of inventory.domains) {
  const file = path.join(portsRoot, domain.name, "mod.rs");
  if (!fs.existsSync(file)) {
    failures.push(`缺少 Port 领域模块：${domain.name}/mod.rs`);
    continue;
  }
  const text = fs.readFileSync(file, "utf8");
  for (const token of bannedInfrastructure) {
    if (text.includes(token)) failures.push(`${domain.name} Port 泄漏基础设施符号：${token}`);
  }
  if (/\b(?:trait|struct|enum|type)\s+\w*Repository\b/.test(text)) {
    failures.push(`${domain.name} Port 出现万能 Repository 命名`);
  }
  if (/pub\s+use\s+[^;]+::\s*\*/.test(text)) {
    failures.push(`${domain.name} Port 禁止 glob re-export`);
  }
  for (const traitName of domain.traits) {
    const matches = text.match(new RegExp(`\\bpub\\s+trait\\s+${traitName}\\b`, "g")) ?? [];
    if (matches.length !== 1) {
      failures.push(`${domain.name} Port trait ${traitName} 数量应为 1，实际 ${matches.length}`);
    } else {
      traitCount += 1;
    }
  }
}

const expectedTraitCount = inventory.domains.reduce((sum, domain) => sum + domain.traits.length, 0);
if (traitCount !== expectedTraitCount) {
  failures.push(`Port trait 总数不一致：期望 ${expectedTraitCount}，实际 ${traitCount}`);
}

const applicationRoot = path.join(root, "crates/application/src");
const concreteImports = [];
const walk = (directory) => {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (entry.isFile() && entry.name.endsWith(".rs")) {
      const text = fs.readFileSync(full, "utf8");
      if (text.includes("football_persistence_postgres")) {
        concreteImports.push(path.relative(root, full).replaceAll("\\", "/"));
      }
    }
  }
};
walk(applicationRoot);
concreteImports.sort();
const expectedImports = [...inventory.sourceScan.directConcretePersistenceImports].sort();
if (JSON.stringify(concreteImports) !== JSON.stringify(expectedImports)) {
  failures.push(
    `Application 具体 PostgreSQL 导入集合变化：期望 ${expectedImports.join(", ")}，实际 ${concreteImports.join(", ")}`,
  );
}

const rootModule = fs.readFileSync(path.join(portsRoot, "mod.rs"), "utf8");
if (!rootModule.includes("pub use error::{PortError, PortErrorKind, PortResult};")) {
  failures.push("Port 根模块缺少统一 PortError/PortResult 出口");
}

if (failures.length > 0) {
  console.error("Application Ports 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}

console.log(
  `Application Ports 验证通过：${inventory.domains.length} 个职责域、${traitCount} 个最小 Port trait；` +
  `${inventory.sourceScan.applicationUsedPersistenceMethods}/${inventory.sourceScan.persistencePublicAsyncMethods} 个持久化异步方法已纳入真实调用面扫描，具体 PostgreSQL 导入仍仅位于组合根。`,
);
