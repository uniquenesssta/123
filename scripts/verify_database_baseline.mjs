import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));

function parseArguments(argv) {
  const options = {
    root: path.resolve(scriptDirectory, ".."),
    contract: null,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === "--root") {
      const value = argv[index + 1];
      if (!value) throw new Error("--root 需要路径参数");
      options.root = path.resolve(value);
      index += 1;
      continue;
    }
    if (argument === "--contract") {
      const value = argv[index + 1];
      if (!value) throw new Error("--contract 需要路径参数");
      options.contract = path.resolve(value);
      index += 1;
      continue;
    }
    throw new Error(`未知参数：${argument}`);
  }

  if (!options.contract) {
    options.contract = path.join(options.root, "architecture", "database-baseline.json");
  }
  return options;
}

function normalizeRelativePath(value) {
  return value.split(path.sep).join("/").replace(/^\.\//, "");
}

function resolveRepositoryFile(root, relativePath, label) {
  if (typeof relativePath !== "string" || relativePath.trim() === "") {
    throw new Error(`${label} 必须是非空路径`);
  }
  const normalized = normalizeRelativePath(relativePath);
  if (
    path.isAbsolute(relativePath) ||
    normalized === ".." ||
    normalized.startsWith("../") ||
    normalized.includes("/../")
  ) {
    throw new Error(`${label} 不能越出仓库根目录：${relativePath}`);
  }
  const absolute = path.join(root, normalized);
  if (!fs.existsSync(absolute)) {
    throw new Error(`${label} 不存在：${normalized}`);
  }
  const stat = fs.lstatSync(absolute);
  if (!stat.isFile() || stat.isSymbolicLink()) {
    throw new Error(`${label} 必须是普通文件：${normalized}`);
  }
  return { absolute, relative: normalized };
}

function canonicalize(buffer, mode) {
  if (mode !== "utf8-lf") throw new Error(`不支持的指纹模式：${mode}`);
  return Buffer.from(buffer.toString("utf8").replace(/\r\n?/g, "\n"), "utf8");
}

function gitBlobSha1(buffer) {
  const header = Buffer.from(`blob ${buffer.length}\0`, "utf8");
  return crypto.createHash("sha1").update(header).update(buffer).digest("hex");
}

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function fingerprintFromBlobSha(blobSha) {
  return sha256(Buffer.from(`git-blob-sha1:${blobSha}`, "utf8"));
}

function readJson(filePath, label) {
  try {
    return JSON.parse(fs.readFileSync(filePath, "utf8"));
  } catch (error) {
    throw new Error(`无法读取${label} ${filePath}：${error.message}`);
  }
}

function collectMigrationFiles(root, policy) {
  const directory = normalizeRelativePath(policy.directory);
  const absolute = path.join(root, directory);
  if (!fs.existsSync(absolute) || !fs.lstatSync(absolute).isDirectory()) {
    throw new Error(`迁移目录不存在：${directory}`);
  }
  return fs
    .readdirSync(absolute, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".sql"))
    .map((entry) => `${directory}/${entry.name}`)
    .sort();
}

function assertExactSet(actual, expected, label, failures) {
  const actualSet = new Set(actual);
  const expectedSet = new Set(expected);
  for (const item of expectedSet) {
    if (!actualSet.has(item)) failures.push(`${label}缺失：${item}`);
  }
  for (const item of actualSet) {
    if (!expectedSet.has(item)) failures.push(`${label}出现未登记项：${item}`);
  }
}

function verifyFileFingerprint(root, entry, failures, label) {
  const resolved = resolveRepositoryFile(root, entry.path, label);
  const bytes = canonicalize(fs.readFileSync(resolved.absolute), entry.mode);
  const blobSha = gitBlobSha1(bytes);
  const fingerprint = fingerprintFromBlobSha(blobSha);

  if (blobSha !== entry.git_blob_sha1) {
    failures.push(
      `${label} Git blob 指纹变化：${resolved.relative}，期望 ${entry.git_blob_sha1}，实际 ${blobSha}`,
    );
  }
  if (fingerprint !== entry.fingerprint_sha256) {
    failures.push(
      `${label} SHA-256 指纹变化：${resolved.relative}，期望 ${entry.fingerprint_sha256}，实际 ${fingerprint}`,
    );
  }
  return { path: resolved.relative, fingerprint };
}

function verifyRequiredTokens(root, requirements, failures, label) {
  for (const requirement of requirements ?? []) {
    const resolved = resolveRepositoryFile(root, requirement.path, `${label}文件`);
    const text = fs.readFileSync(resolved.absolute, "utf8").replace(/\r\n?/g, "\n");
    for (const token of requirement.required_tokens ?? []) {
      if (!text.includes(token)) {
        failures.push(`${label}缺少关键契约：${resolved.relative} -> ${token}`);
      }
    }
  }
}

function extractIgnoredIntegrationTests(text) {
  const names = [];
  const pattern =
    /#\[tokio::test\]\s*#\[ignore\s*=\s*"[^"]*"\]\s*async\s+fn\s+([A-Za-z0-9_]+)/g;
  for (const match of text.matchAll(pattern)) names.push(match[1]);
  return names;
}

function extractAllTokioTests(text) {
  const names = [];
  const pattern = /#\[tokio::test\][\s\S]{0,400}?async\s+fn\s+([A-Za-z0-9_]+)/g;
  for (const match of text.matchAll(pattern)) names.push(match[1]);
  return names;
}

function verifyIntegrationTests(root, contract, failures) {
  const integration = contract.postgres_integration;
  const resolved = resolveRepositoryFile(root, integration.test_file, "PostgreSQL 集成测试文件");
  const text = fs.readFileSync(resolved.absolute, "utf8").replace(/\r\n?/g, "\n");
  const ignored = extractIgnoredIntegrationTests(text);
  const all = extractAllTokioTests(text);

  if (new Set(ignored).size !== ignored.length) {
    failures.push("PostgreSQL 集成测试包含重复测试函数名");
  }
  if (ignored.length !== integration.expected_ignored_test_count) {
    failures.push(
      `PostgreSQL 忽略测试数量变化：期望 ${integration.expected_ignored_test_count}，实际 ${ignored.length}`,
    );
  }
  assertExactSet(
    ignored,
    integration.expected_ignored_tests,
    "PostgreSQL 忽略测试",
    failures,
  );
  assertExactSet(all, integration.expected_ignored_tests, "PostgreSQL 全部测试", failures);

  const envToken = `const DATABASE_ENV: &str = "${integration.environment_variable}";`;
  if (!text.includes(envToken)) {
    failures.push(`PostgreSQL 测试环境变量契约变化：缺少 ${envToken}`);
  }
}

function verify(options) {
  const contract = readJson(options.contract, "数据库基线契约");
  if (contract.contract_id !== "football.database-baseline.v1") {
    throw new Error(`不支持的数据库基线契约：${contract.contract_id ?? "缺失"}`);
  }
  if (contract.contract_version !== "1.0.0") {
    throw new Error(`不支持的数据库基线版本：${contract.contract_version ?? "缺失"}`);
  }

  const failures = [];
  const policy = contract.migration_policy;
  const migrationPattern = new RegExp(policy.filename_pattern);
  const expectedPaths = contract.migrations.map((entry) =>
    normalizeRelativePath(entry.path),
  );
  const actualPaths = collectMigrationFiles(options.root, policy);
  assertExactSet(actualPaths, expectedPaths, "迁移文件", failures);

  const versions = [];
  const seenVersions = new Map();
  for (const relativePath of actualPaths) {
    const fileName = path.posix.basename(relativePath);
    const match = migrationPattern.exec(fileName);
    if (!match?.groups?.version) {
      failures.push(`迁移文件名不符合规范：${relativePath}`);
      continue;
    }
    const version = Number.parseInt(match.groups.version, 10);
    versions.push(version);
    if (seenVersions.has(version)) {
      failures.push(
        `迁移版本重复：${String(version).padStart(4, "0")} -> ${seenVersions.get(version)}, ${relativePath}`,
      );
    } else {
      seenVersions.set(version, relativePath);
    }
  }

  const expectedVersions = [];
  for (let version = policy.first_version; version <= policy.last_version; version += 1) {
    expectedVersions.push(version);
  }
  assertExactSet(
    versions.map(String),
    expectedVersions.map(String),
    "迁移版本",
    failures,
  );
  if (actualPaths.length !== policy.exact_count) {
    failures.push(
      `迁移数量变化：期望 ${policy.exact_count}，实际 ${actualPaths.length}`,
    );
  }

  const verifiedMigrations = [];
  for (const entry of contract.migrations) {
    const fileName = path.posix.basename(entry.path);
    const match = migrationPattern.exec(fileName);
    const declaredVersion = match?.groups?.version
      ? Number.parseInt(match.groups.version, 10)
      : null;
    if (declaredVersion !== entry.version) {
      failures.push(
        `迁移清单版本与文件名不一致：${entry.path}，声明 ${entry.version}，文件 ${declaredVersion}`,
      );
    }
    verifiedMigrations.push(
      verifyFileFingerprint(options.root, entry, failures, "迁移文件"),
    );
  }

  const aggregateInput = verifiedMigrations
    .sort((left, right) => left.path.localeCompare(right.path))
    .map((entry) => `${entry.path}\0${entry.fingerprint}\n`)
    .join("");
  const aggregate = sha256(Buffer.from(aggregateInput, "utf8"));
  if (aggregate !== contract.migration_aggregate_sha256) {
    failures.push(
      `迁移聚合 SHA-256 变化：期望 ${contract.migration_aggregate_sha256}，实际 ${aggregate}`,
    );
  }

  for (const entry of contract.runtime_sources ?? []) {
    verifyFileFingerprint(options.root, entry, failures, "数据库运行源文件");
  }
  verifyRequiredTokens(options.root, contract.runtime_requirements, failures, "数据库运行契约");
  verifyRequiredTokens(options.root, contract.immutable_guards, failures, "不可变约束");
  verifyIntegrationTests(options.root, contract, failures);

  if (failures.length > 0) {
    console.error("数据库基线验证失败：");
    failures.forEach((failure) => console.error(`- ${failure}`));
    process.exitCode = 1;
    return;
  }

  console.log(
    `数据库静态基线验证通过：${actualPaths.length} 个连续迁移、${contract.postgres_integration.expected_ignored_test_count} 个 PostgreSQL 集成测试契约和不可变约束均与冻结清单一致；聚合 SHA-256=${aggregate}`,
  );
}

try {
  verify(parseArguments(process.argv.slice(2)));
} catch (error) {
  console.error(`数据库基线验证无法执行：${error.message}`);
  process.exitCode = 1;
}
