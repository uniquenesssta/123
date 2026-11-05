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

function dependencySection(section) {
  const match = section.match(/^((?:workspace\.|target\.(?:"(?:\\.|[^"])*"|'[^']*'|[A-Za-z0-9_-]+)\.)?(?:dependencies|dev-dependencies|build-dependencies))(?:\.(?:"([^"]+)"|'([^']+)'|([A-Za-z0-9_-]+)))?$/);
  return match ? { section: match[1], key: match[2] ?? match[3] ?? match[4] ?? null } : null;
}

/** Read row, per-dependency table and dotted declarations through one dependency owner. */
export function parseCargoManifestContent(relativePath, content) {
  const dependencies = new Map();
  let section = "", packageName = null;
  const dependency = (scope, key) => {
    const identity = scope + "|" + key;
    if (!dependencies.has(identity)) dependencies.set(identity, { key, packageName: key, path: null, section: scope, workspace: false });
    return dependencies.get(identity);
  };
  const field = (entry, key, value) => {
    if (key === "workspace") { entry.workspace = value === "true"; return; }
    if (key !== "package" && key !== "path") return;
    const literal = value.match(/^(?:"((?:\\.|[^"])*)"|'([^']*)')$/);
    if (!literal) throw new Error(relativePath + ": dependency " + key + " 必须是字符串");
    const text = literal[2] ?? JSON.parse('"' + literal[1] + '"');
    if (key === "package") entry.packageName = text;
    else entry.path = normalizePath(repositoryPath(resolve(dirname(resolveRepositoryPath(relativePath)), text)));
  };
  for (const rawLine of content.split("\n")) {
    const line = stripComment(rawLine).trim();
    if (!line) continue;
    const header = line.match(/^\[([^\]]+)\]$/);
    if (header) {
      section = header[1];
      if (/^(?:(?:workspace\.)?(?:dependencies|dev-dependencies|build-dependencies)(?:\.|$)|target\..*\.(?:dependencies|dev-dependencies|build-dependencies)(?:\.|$))/.test(section) && !dependencySection(section)) throw new Error(relativePath + ": 无法解析依赖表 " + section);
      const scope = dependencySection(section);
      if (scope?.key) dependency(scope.section, scope.key);
      continue;
    }
    if (section === "package") {
      const name = line.match(/^name\s*=\s*["']([^"']+)["']$/);
      if (name) packageName = name[1];
    }
    const scope = dependencySection(section);
    if (!scope) continue;
    const declaration = line.match(/^(?:"([^"]+)"|'([^']+)'|([A-Za-z0-9_-]+))(?:\.(package|path|workspace))?\s*=\s*(.+)$/);
    if (!declaration) throw new Error(relativePath + ": 无法解析依赖声明 " + line);
    const key = declaration[1] ?? declaration[2] ?? declaration[3], property = declaration[4], value = declaration[5];
    const entry = dependency(scope.section, scope.key ?? key);
    if (scope.key || property) field(entry, property ?? key, value);
    else if (value.startsWith("{")) {
      if (!value.endsWith("}")) throw new Error(relativePath + ": 未闭合的依赖行内表 " + key);
      // Read fields at table boundaries, never package/path text embedded in a quoted feature value.
      const fields = value.slice(1,-1).match(/(?:"(?:\\.|[^"])*"|'[^']*'|\[[^\]]*\]|[^,])+/g) ?? [];
      for (const item of fields) {
        const attribute = item.trim().match(/^(\w+)\s*=\s*(.+)$/);
        if (attribute) field(entry, attribute[1], attribute[2].trim());
      }
    } else if (!/^(?:"(?:\\.|[^"])*"|'[^']*')$/.test(value)) throw new Error(relativePath + ": 无法解析依赖值 " + key);
  }
  return { path: relativePath, packageName, dependencies: [...dependencies.values()] };
}

export function parseCargoManifest(relativePath) {
  const manifest = parseCargoManifestContent(relativePath, readText(relativePath));
  if (relativePath === "Cargo.toml" || !manifest.dependencies.some(entry => entry.workspace)) return manifest;
  const shared = parseCargoManifestContent("Cargo.toml", readText("Cargo.toml")).dependencies;
  for (const entry of manifest.dependencies.filter(item => item.workspace)) {
    const inherited = shared.find(item => item.section === "workspace.dependencies" && item.key === entry.key);
    if (!inherited) throw new Error(relativePath + ": workspace dependency 未登记 " + entry.key);
    entry.packageName = inherited.packageName;
    entry.path ??= inherited.path;
  }
  return manifest;
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
