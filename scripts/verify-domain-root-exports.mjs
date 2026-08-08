import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { publicCompatibilityTypeNames, renderDomainRoot } from "./domain-inventory/root-export-policy.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const inventory = JSON.parse(
  readFileSync(resolve(root, "architecture/domain-type-inventory.json"), "utf8"),
);
const libPath = resolve(root, "crates/domain/src/lib.rs");
const actual = readFileSync(libPath, "utf8");
const expected = renderDomainRoot(inventory);

if (/pub\s+use\s+[^;]+::\*\s*;/s.test(actual)) {
  console.error("Domain 根出口禁止 glob re-export。");
  process.exit(1);
}

if (actual !== expected) {
  console.error(
    "Domain 根出口与类型清单不一致。请运行 node scripts/generate-domain-root-exports.mjs 并审查显式 re-export 变更。",
  );
  process.exit(1);
}

const publicTypes = publicCompatibilityTypeNames(inventory);
console.log(
  `Domain 根出口验证通过：${publicTypes.length} 个公共兼容类型全部显式 re-export，0 条 glob export。`,
);
