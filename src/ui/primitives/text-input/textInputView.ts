import { escapeHtml } from "../../../components/format";
import { iconSlot, type UiIcon } from "../../icons/iconSlot";
import { iconButtonView } from "../icon-button";
import { primitiveAttributes, primitiveName, type PrimitiveAttributes } from "../controlAttributes";

export interface TextInputOptions {
  label: string;
  value?: string;
  placeholder?: string;
  type?: "text" | "search" | "email" | "url" | "tel" | "password";
  disabled?: boolean;
  readOnly?: boolean;
  required?: boolean;
  invalid?: boolean;
  maxLength?: number;
  leading?: UiIcon;
  trailing?: UiIcon;
  clearable?: boolean;
  attributes?: PrimitiveAttributes;
}

/** Forms owns label/hint/error layout; this is its control slot. */
export function textInputView(options: TextInputOptions): string {
  return `<span class="ui-text-input" data-ui-primitive="text-input" data-fip-control-scale="40">${options.leading ? iconSlot(options.leading, 16) : ""}<input class="ui-text-input-control" type="${options.type ?? "text"}" aria-label="${primitiveName(options.label)}" value="${escapeHtml(options.value ?? "")}" placeholder="${escapeHtml(options.placeholder ?? "")}" ${options.disabled ? "disabled" : ""} ${options.readOnly ? "readonly" : ""} ${options.required ? "required" : ""} ${options.invalid ? 'aria-invalid="true"' : ""} ${options.maxLength === undefined ? "" : `maxlength="${Math.max(0, Math.trunc(options.maxLength))}"`} ${primitiveAttributes(options.attributes)}>${options.trailing ? iconSlot(options.trailing, 16) : ""}${options.clearable ? iconButtonView({ label: `清除${options.label}`, icon: "close", className: "ui-input-clear", disabled: options.disabled || options.readOnly, attributes: { "data-clear-input": "true", hidden: !options.value } }) : ""}</span>`;
}
