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

const PUBLIC_ROOT_SYMBOLS = new Map([
  ["ai_workspace", ["API_WORKSPACE_SCHEMA_VERSION"]],
  [
    "analytics",
    [
      "AI_ANALYSIS_PACKAGE_FORMAT",
      "AI_ANALYSIS_RESPONSE_FORMAT",
      "ANALYTICS_CALCULATION_VERSION",
    ],
  ],
  [
    "exchange",
    [
      "AI_MATCH_PACKAGE_FORMAT",
      "MATCH_LINEUP_IMPORT_FORMAT",
      "MATCH_LINEUP_IMPORT_LEGACY_FORMAT",
      "PLAYER_IMPORT_FORMAT",
      "PLAYER_MONTHLY_FORMAT",
      "TEAM_MONTHLY_FORMAT",
      "TEAM_PACKAGE_FORMAT",
      "TEAM_PACKAGE_PREVIEW_EXPORT_FORMAT",
    ],
  ],
  ["lineup", ["FORMAL_LINEUP_SNAPSHOT_TYPES"]],
  ["postmatch", ["POSTMATCH_MONITORING_VERSION", "POSTMATCH_SETTLEMENT_VERSION"]],
  [
    "prediction",
    [
      "P4_EVIDENCE_SCHEMA_VERSION",
      "P4_FEATURE_FIELD_COUNT",
      "P4_FREEZE_GRACE_MINUTES",
      "P4_ORCHESTRATION_CONTRACT_VERSION",
      "P4_ORCHESTRATION_PLANNER_VERSION",
      "P4_PERSISTENCE_CONTRACT_VERSION",
      "P4_RESEARCH_LEAD_MINUTES",
      "P4_SNAPSHOT_SCHEMA_VERSION",
      "P4_WORKBENCH_CONTRACT_VERSION",
      "PREDICTION_INPUT_AUDIT_VERSION",
    ],
  ],
  [
    "release",
    ["RELEASE_ACCEPTANCE_CONTRACT_VERSION", "RELEASE_ACCEPTANCE_FIXTURE_VERSION"],
  ],
  [
    "research",
    [
      "P4_EVIDENCE_ROUTE_VERSION",
      "P4_FACT_PIPELINE_CONTRACT_VERSION",
      "P4_RESEARCH_GATEWAY_CONTRACT_VERSION",
      "P4_RESEARCH_OUTPUT_SCHEMA_VERSION",
      "P4_RESEARCH_PROMPT_VERSION",
      "P4_SOURCE_POLICY_VERSION",
    ],
  ],
  ["review", ["MATCH_REVIEW_PACKAGE_FORMAT"]],
]);

function compareNames(left, right) {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

export function publicCompatibilityRootSymbols() {
  return new Map(
    [...PUBLIC_ROOT_SYMBOLS.entries()].map(([moduleName, names]) => [
      moduleName,
      [...names].sort(compareNames),
    ]),
  );
}

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
    const names = grouped.get(moduleName).sort(compareNames);
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

  for (const moduleName of DOMAIN_MODULES) {
    const symbols = [...(PUBLIC_ROOT_SYMBOLS.get(moduleName) ?? [])].sort(compareNames);
    if (symbols.length === 0) continue;
    lines.push(`pub use ${moduleName}::{`);
    for (const symbol of symbols) {
      lines.push(`    ${symbol},`);
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
    .sort(compareNames);
}
