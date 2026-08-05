import { readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8").replace(/\r\n/g, "\n");
const failures = [];
const check = (condition, message) => {
  if (!condition) failures.push(message);
};

const bootstrapRoot = join(root, "src-tauri", "src", "bootstrap");
const expectedFiles = [
  "application.rs",
  "command_registry.rs",
  "error.rs",
  "mod.rs",
  "state.rs",
];
const actualFiles = readdirSync(bootstrapRoot, { withFileTypes: true })
  .filter((entry) => entry.isFile())
  .map((entry) => entry.name)
  .sort();
check(
  JSON.stringify(actualFiles) === JSON.stringify(expectedFiles),
  `Tauri bootstrap 文件集合不正确：${actualFiles.join(", ")}`,
);

const library = read("src-tauri/src/lib.rs");
const moduleRoot = read("src-tauri/src/bootstrap/mod.rs");
const application = read("src-tauri/src/bootstrap/application.rs");
const state = read("src-tauri/src/bootstrap/state.rs");
const registry = read("src-tauri/src/bootstrap/command_registry.rs");
const error = read("src-tauri/src/bootstrap/error.rs");
const commands = read("src-tauri/src/commands.rs");
const commandContract = JSON.parse(read("architecture/command-contract.json"));
const moduleContract = JSON.parse(read("architecture/module-boundaries.json"));
const stateContract = JSON.parse(read("architecture/state-ownership.json"));

check(library.includes("mod bootstrap;"), "src-tauri/src/lib.rs 未声明 bootstrap 模块");
check(library.includes("pub fn run()"), "src-tauri/src/lib.rs 未保留公共 run 入口");
check(library.includes("bootstrap::run();"), "公共 run 未委托 Tauri 组合根");
for (const forbidden of ["tauri::Builder", ".setup(", ".manage(", "generate_handler!"]) {
  check(!library.includes(forbidden), `src-tauri/src/lib.rs 仍承担组合职责：${forbidden}`);
}

const declaredModules = [...moduleRoot.matchAll(/^mod\s+([a-z_]+);$/gm)]
  .map((match) => match[1])
  .sort();
check(
  JSON.stringify(declaredModules) ===
    JSON.stringify(["application", "command_registry", "error", "state"]),
  `bootstrap/mod.rs 模块出口不正确：${declaredModules.join(", ")}`,
);
check(moduleRoot.includes("pub(crate) use state::AppState;"), "bootstrap 未公开唯一 AppState");
check(moduleRoot.includes("application::run();"), "bootstrap 未委托 application 组合入口");

check(application.includes("tauri::Builder::default()"), "application.rs 未构造 Tauri Builder");
check(application.includes("tauri_plugin_dialog::init()"), "application.rs 未保持 dialog 插件注册");
check(application.includes("state::install(app)?"), "application.rs 未委托状态安装");
check(application.includes("command_registry::register(builder)"), "application.rs 未委托命令注册");
check(application.includes("tauri::generate_context!()"), "application.rs 未保持 Tauri context");
check(application.includes("error::expect_startup"), "application.rs 未委托启动错误映射");

check(state.includes("pub struct AppState"), "state.rs 未定义 AppState");
for (const field of [
  "service",
  "config_path",
  "issue_log",
  "runtime_log",
  "openai_profiles",
  "workspace_state",
  "api_workspace_requests",
]) {
  check(new RegExp(`\\bpub ${field}:`).test(state), `AppState 缺少字段：${field}`);
}
for (const required of [
  "app_config_dir()",
  "RuntimeLogStore::discover",
  '"application_started"',
  "app.manage(AppState",
]) {
  check(state.includes(required), `state.rs 缺少原启动状态行为：${required}`);
}
check(!state.includes("#[tauri::command]"), "state.rs 不得实现 Tauri 命令");

const handlerMatches = [...registry.matchAll(/tauri::generate_handler!\s*\[([\s\S]*?)\]\s*\)/g)];
check(handlerMatches.length === 1, `命令注册表必须且只能有一个 generate_handler!，实际 ${handlerMatches.length}`);
const registered = handlerMatches.length
  ? [...handlerMatches[0][1].matchAll(/commands::([a-z0-9_]+)/g)].map((match) => match[1])
  : [];
const expectedCommands = commandContract.commands_in_registration_order ?? [];
check(registered.length === 171, `Tauri 注册命令数量必须为 171，实际 ${registered.length}`);
check(new Set(registered).size === registered.length, "Tauri 注册表存在重复命令");
check(
  JSON.stringify(registered) === JSON.stringify(expectedCommands),
  "Tauri 注册命令名称或顺序与冻结命令契约不一致",
);
check(!registry.includes("#[tauri::command]"), "command_registry.rs 只能登记命令，不能实现命令");

check(error.includes('"足球赛事模型平台启动失败"'), "启动错误提示语义发生变化");
check(
  error.includes("io::Error::other") || error.includes("std::io::Error::other"),
  "启动 I/O 错误映射发生变化",
);
check(commands.includes("pub(crate) use crate::bootstrap::AppState;"), "commands.rs 未使用组合根 AppState");
check(!commands.includes("pub struct AppState"), "commands.rs 仍重复定义 AppState");
check(commands.includes("use uuid::Uuid;"), "commands.rs 丢失 UUID 解析依赖");

check(
  commandContract.sources?.tauri_registration?.file ===
    "src-tauri/src/bootstrap/command_registry.rs",
  "命令契约未指向新的注册所有者",
);
check(
  moduleContract.rust?.tauri_host?.public_entry === "src-tauri/src/lib.rs::run",
  "模块边界契约未保留公共 Tauri 入口",
);
check(
  moduleContract.rust?.tauri_host?.state === "src-tauri/src/bootstrap/state.rs::AppState",
  "模块边界契约未指向新的 AppState 所有者",
);
check(
  moduleContract.tauri_commands?.registration_owner ===
    "src-tauri/src/bootstrap/command_registry.rs",
  "模块边界契约未指向新的命令注册所有者",
);
const tauriAppState = stateContract.states?.find((entry) => entry.id === "tauri.app-state");
check(
  tauriAppState?.owner === "src-tauri/src/bootstrap/state.rs::AppState",
  "状态所有权契约未切换 tauri.app-state owner",
);
check(tauriAppState?.transition === null, "tauri.app-state 仍处于过渡状态");

if (failures.length) {
  throw new Error(`Tauri 组合根验证失败\n${failures.map((item) => `- ${item}`).join("\n")}`);
}

console.log(
  `Tauri 组合根验证通过：5 个职责模块、1 个 AppState 所有者、${registered.length} 个冻结命令及启动错误语义全部一致。`,
);
