import { escapeHtml } from "./format";
import type { CompetitionKind } from "../types";

export const competitionKinds: Array<[CompetitionKind, string]> = [
  ["league", "联赛"],
  ["group_stage", "小组赛"],
  ["knockout_single_leg", "单回合淘汰赛"],
  ["knockout_two_leg", "两回合淘汰赛"],
  ["friendly", "友谊赛"],
  ["custom", "自定义"],
];

export function competitionKindLabel(kind: CompetitionKind | null | undefined): string {
  return competitionKinds.find(([value]) => value === kind)?.[1] ?? "未指定";
}

export function competitionKindOptions(selected: CompetitionKind | null = "custom"): string {
  return competitionKinds
    .map(
      ([value, label]) =>
        `<option value="${value}" ${value === selected ? "selected" : ""}>${escapeHtml(label)}</option>`,
    )
    .join("");
}

export function stageKindOptions(selected: CompetitionKind = "league"): string {
  return competitionKinds
    .filter(([value]) => value !== "friendly")
    .map(
      ([value, label]) =>
        `<option value="${value}" ${value === selected ? "selected" : ""}>${escapeHtml(label)}</option>`,
    )
    .join("");
}
