import { escapeHtml } from "../../../components/format";
export interface ProgressOptions { label: string; value?: number; size?: "Compact" | "Default" }
export function progressView(o: ProgressOptions): string {
  if (!o.label.trim()) throw new Error("Progress requires an accessible name");
  if (o.value !== undefined && !Number.isFinite(o.value)) throw new Error("Progress value must be finite");
  const value = o.value === undefined ? undefined : Math.min(100, Math.max(0, o.value));
  return `<div class="ui-progress" data-ui-display="progress" data-size="${o.size ?? "Compact"}" role="progressbar" aria-label="${escapeHtml(o.label)}" ${value === undefined ? 'data-indeterminate="true"' : `aria-valuemin="0" aria-valuemax="100" aria-valuenow="${value}"`}><span class="ui-progress-fill" ${value === undefined ? "" : `style="width:${value}%"`}></span></div>`;
}
