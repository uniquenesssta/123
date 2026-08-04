import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const contract = JSON.parse(readFileSync(join(root, "contracts/cargo-lock-integrity.json"), "utf8"));
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const lockBuffer = readFileSync(join(root, "Cargo.lock"));
const lock = lockBuffer.toString("utf8");
const failures = [];
const assert = (condition, message) => { if (!condition) failures.push(message); };
const sha256 = createHash("sha256").update(lockBuffer).digest("hex");

assert(contract.release_version === packageJson.version, "Cargo.lock完整性契约版本与项目版本不一致");
assert(contract.cargo_lock_sha256 === sha256, "Cargo.lock哈希与发布契约不一致；请勿用全局替换修改第三方依赖版本");
const blocks = lock.split("[[package]]").slice(1);
const parsed = blocks.map((block) => ({
  name: block.match(/^\s*name = "([^"]+)"/m)?.[1] ?? "",
  version: block.match(/^version = "([^"]+)"/m)?.[1] ?? "",
  checksum: block.match(/^checksum = "([^"]+)"/m)?.[1] ?? null,
  source: block.match(/^source = "([^"]+)"/m)?.[1] ?? null,
}));
for (const item of contract.protected_registry_packages) {
  const match = parsed.find((entry) => entry.name === item.name && entry.source?.includes("crates.io-index"));
  assert(Boolean(match), `Cargo.lock缺少受保护依赖：${item.name}`);
  if (match) {
    assert(match.version === item.version, `${item.name}版本被误改：期望${item.version}，实际${match.version}`);
    assert(match.checksum === item.checksum, `${item.name}校验和被误改`);
  }
}
const localFootball = parsed.filter((entry) => entry.name.startsWith("football-") && entry.source === null);
assert(localFootball.length === 11, `本地workspace包数量异常：${localFootball.length}`);
for (const entry of localFootball) assert(entry.version === packageJson.version, `本地包${entry.name}版本未同步`);
if (failures.length) {
  console.error("Cargo.lock完整性验证失败：");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}
console.log(`Cargo.lock完整性验证通过：${localFootball.length}个本地包版本正确，synstructure保持0.13.2。`);
