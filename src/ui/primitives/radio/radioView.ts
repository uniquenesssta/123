import defaultRing from "../../../assets/controls/radio-default.svg?raw";
import hoverRing from "../../../assets/controls/radio-hover.svg?raw";
import selectedRing from "../../../assets/controls/radio-selected.svg?raw";
import disabledRing from "../../../assets/controls/radio-disabled.svg?raw";
import disabledSelectedRing from "../../../assets/controls/radio-disabled-selected.svg?raw";
import { escapeHtml } from "../../../components/format";
import { primitiveAttributes, primitiveName, type PrimitiveAttributes } from "../controlAttributes";

export interface RadioOptions { label: string; name: string; value: string; checked?: boolean; disabled?: boolean; required?: boolean; showLabel?: boolean; attributes?: PrimitiveAttributes }

export function radioView(options: RadioOptions): string {
  primitiveName(options.name);
  const rings = { default: defaultRing, hover: hoverRing, selected: selectedRing, disabled: disabledRing, "disabled-selected": disabledSelectedRing };
  return `<label class="ui-radio" data-ui-primitive="radio"><input type="radio" name="${escapeHtml(options.name)}" value="${escapeHtml(options.value)}" aria-label="${primitiveName(options.label)}" ${options.checked ? "checked" : ""} ${options.disabled ? "disabled" : ""} ${options.required ? "required" : ""} ${primitiveAttributes(options.attributes)}><span class="ui-radio-mark" aria-hidden="true">${Object.entries(rings).map(([state, source]) => `<span data-radio-state="${state}">${source}</span>`).join("")}</span>${options.showLabel === false ? "" : `<span class="ui-radio-label">${escapeHtml(options.label)}</span>`}</label>`;
}
