import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { isAbsolute, join } from "node:path";
import { resolveNodeDependencyLayout } from "./node-dependency-layout.mjs";

function safeSegments(value, label) {
  if (typeof value !== "string" || !value.trim() || isAbsolute(value)) {
    throw new Error(`${label} 必须是非空相对路径`);
  }
  const segments = value.split(/[\\/]+/).filter(Boolean);
  if (segments.some((segment) => segment === "." || segment === "..")) {
    throw new Error(`${label} 不得包含相对跳转段`);
  }
  return segments;
}

export function resolveNodePackageCli(root, packageName, executablePath) {
  const packageSegments = safeSegments(packageName, "packageName");
  const executableSegments = safeSegments(executablePath, "executablePath");
  const { nodeModulesRoot } = resolveNodeDependencyLayout(root);
  const resolved = join(nodeModulesRoot, ...packageSegments, ...executableSegments);
  if (!existsSync(resolved)) {
    throw new Error(`缺少上一级 Node CLI：${packageName}/${executablePath}`);
  }
  return resolved;
}

export function spawnNodePackageCli({
  root,
  packageName,
  executablePath,
  args = [],
  options = {},
}) {
  const cli = resolveNodePackageCli(root, packageName, executablePath);
  return spawnSync(process.execPath, [cli, ...args], {
    cwd: root,
    stdio: "inherit",
    windowsHide: true,
    ...options,
  });
}
