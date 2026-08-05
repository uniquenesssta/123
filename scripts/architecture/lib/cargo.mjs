import { dirname, resolve } from "node:path";
import {
  normalizePath,
  pathExists,
  readText,
  repositoryPath,
  resolveRepositoryPath,
} from "./repository.mjs";

function stripComment(line) {
  let quoted = false;
  let quote = null;
  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if ((character === '"' || character === "'") && line[index - 1] !== "\\") {
      if (!quoted) {
        quoted = true;
        quote = character;
      } else if (quote === character) {
        quoted = false;
        quote = null;
      }
    }
    if (character === "#" && !quoted) return line.slice(0, index);
  }
  return line;
}

function isDependencySection(section) {
  return section === "dependencies"
    || section === "dev-dependencies"
    || section === "build-dependencies"
    || section.endsWith(".dependencies")
    || section.endsWith(".dev-dependencies")
    || section.endsWith(".build-dependencies");
}

export function parseCargoManifest(relativePath) {
  const content = readText(relativePath);
  const packageMatch = content.match(/\[package\][\s\S]*?^name\s*=\s*["']([^"']+)["']/m);
  const dependencies = [];
  let section = "";

  for (const rawLine of content.split("\n")) {
    const line = stripComment(rawLine).trim();
    if (!line) continue;
    const sectionMatch = line.match(/^\[([^\]]+)\]$/);
    if (sectionMatch) {
      section = sectionMatch[1];
      continue;
    }
    if (!isDependencySection(section)) continue;

    const dependencyMatch = line.match(/^(?:["']([^"']+)["']|([A-Za-z0-9_-]+))\s*=\s*(.+)$/);
    if (!dependencyMatch) continue;
    const key = dependencyMatch[1] ?? dependencyMatch[2];
    const declaration = dependencyMatch[3];
    const packageName = declaration.match(/\bpackage\s*=\s*["']([^"']+)["']/)?.[1] ?? key;
    const pathValue = declaration.match(/\bpath\s*=\s*["']([^"']+)["']/)?.[1] ?? null;
    const resolvedPath = pathValue
      ? normalizePath(repositoryPath(resolve(dirname(resolveRepositoryPath(relativePath)), pathValue)))
      : null;

    dependencies.push({ key, packageName, path: resolvedPath, section });
  }

  return {
    path: relativePath,
    packageName: packageMatch?.[1] ?? null,
    dependencies,
  };
}

export function parseWorkspaceMembers() {
  const content = readText("Cargo.toml");
  const workspaceBlock = content.match(/\[workspace\][\s\S]*?\bmembers\s*=\s*\[([\s\S]*?)\]/);
  if (!workspaceBlock) return [];
  return [...workspaceBlock[1].matchAll(/["']([^"']+)["']/g)].map((match) => normalizePath(match[1])).sort();
}

export function loadWorkspaceGraph(contractCrates) {
  const rootsByPackage = new Map();
  for (const [packageName, definition] of Object.entries(contractCrates)) {
    rootsByPackage.set(packageName, normalizePath(definition.root));
  }

  const manifests = new Map();
  const graph = new Map();
  for (const [packageName, definition] of Object.entries(contractCrates)) {
    const manifestPath = `${normalizePath(definition.root)}/Cargo.toml`;
    if (!pathExists(manifestPath)) {
      manifests.set(packageName, null);
      graph.set(packageName, []);
      continue;
    }

    const manifest = parseCargoManifest(manifestPath);
    manifests.set(packageName, manifest);
    const workspaceDependencies = manifest.dependencies
      .map((dependency) => {
        if (rootsByPackage.has(dependency.packageName)) return dependency.packageName;
        if (!dependency.path) return null;
        return [...rootsByPackage.entries()].find(([, root]) => root === dependency.path)?.[0] ?? null;
      })
      .filter(Boolean);
    graph.set(packageName, [...new Set(workspaceDependencies)].sort());
  }

  return { manifests, graph };
}

export function findCycles(graph) {
  const visiting = new Set();
  const visited = new Set();
  const stack = [];
  const cycles = [];

  function visit(node) {
    if (visiting.has(node)) {
      const start = stack.indexOf(node);
      cycles.push([...stack.slice(start), node]);
      return;
    }
    if (visited.has(node)) return;

    visiting.add(node);
    stack.push(node);
    for (const dependency of graph.get(node) ?? []) visit(dependency);
    stack.pop();
    visiting.delete(node);
    visited.add(node);
  }

  for (const node of graph.keys()) visit(node);
  return cycles;
}
