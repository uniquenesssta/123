import crypto from "node:crypto";
import fs from "node:fs";
import { isVersionAtLeast } from "./version.mjs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const assert = (condition, message) => { if (!condition) throw new Error(message); };

const contractText = read("contracts/release-acceptance-contract.json");
const contract = JSON.parse(contractText);
const schema = JSON.parse(read("schemas/release-acceptance-contract.schema.json"));
const migration = read("crates/persistence-postgres/migrations/0027_release_acceptance.sql");
const application = read("crates/application/src/release_acceptance.rs");
const persistence = read("crates/persistence-postgres/src/release_acceptance.rs");
const commands = read("src-tauri/src/commands/release_acceptance.rs");
const registry = read("src-tauri/src/bootstrap/command_registry.rs");
const client = read("src/api/client.ts");
const pkg = JSON.parse(read("package.json"));
const hash = crypto.createHash("sha256").update(contractText).digest("hex");

assert(schema.properties?.acceptance_mode?.const === contract.acceptance_mode, "发布验收契约与 Schema 模式不一致");
assert(contract.release_version === "0.23.0" && isVersionAtLeast(pkg.version, contract.release_version), "发布验收契约版本与项目版本不兼容");
assert(contract.acceptance_mode === "public_shell_and_runtime", "公开仓库发布验收模式错误");
assert(contract.release_gates.external_provider_required_for_prediction === true, "发布门禁未声明外部模型提供器要求");
assert(contract.release_gates.automatic_parameter_promotion === false, "发布门禁不得允许自动参数晋升");
assert(contract.required_runtime_checks.includes("public_model_boundary"), "发布验收缺少公开模型边界检查");
assert(contract.required_runtime_checks.includes("external_model_runtime"), "发布验收缺少外部模型运行时检查");
assert(migration.includes(hash), "发布验收迁移中的契约哈希不匹配");
assert(migration.includes("'acceptance_mode', 'public_shell_and_runtime'"), "数据库契约仍使用私有固定夹具验收模式");
assert(migration.includes("'external_provider_required', true"), "数据库契约未锁定外部提供器门禁");
assert(application.includes("external provider") || application.includes("外部模型"), "应用验收未报告外部模型运行时边界");
assert(persistence.includes("public_shell_and_runtime"), "持久化验收元数据未使用公开壳模式");

for (const command of ["run_release_acceptance", "list_release_acceptance_runs", "read_release_acceptance_run"]) {
  assert(application.includes(`fn ${command}`), `应用服务缺失：${command}`);
  assert(commands.includes(`fn ${command}`), `Tauri 命令缺失：${command}`);
  assert(registry.includes(command), `Tauri 注册缺失：${command}`);
  assert(client.includes(command), `前端 API 缺失：${command}`);
}

console.log("发布验收验证通过：公开壳与外部模型运行时边界已进入发布门禁。");
