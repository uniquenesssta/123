import type { RouteDecision } from "../../types";
import {routeSourceLabel} from "./predictionLabels";
export interface ScorelineView {
  score: string;
  probability: number;
  cumulativeProbability: number | null;
  rank: number;
}

export interface PredictionViewData {
  homeTeam: string;
  awayTeam: string;
  homeWin: number | null;
  draw: number | null;
  awayWin: number | null;
  btts: number | null;
  over25: number | null;
  homeLambda: number | null;
  awayLambda: number | null;
  homeCleanSheet: number | null;
  awayCleanSheet: number | null;
  scorelines: ScorelineView[];
  modelId: string;
  modelVersion: string;
  parameterVersion: string;
  rulePackage: string;
  rulePackageVersion: string;
  routeSource: string;
  snapshotType: string;
  runId: string;
  durationMs: number | null;
  createdAt: string | null;
  audit: Record<string, unknown>;
  inputAudit: Record<string, unknown>;
}

function objectValue(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function stringValueFrom(
  record: Record<string, unknown>,
  key: string,
  fallback = "",
): string {
  const candidate = record[key];
  return typeof candidate === "string" && candidate.trim().length > 0
    ? candidate
    : fallback;
}

function numberValueFrom(
  record: Record<string, unknown>,
  key: string,
): number | null {
  const candidate = record[key];
  return typeof candidate === "number" && Number.isFinite(candidate)
    ? candidate
    : null;
}

export function predictionViewData(payload: unknown): PredictionViewData {
  const root = objectValue(payload);
  const rootOutput = objectValue(root.output);
  const wrappedOutput =
    Object.keys(objectValue(rootOutput.identity)).length > 0 ? rootOutput : {};
  const directOutput =
    Object.keys(objectValue(root.identity)).length > 0 &&
    Object.keys(objectValue(root.payload)).length > 0
      ? root
      : {};
  const modelPayload =
    Object.keys(wrappedOutput).length > 0
      ? objectValue(wrappedOutput.payload)
      : Object.keys(directOutput).length > 0
        ? objectValue(directOutput.payload)
        : rootOutput;
  const summary =
    Object.keys(wrappedOutput).length > 0
      ? objectValue(wrappedOutput.summary)
      : objectValue(root.summary);
  const identity =
    Object.keys(wrappedOutput).length > 0
      ? objectValue(wrappedOutput.identity)
      : Object.keys(directOutput).length > 0
        ? objectValue(directOutput.identity)
        : root;
  const input = objectValue(root.input);
  const inputHome = objectValue(input.team_a);
  const inputAway = objectValue(input.team_b);
  const route = objectValue(root.route);
  const rawScorelines = Array.isArray(modelPayload.scorelines)
    ? modelPayload.scorelines
    : [];
  const scorelines = rawScorelines
    .map((item, index): ScorelineView | null => {
      const row = objectValue(item);
      const probability = numberValueFrom(row, "probability");
      if (probability === null) return null;
      const goalsA = numberValueFrom(row, "goals_a");
      const goalsB = numberValueFrom(row, "goals_b");
      const score = stringValueFrom(
        row,
        "score",
        goalsA !== null && goalsB !== null ? `${goalsA}-${goalsB}` : "—",
      );
      const rank = numberValueFrom(row, "rank") ?? index + 1;
      return {
        score,
        probability,
        cumulativeProbability: numberValueFrom(row, "cumulative_probability"),
        rank,
      };
    })
    .filter((item): item is ScorelineView => item !== null)
    .sort(
      (left, right) =>
        left.rank - right.rank || right.probability - left.probability,
    );
  const routeSource = stringValueFrom(route, "source");
  const rulePackage = stringValueFrom(
    route,
    "package_display_name",
    stringValueFrom(root, "rule_package_name", "系统默认规则"),
  );
  const audit =
    Object.keys(wrappedOutput).length > 0
      ? objectValue(wrappedOutput.explanation)
      : objectValue(root.explanation);
  const rawInputAudit = objectValue(root.input_audit);
  const inputAudit: Record<string, unknown> = {
    audit_version: stringValueFrom(rawInputAudit, "audit_version", "未记录"),
    readiness_level: stringValueFrom(rawInputAudit, "readiness_level", "not_assessed"),
    readiness_score: numberValueFrom(rawInputAudit, "readiness_score"),
    input_manifest_sha256: stringValueFrom(
      rawInputAudit,
      "input_manifest_sha256",
      stringValueFrom(rawInputAudit, "manifest_sha256", ""),
    ),
    input_sha256: stringValueFrom(
      rawInputAudit,
      "input_sha256",
      stringValueFrom(root, "input_sha256", ""),
    ),
  };

  return {
    homeTeam: stringValueFrom(
      modelPayload,
      "team_a",
      stringValueFrom(inputHome, "name", "主队"),
    ),
    awayTeam: stringValueFrom(
      modelPayload,
      "team_b",
      stringValueFrom(inputAway, "name", "客队"),
    ),
    homeWin:
      numberValueFrom(summary, "home_win") ??
      numberValueFrom(modelPayload, "a_win"),
    draw:
      numberValueFrom(summary, "draw") ??
      numberValueFrom(modelPayload, "draw"),
    awayWin:
      numberValueFrom(summary, "away_win") ??
      numberValueFrom(modelPayload, "b_win"),
    btts:
      numberValueFrom(summary, "btts") ?? numberValueFrom(modelPayload, "btts"),
    over25:
      numberValueFrom(summary, "over_2_5") ??
      numberValueFrom(modelPayload, "over_2_5"),
    homeLambda: numberValueFrom(modelPayload, "lambda_a"),
    awayLambda: numberValueFrom(modelPayload, "lambda_b"),
    homeCleanSheet: numberValueFrom(modelPayload, "clean_sheet_a"),
    awayCleanSheet: numberValueFrom(modelPayload, "clean_sheet_b"),
    scorelines,
    modelId: stringValueFrom(
      identity,
      "model_id",
      stringValueFrom(root, "model_key", "模型"),
    ),
    modelVersion: stringValueFrom(
      identity,
      "model_version",
      stringValueFrom(root, "model_version", "—"),
    ),
    parameterVersion: stringValueFrom(
      identity,
      "parameter_version",
      stringValueFrom(root, "parameter_version", "—"),
    ),
    rulePackage,
    rulePackageVersion: stringValueFrom(
      route,
      "package_version",
      stringValueFrom(root, "rule_package_version", "—"),
    ),
    routeSource:
      routeSource &&
      [
        "explicit_rule_package",
        "stage_binding",
        "season_binding",
        "competition_binding",
        "competition_kind_default",
      ].includes(routeSource)
        ? routeSourceLabel(routeSource as RouteDecision["source"])
        : "系统自动匹配",
    snapshotType: stringValueFrom(root, "snapshot_type", "—"),
    runId: stringValueFrom(root, "execution_mode") === "shadow"
      ? "影子推演（未写入数据库）"
      : stringValueFrom(
          root,
          "run_id",
          stringValueFrom(root, "id", "未写入数据库"),
        ),
    durationMs: numberValueFrom(root, "duration_ms"),
    createdAt: stringValueFrom(root, "created_at") || null,
    audit,
    inputAudit,
  };
}
