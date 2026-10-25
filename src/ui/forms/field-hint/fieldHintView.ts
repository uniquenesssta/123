import { escapeHtml } from "../../../components/format";

export function fieldHintView(id: string, text: string, disabled = false): string {
  return `<p class="ui-field-hint" id="${escapeHtml(id)}" ${disabled ? 'data-disabled="true"' : ""}>${escapeHtml(text)}</p>`;
}
