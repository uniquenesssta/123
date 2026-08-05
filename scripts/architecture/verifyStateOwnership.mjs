import { readFileSync } from "node:fs";
import { join } from "node:path";
import {
  extractRepositoryLocation,
  matchesPathPattern,
  pathExists,
  readJson,
  readText,
  repositoryRoot,
} from "./lib/repository.mjs";
import { VerificationReport } from "./lib/report.mjs";

const report = new VerificationReport("状态所有权验证");
const contract = readJson("architecture/state-ownership.json");
const moduleContract = readJson("architecture/module-boundaries.json");

report.check(contract.contract_id === "football.state-ownership.v1", "状态所有权 contract_id 不正确");
report.check(contract.status === "ACTIVE", `状态所有权契约状态必须为 ACTIVE，当前为 ${contract.status}`);
report.check(contract.policy?.one_current_owner === true, "one_current_owner 必须启用");
report.check(contract.policy?.parallel_or_shadow_owner === false, "parallel_or_shadow_owner 必须禁用");
report.check(moduleContract.policy?.state_contract === "architecture/state-ownership.json", "模块边界契约未引用状态所有权契约");

const states = Array.isArray(contract.states) ? contract.states : [];
report.check(states.length === contract.count, `状态数量 ${states.length} 与声明 ${contract.count} 不一致`);
const ids = new Set();
const stateById = new Map();
for (const state of states) {
  report.check(typeof state.id === "string" && state.id.length > 0, "存在缺少 id 的状态");
  report.check(!ids.has(state.id), `状态 id 重复：${state.id}`);
  ids.add(state.id);
  stateById.set(state.id, state);
  report.check(typeof state.owner === "string" && state.owner.length > 0, `${state.id} 缺少 owner`);
  report.check(typeof state.scope === "string" && state.scope.length > 0, `${state.id} 缺少 scope`);
  report.check(Array.isArray(state.writers) && state.writers.length > 0, `${state.id} 缺少 writers`);
  report.check(Array.isArray(state.forbidden) && state.forbidden.length > 0, `${state.id} 缺少 forbidden`);
}

const externalOwners = new Set(["Windows Credential Manager"]);
const appStateOwner = stateById.get("tauri.app-state")?.owner ?? "";
const appStatePath = extractRepositoryLocation(appStateOwner);

function verifyOwnerSource(state) {
  if (externalOwners.has(state.owner)) return;

  if (state.owner.startsWith("AppState.")) {
    report.check(Boolean(appStatePath), `${state.id} 引用了 AppState 字段，但 tauri.app-state owner 无文件路径`);
    if (!appStatePath || !pathExists(appStatePath)) return;
    const field = state.owner.slice("AppState.".length);
    report.check(new RegExp(`\\b${field.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\b`).test(readText(appStatePath)), `${state.id} owner 字段不存在：${state.owner}`);
    return;
  }

  const location = extractRepositoryLocation(state.owner);
  report.check(Boolean(location), `${state.id} owner 无法解析为仓库文件或外部所有者：${state.owner}`);
  if (!location) return;
  report.check(pathExists(location), `${state.id} owner 文件不存在：${location}`);
  if (!pathExists(location)) return;

  const source = readFileSync(join(repositoryRoot, location), "utf8");
  const suffix = state.owner.split("::")[1] ?? "";
  const symbol = suffix.split(".")[0].trim();
  if (symbol) report.check(source.includes(symbol), `${state.id} owner 符号未在 ${location} 中找到：${symbol}`);
  if (state.owner.includes("localStorage")) report.check(source.includes("localStorage"), `${state.id} owner 未包含 localStorage`);

  for (const forbidden of state.forbidden) {
    if (forbidden.includes(" ") && !forbidden.includes("/")) continue;
    report.check(!matchesPathPattern(location, forbidden), `${state.id} owner ${location} 同时落入 forbidden ${forbidden}`);
  }
}

for (const state of states) {
  verifyOwnerSource(state);
  if (state.transition) {
    report.check(state.transition !== state.owner, `${state.id} transition 不得等于当前 owner`);
  }
}

report.check(stateById.has("browser.lifecycle"), "缺少 browser.lifecycle 状态");
report.check(stateById.has("application.model-registry"), "缺少 application.model-registry 状态");
report.check(stateById.has("persistence.pg-pool"), "缺少 persistence.pg-pool 状态");
report.check(stateById.has("tauri.app-state"), "缺少 tauri.app-state 状态");
report.check(stateById.has("os.openai-api-key"), "缺少 os.openai-api-key 状态");

report.finish(`${states.length} 个状态 id、唯一当前 owner 契约与 owner 源位置`);
