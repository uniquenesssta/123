import type { BootstrapResponse } from "../../types";

export interface HierarchySelection {
  competition_id: string | null;
  season_id: string | null;
  stage_id: string | null;
}

/** Frontend identity guard only. Validity and route resolution remain backend decisions. */
export function reconcileHierarchy(state: BootstrapResponse, requested: HierarchySelection): HierarchySelection {
  const competition = state.data.competitions.find(item => item.id === requested.competition_id && item.is_active);
  const season = competition && state.data.seasons.find(item => item.id === requested.season_id && item.competition_id === competition.id);
  const stage = season && state.data.stages.find(item => item.id === requested.stage_id && item.season_id === season.id && item.competition_id === competition!.id);
  return { competition_id: competition?.id ?? null, season_id: season?.id ?? null, stage_id: stage?.id ?? null };
}

export function assertHierarchy(state: BootstrapResponse, requested: HierarchySelection): void {
  const actual = reconcileHierarchy(state, requested);
  if (Object.keys(actual).some(key => actual[key as keyof HierarchySelection] !== requested[key as keyof HierarchySelection])) {
    throw new Error("赛事层级已变化或上下级不匹配，请重新选择赛事、赛季和阶段");
  }
}

/** Also run after Workspace restoration. Every descendant must match all current ancestors. */
export function syncHierarchyControls(root: ParentNode, state: BootstrapResponse, prefix: "binding-" | "", changed?: string): void {
  const competition = root.querySelector<HTMLSelectElement>(`#${prefix}competition-id`);
  const season = root.querySelector<HTMLSelectElement>(`#${prefix}season-id`);
  const stage = root.querySelector<HTMLSelectElement>(`#${prefix}stage-id`);
  if (!competition || !season || !stage) return;
  if (changed === competition.id) { season.value = ""; stage.value = ""; }
  if (changed === season.id) stage.value = "";
  const selected = reconcileHierarchy(state, { competition_id: competition.value || null, season_id: season.value || null, stage_id: stage.value || null });
  competition.value = selected.competition_id ?? "";
  season.value = selected.season_id ?? "";
  stage.value = selected.stage_id ?? "";
  for (const option of Array.from(season.options)) {
    option.disabled = option.hidden = !!option.value && (!selected.competition_id || option.dataset.competitionId !== selected.competition_id);
  }
  for (const option of Array.from(stage.options)) {
    option.disabled = option.hidden = !!option.value && (!selected.season_id || option.dataset.seasonId !== selected.season_id || option.dataset.competitionId !== selected.competition_id);
  }
  season.disabled = !selected.competition_id;
  stage.disabled = !selected.season_id;
  if (prefix === "") {
    const kind = root.querySelector<HTMLSelectElement>("#competition-kind");
    const stageKind = state.data.stages.find(item => item.id === selected.stage_id)?.stage_kind;
    const competitionKind = state.data.competitions.find(item => item.id === selected.competition_id)?.competition_kind;
    if (kind && (stageKind || competitionKind)) kind.value = stageKind ?? competitionKind!;
  }
}
