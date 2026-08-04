import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const readJson = (path) => JSON.parse(readFileSync(join(root, path), "utf8"));
const read = (path) => readFileSync(join(root, path), "utf8");
const failures = [];
const assert = (condition, message) => { if (!condition) failures.push(message); };
const contract = readJson("contracts/release-readiness-contract.json");
const pkg = readJson("package.json");
const lock = readJson("package-lock.json");
const tauri = readJson("src-tauri/tauri.conf.json");
const cargo = read("Cargo.toml");
const readme = read("README.md");
const provider = readJson("contracts/model-provider-boundary-contract.json");

assert(contract.contract_id === "football.public-release-readiness.v1", "公开发布契约 ID 错误");
assert(contract.release_version === pkg.version, "公开发布契约与项目版本不一致");
assert(lock.version === pkg.version && lock.packages?.[""]?.version === pkg.version, "package-lock 根版本未同步");
assert(tauri.version === pkg.version, "Tauri 版本未同步");
assert(cargo.includes(`version = "${pkg.version}"`), "Cargo workspace 版本未同步");
assert(readme.includes("## 0.23.0 变更记录"), "README 缺少公开拆分变更记录");
assert(readme.includes("外部模型") && readme.includes("不会静默回退"), "README 未明确外部模型失败语义");
assert(provider.provider_kind === "external" && provider.bundled_runtime === false, "公开模型提供器边界错误");

for (const name of contract.required_scripts) {
  assert(typeof pkg.scripts?.[name] === "string", `package.json 缺少脚本：${name}`);
}
assert(pkg.scripts?.["tauri:dev"] === "node scripts/run-tauri.mjs dev", "tauri:dev 未接入受控启动器");
assert(pkg.scripts?.["tauri:build"] === "node scripts/run-tauri.mjs build", "tauri:build 未接入受控启动器");
for (const launcher of contract.required_root_launchers) {
  assert(existsSync(join(root, launcher)), `项目根目录缺少启动入口：${launcher}`);
}
for (const file of contract.required_public_boundary_files) {
  assert(existsSync(join(root, file)), `公开模型边界文件缺失：${file}`);
}
for (const file of contract.forbidden_private_paths) {
  assert(!existsSync(join(root, file)), `公开仓库仍包含私有路径：${file}`);
}

const actualMigrations = readdirSync(join(root, "crates/persistence-postgres/migrations"))
  .filter((name) => name.endsWith(".sql"))
  .sort((left, right) => left.localeCompare(right, "en"));
assert(actualMigrations.length === contract.migration_count, `迁移数量异常：期望 ${contract.migration_count}，实际 ${actualMigrations.length}`);
assert(actualMigrations[0] === contract.first_migration, "首条迁移边界异常");
assert(actualMigrations.at(-1) === contract.last_migration, "末条迁移边界异常");
for (let index = 0; index < actualMigrations.length; index += 1) {
  const expectedPrefix = String(index + 1).padStart(4, "0") + "_";
  assert(actualMigrations[index].startsWith(expectedPrefix), `迁移编号不连续：${actualMigrations[index]}`);
}
for (const item of contract.migrations) {
  const path = join(root, "crates/persistence-postgres/migrations", item.file);
  assert(existsSync(path), `迁移缺失：${item.file}`);
  if (existsSync(path)) {
    const hash = createHash("sha256").update(readFileSync(path)).digest("hex");
    assert(hash === item.sha256, `迁移基线已漂移：${item.file}`);
  }
}

if (failures.length) {
  console.error("公开发布就绪验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log(`公开发布就绪验证通过：版本 ${pkg.version}、${contract.migration_count} 条迁移、外部模型边界和自动验证入口完整。`);
