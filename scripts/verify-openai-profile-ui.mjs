import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outerRoot = resolve(root, "..");
const failures = [];
const assert = (condition, message) => { if (!condition) failures.push(message); };
const read = (path) => readFileSync(join(root, path), "utf8");
const json = (path) => JSON.parse(read(path));
const versionAtLeast = (actual, minimum) => {
  const left = actual.split(".").map(Number);
  const right = minimum.split(".").map(Number);
  for (let index = 0; index < 3; index += 1) {
    if (left[index] !== right[index]) return left[index] > right[index];
  }
  return true;
};

const contract = json("contracts/openai-profile-ui-contract.json");
const schema = json("schemas/openai-profile-ui-contract.schema.json");
const packageJson = json("package.json");
const tauri = json("src-tauri/tauri.conf.json");
const cargo = read("Cargo.toml");
const frontend = read("src/pages/openai.ts");
const client = read("src/api/client.ts");
const main = read("src/main.ts");
const shell = read("src/app/shell.ts");
const navigation = read("src/app/navigation.ts");
const commandRegistry = read("src-tauri/src/bootstrap/command_registry.rs");
const desktopState = read("src-tauri/src/bootstrap/state.rs");
const commands = read("src-tauri/src/commands/openai.rs");
const store = read("src-tauri/src/openai_profiles.rs");
const credentialRoot = "crates/research-gateway/src/credentials";
const credentialExports = read(`${credentialRoot}/mod.rs`);
const credentialKey = read(`${credentialRoot}/key.rs`);
const credentialProvider = read(`${credentialRoot}/provider.rs`);
const credentialStore = read(`${credentialRoot}/store.rs`);
const credentialWindows = read(`${credentialRoot}/windows.rs`);
const credentialBlob = read(`${credentialRoot}/blob.rs`);
const credentialError = read(`${credentialRoot}/error.rs`);
const credentialRedaction = read(`${credentialRoot}/redaction.rs`);
const credentialSources = [credentialExports, credentialKey, credentialProvider, credentialStore, credentialWindows, credentialBlob, credentialError, credentialRedaction];
const gatewayExports = read("crates/research-gateway/src/lib.rs");
const moduleContract = json("architecture/module-boundaries.json");
const gateway = read("crates/research-gateway/src/client.rs");
const parser = read("crates/research-gateway/src/api_example.rs");
const gatewayConfig = read("crates/research-gateway/src/config.rs");
const readme = readFileSync(join(root, "README.md"), "utf8");

function initializerSlice(source, startMarker, endMarker) {
  const start = source.indexOf(startMarker);
  if (start < 0) return "";
  const end = source.indexOf(endMarker, start);
  return end < 0 ? "" : source.slice(start, end);
}

function fieldCount(source, field) {
  return (source.match(new RegExp(`\\b${field}\\s*:`, "g")) ?? []).length;
}

assert(schema.$id === contract.schema_version, "兼容API配置UI契约Schema版本不一致");
for (const key of schema.required ?? []) assert(Object.hasOwn(contract, key), `兼容API配置UI契约缺少${key}`);
assert(contract.contract_key === "openai-profile-ui", "兼容API配置UI契约键错误");
assert(contract.contract_version === "1.1.0", "兼容API配置UI契约版本错误");
assert(contract.release_version === "0.10.2", "兼容API配置UI交付版本错误");
assert(JSON.stringify(contract.provider_scope.supported) === JSON.stringify(["openai_compatible"]), "配置页必须限定为OpenAI-compatible协议族");
assert(contract.provider_scope.official_openai_only === false, "不得把兼容API错误限制为OpenAI官方地址");
assert(contract.provider_scope.multi_provider_switching === false, "不得引入多服务商插件切换层");
assert(JSON.stringify(contract.protocols.supported) === JSON.stringify(["responses", "chat_completions"]), "协议支持范围错误");
assert(JSON.stringify(contract.protocols.formal_research) === JSON.stringify(["responses"]), "P4正式研究协议必须锁定Responses");
assert(versionAtLeast(packageJson.version, "0.10.2"), "当前版本早于兼容API配置UI首次交付版本0.10.2");
assert(tauri.version === packageJson.version, "Tauri版本与package.json不一致");
assert(cargo.includes(`version = "${packageJson.version}"`), "Cargo workspace版本不一致");
assert(readme.includes(`版本 **${packageJson.version}**`), "README版本不一致");

for (const artifact of contract.artifacts) assert(existsSync(join(root, artifact)), `缺少兼容API配置制品：${artifact}`);
for (const command of contract.commands) {
  assert(client.includes(`"${command}"`), `前端API未调用${command}`);
  assert(commandRegistry.includes(`commands::${command}`), `Tauri未注册${command}`);
  assert(commands.includes(`fn ${command}`), `命令源码缺少${command}`);
}

assert(navigation.includes('page: "openai"') && navigation.includes('label: "兼容 API"'), "侧栏缺少兼容API入口");
assert(main.includes('case "openai"'), "页面路由缺少兼容API设置");
assert(frontend.includes("API Example 实时解析"), "页面缺少API Example实时解析入口");
assert(frontend.includes("多个 OpenAI-compatible 配置档案"), "页面未说明兼容API多配置能力");
assert(frontend.includes("完整请求端点"), "页面缺少可编辑请求端点");
assert(frontend.includes("API Key"), "页面缺少API Key编辑入口");
assert(frontend.includes("保存并测试"), "页面缺少连接测试入口");
assert(frontend.includes("Responses") && frontend.includes("Chat Completions"), "页面缺少双协议选择");
assert(main.includes("parseOpenAiApiExampleNow"), "前端缺少API Example实时解析流程");
assert(main.includes("openAiApiExampleTimer"), "API Example解析未使用防抖");
assert(client.includes("parseOpenAiApiExample"), "前端客户端缺少API Example解析命令");

assert(desktopState.includes("openai-profiles.json"), "配置元数据未使用独立本机文件");
assert(store.includes("credential_target(profile_id)"), "密钥未按配置档案隔离");
assert(store.includes("api_key_mask"), "已保存密钥未返回掩码状态");
assert(store.includes("connection_settings_changed"), "连接参数变化后未失效旧测试状态");
assert(store.includes("*state = previous_state"), "凭据写入或删除失败时未回滚配置元数据");
assert(store.includes("parse_api_example"), "保存配置前未执行Rust确定性解析与脱敏");
assert(store.includes("sanitized_example"), "API Example未在持久化前脱敏");
assert(!store.includes("api_key: String"), "持久化配置结构疑似保存明文API Key");
assert(!/derive\([^)]*Debug[^)]*\)\]\s*pub struct OpenAiProfileDraft/s.test(store), "包含API Key的输入结构不得派生Debug");
assert(credentialWindows.includes("CredWriteW"), "Windows凭据管理器缺少写入能力");
assert(credentialWindows.includes("CredDeleteW"), "Windows凭据管理器缺少删除能力");
assert(credentialWindows.includes("CredReadW"), "Windows凭据管理器缺少读取能力");

assert(parser.includes("/chat/completions") && parser.includes("/responses"), "Rust解析器缺少Responses或Chat Completions识别");
assert(credentialRedaction.includes('const API_KEY_PLACEHOLDER: &str = "YOUR_API_KEY";'), "Rust解析器缺少持久化脱敏占位符");
assert(parser.includes("api.gptsapi.net/v1/responses"), "Rust解析器缺少用户提供兼容API回归样本");
assert(parser.includes("prefers_responses_when_markdown_contains_two_examples"), "双示例默认选择Responses的回归测试缺失");
assert(gatewayConfig.includes("request_endpoint"), "网关配置缺少完整请求端点覆盖");
assert(gatewayConfig.includes("token_limit_field"), "网关配置缺少Token字段适配");
assert(gateway.includes("post_json(&endpoint"), "连接测试未向配置端点发送最小POST请求");
assert(gateway.includes("ApiProtocol::ChatCompletions"), "连接测试缺少Chat Completions响应验证");
assert(gateway.includes("P4正式联网研究仅支持Responses协议"), "正式研究未隔离Chat Completions");

// R9-02: credentials and persisted-example redaction have one authoritative owner.
assert(!existsSync(join(root, "crates/research-gateway/src/credentials.rs")), "旧credentials.rs必须删除，不能保留重复owner或转发壳");
assert(!/\b(?:fn|struct|trait|impl)\b/.test(credentialExports), "credentials出口不得承载业务实现");
for (const token of ["pub use key::ApiKey;", "pub use provider::{ApiKeyProvider, DefaultApiKeyProvider};", "pub use store::{delete_windows_api_key, save_windows_api_key, windows_api_key_exists};", "pub(crate) use redaction::{is_placeholder_key, sanitized_api_example};"]) assert(credentialExports.includes(token), `Credentials出口缺少${token}`);
for (const token of ["delete_windows_api_key, save_windows_api_key, windows_api_key_exists, ApiKey, ApiKeyProvider,", "DefaultApiKeyProvider,"]) assert(gatewayExports.includes(token), "原公开凭据接口必须保持");
assert(moduleContract.rust.ports.ApiKeyProvider.owner === `${credentialRoot}/provider.rs`, "ApiKeyProvider清单owner必须与唯一提供者一致");
for (const token of ["pub struct ApiKey(String);", "let mut value = Zeroizing::new(value);", "let mut normalized = Zeroizing::new(value.trim().to_string());", "value.zeroize();", "normalized.len() > 2_560", "normalized.chars().any(char::is_whitespace)", "Ok(Self(std::mem::take(&mut *normalized)))", "self.0.zeroize();", 'formatter.write_str("ApiKey([REDACTED])")']) assert(credentialKey.includes(token), `密钥生命周期兼容/清理边界缺少${token}`);
assert(!/derive\([^)]*(?:Debug|Serialize|Deserialize|Clone)[^)]*\)\]\s*pub struct ApiKey/.test(credentialKey), "密钥不得派生可泄露或复制明文的接口");
for (const token of ["pub trait ApiKeyProvider: Send + Sync", "async fn load(&self, config: &CredentialConfig) -> Result<ApiKey, GatewayError>", 'config.deployment_mode != "server"', "CredentialMode::WindowsCredentialManager", "load_windows_credential(&config.credential_target)", "std::env::var(&config.environment_variable)", "ApiKey::new(value)"]) assert(credentialProvider.includes(token), `凭据来源/桌面隔离边界缺少${token}`);
const saveCredential = initializerSlice(credentialStore, "pub fn save_windows_api_key(", "pub fn delete_windows_api_key(");
assert(saveCredential.indexOf("Zeroizing::new(value)") >= 0 && saveCredential.indexOf("Zeroizing::new(value)") < saveCredential.indexOf("validate_credential_target(target)?"), "凭据目标拒绝前必须持有可清理输入，失败不能遗留明文副本");
assert(saveCredential.indexOf("validate_credential_target(target)?") < saveCredential.indexOf("ApiKey::new("), "凭据目标错误必须保持原优先级");
for (const token of ["target.len() <= 240", "byte.is_ascii_alphanumeric()", "b'-' | b'_' | b'/' | b'.'", "write_windows_credential(target, &key)", "delete_windows_credential(target)", "windows_credential_exists(target)"]) assert(credentialStore.includes(token), `凭据目标/操作兼容边界缺少${token}`);
for (const source of credentialSources.filter(source => source !== credentialWindows)) assert(!/\bCred(?:ReadW|WriteW|DeleteW|Free)\b/.test(source), "Windows凭据系统调用只能由windows适配器持有");
for (const token of ["CRED_TYPE_GENERIC", "CRED_PERSIST_LOCAL_MACHINE", "bytes.zeroize();", "blob.zeroize();", "CredFree(credential.cast())", "Some(1_168)", "decode_credential_blob(&bytes)"]) assert(credentialWindows.includes(token), `Windows原访问/清理/缺席语义缺少${token}`);
assert((credentialBlob.match(/Zeroizing::new\(/g) ?? []).length === 4, "UTF-16两分支的units及临时String必须自动清理");
for (const token of ["looks_like_utf16le(bytes)", "std::str::from_utf8(bytes)", "bytes.len() % 2 == 0", "String::from_utf16(&units)", "take_while(|unit| *unit != 0)", "trim_matches(char::from(0)).trim()", "!trimmed.contains(char::from(0))"]) assert(credentialBlob.includes(token), `原凭据编码/终止符兼容边界缺少${token}`);
assert(credentialError.includes("GatewayErrorCategory::MissingCredential") && credentialError.includes("不要把密钥写入源码或配置文件"), "缺失凭据必须保持原安全恢复建议");
assert(parser.includes("use crate::credentials::API_KEY_PLACEHOLDER;") && credentialExports.includes("pub(crate) use redaction::API_KEY_PLACEHOLDER;"), "原API Example测试必须引用唯一占位符owner，避免迁移后未定义常量");
assert(parser.includes("sanitized_api_example(endpoint.as_str(), body, api_key.as_deref())"), "持久化示例必须统一传入已提取凭据进行脱敏");
for (const token of ["fn sanitize_body(", "fn canonical_curl(", "fn is_placeholder_key(", "fn looks_like_secret("]) assert(!parser.includes(token), `解析器仍持有重复脱敏职责：${token}`);
for (const token of ["value.replace(key, API_KEY_PLACEHOLDER)", "redact_known_key(key, api_key)", "sanitize_body(value, api_key)", "Value::String(redact_known_key(value, api_key))", "Authorization: Bearer {API_KEY_PLACEHOLDER}"]) assert(credentialRedaction.includes(token), `嵌套/对象键/请求头持久化脱敏缺少${token}`);
for (const source of [credentialKey, credentialBlob, credentialRedaction]) assert(!/(?:std::env::var|CredReadW|CredWriteW|CredDeleteW|reqwest::|tokio::spawn|std::fs::)/.test(source), "密钥/解码/脱敏纯职责不得新增IO或后台状态");
for (const [source, names] of [[credentialKey, ["api_key_debug_is_redacted", "api_key_rejects_embedded_whitespace", "empty_and_invalid_keys_keep_exact_safe_errors", "byte_limit_trimming_and_unicode_policy_remain_compatible"]], [credentialProvider, ["desktop_environment_is_rejected_before_reading_missing_variable", "explicit_server_missing_variable_keeps_original_recovery"]], [credentialStore, ["credential_target_rejects_path_injection", "target_limits_and_original_character_policy_remain_compatible", "invalid_targets_stop_all_public_operations_before_native_io"]], [credentialBlob, ["credential_blob_accepts_utf8_and_utf16le", "terminators_trimming_and_utf16_fallback_keep_original_results", "malformed_or_empty_blob_keeps_exact_safe_missing_credential_error"]], [credentialRedaction, ["nested_body_and_object_keys_never_retain_the_extracted_bearer", "existing_secret_heuristic_and_placeholder_policy_remain_compatible", "unmodified_values_and_empty_optional_key_preserve_canonical_template"]], [parser, ["extracted_compatible_key_is_redacted_inside_nested_persisted_template"]]]) for (const name of names) assert(source.includes(`fn ${name}`), `原unit target缺少凭据/脱敏边界测试：${name}`);

const sourceText = [frontend, client, main, store, commands, parser, ...credentialSources].join("\n");
assert(!/localStorage\.(?:setItem|getItem)\([^\n]*(?:openai|api.?key)/i.test(sourceText), "API密钥或配置不得写入localStorage");
assert(!/sk-(?!test-)[A-Za-z0-9_-]{20,}/.test(sourceText), "源码疑似包含真实API Key");
assert(!store.includes("raw_response"), "本机配置文件不得承载API响应");
assert(!/(?:last_tested_at|created_at|updated_at|tested_at|now)\.clone\(\)/.test(store), "兼容API配置时间字段不得触发Clippy clone_on_copy");

const summaryInitializer = initializerSlice(
  store,
  "Ok(OpenAiProfileSummary {",
  "\n        })",
);
assert(summaryInitializer.length > 0, "OpenAiProfileSummary构造器无法定位");
for (const field of ["api_protocol", "api_endpoint", "token_limit_field", "api_workspace_web_search_mode", "api_example_template"]) {
  assert(fieldCount(summaryInitializer, field) === 1, `OpenAiProfileSummary字段${field}必须且只能初始化一次`);
}

const metadataTestInitializer = initializerSlice(
  store,
  "fn profile_metadata_never_serializes_api_key()",
  "store.save(draft)",
);
assert(metadataTestInitializer.length > 0, "密钥不落盘测试构造器无法定位");
for (const field of ["api_protocol", "api_endpoint", "token_limit_field", "api_workspace_web_search_mode", "api_example_template"]) {
  assert(fieldCount(metadataTestInitializer, field) === 1, `OpenAiProfileDraft测试字段${field}必须且只能初始化一次`);
}

if (failures.length) {
  console.error("兼容API Example配置入口验证失败：");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}
console.log("兼容API Example配置入口验证通过：双协议实时解析、端点替换、多档案和正式研究隔离保持；R9-02唯一凭据/脱敏owner及失败清理、嵌套示例边界已锁定。");
