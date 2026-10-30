import { escapeHtml } from "../../components/format";
import type { BootstrapResponse } from "../../types";
export function hierarchyOptions(state: BootstrapResponse) {
  const competitionOptions = state.data.competitions.map((item) => `<option value="${escapeHtml(item.id)}">${escapeHtml(item.name)}</option>`).join("");
  const seasonOptions = state.data.seasons.map((item) => `<option value="${escapeHtml(item.id)}" data-competition-id="${escapeHtml(item.competition_id)}">${escapeHtml(item.competition_name)} · ${escapeHtml(item.name)}</option>`).join("");
  const stageOptions = state.data.stages.map((item) => `<option value="${escapeHtml(item.id)}" data-season-id="${escapeHtml(item.season_id)}" data-competition-id="${escapeHtml(item.competition_id)}">${escapeHtml(item.competition_name)} · ${escapeHtml(item.season_name)} · ${escapeHtml(item.name)}</option>`).join("");
  return { competitionOptions, seasonOptions, stageOptions };
}
