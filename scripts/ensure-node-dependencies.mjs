import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { isDirectExecution } from "./process/execution-context.mjs";
import { resolveNodeDependencyLayout } from "./process/node-dependency-layout.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const lockPath = join(root, "package-lock.json");
const packagePath = join(root, "package.json");
const npmrcPath = join(root, ".npmrc");
const layout = resolveNodeDependencyLayout(root);
const markerPath = layout.markerPath;
const packageJson = JSON.parse(readFileSync(packagePath, "utf8"));
const lockSha256 = createHash("sha256").update(readFileSync(lockPath)).digest("hex");
const expected = {
  schema_version: 2,
  layout: "parent-node-modules",
  package_version: packageJson.version,
  package_lock_sha256: lockSha256,
  tauri_cli_version: packageJson.devDependencies?.["@tauri-apps/cli"] ?? null,
};

function requiredPaths(nodeModulesRoot = layout.nodeModulesRoot) {
  return [
    join(nodeModulesRoot, "@tauri-apps", "cli", "tauri.js"),
    join(nodeModulesRoot, "typescript", "bin", "tsc"),
    join(nodeModulesRoot, "vite", "bin", "vite.js"),
  ];
}

function readMarker() {
  try {
    return JSON.parse(readFileSync(markerPath, "utf8"));
  } catch {
    return null;
  }
}

function markerMatches(marker) {
  return marker?.schema_version === expected.schema_version
    && marker?.layout === expected.layout
    && marker?.package_version === expected.package_version
    && marker?.package_lock_sha256 === expected.package_lock_sha256
    && marker?.tauri_cli_version === expected.tauri_cli_version;
}

function dependenciesReady() {
  return requiredPaths().every(existsSync) && markerMatches(readMarker());
}

function npmCommand() {
  if (process.env.npm_execpath) {
    return { command: process.execPath, args: [process.env.npm_execpath] };
  }
  if (process.platform === "win32") {
    return {
      command: process.env.ComSpec || process.env.COMSPEC || "cmd.exe",
      args: ["/d", "/s", "/c", "npm.cmd"],
    };
  }
  return { command: "npm", args: [] };
}

function removeLegacyRootDependencies() {
  if (!existsSync(layout.legacyRootNodeModules)) return;
  rmSync(layout.legacyRootNodeModules, {
    recursive: true,
    force: true,
    maxRetries: 5,
    retryDelay: 250,
  });
  console.log(`已移除源码根目录内的旧依赖目录：${layout.legacyRootNodeModules}`);
}

function installDependencies() {
  const existingMarker = readMarker();
  if (existsSync(layout.nodeModulesRoot) && !existingMarker) {
    throw new Error(
      `源码根目录上一级已存在未登记的 node_modules，拒绝覆盖：${layout.nodeModulesRoot}`,
    );
  }

  const stagingRoot = mkdtempSync(join(layout.dependencyRoot, ".football-node-deps-"));
  const stagingNodeModules = join(stagingRoot, "node_modules");
  try {
    copyFileSync(packagePath, join(stagingRoot, "package.json"));
    copyFileSync(lockPath, join(stagingRoot, "package-lock.json"));
    if (existsSync(npmrcPath)) {
      copyFileSync(npmrcPath, join(stagingRoot, ".npmrc"));
    }

    const npm = npmCommand();
    console.log("检测到前端开发依赖缺失或锁文件已变化，正在源码根目录上一级执行锁定依赖安装 …");
    const result = spawnSync(npm.command, [...npm.args, "ci", "--include=dev"], {
      cwd: stagingRoot,
      stdio: "inherit",
      windowsHide: true,
    });
    if (result.error) throw result.error;
    if (result.status !== 0) {
      throw new Error(`npm ci 失败，退出码：${result.status ?? "unknown"}`);
    }
    if (!requiredPaths(stagingNodeModules).every(existsSync)) {
      throw new Error("npm ci 完成后仍缺少 Tauri/TypeScript/Vite 执行文件");
    }

    if (existsSync(layout.nodeModulesRoot)) {
      rmSync(layout.nodeModulesRoot, {
        recursive: true,
        force: true,
        maxRetries: 5,
        retryDelay: 250,
      });
    }
    renameSync(stagingNodeModules, layout.nodeModulesRoot);
    writeFileSync(markerPath, `${JSON.stringify(expected, null, 2)}\n`, "utf8");
    removeLegacyRootDependencies();
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
}

export function ensureNodeDependencies({ allowInstall = true } = {}) {
  if (dependenciesReady()) {
    removeLegacyRootDependencies();
    console.log(
      `前端依赖已就绪：${layout.nodeModulesRoot}；Tauri CLI ${expected.tauri_cli_version}。`,
    );
    return;
  }
  if (!allowInstall) {
    throw new Error("前端依赖未就绪；请运行 npm run setup");
  }
  installDependencies();
}

if (isDirectExecution(import.meta.url)) {
  try {
    const checkOnly = process.argv.includes("--check");
    ensureNodeDependencies({ allowInstall: !checkOnly });
    if (process.argv.includes("--install-only")) {
      console.log("依赖同步完成。");
    }
  } catch (error) {
    console.error(`依赖同步失败：${error.message}`);
    process.exit(1);
  }
}
