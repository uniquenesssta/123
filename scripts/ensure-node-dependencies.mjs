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
const lockJson = JSON.parse(readFileSync(lockPath, "utf8"));
const lockSha256 = createHash("sha256").update(readFileSync(lockPath)).digest("hex");
const expected = {
  schema_version: 2,
  layout: "parent-node-modules",
  package_version: packageJson.version,
  package_lock_sha256: lockSha256,
  tauri_cli_version: packageJson.devDependencies?.["@tauri-apps/cli"] ?? null,
};
const directPackageNames = [
  ...Object.keys(packageJson.dependencies ?? {}),
  ...Object.keys(packageJson.devDependencies ?? {}),
];

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

function installedDirectPackagesMatch(nodeModulesRoot = layout.nodeModulesRoot) {
  return directPackageNames.every((packageName) => {
    const expectedVersion = lockJson.packages?.[`node_modules/${packageName}`]?.version;
    if (!expectedVersion) return false;
    try {
      const installed = JSON.parse(
        readFileSync(join(nodeModulesRoot, packageName, "package.json"), "utf8"),
      );
      return installed.version === expectedVersion;
    } catch {
      return false;
    }
  });
}

function dependencyFilesMatch(nodeModulesRoot = layout.nodeModulesRoot) {
  return requiredPaths(nodeModulesRoot).every(existsSync)
    && installedDirectPackagesMatch(nodeModulesRoot);
}

function dependenciesReady() {
  return dependencyFilesMatch() && markerMatches(readMarker());
}

function writeMarker() {
  const temp = `${markerPath}.${process.pid}.tmp`;
  writeFileSync(temp, `${JSON.stringify(expected, null, 2)}\n`, "utf8");
  renameSync(temp, markerPath);
}

function adoptPreparedParentDependencies() {
  if (!existsSync(layout.nodeModulesRoot) || !dependencyFilesMatch()) return false;
  writeMarker();
  console.log(`已复用源码根目录上一级预置依赖：${layout.nodeModulesRoot}`);
  return true;
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
      `源码根目录上一级已存在与锁文件不匹配的 node_modules，拒绝覆盖：${layout.nodeModulesRoot}`,
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
    if (!dependencyFilesMatch(stagingNodeModules)) {
      throw new Error("npm ci 完成后依赖版本或 Tauri/TypeScript/Vite 执行文件与锁文件不一致");
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
    writeMarker();
    removeLegacyRootDependencies();
  } finally {
    rmSync(stagingRoot, { recursive: true, force: true });
  }
}

export function ensureNodeDependencies({ allowInstall = true } = {}) {
  if (dependenciesReady() || adoptPreparedParentDependencies()) {
    removeLegacyRootDependencies();
    console.log(
      `前端依赖已就绪：${layout.nodeModulesRoot}；Tauri CLI ${expected.tauri_cli_version}。`,
    );
    return;
  }
  if (!allowInstall) {
    throw new Error("前端依赖未就绪；请将锁定依赖放到 ../node_modules 或运行 npm run setup");
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
