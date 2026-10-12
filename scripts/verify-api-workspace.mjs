import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { isVersionAtLeast } from "./version.mjs";
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outerRoot = resolve(root, "..");
const failures = [];
const assert = (condition, message) => { if (!condition) failures.push(message); };
const text = (path) => readFileSync(join(root, path), "utf8").replace(/\r\n?/g, "\n");
const json = (path) => JSON.parse(text(path));
const hash = (path) => createHash("sha256").update(text(path), "utf8").digest("hex");
const slice = (source, start, end) => {
  const begin = source.indexOf(start);
  if (begin < 0) return "";
  const finish = source.indexOf(end, begin);
  return finish < 0 ? source.slice(begin) : source.slice(begin, finish);
};

const v1 = json("contracts/api-workspace-contract.json");
const v2 = json("contracts/api-workspace-contract-v2.json");
const v3 = json("contracts/api-workspace-contract-v3.json");
const schema = json("schemas/api-workspace-contract-v3.schema.json");
const packageJson = json("package.json");
const tauri = json("src-tauri/tauri.conf.json");
const migrationV1 = text("crates/persistence-postgres/migrations/0019_api_workspace.sql");
const migrationV2 = text("crates/persistence-postgres/migrations/0020_team_player_management.sql");
const gateway = text("crates/research-gateway/src/client.rs");
const response = text("crates/research-gateway/src/response.rs");
const application = text("crates/application/src/use_cases/ai_workspace/presets/mod.rs");
const persistence = [text("crates/persistence-postgres/src/api_workspace.rs"), ...["catalog", "messages", "assistant_bundle", "generated_files", "usage"].map(name => text(`crates/persistence-postgres/src/adapters/ai_workspace/sessions/${name}.rs`))].join("\n");
const commands = text("src-tauri/src/commands/api_workspace.rs");
const requestRegistryPath = "crates/application/src/use_cases/ai_workspace/cancellation/registry.rs";
const requestRegistry = text(requestRegistryPath);
const requestRegistryExport = text("crates/application/src/use_cases/ai_workspace/cancellation/mod.rs");
const requestRegistryTests = text("crates/application/src/use_cases/ai_workspace/cancellation/tests.rs");
const appState = text("src-tauri/src/bootstrap/state.rs");
const applicationExport = text("crates/application/src/lib.rs");
const requestState = json("architecture/state-ownership.json").states.find(state => state.id === "tauri.api-requests");
const registry = text("src-tauri/src/bootstrap/command_registry.rs");
const client = text("src/api/client.ts");
const page = text("src/pages/apiWorkspace.ts");
const main = text("src/main.ts");
const shell = text("src/app/shell.ts");
const navigation = text("src/app/navigation.ts");
const readme = text("README.md");

assert(v1.contract_id === "football.api-workspace-contract.v1", "历史v1契约ID被破坏");
assert(v2.contract_id === "football.api-workspace-contract.v2", "历史v2契约ID被破坏");
assert(migrationV1.includes(hash("contracts/api-workspace-contract.json")), "0019历史契约哈希不一致");
assert(migrationV2.includes(hash("contracts/api-workspace-contract-v2.json")), "0020历史契约哈希不一致");

assert(schema.$id === v3.contract_id, "AI问答v3 Schema ID不一致");
for (const key of schema.required ?? []) assert(Object.hasOwn(v3, key), `AI问答v3契约缺少${key}`);
assert(v3.contract_version === "3.0.0", "AI问答v3契约版本错误");
assert(v3.baseline_source_version === "0.13.5", "阶段1基线必须是0.13.5");
assert(v3.release_version === "0.14.0", "阶段1发布版本必须是0.14.0");
assert(v3.stage === "H_PRE_STAGE_1", "阶段标识错误");
assert(isVersionAtLeast(packageJson.version, "0.14.0"), "当前项目版本早于AI问答阶段1版本0.14.0");
assert(tauri.version === packageJson.version, "Tauri版本未同步");
assert(readme.includes(`当前版本 **${packageJson.version}**`), "根README当前版本未同步");
assert(readme.includes("## 0.14.0 变更记录"), "根README缺少0.14.0变更记录");
const migratedApplicationArtifact = ["crates/application/src", "api_workspace.rs"].join("/");
const migratedApplicationOwners = [
  "crates/application/src/services/ai_workspace/mod.rs",
  "crates/application/src/use_cases/ai_workspace/mod.rs",
  "crates/application/src/ports/ai_workspace/mod.rs",
  "crates/application/src/composition/adapters/ai_workspace.rs",
];
for (const artifact of v3.artifacts) {
  if (artifact === migratedApplicationArtifact) {
    for (const owner of migratedApplicationOwners) {
      assert(existsSync(join(root, owner)), `AI问答v3迁移后制品不存在：${owner}`);
    }
  } else {
    assert(existsSync(join(root, artifact)), `AI问答v3制品不存在：${artifact}`);
  }
}

for (const command of v3.active_commands) {
  assert(commands.includes(`fn ${command}`), `Tauri命令缺少${command}`);
  assert(registry.includes(`commands::${command}`), `Tauri注册表缺少${command}`);
  assert(client.includes(`"${command}"`), `前端客户端缺少${command}`);
}
for (const command of v3.disabled_legacy_commands) {
  assert(!commands.includes(`fn ${command}`), `阶段1后端仍暴露旧命令：${command}`);
  assert(!registry.includes(`commands::${command}`), `阶段1仍注册旧命令：${command}`);
  assert(!client.includes(`"${command}"`), `阶段1前端仍调用旧命令：${command}`);
}

const plainBody = slice(gateway, "fn build_plain_text_request_body", "fn build_structured_request_body");
const plainResponsesBody = text("crates/research-gateway/src/plain_chat/responses/request.rs").split("#[cfg(test)]")[0];
const plainChatBody = text("crates/research-gateway/src/plain_chat/chat_completions/request.rs").split("#[cfg(test)]")[0];
assert(plainBody.includes("ApiProtocol::Responses"), "纯文本链缺少Responses请求");
assert(plainBody.includes("ApiProtocol::ChatCompletions"), "纯文本链缺少Chat Completions请求");
for (const forbidden of ["tools", "tool_choice", "json_schema", "max_output_tokens", "max_completion_tokens", "background", "store", "reasoning", "metadata"]) {
  assert(![plainBody, plainResponsesBody, plainChatBody].some(source => source.includes(`\"${forbidden}\"`)), `纯文本请求不得发送${forbidden}`);
}
assert(response.includes("parse_plain_text_success_response"), "缺少纯文本响应解析");
assert(gateway.includes("execute_plain_text_with_sink"), "缺少纯文本执行入口");

const send = slice(commands, "pub async fn send_api_workspace_message", "pub async fn cancel_api_workspace_request");
assert(send.includes("AI问答不支持附件"), "新请求未阻断附件");
assert(send.includes("execute_plain_text"), "Tauri发送链未使用纯文本网关");
assert(send.includes("include_context"), "缺少可选只读上下文");
assert(send.includes('"ai_chat"'), "AI问答运行日志未使用独立子系统");
assert(send.includes('"message_chars"') && send.includes('"response_chars"'), "运行日志缺少文本长度");
assert(!send.includes('"prompt"') && !send.includes('"schema"'), "运行日志不得记录问题正文或Schema");
assert(requestRegistry.includes("CancellationToken::new"), "Application注册表缺少请求取消令牌");
assert(requestRegistryExport.includes("pub use registry::{AiWorkspaceRequestLease, AiWorkspaceRequestRegistry};"), "取消目录必须显式导出注册表与清理守卫");
assert(!/\b(?:fn|struct|impl)\b/.test(requestRegistryExport), "取消目录出口不得承载运行时实现");
assert(applicationExport.includes("use_cases::ai_workspace::cancellation::{") && applicationExport.includes("AiWorkspaceRequestLease, AiWorkspaceRequestRegistry"), "Application缺少取消生命周期出口");
assert(appState.includes("pub api_workspace_requests: AiWorkspaceRequestRegistry") && appState.includes("api_workspace_requests: AiWorkspaceRequestRegistry::default()"), "AppState必须持有每个应用独立的Application注册表");
assert(!appState.includes("HashMap") && !commands.includes("CancellationToken::new") && !/api_workspace_requests\s*\.lock\(/.test(commands), "Tauri不得保留取消map或直接锁操作");
assert(requestState?.owner === `${requestRegistryPath}::AiWorkspaceRequestRegistry` && JSON.stringify(requestState.writers) === JSON.stringify([requestRegistryPath]), "取消状态唯一owner/writer登记不一致");
const registrationIndex = send.indexOf(".register(request_id)");
assert(registrationIndex >= 0 && registrationIndex < send.indexOf("gateway_for(Some(profile_id))") && registrationIndex < send.indexOf(".create_api_workspace_session("), "请求ID必须在profile/会话副作用之前注册");
assert(/execute_plain_text\(&request, registration\.cancellation\(\)\)\s*\.await;\s*drop\(registration\);\s*let execution = match execution_result/.test(send), "网关必须使用守卫令牌，并在成功/失败响应处理前释放登记");
const cancel = slice(commands, "pub async fn cancel_api_workspace_request", "pub async fn archive_api_workspace_session");
assert(cancel.includes("let request_id = request_id.trim();") && cancel.includes('Err("AI问答请求 ID 不能为空".to_string())') && cancel.includes("Ok(state.api_workspace_requests.cancel(request_id))"), "取消命令必须保留trim/空ID错误并委托注册表");
const registration = slice(requestRegistry, "pub fn register", "pub fn cancel");
assert(registration.includes("requests.entry(request_id.to_owned())") && registration.includes('Entry::Occupied(_) => Err("AI问答请求 ID 已在使用")') && registration.includes("Entry::Vacant(entry) =>"), "重复请求ID不得替换活动令牌");
assert(!registration.includes("requests.insert("), "禁止先覆盖map后拒绝重复ID");
const cancellation = slice(requestRegistry, "pub fn cancel", "fn lock");
assert(/let token = self\.lock\(\)\.get\(request_id\)\.cloned\(\);\s*if let Some\(token\) = token/.test(cancellation) && cancellation.includes("token.cancel();"), "取消必须在释放map锁后通知已有令牌");
assert(requestRegistry.includes("sync::{Arc, Mutex, MutexGuard}") && !requestRegistry.includes(".await") && requestRegistry.includes("unwrap_or_else(|poisoned| poisoned.into_inner())"), "同步清理只能持有短时map锁并安全处理poison");
assert(!/#\[derive\([^\]]*Clone[^\]]*\)\]\s*pub struct AiWorkspaceRequestLease/.test(requestRegistry) && !requestRegistry.includes("impl Clone for AiWorkspaceRequestLease"), "请求清理守卫必须保持唯一不可克隆的owner");
assert(/impl Drop for AiWorkspaceRequestLease\s*\{\s*fn drop\(&mut self\)\s*\{\s*self\.registry\.lock\(\)\.remove\(&self\.request_id\);\s*self\.cancellation\.cancel\(\);/.test(requestRegistry), "所有退出路径必须先移除登记、释放锁，再取消令牌");
for (const test of [
  "registry_clones_share_requests_but_application_instances_are_isolated",
  "duplicate_registration_preserves_the_running_requests_cancellation",
  "cancellation_and_cleanup_leave_other_requests_active",
  "completed_request_releases_its_id_for_a_fresh_uncancelled_request",
  "preparation_error_releases_the_request_without_reaching_execution",
  "aborting_a_request_task_cancels_its_token_and_releases_its_id",
  "panic_unwinding_cleans_up_the_request_lease",
  "concurrent_duplicate_ids_have_one_owner_and_one_cancellable_token",
]) assert(requestRegistryTests.includes(`fn ${test}(`), `原Application unit target缺少生命周期覆盖：${test}`);
assert(commands.includes("archive_api_workspace_session"), "缺少会话归档命令");

assert(persistence.includes("WHERE session.status = 'active'"), "常用会话列表未过滤归档项");
assert(persistence.includes("SET status = 'archived'"), "会话删除未采用归档");
assert(persistence.includes("api_workspace_session_archived"), "会话归档缺少审计");
assert(application.includes('"plain_chat"'), "应用层缺少通用纯文本预设");
assert(application.includes("普通文本提问与回答，不联网、不写库、不生成文件"), "应用层未明确纯文本边界");

assert(navigation.includes('page: "api_workspace"') && navigation.includes('label: "AI 问答"'), "AI问答未进入常用主导航");
for (const required of ["搜索会话", "取消请求", "复制回答", "附加当前只读上下文", "历史结构化记录（只读）"]) {
  assert(page.includes(required), `AI问答页面缺少：${required}`);
}
for (const forbidden of ["choose-api-workspace-attachments", "apply-api-workspace-operation", "reject-api-workspace-operation", "export-api-workspace-file", "Token用量", "来源链接"]) {
  assert(!page.includes(forbidden), `AI问答常用页面仍暴露旧功能：${forbidden}`);
}
assert(main.includes("apiWorkspaceSessionSearch"), "会话搜索状态未接入");
assert(main.includes("apiWorkspaceDraftMessage"), "未发送草稿未保留");
assert(main.includes("cancelApiWorkspaceRequest"), "前端未接通取消请求");
assert(main.includes("archiveApiWorkspaceSession"), "前端未接通会话归档");
assert(v3.compatibility.historical_proposals_preserved === true, "历史提案必须保留");
assert(v3.compatibility.historical_generated_files_preserved === true, "历史生成文件必须保留");
assert(v3.compatibility.formal_p4_research_unchanged === true, "P4正式研究边界必须保持");
assert(v3.compatibility.database_migration_required === false, "阶段1不应新增数据库迁移");
assert(v3.compatibility.integration_point_h_started === false, "不得提前进入接入点H");

if (failures.length) {
  console.error("AI问答阶段1契约验证失败：");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}
console.log("AI问答阶段1契约验证通过：双协议纯文本、只读上下文、取消、会话归档、历史兼容和P4隔离均已锁定。");
