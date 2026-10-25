import { escapeHtml } from "../../../components/format";
import { iconSlot } from "../../icons/iconSlot";

export function fieldErrorView(id: string, text: string): string {
  return `<p class="ui-field-error" id="${escapeHtml(id)}">${iconSlot("alert-circle", 16)}<span>${escapeHtml(text)}</span></p>`;
}
