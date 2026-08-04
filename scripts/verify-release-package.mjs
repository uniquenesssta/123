import { existsSync, readFileSync, readdirSync } from "node:fs";
import { basename, join, resolve } from "node:path";

const root = resolve(process.argv[2] ?? process.cwd());
const failures = [];
const forbiddenDirectories = new Set([
  "node_modules", "dist", "target", ".cargo-target", ".git", "logs", "coverage",
  ".cache", ".vite", "__pycache__",
]);
const forbiddenFilePattern = /(?:^|\/)(?:football-runtime(?:-[^/]*)?\.jsonl|football-runtime\.log)$|\.(?:tmp|temp|bak|old|orig|log|zip|7z|rar)$/i;
const privatePathPatterns = [
  /^crates\/model-p[47](?:\/|$)/i,
  /^src-tauri\/resources\/defaults(?:\/|$)/i,
  /^src-tauri\/resources\/research\/p[47]_/i,
  /^contracts\/p[47]-.*\.json$/i,
  /^schemas\/p[47]-.*\.json$/i,
  /^scripts\/verify-p[47]-.*\.mjs$/i,
  /^docs\/P[47]_INTEGRATION\.md$/i,
];
const requiredMarkers = [
  "README.md", "ALL_AI_CODE.md", "AI_PROJECT_RULES.md", ".gitignore",
  "package.json", "package-lock.json", "Cargo.toml", "Cargo.lock",
  "crates/model-api/src/lib.rs", "crates/model-stub/src/lib.rs",
  "contracts/model-provider-boundary-contract.json",
  "scripts/verify-public-model-boundary.mjs", ".github/workflows/ci.yml",
  "启动平台.bat", "验证平台.bat", "验收平台.bat",
];

if (!existsSync(join(root, "package.json"))) {
  console.error(`无法识别公开项目根目录：${root}`);
  process.exit(1);
}

function walk(directory, relative = "") {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const rel = relative ? `${relative}/${entry.name}` : entry.name;
    if (entry.isSymbolicLink()) {
      failures.push(`发布包不得包含符号链接：${rel}`);
      continue;
    }
    if (privatePathPatterns.some((pattern) => pattern.test(rel))) {
      failures.push(`发布包包含私有模型资产：${rel}`);
      continue;
    }
    if (entry.isDirectory()) {
      if (forbiddenDirectories.has(entry.name) || entry.name.startsWith("backup-")) {
        failures.push(`发布包包含构建、缓存、日志或 Git 目录：${rel}`);
      } else {
        walk(join(directory, entry.name), rel);
      }
    } else if (entry.isFile() && forbiddenFilePattern.test(rel)) {
      failures.push(`发布包包含运行时、临时或压缩备份文件：${rel}`);
    }
  }
}

walk(root);
for (const marker of requiredMarkers) {
  if (!existsSync(join(root, marker))) failures.push(`发布包缺少：${marker}`);
}
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
if (pkg.scripts?.["verify:public-model-boundary"] !== "node scripts/verify-public-model-boundary.mjs") {
  failures.push("发布包缺少公开模型边界脚本入口");
}
if (failures.length) {
  console.error("发布包洁净度验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log(`发布包洁净度验证通过：${basename(root)} 不含私有模型资产、依赖缓存、构建产物、日志或 Git 历史。`);
