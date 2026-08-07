import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const compatibility = read("crates/persistence-postgres/src/migration_compatibility.rs");
const connection = read("crates/persistence-postgres/src/connection.rs");
const library = read("crates/persistence-postgres/src/lib.rs");

check(library.includes("mod migration_compatibility;"), "PostgreSQL crate 未注册迁移兼容模块");
check(connection.includes("reconcile_known_legacy_migrations(&self.pool).await?;"), "数据库 migrate 未调用历史兼容桥");
check(
  connection.indexOf("reconcile_known_legacy_migrations(&self.pool).await?;") < connection.indexOf("MIGRATOR.run(&self.pool).await?;"),
  "历史兼容桥必须在 SQLx Migrator 校验迁移账本之前执行",
);
check(compatibility.includes("pg_advisory_xact_lock"), "历史兼容桥缺少事务级并发锁");
check(compatibility.includes("p4-software-integration") && compatibility.includes("LEGACY_INTEGRATION_CONTRACT_SHA256"), "历史兼容桥缺少已知旧数据库来源识别");
check(compatibility.includes("COMPATIBLE_MIGRATION_VERSIONS: [i64; 11] = [12, 13, 14, 15, 16, 17, 18, 25, 26, 27, 31]"), "历史迁移兼容白名单发生变化");
check(compatibility.includes("Sha384::digest(sql.as_bytes())"), "SQLx 迁移 checksum 未按 SHA-384 生成");
check(compatibility.includes("UPDATE public._sqlx_migrations SET checksum=$2") && compatibility.includes("success=true"), "兼容桥没有只更新成功迁移的 checksum");
check(
  compatibility.includes("RENAME COLUMN golden_master_sha256 TO provider_fixture_sha256"),
  "已知旧模型制品账本必须通过列重命名保留不可变记录，禁止行级复制",
);
check(
  !compatibility.includes("UPDATE model.engine_artifacts SET provider_fixture_sha256"),
  "历史兼容桥不得 UPDATE 不可变 model.engine_artifacts 记录",
);
check(
  compatibility.includes("RENAME CONSTRAINT engine_artifacts_golden_master_sha256_check TO engine_artifacts_provider_fixture_sha256_check"),
  "历史模型制品字段约束没有同步到公开字段名",
);
check(compatibility.includes("为保护数据，未修改迁移账本"), "未知迁移历史没有 fail-closed 保护语义");
for (const destructive of ["DELETE FROM public._sqlx_migrations", "TRUNCATE", "DROP TABLE", "DROP SCHEMA"]) {
  check(!compatibility.includes(destructive), `历史兼容桥包含禁止的数据破坏操作：${destructive}`);
}

if (failures.length) {
  console.error("数据库历史迁移兼容验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("数据库历史迁移兼容验证通过：仅识别已登记旧来源，模型制品字段通过无行更新重命名桥接，固定 11 个版本按 SQLx SHA-384 对账，未知历史保持 fail-closed。");
