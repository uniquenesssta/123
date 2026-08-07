import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { ensureNodeDependencies } from "./ensure-node-dependencies.mjs";
import { spawnNodePackageCli } from "./process/node-package-cli.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

try {
  ensureNodeDependencies();
  const result = spawnNodePackageCli({
    root,
    packageName: "vite",
    executablePath: "bin/vite.js",
    args: process.argv.slice(2),
  });
  if (result.error) throw result.error;
  process.exit(result.status ?? 1);
} catch (error) {
  console.error(`Vite 启动失败：${error.message}`);
  process.exit(1);
}
