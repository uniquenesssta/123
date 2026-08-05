import { readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, extname, resolve } from "node:path";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDir, "..");
const clientSource = readFileSync(resolve(projectRoot, "src/api/client.ts"), "utf8");
const desktopSource = readFileSync(resolve(projectRoot, "src-tauri/src/bootstrap/command_registry.rs"), "utf8");
const commandDirectory = resolve(projectRoot, "src-tauri/src/commands");

function uniqueSorted(values) {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

function duplicates(values) {
  const counts = new Map();
  for (const value of values) {
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }
  return [...counts.entries()]
    .filter(([, count]) => count > 1)
    .map(([value]) => value)
    .sort((left, right) => left.localeCompare(right));
}

function difference(left, right) {
  return left.filter((value) => !right.includes(value));
}

const frontendCommands = [];
for (const match of clientSource.matchAll(/\b(?:invoke|tauriInvoke)(?:<[^>]+>)?\(\s*["']([a-z0-9_]+)["']/g)) {
  frontendCommands.push(match[1]);
}

const handlerMatch = desktopSource.match(/tauri::generate_handler!\[([\s\S]*?)\]\s*\)/);
if (!handlerMatch) {
  throw new Error("未找到 Tauri generate_handler 命令注册列表");
}

const registeredCommands = [];
for (const match of handlerMatch[1].matchAll(/commands::([a-z0-9_]+)/g)) {
  registeredCommands.push(match[1]);
}

function rustFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...rustFiles(path));
    } else if (entry.isFile() && extname(entry.name) === ".rs") {
      files.push(path);
    }
  }
  return files;
}

const sourceCommands = [];
for (const path of rustFiles(commandDirectory)) {
  const source = readFileSync(path, "utf8");
  for (const match of source.matchAll(/#\[tauri::command\]\s*pub\s+(?:async\s+)?fn\s+([a-z0-9_]+)/g)) {
    sourceCommands.push(match[1]);
  }
}

const frontend = uniqueSorted(frontendCommands);
const registered = uniqueSorted(registeredCommands);
const source = uniqueSorted(sourceCommands);
const failures = [
  ["前端存在但未注册", difference(frontend, registered)],
  ["后端已注册但前端未使用", difference(registered, frontend)],
  ["源码命令未注册", difference(source, registered)],
  ["注册项没有对应源码命令", difference(registered, source)],
  ["后端命令重复注册", duplicates(registeredCommands)],
  ["源码命令名称重复", duplicates(sourceCommands)],
].filter(([, values]) => values.length > 0);

if (failures.length) {
  throw new Error(`Tauri 命令契约不一致\n${failures
    .map(([label, values]) => `${label}：${values.join(", ")}`)
    .join("\n")}`);
}

if (frontend.length === 0) {
  throw new Error("未发现任何 Tauri 命令");
}

console.log(`Tauri 命令契约通过：前端、注册表与源码共 ${frontend.length} 个命令完全一致。`);
