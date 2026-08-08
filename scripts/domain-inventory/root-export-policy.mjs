const DOMAIN_MODULES = [
  "ai_workspace",
  "analytics",
  "coach",
  "competition",
  "exchange",
  "formation",
  "lineup",
  "match_record",
  "player",
  "postmatch",
  "prediction",
  "release",
  "research",
  "review",
  "routing",
  "shared",
  "team",
];

export function renderDomainRoot(inventory) {
  if (!inventory || !Array.isArray(inventory.types)) {
    throw new Error("Domain 类型清单缺少 types 数组");
  }

  const grouped = new Map(DOMAIN_MODULES.map((moduleName) => [moduleName, []]));
  const seen = new Set();

  for (const entry of inventory.types) {
    if (!entry.publicCompatibilityType) {
      continue;
    }
    if (!grouped.has(entry.targetModule)) {
      throw new Error(`公共兼容类型 ${entry.typeName} 缺少受支持的目标模块：${entry.targetModule}`);
    }
    if (seen.has(entry.typeName)) {
      throw new Error(`公共兼容类型重复：${entry.typeName}`);
    }
    seen.add(entry.typeName);
    grouped.get(entry.targetModule).push(entry.typeName);
  }

  if (seen.size !== inventory.summary.publicCompatibilityTypeCount) {
    throw new Error(
      `公共兼容类型数量不一致：清单 ${inventory.summary.publicCompatibilityTypeCount}，根出口 ${seen.size}`,
    );
  }

  const lines = [];
  for (const moduleName of DOMAIN_MODULES) {
    lines.push(`pub mod ${moduleName};`);
  }
  lines.push("");

  for (const moduleName of DOMAIN_MODULES) {
    const names = grouped.get(moduleName).sort((left, right) => left.localeCompare(right, "en"));
    if (names.length === 0) {
      throw new Error(`目标模块没有公共兼容类型：${moduleName}`);
    }
    lines.push(`pub use ${moduleName}::{`);
    for (const name of names) {
      lines.push(`    ${name},`);
    }
    lines.push("};");
    lines.push("");
  }

  lines.push(
    "pub(crate) use shared::defaults::{default_confidence, default_team_page_limit, default_true};",
  );
  lines.push("");
  return lines.join("\n");
}

export function publicCompatibilityTypeNames(inventory) {
  return inventory.types
    .filter((entry) => entry.publicCompatibilityType)
    .map((entry) => entry.typeName)
    .sort((left, right) => left.localeCompare(right, "en"));
}
