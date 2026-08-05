import { dirname, extname, resolve } from "node:path";
import {
  normalizePath,
  pathExists,
  repositoryPath,
  resolveRepositoryPath,
} from "./repository.mjs";

const sourceExtensions = [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"];

function removeComments(source) {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/(^|[^:])\/\/.*$/gm, "$1");
}

export function parseJavaScriptImports(source) {
  const clean = removeComments(source);
  const specifiers = [];
  const patterns = [
    /\b(?:import|export)\s+(?:type\s+)?(?:[^"'`;]*?\s+from\s+)?["']([^"']+)["']/g,
    /\bimport\s*\(\s*["']([^"']+)["']\s*\)/g,
    /\brequire\s*\(\s*["']([^"']+)["']\s*\)/g,
  ];

  for (const pattern of patterns) {
    for (const match of clean.matchAll(pattern)) {
      specifiers.push(match[1]);
    }
  }

  return [...new Set(specifiers)];
}

export function resolveRelativeImport(importer, specifier) {
  if (!specifier.startsWith(".")) return null;

  const importerDirectory = dirname(resolveRepositoryPath(importer));
  const unresolved = resolve(importerDirectory, specifier);
  const candidates = [];

  if (extname(unresolved)) {
    candidates.push(unresolved);
  } else {
    candidates.push(unresolved);
    for (const extension of sourceExtensions) candidates.push(`${unresolved}${extension}`);
    for (const extension of sourceExtensions) candidates.push(resolve(unresolved, `index${extension}`));
  }

  for (const candidate of candidates) {
    const relativeCandidate = repositoryPath(candidate);
    if (pathExists(relativeCandidate)) return normalizePath(relativeCandidate);
  }

  return normalizePath(repositoryPath(unresolved));
}

export function isPackageImport(specifier, packageName) {
  return specifier === packageName || specifier.startsWith(`${packageName}/`);
}
