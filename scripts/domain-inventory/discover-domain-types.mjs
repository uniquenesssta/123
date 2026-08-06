import { createHash } from "node:crypto";
import { listFiles, readText } from "./files.mjs";

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function maskTestModules(source) {
  const pattern = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{/g;
  const chars = [...source];
  let match;
  while ((match = pattern.exec(source)) !== null) {
    const open = source.indexOf("{", match.index);
    let depth = 0;
    let end = open;
    for (; end < source.length; end += 1) {
      if (source[end] === "{") depth += 1;
      if (source[end] === "}") {
        depth -= 1;
        if (depth === 0) {
          end += 1;
          break;
        }
      }
    }
    for (let index = match.index; index < end; index += 1) {
      if (chars[index] !== "\n") chars[index] = " ";
    }
    pattern.lastIndex = end;
  }
  return chars.join("");
}

function normalizeAttribute(attribute) {
  return attribute.replace(/\s+/g, " ").trim();
}

function collectAttributes(lines, declarationLine) {
  const attributes = [];
  let index = declarationLine - 1;
  while (index >= 0) {
    const trimmed = lines[index].trim();
    if (trimmed === "") {
      if (attributes.length === 0) break;
      index -= 1;
      continue;
    }
    if (!trimmed.startsWith("#[")) break;
    attributes.unshift(normalizeAttribute(trimmed));
    index -= 1;
  }
  return attributes;
}

function collectItem(lines, declarationLine, kind) {
  const declaration = lines[declarationLine];
  if (kind === "type") return declaration.trim();
  const tupleOrUnit = !declaration.includes("{") && (declaration.includes("(") || declaration.trimEnd().endsWith(";"));
  if (tupleOrUnit) {
    const output = [];
    for (let index = declarationLine; index < lines.length; index += 1) {
      output.push(lines[index]);
      if (lines[index].includes(";")) break;
    }
    return output.join("\n").trim();
  }
  let depth = 0;
  let started = false;
  const output = [];
  for (let index = declarationLine; index < lines.length; index += 1) {
    const line = lines[index];
    output.push(line);
    for (const char of line) {
      if (char === "{") {
        depth += 1;
        started = true;
      } else if (char === "}") {
        depth -= 1;
      }
    }
    if (started && depth === 0) break;
  }
  return output.join("\n").trim();
}

function parseDerives(attributes) {
  const derive = attributes.find((attribute) => attribute.startsWith("#[derive("));
  if (!derive) return [];
  return derive
    .slice("#[derive(".length, -2)
    .split(",")
    .map((value) => value.trim())
    .filter(Boolean)
    .sort();
}

function parseSerde(attributes) {
  const serdeAttributes = attributes.filter((attribute) => attribute.startsWith("#[serde("));
  const joined = serdeAttributes.join(" ");
  const value = (name) => joined.match(new RegExp(name + "\\s*=\\s*\"([^\"]+)\""))?.[1] ?? null;
  return {
    attributes: serdeAttributes,
    rename: value("rename"),
    renameAll: value("rename_all"),
    tag: value("tag"),
    content: value("content"),
    transparent: /\btransparent\b/.test(joined),
    untagged: /\buntagged\b/.test(joined),
  };
}

function collectMemberSerde(itemSource, kind) {
  if (kind === "type") return [];
  const bodyStart = itemSource.indexOf("{");
  const bodyEnd = itemSource.lastIndexOf("}");
  if (bodyStart < 0 || bodyEnd <= bodyStart) return [];
  const lines = itemSource.slice(bodyStart + 1, bodyEnd).split("\n");
  const members = [];
  let pending = [];
  let depth = 0;
  for (const rawLine of lines) {
    const trimmed = rawLine.trim();
    if (depth === 0 && trimmed.startsWith("#[")) {
      pending.push(normalizeAttribute(trimmed));
      continue;
    }
    if (depth === 0 && trimmed && !trimmed.startsWith("//")) {
      const field = trimmed.match(/^(?:pub(?:\([^)]*\))?\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:/);
      const variant = kind === "enum" ? trimmed.match(/^([A-Za-z_][A-Za-z0-9_]*)\s*(?:[,({=]|$)/) : null;
      const name = field?.[1] ?? variant?.[1] ?? null;
      if (name) {
        members.push({
          name,
          serdeAttributes: pending.filter((attribute) => attribute.startsWith("#[serde(")),
        });
        pending = [];
      } else if (!trimmed.endsWith(",")) {
        pending = [];
      }
    }
    for (const char of rawLine) {
      if (char === "{" || char === "(") depth += 1;
      if (char === "}" || char === ")") depth = Math.max(0, depth - 1);
    }
  }
  return members;
}

function moduleName(sourcePath) {
  if (sourcePath.endsWith("/lib.rs")) return "football_domain";
  return "football_domain::" + sourcePath.split("/").at(-1).replace(/\.rs$/, "");
}

export function discoverDomainTypes(root) {
  const files = listFiles(root, "crates/domain/src", (path) => path.endsWith(".rs"));
  const output = [];
  for (const sourcePath of files) {
    const original = readText(root, sourcePath);
    const source = maskTestModules(original);
    const lines = source.split("\n");
    for (let lineIndex = 0; lineIndex < lines.length; lineIndex += 1) {
      const line = lines[lineIndex];
      const match = line.match(/^\s*(pub(?:\([^)]*\))?\s+)?(struct|enum|type)\s+([A-Za-z_][A-Za-z0-9_]*)\b/);
      if (!match) continue;
      const visibilityToken = (match[1] ?? "").trim();
      const kind = match[2];
      const typeName = match[3];
      const attributes = collectAttributes(lines, lineIndex);
      if (attributes.some((attribute) => /cfg\s*\(\s*test\s*\)/.test(attribute))) continue;
      const itemSource = collectItem(lines, lineIndex, kind);
      const derives = parseDerives(attributes);
      output.push({
        typeName,
        kind,
        visibility: visibilityToken === "pub" ? "public" : visibilityToken || "private",
        currentPath: sourcePath,
        currentModule: moduleName(sourcePath),
        sourceLine: lineIndex + 1,
        derives,
        serde: parseSerde(attributes),
        members: collectMemberSerde(itemSource, kind),
        declarationSha256: sha256(attributes.join("\n") + "\n" + itemSource),
      });
    }
  }
  return output.sort((left, right) => left.typeName.localeCompare(right.typeName));
}
