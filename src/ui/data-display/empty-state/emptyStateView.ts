import { escapeHtml } from "../../../components/format";
import { buttonView, type ButtonOptions } from "../../primitives/button";
import { iconSlot, type UiIcon } from "../../icons/iconSlot";
export interface EmptyStateOptions { title: string; description?: string; size?: "Compact" | "Default"; icon?: UiIcon; action?: ButtonOptions }
export function emptyStateView(o: EmptyStateOptions): string {
  return `<div class="ui-empty-state" data-ui-display="empty-state" data-size="${o.size ?? "Compact"}">${o.icon ? iconSlot(o.icon, 20) : ""}<p class="ui-empty-title">${escapeHtml(o.title)}</p>${o.description === undefined ? "" : `<p class="ui-empty-description">${escapeHtml(o.description)}</p>`}${o.action ? buttonView({ ...o.action, variant: "secondary", size: 32 }) : ""}</div>`;
}
