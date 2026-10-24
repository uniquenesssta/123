import { escapeHtml } from "../../../components/format";
import { iconSlot, type UiIcon } from "../../icons/iconSlot";

/** Minimal approved Button/Icon32 dependency; broader primitive variants belong to R12-11. */
export function iconButtonView(action: string, label: string, icon: UiIcon, className = "", attributes = ""): string {
  return `<button type="button" class="ui-icon-button ${className}" data-action="${escapeHtml(action)}" title="${escapeHtml(label)}" aria-label="${escapeHtml(label)}" ${attributes}>${iconSlot(icon, 16)}</button>`;
}
