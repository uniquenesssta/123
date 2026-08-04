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
const desktop = read("src-tauri/src/lib.rs");
const commands = read("src-tauri/src/commands/openai.rs");
const store = read("src-tauri/src/openai_profiles.rs");
const credentials = read("crates/research-gateway/src/credentials.rs");
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
  assert(desktop.includes(`commands::${command}`), `Tauri未注册${command}`);
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

assert(desktop.includes("openai-profiles.json"), "配置元数据未使用独立本机文件");
assert(store.includes("credential_target(profile_id)"), "密钥未按配置档案隔离");
assert(store.includes("api_key_mask"), "已保存密钥未返回掩码状态");
assert(store.includes("connection_settings_changed"), "连接参数变化后未失效旧测试状态");
assert(store.includes("*state = previous_state"), "凭据写入或删除失败时未回滚配置元数据");
assert(store.includes("parse_api_example"), "保存配置前未执行Rust确定性解析与脱敏");
assert(store.includes("sanitized_example"), "API Example未在持久化前脱敏");
assert(!store.includes("api_key: String"), "持久化配置结构疑似保存明文API Key");
assert(!/derive\([^)]*Debug[^)]*\)\]\s*pub struct OpenAiProfileDraft/s.test(store), "包含API Key的输入结构不得派生Debug");
assert(credentials.includes("CredWriteW"), "Windows凭据管理器缺少写入能力");
assert(credentials.includes("CredDeleteW"), "Windows凭据管理器缺少删除能力");
assert(credentials.includes("CredReadW"), "Windows凭据管理器缺少读取能力");

assert(parser.includes("/chat/completions") && parser.includes("/responses"), "Rust解析器缺少Responses或Chat Completions识别");
assert(parser.includes("YOUR_API_KEY"), "Rust解析器缺少持久化脱敏占位符");
assert(parser.includes("api.gptsapi.net/v1/responses"), "Rust解析器缺少用户提供兼容API回归样本");
assert(parser.includes("prefers_responses_when_markdown_contains_two_examples"), "双示例默认选择Responses的回归测试缺失");
assert(gatewayConfig.includes("request_endpoint"), "网关配置缺少完整请求端点覆盖");
assert(gatewayConfig.includes("token_limit_field"), "网关配置缺少Token字段适配");
assert(gateway.includes("post_json(&endpoint"), "连接测试未向配置端点发送最小POST请求");
assert(gateway.includes("ApiProtocol::ChatCompletions"), "连接测试缺少Chat Completions响应验证");
assert(gateway.includes("P4正式联网研究仅支持Responses协议"), "正式研究未隔离Chat Completions");

const sourceText = [frontend, client, main, store, commands, parser].join("\n");
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
console.log("兼容API Example配置入口验证通过：双协议实时解析、端点替换、多档案、安全密钥和正式研究隔离均已锁定。");
