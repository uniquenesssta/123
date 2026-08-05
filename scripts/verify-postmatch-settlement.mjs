import crypto from "node:crypto";
import fs from "node:fs";
import { isVersionAtLeast } from "./version.mjs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const assert = (condition, message) => { if (!condition) throw new Error(message); };

const contractText = read("contracts/postmatch-settlement-contract.json");
const contract = JSON.parse(contractText);
const schema = JSON.parse(read("schemas/postmatch-settlement-contract.schema.json"));
const migration = read(contract.migration);
const application = read("crates/application/src/postmatch.rs");
const persistence = read("crates/persistence-postgres/src/postmatch.rs");
const commands = read("src-tauri/src/commands/postmatch.rs");
const registry = read("src-tauri/src/bootstrap/command_registry.rs");
const client = read("src/api/client.ts");
const pkg = JSON.parse(read("package.json"));
const hash = crypto.createHash("sha256").update(contractText).digest("hex");

assert(schema.properties?.contract_id?.const === contract.contract_id, "赛后结算契约与 Schema ID 不一致");
assert(contract.release_version === "0.22.0" && isVersionAtLeast(pkg.version, contract.release_version), "赛后结算契约版本与项目版本不兼容");
assert(contract.stage === "H" && contract.delivery_phase === "INTEGRATION_H_COMPLETE", "赛后结算阶段标识错误");
assert(contract.settlement_gate.official_result_required === true, "正式赛果门禁缺失");
assert(contract.settlement_gate.finalized_review_required === true, "最终复盘门禁缺失");
assert(contract.settlement_gate.settlement_records_immutable === true, "结算记录不可变边界缺失");
assert(contract.evidence_scoring.manual_verdict_required === true, "证据人工判定门禁缺失");
assert(contract.evidence_scoring.snapshot_cutoff_enforced === true, "证据快照截止门禁缺失");
assert(contract.provider_scoring.provider_owned_scoring_policy === true, "供应商评分策略所有权边界错误");
assert(contract.drift_monitoring.provider_owned_thresholds === true, "漂移阈值不应固化在公开仓库");
assert(contract.parameter_lifecycle.automatic_promotion === false, "赛后结算不得触发自动参数晋升");
assert(contract.parameter_lifecycle.provider_state === "NOT_BUNDLED", "公开仓库不得声明已捆绑模型提供器");
assert(contract.compatibility.external_provider_required === true, "赛后结算未声明外部提供器依赖");
assert(migration.includes(hash), "赛后结算迁移中的契约哈希不匹配");
for (const table of ["postmatch_settlements", "evidence_scoring_items", "evidence_scoring_decisions", "provider_score_snapshots", "postmatch_drift_runs", "postmatch_drift_findings"]) {
  assert(migration.includes(table), `赛后结算账本缺失：${table}`);
}
assert(migration.includes("'provider_state', 'NOT_BUNDLED'") && migration.includes("'automatic_parameter_promotion', false"), "数据库契约未锁定公开提供器边界或人工晋升");
assert(persistence.includes("settle_postmatch_review") && persistence.includes("refresh_postmatch_monitoring"), "赛后结算持久化链路缺失");

for (const command of contract.commands) {
  assert(application.includes(`fn ${command}`), `应用服务缺失：${command}`);
  assert(commands.includes(`fn ${command}`), `Tauri 命令缺失：${command}`);
  assert(registry.includes(command), `Tauri 注册缺失：${command}`);
  assert(client.includes(command), `前端 API 缺失：${command}`);
}

console.log("赛后结算验证通过：公开提供器边界、不可变结算、人工证据判定和漂移记录链路完整。");
