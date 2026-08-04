import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const lockPath = join(root, "package-lock.json");
const packagePath = join(root, "package.json");
const markerPath = join(root, "node_modules", ".football-deps-lock");
const packageJson = JSON.parse(readFileSync(packagePath, "utf8"));
const lockSha256 = createHash("sha256").update(readFileSync(lockPath)).digest("hex");
const expected = {
  schema_version: 1,
  package_version: packageJson.version,
  package_lock_sha256: lockSha256,
  tauri_cli_version: packageJson.devDependencies?.["@tauri-apps/cli"] ?? null,
};
const required = [
  join(root, "node_modules", "@tauri-apps", "cli", "tauri.js"),
  join(root, "node_modules", "typescript", "bin", "tsc"),
  join(root, "node_modules", "vite", "bin", "vite.js"),
];

function readMarker() {
  try { return JSON.parse(readFileSync(markerPath, "utf8")); } catch { return null; }
}
function dependenciesReady() {
  const marker = readMarker();
  return required.every(existsSync)
    && marker?.schema_version === expected.schema_version
    && marker?.package_version === expected.package_version
    && marker?.package_lock_sha256 === expected.package_lock_sha256
    && marker?.tauri_cli_version === expected.tauri_cli_version;
}
function npmCommand() {
  if (process.env.npm_execpath) return { command: process.execPath, args: [process.env.npm_execpath] };
  return { command: process.platform === "win32" ? "npm.cmd" : "npm", args: [] };
}
function installDependencies() {
  const npm = npmCommand();
  console.log("检测到前端开发依赖缺失或锁文件已变化，正在执行 npm ci --include=dev …");
  const result = spawnSync(npm.command, [...npm.args, "ci", "--include=dev"], {
    cwd: root, stdio: "inherit", windowsHide: true,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`npm ci 失败，退出码：${result.status ?? "unknown"}`);
  if (!required.every(existsSync)) throw new Error("npm ci 完成后仍缺少 Tauri/TypeScript/Vite 本地执行文件");
  mkdirSync(dirname(markerPath), { recursive: true });
  const temp = `${markerPath}.${process.pid}.tmp`;
  writeFileSync(temp, JSON.stringify(expected, null, 2) + "\n", "utf8");
  renameSync(temp, markerPath);
}

export function ensureNodeDependencies({ allowInstall = true } = {}) {
  if (dependenciesReady()) {
    console.log(`前端依赖已就绪：Tauri CLI ${expected.tauri_cli_version}。`);
    return;
  }
  if (!allowInstall) throw new Error("前端依赖未就绪；请运行 npm run setup");
  rmSync(markerPath, { force: true });
  installDependencies();
}

const direct = process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (direct) {
  try {
    const checkOnly = process.argv.includes("--check");
    ensureNodeDependencies({ allowInstall: !checkOnly });
    if (process.argv.includes("--install-only")) console.log("依赖同步完成。");
  } catch (error) {
    console.error(`依赖同步失败：${error.message}`);
    process.exit(1);
  }
}
