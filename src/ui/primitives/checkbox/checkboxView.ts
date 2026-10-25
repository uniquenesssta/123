import { escapeHtml } from "../../../components/format";
import { iconSlot } from "../../icons/iconSlot";
import { primitiveAttributes, primitiveName, type PrimitiveAttributes } from "../controlAttributes";

export interface CheckboxOptions { label: string; checked?: boolean; indeterminate?: boolean; disabled?: boolean; showLabel?: boolean; value?: string; attributes?: PrimitiveAttributes }

export function checkboxView(options: CheckboxOptions): string {
  return `<label class="ui-checkbox" data-ui-primitive="checkbox"><input type="checkbox" aria-label="${primitiveName(options.label)}" ${options.checked ? "checked" : ""} ${options.disabled ? "disabled" : ""} ${options.indeterminate ? 'data-indeterminate="true"' : ""} value="${escapeHtml(options.value ?? "on")}" ${primitiveAttributes(options.attributes)}><span class="ui-checkbox-mark" aria-hidden="true"><span class="ui-checkbox-check">${iconSlot("check", 16)}</span><span class="ui-checkbox-mixed">${iconSlot("minus", 16)}</span></span>${options.showLabel === false ? "" : `<span class="ui-checkbox-label">${escapeHtml(options.label)}</span>`}</label>`;
}
