import { createHash } from "node:crypto";
import { discoverDomainTypes } from "./discover-domain-types.mjs";
import { discoverTypeUsage } from "./discover-type-usage.mjs";
import { resolveTarget } from "./target-module-policy.mjs";

function sha256(value) {
  return createHash("sha256").update(value).digest("hex");
}

function countBy(values, field) {
  return Object.fromEntries(
    [...new Set(values.map((value) => value[field]))]
      .sort()
      .map((key) => [key, values.filter((value) => value[field] === key).length]),
  );
}

export function createDomainTypeInventory(root) {
  const types = discoverDomainTypes(root);
  const keys = new Set();
  for (const type of types) {
    const key = `${type.currentPath}::${type.typeName}`;
    if (keys.has(key)) throw new Error("Domain 类型重复登记：" + key);
    keys.add(key);
  }
  const usage = discoverTypeUsage(root, types);
  const entries = types.map((type) => {
    const target = resolveTarget(type);
    const callers = usage.byType.get(`${type.currentPath}::${type.typeName}`);
    const publicCompatibilityType = type.visibility === "public";
    return {
      ...type,
      ...target,
      serializationName: type.serde.rename ?? type.typeName,
      publicCompatibilityType,
      compatibilityLevel: publicCompatibilityType ? "root_public_compatibility" : "module_internal",
      currentRootExport: publicCompatibilityType && type.currentPath !== "crates/domain/src/lib.rs" ? "glob_reexport" : publicCompatibilityType ? "root_definition" : "not_exported",
      targetRootExport: publicCompatibilityType ? "explicit_reexport" : "not_exported",
      databaseMapping: callers.databaseMappings.length > 0 ? "referenced_by_postgres_adapter" : "none_detected",
      ...callers,
    };
  });
  const sourceDigest = sha256(entries.map((entry) => entry.currentPath + "\0" + entry.declarationSha256).join("\0"));
  return {
    schemaVersion: "football.domain-type-inventory.v1",
    sourceRoot: "crates/domain/src",
    callerScope: ["crates/**/*.rs", "src-tauri/**/*.rs"],
    contract: {
      requiredEntryFields: [
        "typeName",
        "kind",
        "visibility",
        "currentPath",
        "targetModule",
        "targetPath",
        "serializationName",
        "serde",
        "databaseMappings",
        "domainCallers",
        "externalCallers",
        "publicCompatibilityType",
      ],
      currentPublicExportPolicy: "root glob re-export is recorded as current debt and removed only in R2-08",
      targetPublicExportPolicy: "explicit re-export only",
    },
    sourceDigest,
    rustUsageDigest: usage.usageDigest,
    summary: {
      typeCount: entries.length,
      publicCompatibilityTypeCount: entries.filter((entry) => entry.publicCompatibilityType).length,
      internalTypeCount: entries.filter((entry) => !entry.publicCompatibilityType).length,
      domainSourceFileCount: new Set(entries.map((entry) => entry.currentPath)).size,
      scannedRustFileCount: usage.rustFileCount,
      databaseMappedTypeCount: entries.filter((entry) => entry.databaseMappings.length > 0).length,
      byTargetModule: countBy(entries, "targetModule"),
      byTargetTask: countBy(entries, "targetTask"),
    },
    types: entries,
  };
}
