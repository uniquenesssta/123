import { escapeHtml } from "../../../components/format";
import { primitiveAttributes, primitiveName, type PrimitiveAttributes } from "../controlAttributes";

export interface TextAreaOptions {
  label: string; value?: string; placeholder?: string; height?: 96 | 160; resize?: boolean;
  disabled?: boolean; readOnly?: boolean; required?: boolean; invalid?: boolean; maxLength?: number; attributes?: PrimitiveAttributes;
}

export function textAreaView(options: TextAreaOptions): string {
  return `<textarea class="ui-text-area" data-ui-primitive="text-area" data-height="${options.height ?? 96}" data-resize="${options.resize !== false}" aria-label="${primitiveName(options.label)}" placeholder="${escapeHtml(options.placeholder ?? "")}" ${options.disabled ? "disabled" : ""} ${options.readOnly ? "readonly" : ""} ${options.required ? "required" : ""} ${options.invalid ? 'aria-invalid="true"' : ""} ${options.maxLength === undefined ? "" : `maxlength="${Math.max(0, Math.trunc(options.maxLength))}"`} ${primitiveAttributes(options.attributes)}>${escapeHtml(options.value ?? "")}</textarea>`;
}
