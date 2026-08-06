import { writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createDomainTypeInventory } from "./domain-inventory/inventory-document.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const output = resolve(root, process.argv[2] ?? "architecture/domain-type-inventory.json");
const inventory = createDomainTypeInventory(root);
writeFileSync(output, JSON.stringify(inventory, null, 2) + "\n", "utf8");
console.log("Domain 类型清单已生成：" + inventory.summary.typeCount + " 个类型，" + inventory.summary.domainSourceFileCount + " 个来源文件。");
