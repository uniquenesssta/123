import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = process.cwd();
const scanRoots = [join(root, "crates"), join(root, "src-tauri", "src")];

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function collectRustFiles(directory, output = []) {
  for (const entry of readdirSync(directory)) {
    const path = join(directory, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      collectRustFiles(path, output);
    } else if (entry.endsWith(".rs")) {
      output.push(path);
    }
  }
  return output;
}

function rawStringStart(source, index) {
  if (source[index] !== "r") return null;
  let cursor = index + 1;
  let hashes = 0;
  while (source[cursor] === "#") {
    hashes += 1;
    cursor += 1;
  }
  if (source[cursor] !== '"') return null;
  return { hashes, contentStart: cursor + 1 };
}

function skipQuoted(source, index, quote) {
  let cursor = index + 1;
  while (cursor < source.length) {
    if (source[cursor] === "\\") {
      cursor += 2;
      continue;
    }
    if (source[cursor] === quote) return cursor + 1;
    cursor += 1;
  }
  return source.length;
}

function skipRawString(source, index, hashes, contentStart) {
  const suffix = `"${"#".repeat(hashes)}`;
  const end = source.indexOf(suffix, contentStart);
  return end === -1 ? source.length : end + suffix.length;
}

function skipBlockComment(source, index) {
  let cursor = index + 2;
  let depth = 1;
  while (cursor < source.length && depth > 0) {
    if (source.startsWith("/*", cursor)) {
      depth += 1;
      cursor += 2;
    } else if (source.startsWith("*/", cursor)) {
      depth -= 1;
      cursor += 2;
    } else {
      cursor += 1;
    }
  }
  return cursor;
}

function matchingBrace(source, openingBrace) {
  let depth = 0;
  let cursor = openingBrace;
  while (cursor < source.length) {
    if (source.startsWith("//", cursor)) {
      const end = source.indexOf("\n", cursor + 2);
      cursor = end === -1 ? source.length : end + 1;
      continue;
    }
    if (source.startsWith("/*", cursor)) {
      cursor = skipBlockComment(source, cursor);
      continue;
    }
    const raw = rawStringStart(source, cursor);
    if (raw) {
      cursor = skipRawString(source, cursor, raw.hashes, raw.contentStart);
      continue;
    }
    if (source[cursor] === '"') {
      cursor = skipQuoted(source, cursor, '"');
      continue;
    }
    if (source[cursor] === "'") {
      const next = source[cursor + 1];
      const nextNext = source[cursor + 2];
      const looksLikeChar = next === "\\" || nextNext === "'";
      if (looksLikeChar) {
        cursor = skipQuoted(source, cursor, "'");
        continue;
      }
    }
    if (source[cursor] === "{") depth += 1;
    if (source[cursor] === "}") {
      depth -= 1;
      if (depth === 0) return cursor;
    }
    cursor += 1;
  }
  return -1;
}

function stripTrivia(source) {
  let cursor = 0;
  while (cursor < source.length) {
    if (/\s/.test(source[cursor])) {
      cursor += 1;
      continue;
    }
    if (source.startsWith("//", cursor)) {
      const end = source.indexOf("\n", cursor + 2);
      cursor = end === -1 ? source.length : end + 1;
      continue;
    }
    if (source.startsWith("/*", cursor)) {
      cursor = skipBlockComment(source, cursor);
      continue;
    }
    break;
  }
  return source.slice(cursor);
}

const files = scanRoots.flatMap((directory) => collectRustFiles(directory));
const testModulePattern = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+tests\s*\{/g;
const forbiddenAllows = [
  "clippy::items_after_test_module",
  "clippy::clone_on_copy",
  "clippy::redundant_closure",
  "clippy::field_reassign_with_default",
  "clippy::uninlined_format_args",
];
const pythonPercentFormatPattern = /\{[A-Za-z_][A-Za-z0-9_]*:[^}\r\n]*%\}/g;

for (const file of files) {
  const source = readFileSync(file, "utf8");
  const display = relative(root, file).replaceAll("\\", "/");
  for (const lint of forbiddenAllows) {
    assert(!source.includes(`allow(${lint})`), `${display} 不得通过 #[allow(${lint})] 掩盖严格校验错误`);
  }
  const pythonPercentFormats = [...source.matchAll(pythonPercentFormatPattern)];
  assert(
    pythonPercentFormats.length === 0,
    `${display} 含 Rust 不支持的 Python 风格百分比格式占位符：${pythonPercentFormats.map((item) => item[0]).join(", ")}`,
  );
  if (display === "src-tauri/src/runtime_log.rs") {
    assert(!source.includes("gateway_attempt_sink"), `${display} 不得恢复阶段 1 已停用的结构化网关尝试日志适配器`);
    assert(!source.includes("RuntimeGatewayAttemptSink"), `${display} 不得保留无调用方的网关日志 Sink`);
  }
  if (display === "src-tauri/src/commands/database.rs") {
    assert(!/let\s+Some\(options\)\s*=\s*config\.database\s+else/.test(source), `${display} 的可选数据库配置应使用 ? 提前返回`);
  }
  if (display === "src-tauri/src/openai_profiles.rs") {
    assert(!source.includes('"{}；同时无法回滚兼容 API配置元数据：{rollback_error}"'), `${display} 不得恢复未内联的回滚错误格式化参数`);
  }
  for (const match of source.matchAll(testModulePattern)) {
    const openingBrace = match.index + match[0].lastIndexOf("{");
    const closingBrace = matchingBrace(source, openingBrace);
    assert(closingBrace >= 0, `${display} 的测试模块括号不完整`);
    const remainder = stripTrivia(source.slice(closingBrace + 1));
    assert(remainder.length === 0, `${display} 在 #[cfg(test)] mod tests 之后仍存在项目；测试模块必须位于文件末尾`);
  }
}

console.log(`Rust 源码卫生门禁通过：${files.length} 个文件未发现非法百分比格式占位符、测试模块后置项目、已知桌面端 Clippy 回归或本轮 Clippy 抑制。`);
