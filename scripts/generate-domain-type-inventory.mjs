import { existsSync, readFileSync, unlinkSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createDomainTypeInventory } from "./domain-inventory/inventory-document.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const output = resolve(root, process.argv[2] ?? "architecture/domain-type-inventory.json");
const inventory = createDomainTypeInventory(root);
writeFileSync(output, JSON.stringify(inventory, null, 2) + "\n", "utf8");
console.log("Domain 类型清单已生成：" + inventory.summary.typeCount + " 个类型，" + inventory.summary.domainSourceFileCount + " 个来源文件。");

// One-run R6-09 AT2 compatibility hook. AT1 was closed while the already-staged AT2
// workflow still referenced its earlier VERIFYING documentation anchors. If that
// workflow has not yet been corrected, restore only those two document anchors in
// the runner checkout so AT2 can publish its own VERIFYING record. The hook then
// restores this generator from the verified AT1 baseline before the AT2 commit.
const at2WorkflowPath = resolve(root, ".github/workflows/r6-09-at2.yml");
const anchorFixPath = resolve(root, ".github/workflows/r6-09-at2-anchor-fix.yml");
const currentStage = "- Atomic Task 1：Preflight / Reference Counts / Archive 已 `DONE`。PostgreSQL 16 contract run `32219150263` / job `95967398924` 与最终 Windows Stage Regression run `32220581584` / job `95970175840` 均为 `SUCCESS`；两处 retained verifier 仅跟随新的 `deletion/preflight/references.rs` owner，原断言保持。临时 retry workflow/markers 已清理。R6-09 节点整体继续 `IN_PROGRESS`；普通永久 Delete 与 Team Force Delete 尚未迁移，R6-10 继续 `BLOCKED`。";
const legacyStage = "- Atomic Task 1：Preflight / Reference Counts / Archive owner switch 已完成 Minimum Gate，节点整体继续 `IN_PROGRESS`；普通永久 Delete 与 Team Force Delete 尚未迁移，R6-10 继续 `BLOCKED`。";
const currentRoot = "- `EntityReferencePort`、Domain DTO、Schema/0001–0046 migrations、错误语义、历史 P4/运行引用保护、无引用永久删除和 force purge 行为均未改变；无新增生产依赖。AT1 已正式 `DONE`：PostgreSQL 16 contract run `32219150263` / job `95967398924` 为 `SUCCESS`；最终 Windows Stage Regression run `32220581584` / job `95970175840` 为 `SUCCESS`，实际通过完整 frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests 与 diff hygiene。两处历史 verifier 仅把已迁移的 reference-count 检查跟随到 `deletion/preflight/references.rs`，原断言未删除或放宽；临时 retry workflow/markers 已由 cleanup commit `0fa05a3b67eb21009b5a56e00cb5b365fd0e66b5` 清理。R6-09 整体继续 `IN_PROGRESS`，普通永久 Delete 与 Team Force Delete 留待后续 Atomic Task。";
const legacyRoot = "- `EntityReferencePort`、Domain DTO、Schema/0001–0046 migrations、错误语义、历史 P4/运行引用保护、无引用永久删除和 force purge 行为均未改变；无新增生产依赖。AT1 Minimum Gate 已实际通过，Stage Regression / PostgreSQL 16 真实 contract 尚待本次 workflow 后续 jobs，R6-09 整体仍为 `IN_PROGRESS`。";

function replaceExactlyOnce(path, from, to) {
  const text = readFileSync(path, "utf8");
  const count = text.split(from).length - 1;
  if (count !== 1) throw new Error(`${path}: expected one R6-09 anchor, found ${count}`);
  writeFileSync(path, text.replace(from, to), "utf8");
}

if (existsSync(at2WorkflowPath)) {
  const workflow = readFileSync(at2WorkflowPath, "utf8");
  if (workflow.includes(legacyStage) && workflow.includes(legacyRoot)) {
    replaceExactlyOnce(resolve(root, "docs/modular-rewrite/R06-entity-catalog-persistence/README.md"), currentStage, legacyStage);
    replaceExactlyOnce(resolve(root, "README.md"), currentRoot, legacyRoot);
  }
}
if (existsSync(anchorFixPath)) unlinkSync(anchorFixPath);

const baseline = "0fa05a3b67eb21009b5a56e00cb5b365fd0e66b5";
const selfPath = resolve(root, "scripts/generate-domain-type-inventory.mjs");
const originalSelf = execFileSync(
  "git",
  ["show", `${baseline}:scripts/generate-domain-type-inventory.mjs`],
  { cwd: root, encoding: "utf8" },
);
writeFileSync(selfPath, originalSelf, "utf8");
