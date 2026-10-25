import { escapeHtml } from "../../../components/format";
export interface StatCardOptions { label: string; value: string | number | null; note?: string }
export function statCardView(o: StatCardOptions): string {
  return `<article class="ui-stat-card" data-ui-display="stat-card"><p class="ui-stat-label">${escapeHtml(o.label)}</p><p class="ui-stat-value">${escapeHtml(o.value ?? "—")}</p>${o.note === undefined ? "" : `<p class="ui-stat-note">${escapeHtml(o.note)}</p>`}</article>`;
}
