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
const application = text("crates/application/src/api_workspace.rs");
const persistence = text("crates/persistence-postgres/src/api_workspace.rs");
const commands = text("src-tauri/src/commands/api_workspace.rs");
const registry = text("src-tauri/src/lib.rs");
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
for (const artifact of v3.artifacts) assert(existsSync(join(root, artifact)), `AI问答v3制品不存在：${artifact}`);

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
assert(plainBody.includes("ApiProtocol::Responses"), "纯文本链缺少Responses请求");
assert(plainBody.includes("ApiProtocol::ChatCompletions"), "纯文本链缺少Chat Completions请求");
for (const forbidden of ["tools", "tool_choice", "json_schema", "max_output_tokens", "max_completion_tokens", "background", "store", "reasoning", "metadata"]) {
  assert(!plainBody.includes(`\"${forbidden}\"`), `纯文本请求不得发送${forbidden}`);
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
assert(commands.includes("CancellationToken::new"), "缺少请求取消令牌");
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
