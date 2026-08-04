import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(scriptDirectory, "..");
const dryRun = process.argv.includes("--dry-run");
const unknown = process.argv.slice(2).filter((argument) => argument !== "--dry-run");
if (unknown.length > 0) {
  console.error(`未知参数：${unknown.join(", ")}`);
  process.exit(1);
}

const environmentVariable = "FOOTBALL_TEST_DATABASE_URL";
const connectionUrl = process.env[environmentVariable];
if (!connectionUrl) {
  console.error(
    `缺少 ${environmentVariable}。必须指向专用、允许彻底清空的 PostgreSQL 测试数据库。`,
  );
  process.exit(1);
}

let parsed;
try {
  parsed = new URL(connectionUrl);
} catch {
  console.error(`${environmentVariable} 不是有效 URL。`);
  process.exit(1);
}

if (!["postgres:", "postgresql:"].includes(parsed.protocol)) {
  console.error(`${environmentVariable} 必须使用 postgres:// 或 postgresql://。`);
  process.exit(1);
}
const databaseName = decodeURIComponent(parsed.pathname.replace(/^\/+/, "").split("/")[0] ?? "");
if (!databaseName || !databaseName.toLowerCase().includes("test")) {
  console.error(
    `已拒绝执行：数据库名称必须包含 test，当前名称为 ${databaseName || "空"}。`,
  );
  process.exit(1);
}

const verify = spawnSync(
  process.execPath,
  [path.join(root, "scripts", "verify_database_baseline.mjs")],
  { cwd: root, stdio: "inherit", env: process.env },
);
if (verify.status !== 0) process.exit(verify.status ?? 1);

const cargoArguments = [
  "test",
  "--locked",
  "-p",
  "football-persistence-postgres",
  "--test",
  "postgres_integration",
  "--",
  "--ignored",
  "--test-threads=1",
];

if (dryRun) {
  console.log(
    `数据库执行前检通过：目标数据库=${databaseName}；将执行 cargo ${cargoArguments.join(" ")}`,
  );
  process.exit(0);
}

const cargo = spawnSync("cargo", cargoArguments, {
  cwd: root,
  stdio: "inherit",
  env: process.env,
});
if (cargo.error) {
  console.error(`无法启动 cargo：${cargo.error.message}`);
  process.exit(1);
}
process.exit(cargo.status ?? 1);
