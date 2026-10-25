import { escapeHtml } from "../../../components/format";
import { iconSlot } from "../../icons/iconSlot";
import { primitiveAttributes, primitiveName, type PrimitiveAttributes } from "../controlAttributes";

export interface SelectOption { value: string; label: string; disabled?: boolean }
export interface SelectOptions { id: string; label: string; options: readonly SelectOption[]; value?: string; placeholder?: string; disabled?: boolean; required?: boolean; invalid?: boolean; attributes?: PrimitiveAttributes }

export function selectView(options: SelectOptions): string {
  const id = primitiveName(options.id), name = primitiveName(options.label);
  if (options.attributes?.id !== undefined) throw new Error("Use SelectOptions.id for the interactive control id");
  const triggerAttributes = Object.fromEntries(Object.entries(options.attributes ?? {}).filter(([key]) => key.startsWith("aria-")));
  const nativeAttributes = Object.fromEntries(Object.entries(options.attributes ?? {}).filter(([key]) => !key.startsWith("aria-")));
  const values = options.options.map(option => option.value);
  if (values.some(value => !value) || new Set(values).size !== values.length) throw new Error("Select requires unique non-empty option values");
  const selected = options.options.find(option => option.value === options.value);
  const placeholder = options.placeholder ?? "请选择";
  return `<span class="ui-select" data-ui-primitive="select" data-fip-control-scale="40"><select class="ui-select-native" tabindex="-1" aria-label="${name}" ${primitiveAttributes(nativeAttributes)} ${options.disabled ? "disabled" : ""} ${options.required ? "required" : ""}><option value="" ${selected ? "" : "selected"}>${escapeHtml(placeholder)}</option>${options.options.map(option => `<option value="${escapeHtml(option.value)}" ${option === selected ? "selected" : ""} ${option.disabled ? "disabled" : ""}>${escapeHtml(option.label)}</option>`).join("")}</select><button type="button" class="ui-select-trigger" id="${id}" role="combobox" aria-label="${name}" ${primitiveAttributes(triggerAttributes)} aria-controls="${id}-options" aria-expanded="false" aria-haspopup="listbox" ${options.required ? 'aria-required="true"' : ""} ${options.invalid ? 'aria-invalid="true"' : ""} ${options.disabled ? "disabled" : ""}><span class="ui-select-value" data-placeholder="${!selected}">${escapeHtml(selected?.label ?? placeholder)}</span>${iconSlot("chevron-down", 16)}</button><span class="ui-select-options" id="${id}-options" role="listbox" aria-label="${name}" hidden>${options.options.map((option, index) => `<span class="ui-select-option" role="option" id="${id}-option-${index}" data-value="${escapeHtml(option.value)}" aria-selected="${option === selected}" aria-disabled="${!!option.disabled}"><span>${escapeHtml(option.label)}</span>${iconSlot("check", 16)}</span>`).join("")}</span></span>`;
}
