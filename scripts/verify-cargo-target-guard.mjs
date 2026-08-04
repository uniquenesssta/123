import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
const cargoConfig = fs.readFileSync(path.join(root, ".cargo", "config.toml"), "utf8");
const guardSource = fs.readFileSync(path.join(root, "scripts", "prepare-cargo-target.mjs"), "utf8");
const tauriRunner = fs.readFileSync(path.join(root, "scripts", "run-tauri.mjs"), "utf8");

const failures = [];
const expectedGuard = "node scripts/prepare-cargo-target.mjs";
for (const scriptName of ["cargo:test", "cargo:clippy", "verify:rust"]) {
  const command = packageJson.scripts?.[scriptName];
  if (typeof command !== "string" || !command.includes(expectedGuard)) {
    failures.push(`package.json 脚本 ${scriptName} 未接入 Cargo 路径保护`);
  }
}
for (const scriptName of ["tauri", "tauri:dev", "tauri:build"]) {
  const command = packageJson.scripts?.[scriptName];
  if (typeof command !== "string" || !command.includes("run-tauri.mjs")) {
    failures.push(`package.json 脚本 ${scriptName} 未接入阶段7 Tauri 启动器`);
  }
}
if (!tauriRunner.includes("prepare-cargo-target.mjs")) {
  failures.push("阶段7 Tauri 启动器未接入 Cargo 路径保护");
}

if (!/target-dir\s*=\s*"\.\.\/\.cargo-target"/.test(cargoConfig)) {
  failures.push(".cargo/config.toml 的 target-dir 不再是 ../.cargo-target");
}

for (const token of [
  "target-location.json",
  "migration_bundle_sha256",
  "computeMigrationFingerprint",
  "项目目录移动后仍引用旧绝对路径的 Cargo 缓存",
  "数据库迁移文件内容变化后仍保留旧的 SQLx 嵌入缓存",
  "assertSafeTargetPath",
  "fs.rmSync",
]) {
  if (!guardSource.includes(token)) {
    failures.push(`Cargo 路径保护缺少关键边界：${token}`);
  }
}

if (failures.length > 0) {
  console.error("Cargo 路径保护验证失败：");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("Cargo 路径保护验证通过：项目移动或数据库迁移内容变化后会在编译前安全清理失效的 Rust/Tauri缓存。");
