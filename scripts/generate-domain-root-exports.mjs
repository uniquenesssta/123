import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { renderDomainRoot } from "./domain-inventory/root-export-policy.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const inventory = JSON.parse(
  readFileSync(resolve(root, "architecture/domain-type-inventory.json"), "utf8"),
);
const outputPath = resolve(root, "crates/domain/src/lib.rs");
writeFileSync(outputPath, renderDomainRoot(inventory), "utf8");
console.log(
  `Domain 根出口已生成：${inventory.summary.publicCompatibilityTypeCount} 个公共兼容类型及登记公共根符号，显式 re-export。`,
);
