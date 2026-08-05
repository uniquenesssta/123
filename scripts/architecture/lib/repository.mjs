import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, extname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ignoredDirectories = new Set([
  ".git",
  ".cargo-target",
  "dist",
  "node_modules",
  "target",
]);

export const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

export function normalizePath(value) {
  return value.replaceAll("\\", "/").replace(/^\.\//, "");
}

export function resolveRepositoryPath(relativePath) {
  return resolve(repositoryRoot, normalizePath(relativePath));
}

export function repositoryPath(absolutePath) {
  return normalizePath(relative(repositoryRoot, absolutePath));
}

export function pathExists(relativePath) {
  return existsSync(resolveRepositoryPath(relativePath));
}

export function readText(relativePath) {
  return readFileSync(resolveRepositoryPath(relativePath), "utf8").replaceAll("\r\n", "\n");
}

export function readJson(relativePath) {
  try {
    return JSON.parse(readText(relativePath));
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`无法读取 JSON ${relativePath}：${message}`);
  }
}

export function listFiles(roots, { extensions = null } = {}) {
  const result = [];
  const normalizedExtensions = extensions
    ? new Set(extensions.map((value) => value.toLowerCase()))
    : null;

  function visit(absolutePath) {
    if (!existsSync(absolutePath)) return;
    const stats = statSync(absolutePath);
    if (stats.isFile()) {
      if (!normalizedExtensions || normalizedExtensions.has(extname(absolutePath).toLowerCase())) {
        result.push(repositoryPath(absolutePath));
      }
      return;
    }
    if (!stats.isDirectory()) return;

    const directoryName = normalizePath(absolutePath).split("/").at(-1);
    if (ignoredDirectories.has(directoryName)) return;

    for (const entry of readdirSync(absolutePath, { withFileTypes: true })) {
      if (entry.isDirectory() && ignoredDirectories.has(entry.name)) continue;
      visit(join(absolutePath, entry.name));
    }
  }

  for (const root of roots) visit(resolveRepositoryPath(root));
  return result.sort();
}

export function matchesPathPattern(filePath, pattern) {
  const value = normalizePath(filePath);
  const normalizedPattern = normalizePath(pattern);

  if (!normalizedPattern.includes("*") && !normalizedPattern.includes("?")) {
    return value === normalizedPattern || value.startsWith(`${normalizedPattern}/`);
  }

  let expression = "^";
  for (let index = 0; index < normalizedPattern.length; index += 1) {
    const character = normalizedPattern[index];
    if (character === "*" && normalizedPattern[index + 1] === "*") {
      expression += ".*";
      index += 1;
    } else if (character === "*") {
      expression += "[^/]*";
    } else if (character === "?") {
      expression += "[^/]";
    } else {
      expression += character.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    }
  }
  expression += "$";
  return new RegExp(expression).test(value);
}

export function extractRepositoryLocation(value) {
  if (typeof value !== "string") return null;
  const normalized = normalizePath(value);
  const match = normalized.match(/(?:^|\s)((?:src-tauri|src|crates|migrations|architecture)\/[A-Za-z0-9_./-]+\.(?:rs|ts|tsx|js|mjs|json|toml))/);
  return match?.[1] ?? null;
}
