import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";

const scriptDirectory = path.dirname(fileURLToPath(import.meta.url));

function parseArguments(argv) {
  const options = {
    root: path.resolve(scriptDirectory, ".."),
    manifest: null,
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
    if (argument === "--manifest") {
      const value = argv[index + 1];
      if (!value) throw new Error("--manifest 需要路径参数");
      options.manifest = path.resolve(value);
      index += 1;
      continue;
    }
    throw new Error(`未知参数：${argument}`);
  }

  if (!options.manifest) {
    options.manifest = path.join(options.root, "architecture", "protected-assets.json");
  }

  return options;
}

function normalizeRelativePath(value) {
  return value.split(path.sep).join("/").replace(/^\.\//, "");
}

function validateRelativePath(value, fieldName) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${fieldName} 必须是非空字符串`);
  }
  const normalized = normalizeRelativePath(value);
  if (
    path.isAbsolute(value) ||
    normalized === ".." ||
    normalized.startsWith("../") ||
    normalized.includes("/../")
  ) {
    throw new Error(`${fieldName} 不能越出仓库根目录：${value}`);
  }
  return normalized;
}

function canonicalize(buffer, mode) {
  if (mode === "binary") return buffer;
  if (mode !== "utf8-lf") {
    throw new Error(`不支持的指纹模式：${mode}`);
  }
  const text = buffer.toString("utf8").replace(/\r\n?/g, "\n");
  return Buffer.from(text, "utf8");
}

function sha1GitBlob(buffer) {
  const header = Buffer.from(`blob ${buffer.length}\0`, "utf8");
  return crypto.createHash("sha1").update(header).update(buffer).digest("hex");
}

function sha256(value) {
  return crypto.createHash("sha256").update(value).digest("hex");
}

function fingerprintFromBlobSha(blobSha) {
  return sha256(Buffer.from(`git-blob-sha1:${blobSha}`, "utf8"));
}

function globToRegExp(pattern) {
  const normalized = validateRelativePath(pattern, "forbidden_paths.pattern");
  let expression = "^";

  for (let index = 0; index < normalized.length; index += 1) {
    const character = normalized[index];
    const next = normalized[index + 1];

    if (character === "*" && next === "*") {
      const after = normalized[index + 2];
      if (after === "/") {
        expression += "(?:.*/)?";
        index += 2;
      } else {
        expression += ".*";
        index += 1;
      }
      continue;
    }
    if (character === "*") {
      expression += "[^/]*";
      continue;
    }
    if (character === "?") {
      expression += "[^/]";
      continue;
    }
    if ("\\^$+?.()|{}[]".includes(character)) {
      expression += `\\${character}`;
    } else {
      expression += character;
    }
  }

  expression += "$";
  return new RegExp(expression);
}

function listFiles(rootDirectory, relativeDirectory = "") {
  const absoluteDirectory = path.join(rootDirectory, relativeDirectory);
  if (!fs.existsSync(absoluteDirectory)) return [];

  const result = [];
  for (const entry of fs.readdirSync(absoluteDirectory, { withFileTypes: true })) {
    const relativePath = normalizeRelativePath(path.join(relativeDirectory, entry.name));
    if (entry.isSymbolicLink()) {
      result.push(relativePath);
      continue;
    }
    if (entry.isDirectory()) {
      result.push(...listFiles(rootDirectory, relativePath));
    } else if (entry.isFile()) {
      result.push(relativePath);
    }
  }
  return result.sort();
}

function assertExactFileSet(actual, expected, label, failures) {
  const actualSet = new Set(actual);
  const expectedSet = new Set(expected);

  for (const file of expectedSet) {
    if (!actualSet.has(file)) failures.push(`${label} 缺少受保护文件：${file}`);
  }
  for (const file of actualSet) {
    if (!expectedSet.has(file)) failures.push(`${label} 出现未登记文件：${file}`);
  }
}

function readManifest(manifestPath) {
  let parsed;
  try {
    parsed = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  } catch (error) {
    throw new Error(`无法读取保护资产清单 ${manifestPath}：${error.message}`);
  }

  if (parsed.manifest_version !== "1.0.0") {
    throw new Error(`不支持的清单版本：${parsed.manifest_version ?? "缺失"}`);
  }
  if (!Array.isArray(parsed.protected_files) || parsed.protected_files.length === 0) {
    throw new Error("protected_files 必须是非空数组");
  }
  if (!Array.isArray(parsed.forbidden_paths)) {
    throw new Error("forbidden_paths 必须是数组");
  }
  return parsed;
}

function verify(options) {
  const manifest = readManifest(options.manifest);
  const failures = [];
  const seenPaths = new Set();
  const verifiedEntries = [];

  for (const entry of manifest.protected_files) {
    const relativePath = validateRelativePath(entry.path, "protected_files.path");
    if (seenPaths.has(relativePath)) {
      failures.push(`保护资产清单包含重复路径：${relativePath}`);
      continue;
    }
    seenPaths.add(relativePath);

    const absolutePath = path.join(options.root, relativePath);
    if (!fs.existsSync(absolutePath)) {
      failures.push(`受保护文件缺失：${relativePath}`);
      continue;
    }

    const stat = fs.lstatSync(absolutePath);
    if (stat.isSymbolicLink()) {
      failures.push(`受保护文件不能是符号链接：${relativePath}`);
      continue;
    }
    if (!stat.isFile()) {
      failures.push(`受保护路径不是普通文件：${relativePath}`);
      continue;
    }

    const canonicalBytes = canonicalize(fs.readFileSync(absolutePath), entry.mode);
    const blobSha = sha1GitBlob(canonicalBytes);
    const fingerprint = fingerprintFromBlobSha(blobSha);

    if (blobSha !== entry.git_blob_sha1) {
      failures.push(
        `受保护文件 Git blob 指纹变化：${relativePath}，期望 ${entry.git_blob_sha1}，实际 ${blobSha}`,
      );
    }
    if (fingerprint !== entry.fingerprint_sha256) {
      failures.push(
        `受保护文件 SHA-256 指纹变化：${relativePath}，期望 ${entry.fingerprint_sha256}，实际 ${fingerprint}`,
      );
    }

    verifiedEntries.push({
      path: relativePath,
      fingerprint_sha256: fingerprint,
    });
  }

  for (const rootEntry of manifest.protected_roots ?? []) {
    const rootPath = validateRelativePath(rootEntry.path, "protected_roots.path");
    const absoluteRoot = path.join(options.root, rootPath);
    if (!fs.existsSync(absoluteRoot)) {
      failures.push(`受保护目录缺失：${rootPath}`);
      continue;
    }
    if (!fs.lstatSync(absoluteRoot).isDirectory()) {
      failures.push(`受保护目录不是目录：${rootPath}`);
      continue;
    }

    const actualFiles = listFiles(options.root, rootPath);
    const expectedFiles = (rootEntry.allowed_files ?? []).map((file) =>
      validateRelativePath(file, "protected_roots.allowed_files"),
    );
    assertExactFileSet(actualFiles, expectedFiles, `受保护目录 ${rootPath}`, failures);
  }

  const ignoredTopLevelDirectories = new Set([
    ".git",
    "node_modules",
    "dist",
    "target",
    ".cargo-target",
    "verification-logs",
  ]);
  const allRepositoryFiles = listFiles(options.root).filter((relativePath) => {
    const firstSegment = relativePath.split("/")[0];
    return !ignoredTopLevelDirectories.has(firstSegment);
  });

  for (const patternEntry of manifest.protected_patterns ?? []) {
    const rootPath = validateRelativePath(patternEntry.root, "protected_patterns.root");
    const matcher = globToRegExp(
      `${rootPath.replace(/\/$/, "")}/${patternEntry.pattern}`,
    );
    const actualFiles = allRepositoryFiles.filter((relativePath) => matcher.test(relativePath));
    const expectedFiles = (patternEntry.allowed_files ?? []).map((file) =>
      validateRelativePath(file, "protected_patterns.allowed_files"),
    );
    assertExactFileSet(
      actualFiles,
      expectedFiles,
      `受保护模式 ${rootPath}/${patternEntry.pattern}`,
      failures,
    );
  }

  for (const forbidden of manifest.forbidden_paths) {
    const pattern = validateRelativePath(forbidden.pattern, "forbidden_paths.pattern");
    const matcher = globToRegExp(pattern);
    const matches = allRepositoryFiles.filter((relativePath) => matcher.test(relativePath));

    const directoryCandidate = pattern.endsWith("/**")
      ? pattern.slice(0, -3)
      : null;
    if (directoryCandidate) {
      const absoluteDirectory = path.join(options.root, directoryCandidate);
      if (fs.existsSync(absoluteDirectory)) {
        matches.unshift(directoryCandidate);
      }
    }

    for (const match of [...new Set(matches)]) {
      failures.push(
        `发现禁止进入公开仓库的资产：${match}（${forbidden.category ?? "未分类"}）`,
      );
    }
  }

  const aggregateInput = verifiedEntries
    .sort((left, right) => {
    if (left.path < right.path) return -1;
    if (left.path > right.path) return 1;
    return 0;
  })
    .map((entry) => `${entry.path}\0${entry.fingerprint_sha256}\n`)
    .join("");
  const aggregate = sha256(Buffer.from(aggregateInput, "utf8"));
  if (aggregate !== manifest.aggregate_sha256) {
    failures.push(
      `保护资产聚合 SHA-256 变化：期望 ${manifest.aggregate_sha256}，实际 ${aggregate}`,
    );
  }

  if (failures.length > 0) {
    console.error("保护资产验证失败：");
    for (const failure of failures) console.error(`- ${failure}`);
    process.exitCode = 1;
    return;
  }

  console.log(
    `保护资产验证通过：${verifiedEntries.length} 个文件指纹一致，私有 P4/P7 资产继续缺席，聚合 SHA-256=${aggregate}`,
  );
}

try {
  verify(parseArguments(process.argv.slice(2)));
} catch (error) {
  console.error(`保护资产验证无法执行：${error.message}`);
  process.exitCode = 1;
}
