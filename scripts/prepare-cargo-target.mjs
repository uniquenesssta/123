import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const SOURCE_ROOT = path.resolve(SCRIPT_DIR, "..");
const TARGET_ROOT = path.resolve(SOURCE_ROOT, "..", ".cargo-target");
const MIGRATIONS_ROOT = path.join(
  SOURCE_ROOT,
  "crates",
  "persistence-postgres",
  "migrations",
);
const STAMP_PATH = path.join(SOURCE_ROOT, ".cargo", "target-location.json");
const STAMP_SCHEMA_VERSION = 2;

function normalizeForComparison(value) {
  const normalized = path.normalize(path.resolve(value));
  return process.platform === "win32" ? normalized.toLowerCase() : normalized;
}

function assertSafeTargetPath() {
  const expected = normalizeForComparison(path.resolve(SOURCE_ROOT, "..", ".cargo-target"));
  const actual = normalizeForComparison(TARGET_ROOT);
  const source = normalizeForComparison(SOURCE_ROOT);
  const filesystemRoot = normalizeForComparison(path.parse(TARGET_ROOT).root);

  if (actual !== expected || actual === source || actual === filesystemRoot) {
    throw new Error(`拒绝清理不安全的 Cargo 目标目录：${TARGET_ROOT}`);
  }
}

function directoryHasEntries(directory) {
  try {
    return fs.readdirSync(directory).length > 0;
  } catch (error) {
    if (error?.code === "ENOENT") {
      return false;
    }
    throw error;
  }
}

function migrationFiles() {
  return fs
    .readdirSync(MIGRATIONS_ROOT, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".sql"))
    .map((entry) => entry.name)
    .sort((left, right) => left.localeCompare(right, "en"));
}

function computeMigrationFingerprint() {
  const hash = crypto.createHash("sha256");
  for (const filename of migrationFiles()) {
    hash.update(filename, "utf8");
    hash.update("\0", "utf8");
    hash.update(fs.readFileSync(path.join(MIGRATIONS_ROOT, filename)));
    hash.update("\0", "utf8");
  }
  return hash.digest("hex");
}

function readStamp() {
  try {
    const parsed = JSON.parse(fs.readFileSync(STAMP_PATH, "utf8"));
    if (
      parsed?.schema_version !== STAMP_SCHEMA_VERSION ||
      typeof parsed?.source_root !== "string" ||
      typeof parsed?.target_root !== "string" ||
      !/^[0-9a-f]{64}$/.test(parsed?.migration_bundle_sha256 ?? "")
    ) {
      return { status: "invalid" };
    }
    return { status: "valid", value: parsed };
  } catch (error) {
    if (error?.code === "ENOENT") {
      return { status: "missing" };
    }
    return { status: "invalid" };
  }
}

function removeStaleTarget(reason) {
  assertSafeTargetPath();
  if (!fs.existsSync(TARGET_ROOT)) {
    return;
  }

  console.log(`检测到${reason}，正在清理失效的 Rust/Tauri 构建缓存：`);
  console.log(TARGET_ROOT);
  try {
    fs.rmSync(TARGET_ROOT, {
      recursive: true,
      force: true,
      maxRetries: 5,
      retryDelay: 300,
    });
  } catch (error) {
    throw new Error(
      `无法清理失效的 Cargo 缓存。请关闭 tauri、cargo、rustc 和 Vite 进程后重试。\n${error.message}`,
      { cause: error },
    );
  }
}

function writeStamp(migrationBundleSha256) {
  fs.mkdirSync(path.dirname(STAMP_PATH), { recursive: true });
  fs.mkdirSync(TARGET_ROOT, { recursive: true });

  const payload = {
    schema_version: STAMP_SCHEMA_VERSION,
    source_root: path.resolve(SOURCE_ROOT),
    target_root: path.resolve(TARGET_ROOT),
    migration_bundle_sha256: migrationBundleSha256,
  };
  fs.writeFileSync(STAMP_PATH, `${JSON.stringify(payload, null, 2)}\n`, "utf8");
}

function main() {
  assertSafeTargetPath();
  const migrationBundleSha256 = computeMigrationFingerprint();
  const stamp = readStamp();
  const targetHasEntries = directoryHasEntries(TARGET_ROOT);

  if (stamp.status === "missing" && targetHasEntries) {
    removeStaleTarget("未登记来源路径的既有 Cargo 缓存");
  } else if (stamp.status === "invalid" && targetHasEntries) {
    removeStaleTarget("无法验证来源路径或迁移版本的 Cargo 缓存");
  } else if (stamp.status === "valid") {
    const sourceChanged =
      normalizeForComparison(stamp.value.source_root) !== normalizeForComparison(SOURCE_ROOT);
    const targetChanged =
      normalizeForComparison(stamp.value.target_root) !== normalizeForComparison(TARGET_ROOT);
    const migrationsChanged =
      stamp.value.migration_bundle_sha256 !== migrationBundleSha256;
    if ((sourceChanged || targetChanged) && targetHasEntries) {
      removeStaleTarget("项目目录移动后仍引用旧绝对路径的 Cargo 缓存");
    } else if (migrationsChanged && targetHasEntries) {
      removeStaleTarget("数据库迁移文件内容变化后仍保留旧的 SQLx 嵌入缓存");
    }
  }

  writeStamp(migrationBundleSha256);
  console.log(`Cargo 目标目录已确认：${TARGET_ROOT}`);
  console.log(`数据库迁移包指纹：${migrationBundleSha256.slice(0, 12)}…`);
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
