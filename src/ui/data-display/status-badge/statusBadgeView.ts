import { badgeView } from "../../primitives/badge";
export interface StatusBadgeOptions { label: string; tone?: "Neutral" | "Info" | "Success" | "Warning" | "Danger"; style?: "Filled" | "Outline" }
/** The host supplies a status label/tone; no domain state is inferred here. */
export function statusBadgeView(o: StatusBadgeOptions): string {
  return `<span class="ui-status-badge" data-ui-display="status-badge">${badgeView(o.label, o.tone, o.style)}</span>`;
}
