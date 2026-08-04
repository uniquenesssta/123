import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { ensureNodeDependencies } from "./ensure-node-dependencies.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
function runNodeScript(name) {
  const result = spawnSync(process.execPath, [join(root, "scripts", name)], {
    cwd: root, stdio: "inherit", windowsHide: true,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
try {
  ensureNodeDependencies();
  runNodeScript("verify-cargo-lock.mjs");
  runNodeScript("verify-cargo-lock-sync.mjs");
  runNodeScript("prepare-cargo-target.mjs");
  const tauri = join(root, "node_modules", "@tauri-apps", "cli", "tauri.js");
  const result = spawnSync(process.execPath, [tauri, ...process.argv.slice(2)], {
    cwd: root, stdio: "inherit", windowsHide: true,
  });
  if (result.error) throw result.error;
  process.exit(result.status ?? 1);
} catch (error) {
  console.error(`Tauri 启动失败：${error.message}`);
  process.exit(1);
}
