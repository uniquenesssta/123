import { escapeHtml } from "../../../components/format";
import type { PrimitiveAttributes } from "../../primitives/controlAttributes";
import { fieldLabelView } from "../field-label";
import { fieldHintView } from "../field-hint";
import { fieldErrorView } from "../field-error";

export interface FieldBinding { id: string; label: string; required: boolean; disabled: boolean; invalid: boolean; attributes: PrimitiveAttributes }
export interface FormFieldOptions {
  id: string; label: string; hint?: string; error?: string; required?: boolean; disabled?: boolean; hiddenLabel?: boolean; describedBy?: readonly string[];
  /** Trusted application renderer, never unescaped user-provided HTML. */
  control(binding: FieldBinding): string;
}

export function formFieldView(o: FormFieldOptions): string {
  if (!/^[A-Za-z][\w:.-]*$/.test(o.id)) throw new Error("A field requires a stable document-unique control id");
  const { id, label } = o, invalid = !!o.error;
  const messageId = o.error ? `${id}-error` : o.hint ? `${id}-hint` : "";
  const description = [...new Set([...(o.describedBy ?? []), messageId].filter(Boolean))].join(" ");
  const attributes: PrimitiveAttributes = { id, "aria-labelledby": `${id}-label`, ...(description ? { "aria-describedby": description } : {}), ...(invalid ? { "aria-invalid": "true", "aria-errormessage": messageId } : {}) };
  return `<div class="ui-form-field" data-ui-form-field="${escapeHtml(id)}">${fieldLabelView({ id: `${id}-label`, controlId: id, label, required: o.required, disabled: o.disabled, hidden: o.hiddenLabel })}${o.control({ id, label, required: !!o.required, disabled: !!o.disabled, invalid, attributes })}<div data-field-message>${o.error ? fieldErrorView(messageId, o.error) : o.hint ? fieldHintView(messageId, o.hint, o.disabled) : ""}</div></div>`;
}
