import type { BootstrapResponse, CompetitionDraft, CompetitionKind, SeasonDraft, StageDraft, RoundDraft } from "../../types";

export function formValue(root: ParentNode, id: string): string {
  const input = root.querySelector<HTMLInputElement | HTMLSelectElement>(`#${id}`);
  if (!input) throw new Error(`缺少表单字段：${id}`);
  return input.value.trim();
}
export function readCompetitionDraft(root: ParentNode): CompetitionDraft {
  return { code: "", name: formValue(root, "competition-name"), country_code: formValue(root, "competition-country") || null, timezone: formValue(root, "competition-timezone"), competition_kind: formValue(root, "new-competition-kind") as CompetitionKind, metadata: {} };
}
export function readHierarchyDrafts(root: ParentNode, state: BootstrapResponse) {
  return {
    season(): SeasonDraft {
      const id = formValue(root, "season-competition-id");
      if (!state.data.competitions.some(item => item.id === id && item.is_active)) throw new Error("请选择当前有效赛事");
      return { competition_id: id, name: formValue(root, "season-name"), starts_on: formValue(root, "season-starts-on") || null, ends_on: formValue(root, "season-ends-on") || null, status: formValue(root, "season-status") as SeasonDraft["status"], metadata: {} };
    },
    stage(): StageDraft {
      const id = formValue(root, "stage-season-id");
      if (!state.data.seasons.some(item => item.id === id)) throw new Error("请选择当前有效赛季");
      return { season_id: id, code: "", name: formValue(root, "stage-name"), stage_kind: formValue(root, "stage-kind") as CompetitionKind, sequence_no: Number(formValue(root, "stage-sequence")), rules: {} };
    },
    round(): RoundDraft {
      const id = formValue(root, "round-stage-id");
      if (!state.data.stages.some(item => item.id === id)) throw new Error("请选择当前有效阶段");
      return { stage_id: id, code: "", name: formValue(root, "round-name"), sequence_no: Number(formValue(root, "round-sequence")), starts_at: null, ends_at: null };
    },
  };
}
