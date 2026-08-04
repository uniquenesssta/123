import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const nodeChecks = [
  "verify-public-model-boundary.mjs",
  "verify-global-name-search.mjs",
  "verify-search-query-state.mjs",
  "verify-player-role-inheritance.mjs",
  "verify-lineup-preset-navigation.mjs",
  "verify-lineup-preset-editor.mjs",
  "verify-inline-workspace-and-resource-scroll.mjs",
  "verify-player-pagination-team-filter.mjs",
  "verify-dual-level-navigation.mjs",
  "verify-core-workspace-hierarchy.mjs",
  "verify-extended-workspace-hierarchy.mjs",
  "verify-global-visual-system.mjs",
  "verify-reported-layout-regressions.mjs",
  "verify-windows-acceptance.mjs",
  "verify-openai-profile-ui.mjs",
  "verify-api-workspace.mjs",
  "verify-team-player-management.mjs",
  "verify-entity-deletion.mjs",
  "verify-force-team-delete.mjs",
  "verify-team-package.mjs",
  "verify-team-package-import-normalization.mjs",
  "verify-import-row-identity.mjs",
  "verify-entity-relationships.mjs",
  "verify-formation-usage.mjs",
  "verify-monthly-workbooks.mjs",
  "verify-match-lineup-chain.mjs",
  "verify-match-workflow-ui.mjs",
  "verify-history-scoreline-ui.mjs",
  "verify-stage1-balanced-ui.mjs",
  "verify-stage2-responsive-catalog-ui.mjs",
  "verify-workspace-ui.mjs",
  "verify-postmatch-settlement.mjs",
  "verify-parameter-lifecycle.mjs",
  "verify-api-runtime-diagnostics.mjs",
  "verify-api-compatible-transport.mjs",
  "verify-api-workspace-research-control.mjs",
  "verify-rust-source-hygiene.mjs",
  "verify-cargo-lock.mjs",
  "verify-package-manager.mjs",
  "verify-cargo-target-guard.mjs",
  "verify-database-reset.mjs",
  "verify-command-contract.mjs",
  "verify-entity-resource-center.mjs",
  "verify-team-package-localized-names.mjs",
  "verify-analysis-history-workflow.mjs",
  "verify-match-review-package.mjs",
  "verify-player-navigation-context.mjs",
  "verify-stage-a-architecture.mjs",
  "verify-task-ui.mjs",
  "verify-task-ui-screenshots.mjs",
  "verify-profile-editor-persistence.mjs",
  "verify-match-event-facts.mjs",
  "verify-team-package-preview-recovery.mjs",
  "verify-searchable-hierarchy-and-team-binding.mjs",
  "verify-searchable-select-diagnostics.mjs",
  "verify-searchable-select-ui.mjs",
  "verify-team-package-real-import-recovery.mjs",
  "verify-stage-e1-ui.mjs",
  "verify-stage-e1-followup.mjs",
  "verify-stage-e2-lineup-presets.mjs",
  "verify-lineup-scroll-continuity.mjs",
  "verify-release-acceptance.mjs",
  "verify-release-readiness.mjs",
];

function run(command, args, label) {
  console.log(`\n[verify] ${label}`);
  const result = spawnSync(command, args, {
    cwd: root,
    stdio: "inherit",
    windowsHide: true,
  });
  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }
  if (result.status !== 0) process.exit(result.status ?? 1);
}

for (const script of nodeChecks) {
  const path = join(root, "scripts", script);
  if (!existsSync(path)) {
    console.error(`缺少前端验证脚本：scripts/${script}`);
    process.exit(1);
  }
  run(process.execPath, [path], script);
}

const binSuffix = process.platform === "win32" ? ".cmd" : "";
run(join(root, "node_modules", ".bin", `tsc${binSuffix}`), ["--noEmit"], "TypeScript");
run(join(root, "node_modules", ".bin", `vite${binSuffix}`), ["build"], "Vite build");
