const FILE_POLICY = new Map([
  ["analytics.rs", ["analytics", "R2-07"]],
  ["api_workspace.rs", ["ai_workspace", "R2-07"]],
  ["exchange.rs", ["exchange", "R2-07"]],
  ["spreadsheet.rs", ["exchange", "R2-07"]],
  ["monthly_workbook.rs", ["exchange", "R2-07"]],
  ["team_package.rs", ["exchange", "R2-07"]],
  ["fact_pipeline.rs", ["research", "R2-05"]],
  ["research_gateway.rs", ["research", "R2-05"]],
  ["match_event.rs", ["review", "R2-06"]],
  ["match_review_package.rs", ["review", "R2-06"]],
  ["match_review_workflow.rs", ["review", "R2-06"]],
  ["review.rs", ["review", "R2-06"]],
  ["postmatch.rs", ["postmatch", "R2-06"]],
  ["release_acceptance.rs", ["release", "R2-07"]],
]);

const DIRECTORY_POLICY = new Map([
  ["coach", ["coach", "R2-03"]],
  ["competition", ["competition", "R2-02"]],
  ["formation", ["formation", "R2-03"]],
  ["lineup", ["lineup", "R2-04"]],
  ["match_record", ["match_record", "R2-04"]],
  ["player", ["player", "R2-03"]],
  ["prediction", ["prediction", "R2-05"]],
  ["routing", ["routing", "R2-02"]],
  ["shared", ["shared", "R2-03"]],
  ["team", ["team", "R2-03"]],
]);

function rootPolicy(typeName) {
  if (["MatchContext", "PredictionSummary", "PersistedModelRun"].includes(typeName)) return ["prediction", "R2-05"];
  if (typeName === "ModelIdentity" || typeName === "RuleRouting" || typeName.startsWith("Route") || typeName.startsWith("CompetitionBinding") || typeName === "ResolvedCompetitionContext") return ["routing", "R2-02"];
  if (typeName.startsWith("Competition") || typeName.startsWith("Season") || typeName.startsWith("Stage") || typeName.startsWith("Round") || typeName.startsWith("RulePackage") || typeName === "RuleSourceReference") return ["competition", "R2-02"];
  if (typeName === "MatchStatus" || typeName === "MatchDraft" || typeName === "MatchRecord") return ["match_record", "R2-04"];
  if (typeName === "LineupType" || typeName.startsWith("Lineup") || typeName.startsWith("TeamLineupPreset")) return ["lineup", "R2-04"];
  if (typeName.startsWith("Coach") || typeName.startsWith("TeamCoach")) return ["coach", "R2-03"];
  if (typeName.startsWith("Formation") || typeName === "ResolvedFormationDistribution") return ["formation", "R2-03"];
  if (typeName.startsWith("Team")) return ["team", "R2-03"];
  if (typeName === "PreferredFoot" || typeName === "PlayerStatus" || typeName === "AvailabilityStatus" || typeName.startsWith("Player")) return ["player", "R2-03"];
  if (typeName.startsWith("Entity") || typeName.startsWith("Bulk") || typeName.startsWith("DataProvider") || typeName.startsWith("ExternalEntity") || typeName === "AbilityDimensionRecord" || typeName === "PositionReference") return ["shared", "R2-03"];
  throw new Error("R2 目标模块策略缺少根类型：" + typeName);
}

function directoryPolicy(currentPath) {
  const prefix = "crates/domain/src/";
  if (!currentPath.startsWith(prefix)) return null;
  const relative = currentPath.slice(prefix.length);
  const directory = relative.includes("/") ? relative.split("/", 1)[0] : null;
  return directory ? DIRECTORY_POLICY.get(directory) ?? null : null;
}

export function resolveTarget(type) {
  const fileName = type.currentPath.split("/").at(-1);
  const policy = directoryPolicy(type.currentPath) ?? (fileName === "lib.rs" ? rootPolicy(type.typeName) : FILE_POLICY.get(fileName));
  if (!policy) throw new Error("R2 目标模块策略缺少来源文件：" + type.currentPath);
  const [targetModule, targetTask] = policy;
  return {
    targetModule,
    targetTask,
    targetPath: "crates/domain/src/" + targetModule + "/",
  };
}
