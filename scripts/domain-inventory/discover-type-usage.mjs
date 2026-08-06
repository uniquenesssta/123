import { createHash } from "node:crypto";
import { listFiles, readText } from "./files.mjs";

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function isDeclarationLine(line, typeName) {
  return new RegExp("\\b(?:struct|enum|type)\\s+" + typeName + "\\b").test(line);
}

export function discoverTypeUsage(root, types) {
  const rustFiles = listFiles(root, ".", (path) => path.endsWith(".rs"));
  const sources = rustFiles.map((path) => ({ path, content: readText(root, path) }));
  const usageDigest = sha256(sources.map(({ path, content }) => path + "\0" + content).join("\0"));
  const byType = new Map();

  for (const type of types) {
    const word = new RegExp("\\b" + type.typeName + "\\b");
    const domainCallers = [];
    const externalCallers = [];
    const databaseMappings = [];
    for (const source of sources) {
      if (!word.test(source.content)) continue;
      if (source.path === type.currentPath) {
        const lines = source.content.split("\n");
        const occurrences = lines.filter((line) => word.test(line) && !isDeclarationLine(line, type.typeName));
        if (occurrences.length === 0) continue;
      }
      if (source.path.startsWith("crates/domain/")) domainCallers.push(source.path);
      else externalCallers.push(source.path);
      if (source.path.startsWith("crates/persistence-postgres/src/")) databaseMappings.push(source.path);
    }
    byType.set(`${type.currentPath}::${type.typeName}`, {
      domainCallers: [...new Set(domainCallers)].sort(),
      externalCallers: [...new Set(externalCallers)].sort(),
      databaseMappings: [...new Set(databaseMappings)].sort(),
    });
  }

  return { byType, usageDigest, rustFileCount: rustFiles.length };
}
