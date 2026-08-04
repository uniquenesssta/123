import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const read = (relativePath) =>
  fs.readFileSync(path.join(root, relativePath), "utf8");
const assert = (condition, message) => {
  if (!condition) {
    throw new Error(message);
  }
};

const workspace = read("pnpm-workspace.yaml");
const packageJson = JSON.parse(read("package.json"));
const tauriConfig = JSON.parse(read("src-tauri/tauri.conf.json"));

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
  packageJson.scripts?.["tauri:dev"] === "node scripts/run-tauri.mjs dev",
  "tauri:dev 必须使用阶段7自动依赖启动器",
);
assert(
  read("scripts/run-tauri.mjs").includes("prepare-cargo-target.mjs"),
  "阶段7启动器必须保留项目路径缓存保护",
);
assert(
  tauriConfig.build?.beforeDevCommand === "npm run dev",
  "Tauri beforeDevCommand 必须固定使用 npm run dev",
);
assert(
  tauriConfig.build?.beforeBuildCommand === "npm run build",
  "Tauri beforeBuildCommand 必须固定使用 npm run build",
);

console.log("包管理器入口验证通过：npm 为推荐入口；pnpm 11 已仅批准 esbuild 构建脚本。");
