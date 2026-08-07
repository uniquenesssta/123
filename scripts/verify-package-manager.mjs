import fs from "node:fs";
import path from "node:path";
import { resolveNodeDependencyLayout } from "./process/node-dependency-layout.mjs";

const root = process.cwd();
const read = (relativePath) =>
  fs.readFileSync(path.join(root, relativePath), "utf8");
const assert = (condition, message) => {
  if (!condition) {
    throw new Error(message);
  }
};

const workspace = read("pnpm-workspace.yaml");
const npmrc = read(".npmrc");
const packageJson = JSON.parse(read("package.json"));
const tauriConfig = JSON.parse(read("src-tauri/tauri.conf.json"));
const dependencyLayout = resolveNodeDependencyLayout(root);

assert(
  /^allowBuilds:\s*\n(?:[ \t].*\n)*?[ \t]+esbuild:\s*true\s*$/m.test(workspace),
  "pnpm-workspace.yaml 必须显式允许 esbuild 安装脚本",
);
assert(
  !/dangerouslyAllowAllBuilds:\s*true/.test(workspace),
  "不得放开所有依赖安装脚本",
);
assert(
  fs.existsSync(path.join(root, "package-lock.json")),
  "npm 可复现安装必须保留 package-lock.json",
);
assert(
  dependencyLayout.nodeModulesRoot === path.resolve(root, "..", "node_modules"),
  "Node 依赖必须固定读取源码根目录上一级的 node_modules",
);
assert(
  dependencyLayout.nodeModulesRoot !== path.join(root, "node_modules"),
  "Node 依赖不得读取源码根目录内的 node_modules",
);
assert(
  /^cache=\.\.\/.npm-cache\s*$/m.test(npmrc),
  ".npmrc 必须将 npm 缓存放到源码根目录上一级",
);
assert(
  packageJson.scripts?.dev === "node scripts/run-vite.mjs --host 127.0.0.1 --port 1420",
  "dev 必须通过上一级依赖 Vite 启动器",
);
assert(
  packageJson.scripts?.preview === "node scripts/run-vite.mjs preview",
  "preview 必须通过上一级依赖 Vite 启动器",
);
assert(
  packageJson.scripts?.["tauri:dev"] === "node scripts/run-tauri.mjs dev",
  "tauri:dev 必须使用自动依赖启动器",
);
assert(
  read("scripts/run-tauri.mjs").includes("resolveNodePackageCli"),
  "Tauri 启动器必须从统一的上一级 Node 依赖边界解析 CLI",
);
assert(
  read("scripts/run-tauri.mjs").includes("prepare-cargo-target.mjs"),
  "启动器必须保留项目路径缓存保护",
);
assert(
  read("scripts/ensure-node-dependencies.mjs").includes("resolveNodeDependencyLayout"),
  "依赖同步器必须使用统一的上一级依赖布局",
);
assert(
  tauriConfig.build?.beforeDevCommand === "npm run dev",
  "Tauri beforeDevCommand 必须固定使用 npm run dev",
);
assert(
  tauriConfig.build?.beforeBuildCommand === "npm run build",
  "Tauri beforeBuildCommand 必须固定使用 npm run build",
);

console.log("包管理器入口验证通过：Node 依赖与 npm 缓存均位于源码根目录上一级；npm 锁文件与 pnpm 构建脚本边界保持不变。");
