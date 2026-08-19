import fs from "node:fs";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = new URL("../", import.meta.url);
const branch = "rewrite/r6-entity-catalog-persistence";
const read = (path) => fs.readFileSync(new URL(path, root), "utf8");
const write = (path, content) => fs.writeFileSync(new URL(path, root), content, "utf8");

function replaceOnce(path, oldText, newText) {
  const text = read(path);
  const first = text.indexOf(oldText);
  if (first < 0 || text.indexOf(oldText, first + oldText.length) >= 0) {
    throw new Error(`${path}: closeout anchor must exist exactly once`);
  }
  write(path, text.replace(oldText, newText));
}

replaceOnce(
  "README.md",
  "- `EntityReferencePort`、Domain DTO、Schema/0001–0046 migrations、错误语义、历史 P4/运行引用保护、无引用永久删除和 force purge 行为均未改变；无新增生产依赖。AT1 Minimum Gate 已实际通过，Stage Regression / PostgreSQL 16 真实 contract 尚待本次 workflow 后续 jobs，R6-09 整体仍为 `IN_PROGRESS`。",
  "- `EntityReferencePort`、Domain DTO、Schema/0001–0046 migrations、错误语义、历史 P4/运行引用保护、无引用永久删除和 force purge 行为均未改变；无新增生产依赖。AT1 已正式 `DONE`：PostgreSQL 16 contract run `32219150263` / job `95967398924` 为 `SUCCESS`；最终 Windows Stage Regression run `32220581584` / job `95970175840` 为 `SUCCESS`，实际通过完整 frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests 与 diff hygiene。两处历史 verifier 仅把已迁移的 reference-count 检查跟随到 `deletion/preflight/references.rs`，原断言未删除或放宽；临时 retry workflow/markers 已由 cleanup commit `0fa05a3b67eb21009b5a56e00cb5b365fd0e66b5` 清理。R6-09 整体继续 `IN_PROGRESS`，普通永久 Delete 与 Team Force Delete 留待后续 Atomic Task。",
);

replaceOnce(
  "docs/modular-rewrite/R06-entity-catalog-persistence/README.md",
  "- Atomic Task 1：Preflight / Reference Counts / Archive owner switch 已完成 Minimum Gate，节点整体继续 `IN_PROGRESS`；普通永久 Delete 与 Team Force Delete 尚未迁移，R6-10 继续 `BLOCKED`。",
  "- Atomic Task 1：Preflight / Reference Counts / Archive 已 `DONE`。PostgreSQL 16 contract run `32219150263` / job `95967398924` 与最终 Windows Stage Regression run `32220581584` / job `95970175840` 均为 `SUCCESS`；两处 retained verifier 仅跟随新的 `deletion/preflight/references.rs` owner，原断言保持。临时 retry workflow/markers 已清理。R6-09 节点整体继续 `IN_PROGRESS`；普通永久 Delete 与 Team Force Delete 尚未迁移，R6-10 继续 `BLOCKED`。",
);

const taskPath = "docs/modular-rewrite/R06-entity-catalog-persistence/R06-09-archive-delete-and-force-delete.md";
replaceOnce(
  taskPath,
  "## Atomic Task 1 — Preflight / Reference Counts / Archive\n\n",
  "## Atomic Task 1 — Preflight / Reference Counts / Archive\n\n**状态：DONE**\n\n",
);
replaceOnce(
  taskPath,
  "- Stage Regression 与 PostgreSQL 16 真实 contract 由本 workflow 后续独立 jobs 执行；在成功前 AT1 不关闭。",
  "- PostgreSQL 16 真实 contract：run `32219150263` / job `95967398924` 为 `SUCCESS`。\n- Windows Stage Regression 首轮 run `32219150263` / job `95967398932` 因历史 `verify-match-review-package.mjs` 仍读取旧 `entity_catalog.rs` fail-fast；第一次 retry run `32220406479` / job `95969707027` 又发现 `verify-stage-e2-lineup-presets.mjs` 仍绑定旧 owner。两处只将原 reference-count 断言跟随到 `adapters/catalog/deletion/preflight/references.rs`，未删除、跳过或放宽门禁。\n- 最终 Windows retry run `32220581584` / job `95970175840` 为 `SUCCESS`：AT1 current-head verifier、`npm run verify:frontend`、rustfmt、workspace Clippy `-D warnings`、workspace tests 与 `git diff --check` 全部实际通过。AT1 据此关闭为 `DONE`。",
);
replaceOnce(
  taskPath,
  "- 删除/移动：AT1 无生产文件删除；临时 workflow/helper 在提交时清理。",
  "- 删除/移动：AT1 无生产文件删除；临时 retry workflow、failure/run marker 已由 cleanup commit `0fa05a3b67eb21009b5a56e00cb5b365fd0e66b5` 清理。",
);

const packagePath = new URL("package.json", root);
const pkg = JSON.parse(fs.readFileSync(packagePath, "utf8"));
const temporarySetup = "node scripts/r6-09-at1-doc-closeout.mjs && node scripts/ensure-node-dependencies.mjs --install-only";
if (pkg.scripts?.setup !== temporarySetup) {
  throw new Error(`package.json setup does not match temporary closeout hook: ${pkg.scripts?.setup}`);
}
pkg.scripts.setup = "node scripts/ensure-node-dependencies.mjs --install-only";
fs.writeFileSync(packagePath, `${JSON.stringify(pkg, null, 2)}\n`, "utf8");

for (const path of [
  ".github/workflows/r6-09-at1-doc-closeout.yml",
  ".r6-09-doc-closeout-trigger.txt",
]) {
  const target = new URL(path, root);
  if (fs.existsSync(target)) fs.unlinkSync(target);
}
fs.unlinkSync(fileURLToPath(import.meta.url));

const runGit = (args) => execFileSync("git", args, { cwd: fileURLToPath(root), stdio: "inherit" });
runGit(["config", "user.name", "github-actions[bot]"]);
runGit(["config", "user.email", "41898282+github-actions[bot]@users.noreply.github.com"]);
runGit([
  "add",
  "README.md",
  "package.json",
  "docs/modular-rewrite/R06-entity-catalog-persistence/README.md",
  taskPath,
  ".github/workflows/r6-09-at1-doc-closeout.yml",
  ".r6-09-doc-closeout-trigger.txt",
  "scripts/r6-09-at1-doc-closeout.mjs",
]);
runGit(["diff", "--cached", "--check"]);
runGit(["commit", "-m", "docs(r6-09): close AT1 verification record"]);
runGit(["push", "origin", `HEAD:${branch}`]);
