import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const result = spawnSync(
  "cargo",
  ["metadata", "--locked", "--format-version", "1"],
  {
    cwd: root,
    encoding: "utf8",
    windowsHide: true,
    maxBuffer: 32 * 1024 * 1024,
  },
);

if (result.error) {
  console.error("Cargo.lock语义同步验证无法启动 Cargo：");
  console.error(result.error.message);
  process.exit(1);
}

if (result.status !== 0) {
  console.error("Cargo.toml 与 Cargo.lock 语义不同步，已停止后续 Rust 校验。");
  const details = `${result.stderr ?? ""}${result.stdout ?? ""}`.trim();
  if (details) console.error(details);
  console.error("请先在受控环境中同步 Cargo.lock；不要删除 --locked 或绕过该门禁。");
  process.exit(result.status ?? 1);
}

let metadata;
try {
  metadata = JSON.parse(result.stdout);
} catch {
  console.error("Cargo metadata 返回了无法解析的结果，已停止后续 Rust 校验。");
  process.exit(1);
}

const localPackages = metadata.packages.filter(
  (item) => item.name.startsWith("football-") && item.source === null,
);
if (localPackages.length !== 11) {
  console.error(`Cargo workspace 本地包数量异常：期望11，实际${localPackages.length}`);
  process.exit(1);
}

console.log("Cargo.lock语义同步验证通过：Cargo.toml 与锁文件一致，--locked 无需更新。");
