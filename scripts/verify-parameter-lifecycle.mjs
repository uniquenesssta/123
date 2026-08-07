import fs from "node:fs";
import { isVersionAtLeast } from "./version.mjs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const assert = (condition, message) => { if (!condition) throw new Error(message); };
const HISTORICAL_MIGRATION_CONTRACT_SHA256 = "8e53460bd59797138a7c7977c3d8379bb0704cd80ae7df30f69082adb79aa4e3";

const contractText = read("contracts/parameter-lifecycle-contract.json");
const contract = JSON.parse(contractText);
const schema = JSON.parse(read("schemas/parameter-lifecycle-contract.schema.json"));
const migration = read(contract.migration);
const application = read("crates/application/src/analytics.rs");
const persistence = read("crates/persistence-postgres/src/parameter_lifecycle.rs");
const commands = read("src-tauri/src/commands/analytics.rs");
const registry = read("src-tauri/src/bootstrap/command_registry.rs");
const client = read("src/api/client.ts");
const pkg = JSON.parse(read("package.json"));

assert(schema.properties?.contract_id?.const === contract.contract_id, "参数生命周期契约与 Schema ID 不一致");
assert(contract.release_version === "0.21.0" && isVersionAtLeast(pkg.version, contract.release_version), "参数生命周期契约版本与项目版本不兼容");
assert(contract.delivery_phase === "INTEGRATION_I_STAGE_1", "参数生命周期交付阶段标识错误");
assert(contract.integration_point_h_required === true, "参数生命周期未保留接入点 H 门禁");
assert(contract.required_h_contract_key === "p4-postmatch-settlement", "持久化兼容契约键发生变化");
assert(contract.provider_state === "NOT_BUNDLED", "公开仓库不得声明已捆绑模型提供器");
assert(contract.candidate_generation.provider_owned_parameters === true, "候选参数所有权边界未归属外部提供器");
assert(contract.promotion.automatic_promotion === false, "公开仓库不得自动晋升参数");
assert(contract.promotion.manual_confirmation_required === true && contract.promotion.rollback_supported === true, "人工确认或回滚契约缺失");
assert(contract.compatibility.external_provider_required === true, "参数生命周期未声明外部提供器依赖");
assert(migration.includes(HISTORICAL_MIGRATION_CONTRACT_SHA256), "参数生命周期历史迁移契约哈希不匹配");
for (const table of ["parameter_shadow_validations", "parameter_promotion_decisions", "parameter_binding_changes"]) {
  assert(migration.includes(table), `参数生命周期账本缺失：${table}`);
}
for (const trigger of ["parameter_shadow_validations_immutable", "parameter_promotion_decisions_immutable", "parameter_binding_changes_immutable"]) {
  assert(migration.includes(trigger), `参数生命周期不可变触发器缺失：${trigger}`);
}
assert(migration.includes("'automatic_promotion', false") && migration.includes("'p4_4_state', 'SHADOW_ONLY'"), "数据库历史契约未锁定人工晋升或 P4.4 影子边界");
for (const status of contract.candidate_statuses) {
  assert(migration.includes(`'${status}'`), `数据库候选状态缺失：${status}`);
}
assert(persistence.includes("p4-postmatch-settlement"), "持久化层未保留接入点 H 兼容门禁");
assert(application.includes("binding_unchanged") && application.includes("finite_probabilities"), "影子验证核心安全门禁缺失");

for (const command of contract.commands) {
  assert(application.includes(`fn ${command}`), `应用服务缺失：${command}`);
  assert(commands.includes(`fn ${command}`), `Tauri 命令缺失：${command}`);
  assert(registry.includes(command), `Tauri 注册缺失：${command}`);
  assert(client.includes(command), `前端 API 缺失：${command}`);
}

console.log("参数生命周期验证通过：当前公开提供器边界与冻结历史迁移分别验证，人工晋升、不可变账本和回滚链路完整。");
