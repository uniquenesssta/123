import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const failures = [];
const assert = (condition, message) => { if (!condition) failures.push(message); };
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));

const forbiddenPaths = [
  "crates/model-p4",
  "crates/model-p7",
  "src-tauri/resources/defaults",
  "docs/P4_INTEGRATION.md",
  "docs/P7_INTEGRATION.md",
];
for (const relative of forbiddenPaths) {
  assert(!exists(relative), `公开包仍包含私有模型路径：${relative}`);
}

const patternDirectories = [
  ["contracts", /^p[47]-.*\.json$/i],
  ["schemas", /^p[47]-.*\.json$/i],
  ["scripts", /^verify-p[47]-.*\.mjs$/i],
  ["src-tauri/resources/research", /^p[47]_.*\.(json|txt)$/i],
];
for (const [directory, pattern] of patternDirectories) {
  const absolute = path.join(root, directory);
  if (!fs.existsSync(absolute)) continue;
  for (const name of fs.readdirSync(absolute)) {
    assert(!pattern.test(name), `公开包仍包含私有模型资产：${directory}/${name}`);
  }
}

for (const required of [
  "crates/model-api/src/lib.rs",
  "crates/model-stub/src/lib.rs",
  "crates/application/src/model_shell/mod.rs",
  "contracts/model-provider-boundary-contract.json",
  "src-tauri/resources/research/public_research_prompt.txt",
  "schemas/research-output.schema.json",
]) {
  assert(exists(required), `公开模型入口缺失：${required}`);
}

const cargoText = [read("Cargo.toml"), read("crates/application/Cargo.toml"), read("Cargo.lock")].join("\n");
for (const token of ["football-model-p4", "football-model-p7", "crates/model-p4", "crates/model-p7"]) {
  assert(!cargoText.includes(token), `Cargo 仍引用私有模型 crate：${token}`);
}
assert(cargoText.includes("football-model-stub"), "Cargo 未接入公开模型 Stub");

const sourceRoots = ["crates", "src-tauri/src"];
const sourceFiles = [];
const walk = (directory) => {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) walk(absolute);
    else if (/\.(rs|toml)$/.test(entry.name)) sourceFiles.push(absolute);
  }
};
for (const relative of sourceRoots) walk(path.join(root, relative));
const sourceText = sourceFiles.map((file) => fs.readFileSync(file, "utf8")).join("\n");
for (const token of ["football_model_p4", "football_model_p7", "P4Model", "P7Model"]) {
  assert(!sourceText.includes(token), `源码仍直接调用私有模型实现：${token}`);
}

const stub = read("crates/model-stub/src/lib.rs");
const api = read("crates/model-api/src/lib.rs");
assert(stub.includes("ModelError::Unavailable"), "公开 Stub 未返回显式不可用错误");
assert(api.includes("Unavailable(String)"), "模型 API 缺少显式不可用错误类型");
assert(stub.includes("external-provider"), "公开 Stub 未声明外部提供器边界");

const sensitiveFiles = [
  "crates/application/src/rule_packages.rs",
  "crates/application/src/release_acceptance.rs",
  "crates/application/src/model_shell/fixtures.rs",
  "crates/model-stub/src/lib.rs",
  "crates/persistence-postgres/src/team_features.rs",
  "crates/persistence-postgres/src/match_prediction.rs",
  "contracts/model-provider-boundary-contract.json",
  "docs/ARCHITECTURE.md",
  "docs/RULE_PACKAGES.md",
];
const sensitiveTokens = [
  "p4_p2_time_forward",
  "P4_FOUR_LAYER",
  "P4_CORE_MATH",
  "dixon_coles",
  "world_cup_knockout_rho",
  "training_samples",
  "validation_samples",
  "rho=-0.13",
  "rho\": -0.13",
  "matrix_cell_count: 169",
];
for (const relative of sensitiveFiles) {
  const text = read(relative).toLowerCase();
  for (const rawToken of sensitiveTokens) {
    assert(!text.includes(rawToken.toLowerCase()), `公开边界文件泄露模型内部细节：${relative} -> ${rawToken}`);
  }
}

// Every compile-time include must resolve inside the public package.
for (const file of sourceFiles.filter((value) => value.endsWith(".rs"))) {
  const text = fs.readFileSync(file, "utf8");
  const regex = /include_str!\(\s*"([^"]+)"\s*\)/gs;
  for (const match of text.matchAll(regex)) {
    const target = path.resolve(path.dirname(file), match[1]);
    assert(fs.existsSync(target), `include_str! 指向不存在的公开文件：${path.relative(root, file)} -> ${match[1]}`);
  }
}

const provider = JSON.parse(read("contracts/model-provider-boundary-contract.json"));
assert(provider.provider_kind === "external", "模型提供器契约不是 external");
assert(provider.bundled_runtime === false, "公开包错误声明已捆绑模型运行时");
assert(provider.bundled_parameters === false, "公开包错误声明已捆绑模型参数");
assert(provider.bundled_fixtures === false, "公开包错误声明已捆绑固定回归资产");
assert(provider.failure_mode === "explicit_unavailable_error", "公开模型缺少显式失败语义");

if (failures.length > 0) {
  console.error("公开模型边界验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("公开模型边界验证通过：入口保留，私有模型源码、参数、Profile、固定资产和直接依赖均未进入公开包。");
