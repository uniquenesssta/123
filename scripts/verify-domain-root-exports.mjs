import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { publicCompatibilityTypeNames } from "./domain-inventory/root-export-policy.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const inventory = JSON.parse(
  readFileSync(resolve(root, "architecture/domain-type-inventory.json"), "utf8"),
);
const libPath = resolve(root, "crates/domain/src/lib.rs");
const actual = readFileSync(libPath, "utf8");

if (/pub\s+use\s+[^;]+::\*\s*;/s.test(actual)) {
  console.error("Domain 根出口禁止 glob re-export。");
  process.exit(1);
}

if (/^\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|union|trait|impl|fn|const|static|type)\b/m.test(actual)) {
  console.error("Domain 根文件只能声明模块和 re-export，不得承载领域定义或实现。");
  process.exit(1);
}

const expectedModules = [...new Set(inventory.types.map((entry) => entry.targetModule))].sort();
const declaredModules = [...actual.matchAll(/^pub mod ([a-z0-9_]+);$/gm)]
  .map((match) => match[1])
  .sort();
if (JSON.stringify(declaredModules) !== JSON.stringify(expectedModules)) {
  console.error(
    `Domain 根模块声明不一致：expected=${expectedModules.join(",")}; actual=${declaredModules.join(",")}`,
  );
  process.exit(1);
}

const expectedByModule = new Map(expectedModules.map((moduleName) => [moduleName, []]));
for (const entry of inventory.types) {
  if (entry.publicCompatibilityType) {
    expectedByModule.get(entry.targetModule).push(entry.typeName);
  }
}
for (const names of expectedByModule.values()) {
  names.sort();
}

const actualByModule = new Map(expectedModules.map((moduleName) => [moduleName, []]));
for (const match of actual.matchAll(/pub\s+use\s+([a-z0-9_]+)::\{([^}]*)\};/gs)) {
  const moduleName = match[1];
  if (!actualByModule.has(moduleName)) {
    console.error(`Domain 根出口出现未知模块：${moduleName}`);
    process.exit(1);
  }
  const names = match[2]
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean);
  actualByModule.get(moduleName).push(...names);
}

for (const moduleName of expectedModules) {
  const expected = expectedByModule.get(moduleName);
  const actualNames = actualByModule.get(moduleName).sort();
  if (JSON.stringify(actualNames) !== JSON.stringify(expected)) {
    console.error(
      `Domain 根出口 ${moduleName} 与类型清单不一致：expected=${expected.length}; actual=${actualNames.length}`,
    );
    process.exit(1);
  }
}

const publicTypes = publicCompatibilityTypeNames(inventory);
const exportedTypes = [...actualByModule.values()].flat();
if (new Set(exportedTypes).size !== publicTypes.length) {
  console.error("Domain 根出口包含重复公共类型或遗漏公共兼容类型。");
  process.exit(1);
}

console.log(
  `Domain 根出口验证通过：${publicTypes.length} 个公共兼容类型全部显式 re-export，0 条 glob export。`,
);
