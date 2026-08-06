import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createDomainTypeInventory } from "./domain-inventory/inventory-document.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const inventoryPath = resolve(root, "architecture/domain-type-inventory.json");
const committed = JSON.parse(readFileSync(inventoryPath, "utf8"));
const actual = createDomainTypeInventory(root);
const expectedText = JSON.stringify(committed, null, 2);
const actualText = JSON.stringify(actual, null, 2);

if (expectedText !== actualText) {
  console.error("Domain 类型与契约清单已漂移。请运行 node scripts/generate-domain-type-inventory.mjs 并审查变更。");
  process.exit(1);
}

if (committed.summary.typeCount < 1 || committed.types.length !== committed.summary.typeCount) {
  console.error("Domain 类型清单为空或汇总数量不一致。");
  process.exit(1);
}

const missingContracts = committed.types.filter((entry) =>
  !entry.currentPath || !entry.targetModule || !entry.targetPath || !entry.serializationName ||
  !Array.isArray(entry.databaseMappings) || !Array.isArray(entry.externalCallers),
);
if (missingContracts.length > 0) {
  console.error("Domain 类型清单缺少必要契约字段：" + missingContracts.map((entry) => entry.typeName).join(", "));
  process.exit(1);
}

console.log(
  "Domain 类型清单验证通过：" + committed.summary.typeCount + " 个类型、" +
  committed.summary.publicCompatibilityTypeCount + " 个公共兼容类型、" +
  committed.summary.databaseMappedTypeCount + " 个 PostgreSQL 映射类型。",
);
