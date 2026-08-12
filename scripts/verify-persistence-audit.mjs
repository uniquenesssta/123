import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const files = {
  module: "crates/persistence-postgres/src/audit/mod.rs",
  event: "crates/persistence-postgres/src/audit/audit_event.rs",
  payload: "crates/persistence-postgres/src/audit/audit_payload.rs",
  hash: "crates/persistence-postgres/src/audit/audit_hash.rs",
  writer: "crates/persistence-postgres/src/audit/write_audit_event.rs",
  library: "crates/persistence-postgres/src/lib.rs",
  player: "crates/persistence-postgres/src/player_catalog.rs",
};
for (const [label, relative] of Object.entries(files)) check(exists(relative), `R4-02 缺少 ${label} owner：${relative}`);
const sources = Object.fromEntries(Object.entries(files).map(([key, relative]) => [key, read(relative)]));
check(sources.library.includes("mod audit;"), "lib.rs 未注册 audit 模块");
check(sources.library.includes("pub(crate) use audit::{sha256_json, write_audit_event};"), "lib.rs 未保持 crate 内 Audit 兼容出口");
check(!sources.library.includes("INSERT INTO audit.events"), "lib.rs 仍承载 Audit SQL 实现");
check(!sources.library.includes("async fn write_audit_event") && !sources.library.includes("fn sha256_json<"), "lib.rs 仍承载 Audit helper 实现");
check(sources.module.includes("mod audit_event;") && sources.module.includes("mod audit_payload;") && sources.module.includes("mod audit_hash;") && sources.module.includes("mod write_audit_event;"), "audit/mod.rs 未显式声明全部职责文件");
check(sources.event.includes("struct AuditEvent") && sources.event.includes("struct AuditEntityId") && sources.event.includes("Uuid::new_v4()"), "AuditEvent/entity ID owner 不完整");
check(sources.payload.includes("struct AuditPayload(Value)") && sources.payload.includes("into_value"), "AuditPayload 未保持 JSON Value 原样传递");
for (const token of ["serde_json::to_vec(value)?", "Sha256::new()", "hasher.update(bytes)", "hex::encode(hasher.finalize())"]) check(sources.hash.includes(token), `sha256_json 算法契约缺失：${token}`);
for (const token of ["Transaction<'_, Postgres>", "INSERT INTO audit.events (id, event_type, entity_type, entity_id, payload)", "VALUES ($1, $2, $3, $4, $5)", ".execute(&mut **tx)", ".await?;"]) check(sources.writer.includes(token), `Audit writer 事务/错误传播契约缺失：${token}`);

const rustRoot = path.join(root, "crates/persistence-postgres/src");
const rustFiles = fs.readdirSync(rustRoot, { recursive: true, withFileTypes: true })
  .filter((entry) => entry.isFile() && entry.name.endsWith(".rs"))
  .map((entry) => path.join(entry.parentPath ?? entry.path, entry.name));
const rawInsertOwners = rustFiles.filter((file) => fs.readFileSync(file, "utf8").includes("INSERT INTO audit.events"));
check(rawInsertOwners.length === 1 && rawInsertOwners[0].replaceAll("\\", "/").endsWith("/audit/write_audit_event.rs"), `Audit INSERT 必须只有唯一 writer owner，实际：${rawInsertOwners.join(", ")}`);
const combined = rustFiles.map((file) => fs.readFileSync(file, "utf8")).join("\n");
check(!combined.includes("async fn audit_in_tx("), "旧 player audit_in_tx helper 仍存在");
check(!combined.includes("async fn audit(\n"), "旧 pool audit helper 仍存在");
check(sources.player.includes("let mut tx = self.pool.begin().await?;") && sources.player.includes('"team_created"') && sources.player.includes("tx.commit().await?;"), "create_team 未将业务写入与审计收敛到同一事务");

if (failures.length) {
  console.error("R4-02 Audit 基础设施验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R4-02 Audit 基础设施验证通过：AuditEvent/EntityId/Payload/Hash/Writer 已形成唯一职责 owner，所有生产审计 INSERT 收敛到业务事务内统一 writer。");
