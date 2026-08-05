import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(resolve(root, path), "utf8");

const domain = read("crates/domain/src/lib.rs");
const catalog = read("crates/persistence-postgres/src/player_catalog.rs");
const connection = read("crates/persistence-postgres/src/connection.rs");
const integration = read("crates/persistence-postgres/tests/postgres_integration.rs");
const command = read("src-tauri/src/commands/database.rs");
const applicationDatabase = read("crates/application/src/database.rs");
const tauri = read("src-tauri/src/bootstrap/command_registry.rs");
const client = read("src/api/client.ts");
const page = read("src/pages/database.ts");
const main = read("src/main.ts");

function requireText(source, token, label) {
  if (!source.includes(token)) throw new Error(`数据库彻底清空契约缺失：${label}`);
}

function functionBody(source, name) {
  const start = source.indexOf(`fn ${name}`);
  if (start < 0) throw new Error(`未找到 Rust 函数：${name}`);
  const brace = source.indexOf("{", start);
  let depth = 0;
  for (let index = brace; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return source.slice(start, index + 1);
  }
  throw new Error(`Rust 函数未闭合：${name}`);
}

const teamRecord = domain.match(/pub struct TeamRecord\s*\{([\s\S]*?)\n\}/)?.[1] ?? "";
const playerRecord = domain.match(/pub struct PlayerRecord\s*\{([\s\S]*?)\n\}/)?.[1] ?? "";
if (/localized_name/.test(teamRecord) || /localized_name/.test(playerRecord)) {
  throw new Error("TeamRecord / PlayerRecord 不应重复承载本地化姓名；本地化姓名属于列表或别名读取模型");
}
const teamMapper = functionBody(catalog, "team_record_from_row");
const playerMapper = functionBody(catalog, "player_record_from_row");
if (/localized_name/.test(teamMapper) || /localized_name/.test(playerMapper)) {
  throw new Error("基础 TeamRecord / PlayerRecord 行映射仍写入不存在的 localized_name 字段");
}

requireText(connection, "pub async fn reset_to_pristine", "持久化层彻底重建入口");
for (const schema of [
  "ai_workspace",
  "analytics",
  "audit",
  "catalog",
  "feature",
  "football",
  "model",
  "platform",
  "research",
  "review",
]) {
  requireText(connection, `\"${schema}\"`, `清理应用 schema：${schema}`);
}
requireText(connection, "DROP TABLE IF EXISTS public._sqlx_migrations", "清除迁移账本");
requireText(connection, "self.migrate().await", "从迁移重新建立空白结构");
requireText(connection, "football-platform-destructive-reset", "数据库级互斥锁");
requireText(integration, "destructive_reset_rebuilds_an_empty_migrated_database", "专用数据库集成测试");
requireText(integration, "assert_eq!(team_count, 0)", "清空后业务数据为空断言");

requireText(command, "pub async fn reset_database", "Tauri 清空命令");
requireText(command, "confirmation.trim() != current_health.database_name", "后端数据库名称二次校验");
requireText(command, "state.service.disconnect_database().await", "清空前停止活动数据库服务");
requireText(command, "reset_store.reset_to_pristine().await", "执行彻底重建");
requireText(command, "state.service.connect_database(options)", "清空后自动恢复连接");
requireText(command, "state.service.ensure_p4_orchestration_worker()", "清空后恢复P4后台工作器");
requireText(applicationDatabase, "pub fn ensure_p4_orchestration_worker", "P4后台工作器幂等恢复入口");
requireText(tauri, "commands::reset_database", "Tauri 命令注册");

requireText(client, 'invoke("reset_database", { confirmation })', "前端 API 调用");
requireText(page, 'data-action="request-reset-database"', "数据库页危险按钮");
requireText(main, "requestDatabaseReset", "输入数据库名称确认弹窗");
requireText(main, 'data-action="execute-database-reset"', "最终清空按钮");
requireText(main, "api.clearWorkspaceState()", "清空后移除失效工作区引用");
requireText(main, "window.location.reload()", "清空后重启界面状态");

console.log("数据库编译修复与彻底清空契约通过：记录映射、后端重建、强确认、自动重连和前端状态重置均已锁定。");
