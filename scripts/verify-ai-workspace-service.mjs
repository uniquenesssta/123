import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relativePath) =>
  fs.readFileSync(path.join(root, relativePath), "utf8").replace(/^\uFEFF/, "").replaceAll("\r\n", "\n");
const failures = [];
const check = (condition, message) => {
  if (!condition) failures.push(message);
};

const serviceRoot = "crates/application/src/services/ai_workspace";
const useCaseRoot = "crates/application/src/use_cases/ai_workspace";
const adapter = "crates/application/src/composition/adapters/ai_workspace.rs";
const legacyOwner = ["crates/application/src", "api_workspace.rs"].join("/");
const useCases = [
  "usage_totals",
  "list_sessions",
  "create_session",
  "archive_session",
  "read_session",
  "append_user_message",
  "append_assistant_bundle",
  "context",
  "read_operation",
  "read_generated_file",
  "apply_operation",
  "reject_operation",
];
const sessionUseCases = new Set(["usage_totals", "list_sessions", "create_session", "archive_session", "read_session", "append_user_message", "append_assistant_bundle", "read_generated_file"]);
const useCaseFile = name => sessionUseCases.has(name) ? `${useCaseRoot}/sessions/${name}.rs` : `${useCaseRoot}/${name}/mod.rs`;
const publicMethods = [
  "api_workspace_openai_usage_totals",
  "list_api_workspace_sessions",
  "create_api_workspace_session",
  "archive_api_workspace_session",
  "read_api_workspace_session",
  "append_api_workspace_user_message",
  "append_api_workspace_assistant_bundle",
  "api_workspace_context",
  "read_api_workspace_operation",
  "read_api_workspace_generated_file",
  "apply_api_workspace_operation",
  "reject_api_workspace_operation",
];

check(!fs.existsSync(path.join(root, legacyOwner)), `旧 AI Workspace owner 仍存在：${legacyOwner}`);
for (const file of [
  `${serviceRoot}/mod.rs`,
  `${serviceRoot}/service/mod.rs`,
  `${serviceRoot}/service/sessions.rs`,
  `${serviceRoot}/service/context.rs`,
  `${serviceRoot}/service/operations.rs`,
  `${serviceRoot}/facade/mod.rs`,
  `${serviceRoot}/facade/sessions.rs`,
  `${serviceRoot}/facade/context.rs`,
  `${serviceRoot}/facade/operations.rs`,
  `${useCaseRoot}/mod.rs`,
  `${useCaseRoot}/presets/mod.rs`,
  `${useCaseRoot}/read_attachments/mod.rs`,
  `${useCaseRoot}/apply_operation/dispatch.rs`,
  `${useCaseRoot}/apply_operation/payload.rs`,
  `${useCaseRoot}/apply_operation/metadata.rs`,
  adapter,
]) {
  check(fs.existsSync(path.join(root, file)), `缺少 AI Workspace 模块文件：${file}`);
}
for (const useCase of useCases) {
  check(fs.existsSync(path.join(root, useCaseFile(useCase))), `AI Workspace Use Case 缺少唯一职责文件：${useCase}`);
}

const service = [
  read(`${serviceRoot}/service/sessions.rs`),
  read(`${serviceRoot}/service/context.rs`),
  read(`${serviceRoot}/service/operations.rs`),
].join("\n");
const facade = [
  read(`${serviceRoot}/facade/sessions.rs`),
  read(`${serviceRoot}/facade/context.rs`),
  read(`${serviceRoot}/facade/operations.rs`),
].join("\n");
const library = read("crates/application/src/lib.rs");
const composition = read("crates/application/src/composition/application_composition.rs");
const applicationService = read("crates/application/src/service/application_service.rs");
const ports = read("crates/application/src/ports/ai_workspace/mod.rs");
const adapterText = read(adapter);
const commands = read("src-tauri/src/commands/api_workspace.rs");

check(!library.includes("mod api_workspace;"), "lib.rs 仍登记旧 AI Workspace owner");
check(composition.includes("ai_workspace: AiWorkspaceService"), "ApplicationComposition 未聚合 AiWorkspaceService");
check(composition.includes("ai_workspace: AiWorkspaceService::new()"), "组合根未构造唯一 AiWorkspaceService");
check(applicationService.includes("pub(crate) ai_workspace: AiWorkspaceService"), "ApplicationService 未持有 AiWorkspaceService");
check(applicationService.includes("ai_workspace: parts.ai_workspace"), "ApplicationService 未从组合根接收 AiWorkspaceService");
for (const useCase of useCases) check(service.includes(`fn ${useCase}`), `AiWorkspaceService 缺少方法：${useCase}`);
for (const method of publicMethods) check(facade.includes(`pub async fn ${method}`), `ApplicationService 兼容入口缺少：${method}`);
check((facade.match(/self\.ai_workspace/g) ?? []).length >= publicMethods.length, "AI Workspace facade 未统一委托 AiWorkspaceService");

for (const token of [
  "pub trait ApiWorkspaceSessionPort",
  "pub trait ApiWorkspaceOperationPort",
  "pub struct SerializedApiWorkspaceOperationResult",
  "async fn usage_totals(",
  "async fn create_session(",
  "async fn list_sessions(",
  "async fn read_session(",
  "async fn archive_session(",
  "async fn append_message(",
  "async fn append_assistant_bundle(",
  "async fn read_generated_file(",
  "async fn claim_operation(",
  "async fn complete_operation(",
  "async fn reject_operation(",
  "async fn read_operation(",
]) check(ports.includes(token), `AI Workspace Port 缺少真实能力：${token}`);
check(!ports.includes("serde_json::Value"), "AI Workspace Port 暴露裸 JSON Value");
check(adapterText.includes("impl ApiWorkspaceSessionPort for PersistenceStore"), "ApiWorkspaceSessionPort 未由 PersistenceStore 适配");
check(adapterText.includes("impl ApiWorkspaceOperationPort for PersistenceStore"), "ApiWorkspaceOperationPort 未由 PersistenceStore 适配");
for (const call of [
  "api_workspace_usage_totals",
  "create_api_workspace_session",
  "list_api_workspace_sessions",
  "read_api_workspace_session",
  "archive_api_workspace_session",
  "append_api_workspace_message",
  "append_api_workspace_assistant_bundle",
  "read_api_workspace_generated_file",
  "claim_api_workspace_operation",
  "complete_api_workspace_operation",
  "reject_api_workspace_operation",
  "read_api_workspace_operation",
]) check(adapterText.includes(call), `AI Workspace adapter 缺少持久化委托：${call}`);

const banned = ["football_persistence_postgres", "sqlx::", "PostgresStore", "PersistenceStore"];
for (const relativePath of [
  `${serviceRoot}/service/sessions.rs`,
  `${serviceRoot}/service/context.rs`,
  `${serviceRoot}/service/operations.rs`,
  `${serviceRoot}/facade/sessions.rs`,
  `${serviceRoot}/facade/context.rs`,
  `${serviceRoot}/facade/operations.rs`,
  ...useCases.map(useCaseFile),
  `${useCaseRoot}/apply_operation/dispatch.rs`,
  `${useCaseRoot}/apply_operation/payload.rs`,
  `${useCaseRoot}/apply_operation/metadata.rs`,
  `${useCaseRoot}/presets/mod.rs`,
  `${useCaseRoot}/read_attachments/mod.rs`,
]) {
  const text = read(relativePath);
  for (const token of banned) check(!text.includes(token), `${relativePath} 泄漏基础设施符号：${token}`);
}

const presets = read(`${useCaseRoot}/presets/mod.rs`);
for (const key of [
  "plain_chat",
  "match_research",
  "availability_verification",
  "lineup_player_cleanup",
  "player_profile_completion",
  "team_profile_completion",
  "file_structuring",
  "database_quality_audit",
  "custom_analysis",
]) check(presets.includes(`\"${key}\"`), `AI Workspace 预设兼容键丢失：${key}`);
for (const text of [
  "未知API协作预设：{key}",
  "普通文本提问与回答，不联网、不写库、不生成文件",
  "web_search_enabled: false",
  "allowed_operation_types: Vec::new()",
]) check(presets.includes(text), `AI Workspace 预设兼容语义丢失：${text}`);

const createSession = read(useCaseFile("create_session"));
check(createSession.includes("该API协作预设必须选择一场比赛"), "会话创建 required-match 错误语义丢失");
const appendUser = read(useCaseFile("append_user_message"));
check(appendUser.includes("请输入问题或选择附件"), "用户消息空输入错误语义丢失");
const context = read(`${useCaseRoot}/context/mod.rs`);
check(context.includes("API协作实体上下文必须同时提供有效的类型和ID"), "实体上下文错误语义丢失");
check(context.includes("limit: 200"), "通用上下文 200 球员上限被改变");

const operation = [
  read(`${useCaseRoot}/apply_operation/mod.rs`),
  read(`${useCaseRoot}/apply_operation/dispatch.rs`),
  read(`${useCaseRoot}/apply_operation/payload.rs`),
  read(`${useCaseRoot}/apply_operation/metadata.rs`),
].join("\n");
for (const operationType of [
  "add_player_name",
  "assign_player_position",
  "add_player_availability",
  "add_player_dynamic_tag",
  "add_player_ability_observation",
  "add_team_name",
  "update_team_profile",
]) check(operation.includes(`\"${operationType}\"`), `AI Workspace 数据库操作兼容类型丢失：${operationType}`);
for (const text of [
  "不允许的API数据库操作：{other}",
  "数据库提案缺少字段：{key}",
  "字段{key}不是有效UUID",
  "数据库提案缺少数值字段：{key}",
  "字段{key}时间无效",
  "字段{key}日期无效",
  "未知球员可用性状态：{value}",
  "metadata_json必须是JSON对象",
  "api_workspace_operation_id",
  "api_workspace_session_id",
  "api_workspace_rationale",
  "source_urls",
  'complete_operation(operation_id, "applied"',
  'complete_operation(operation_id, "failed"',
]) check(operation.includes(text), `AI Workspace Operation 兼容语义丢失：${text}`);
const reject = read(`${useCaseRoot}/reject_operation/mod.rs`);
check(reject.includes("用户拒绝该数据库提案"), "空拒绝理由默认值被改变");

const attachments = read(`${useCaseRoot}/read_attachments/mod.rs`);
for (const text of [
  "MAX_ATTACHMENT_FILES: usize = 5",
  "MAX_ATTACHMENT_TOTAL_BYTES: u64 = 5 * 1024 * 1024",
  "MAX_TEXT_ATTACHMENT_BYTES: usize = 1_500_000",
  "一次最多选择{MAX_ATTACHMENT_FILES}个附件",
  "附件总大小不能超过5 MiB",
  "附件不存在：",
  "附件缺少扩展名",
  "不支持的附件类型：.{extension}；仅支持 txt、md、json、csv、tsv、xlsx",
  "附件不是UTF-8文本：",
  "Excel附件读取任务失败：{error}",
  "[attachment content truncated by the desktop client]",
  "read_workbook_for_api",
  "Sha256::digest",
]) check(attachments.includes(text), `AI Workspace 附件兼容语义丢失：${text}`);

for (const token of [
  "api_workspace_preset_spec",
  "api_workspace_preset_specs",
  "api_workspace_presets",
  "read_api_workspace_attachments",
  "ApiWorkspacePresetSpec",
]) check(library.includes(token), `lib.rs 公共 AI Workspace 出口丢失：${token}`);
for (const token of [
  "list_api_workspace_sessions",
  "read_api_workspace_session",
  "send_api_workspace_message",
  "archive_api_workspace_session",
  "AI问答不支持附件；请使用 Excel 工作包维护资料",
  "execute_plain_text",
]) check(commands.includes(token), `Tauri AI Workspace 兼容边界丢失：${token}`);
for (const forbidden of ["apply_api_workspace_operation", "reject_api_workspace_operation"]) {
  check(!commands.includes(`pub async fn ${forbidden}`), `AT3 不应重新暴露已禁用的 Tauri 命令：${forbidden}`);
}

// R9-09: one Session use-case namespace and one PostgreSQL persistence boundary.
const sessionRoot = `${useCaseRoot}/sessions`;
const sessionExports = read(`${sessionRoot}/mod.rs`);
for (const name of sessionUseCases) {
  check(sessionExports.includes(`mod ${name};`) && sessionExports.includes(`pub(crate) use ${name}::execute as ${name};`), `Session用例出口缺少直接登记/导出：${name}`);
  check(!fs.existsSync(path.join(root, useCaseRoot, name)), `旧Session用例目录残留：${name}`);
  check(service.includes(`ai_workspace::sessions::${name}(`) && !service.includes(`ai_workspace::${name}::execute`), `Service未直连唯一Session用例：${name}`);
}
check(!/\b(?:fn|struct|impl)\b/.test(sessionExports), "Session出口不得承载实现或转发");
const pgRoot = "crates/persistence-postgres/src/adapters/ai_workspace/sessions";
const pgNames = ["assistant_bundle", "catalog", "generated_files", "mapping", "messages", "usage", "validation"];
const pg = Object.fromEntries(pgNames.map(name => [name, read(`${pgRoot}/${name}.rs`).split("#[cfg(test)]")[0]]));
const pgExports = read(`${pgRoot}/mod.rs`);
check(read("crates/persistence-postgres/src/adapters/mod.rs").includes("mod ai_workspace;") && read("crates/persistence-postgres/src/adapters/ai_workspace/mod.rs").trim() === "mod sessions;", "PG会话目录未由原adapter根唯一登记");
check(pgNames.every(name => pgExports.includes(`mod ${name};`)) && !/\b(?:fn|struct|impl)\b/.test(pgExports), "PG会话出口必须只登记具名职责");
const legacyPg = read("crates/persistence-postgres/src/api_workspace.rs");
for (const method of ["api_workspace_usage_totals", "create_api_workspace_session", "list_api_workspace_sessions", "archive_api_workspace_session", "read_api_workspace_session", "read_api_workspace_session_summary", "append_api_workspace_message", "append_api_workspace_assistant_bundle", "read_api_workspace_generated_file"]) {
  const count = (Object.values(pg).join("\n").match(new RegExp(`async fn ${method}\\(`, "g")) ?? []).length;
  check(count === 1 && !legacyPg.includes(`async fn ${method}(`), `会话持久化方法必须唯一且清除旧实现：${method}`);
}
for (const method of ["recover_interrupted_api_workspace_operations", "claim_api_workspace_operation", "complete_api_workspace_operation", "reject_api_workspace_operation", "read_api_workspace_operation"]) check(legacyPg.includes(`pub async fn ${method}(`), `原提案状态流转不得混入Session迁移：${method}`);
check(pg.catalog.includes("use crate::api_workspace::operation_from_row;") && pg.assistant_bundle.includes("use crate::api_workspace::validate_operation_draft;") && legacyPg.includes("pub(crate) fn operation_from_row") && legacyPg.includes("pub(crate) fn validate_operation_draft"), "会话历史/回答事务必须复用唯一提案映射/校验");
for (const token of ["WHERE session.status = 'active'", "ORDER BY session.updated_at DESC, session.id DESC", "limit.clamp(1, 500)", "SET status = 'archived', updated_at = now()", "WHERE id = $1 AND status = 'active'", "api_workspace_session_archived", "rows.iter().map(session_from_row).collect()", ".bind(draft.profile_id.trim())", ".bind(draft.title.trim())"]) check(pg.catalog.includes(token), `会话目录原查询/归档/绑定规则缺少${token}`);
check((pg.catalog.match(/ORDER BY created_at, id/g) ?? []).length === 3, "会话详情三类历史必须保持created_at/id原顺序");
for (const token of ["validate_message_draft(draft)?;", ".bind(draft.role.trim())", ".bind(&draft.content)", ".bind(&draft.attachments)", ".bind(&draft.token_usage)", "message_from_row(&row)"]) check(pg.messages.includes(token), `消息写入原绑定/映射缺少${token}`);
check(!pg.messages.includes(".begin()") && pg.messages.indexOf(".fetch_one(&self.pool)") < pg.messages.indexOf("UPDATE ai_workspace.sessions SET updated_at"), "普通消息原两次autocommit及先插入后更新时间顺序不得暗改");
const bundleOrder = ["validate_message_draft(message)?", 'message.role != "assistant"', "self.pool.begin().await?", "INSERT INTO ai_workspace.messages", "for draft in operations", "draft.session_id != message.session_id", "validate_operation_draft(draft)?", "sha256_json(&json!", "INSERT INTO ai_workspace.operation_proposals", "for draft in files", "validate_generated_file_draft(draft)?", "INSERT INTO ai_workspace.generated_files", "UPDATE ai_workspace.sessions", "write_audit_event(", "tx.commit().await?", "self.read_api_workspace_session(message.session_id).await"].map(token => pg.assistant_bundle.indexOf(token));
check(bundleOrder.every((position,index) => position>=0 && (index===0 || position>bundleOrder[index-1])), "回答事务原校验/消息/提案/文件/更新时间/审计/提交/回读顺序漂移");
check((pg.assistant_bundle.match(/\.execute\(&mut \*tx\)/g) ?? []).length === 4 && pg.assistant_bundle.includes("API生成文件与回答会话身份不一致") && pg.assistant_bundle.includes("API数据库提案与回答会话身份不一致"), "回答四写入必须共用原事务，身份不匹配不能绕过");
check(pg.assistant_bundle.includes('"api_workspace_response_recorded"') && pg.assistant_bundle.includes('"provider_response_id": message.provider_response_id') && pg.assistant_bundle.includes(".bind(draft.content_sha256.trim())"), "回答审计/提供方ID/既有文件hash绑定必须保持");
for (const token of ['"user" | "assistant" | "system"', "draft.content.chars().count() > 100_000", "draft.filename.chars().count() > 120", "draft.content.len() > 2 * 1024 * 1024", '"text/plain" | "text/markdown" | "application/json" | "text/csv"']) check(pg.validation.includes(token), `会话载荷原验证边界缺少${token}`);
for (const token of ["session_from_row", "message_from_row", "file_from_row"]) check(pg.mapping.includes(`pub(super) fn ${token}`) && !legacyPg.includes(`fn ${token}`), `会话Row映射必须唯一且限定目录可见：${token}`);
for (const source of Object.values(pg)) for (const token of ["CREATE TABLE", "DELETE FROM ai_workspace", "TRUNCATE", "DROP TABLE", "reqwest", "football_research_gateway", "static "]) check(!source.includes(token), `会话持久化越界引入新副作用/状态：${token}`);
const sessionTests = read(`${sessionRoot}/tests.rs`);
for (const name of ["session_validation_precedes_database_factory_polling", "user_message_keeps_trimmed_text_metadata_and_persistence_error", "session_and_bundle_ports_keep_identity_payload_order_and_errors", "usage_totals_keep_formal_then_workspace_order_and_saturating_counts"]) check(sessionTests.includes(`fn ${name}`), `原Application unit target缺会话契约：${name}`);
const validationTests = read(`${pgRoot}/validation.rs`);
for (const name of ["message_roles_keep_exact_values_and_empty_content_compatibility", "message_limit_counts_unicode_characters_and_keeps_original_error", "file_policy_preserves_name_media_type_and_raw_whitespace_rules", "file_limits_preserve_character_names_byte_payloads_and_supplied_hash"]) check(validationTests.includes(`fn ${name}`), `原Persistence unit target缺载荷边界：${name}`);
const stale = [];
const walk = (directory) => {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (entry.isFile() && /\.(?:rs|mjs|js|ts)$/.test(entry.name)) {
      const relative = path.relative(root, full).replaceAll("\\", "/");
      if (relative === "scripts/verify-ai-workspace-service.mjs") continue;
      const text = fs.readFileSync(full, "utf8");
      if (text.includes(legacyOwner)) stale.push(relative);
    }
  }
};
for (const scanRoot of ["crates/application/src", "scripts"]) walk(path.join(root, scanRoot));
check(stale.length === 0, `仍有源码/验证器硬编码旧 AI Workspace owner：${stale.join(", ")}`);

const packageDefinition = JSON.parse(read("package.json"));
const frontendVerifier = read("scripts/verify-frontend.mjs");
check(packageDefinition.scripts?.["verify:ai-workspace-service"] === "node scripts/verify-ai-workspace-service.mjs", "package.json 未登记 AI Workspace 专项门禁");
check(packageDefinition.scripts?.["verify:architecture"]?.includes("verify-ai-workspace-service.mjs"), "verify:architecture 未接入 AI Workspace 门禁");
check(frontendVerifier.includes('"verify-ai-workspace-service.mjs"'), "完整 frontend 未接入 AI Workspace 门禁");

if (failures.length) {
  console.error("AI Workspace Service 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}
console.log("AI Workspace Service 验证通过：12 个 Application 用例由唯一 AiWorkspaceService 编排，R9-09 Session 用例/PG会话查询与回答事务唯一owner，Context/Operation/Presets/Attachments 职责保持，Ports/PersistenceStore 适配完整且旧 owner 清零。");
