import { requireAccessibleName } from "../../accessibility";
import { escapeHtml } from "../../../components/format";

export interface FieldLabelOptions { id: string; controlId: string; label: string; required?: boolean; disabled?: boolean; hidden?: boolean }

export function fieldLabelView(o: FieldLabelOptions): string {
  requireAccessibleName(o.label);
  return `<label class="ui-field-label" id="${escapeHtml(o.id)}" for="${escapeHtml(o.controlId)}" ${o.disabled ? 'data-disabled="true"' : ""} ${o.hidden ? 'data-visually-hidden="true"' : ""}>${escapeHtml(o.label)}</label>`;
}
