import { textInputView } from "../../primitives/text-input";
import type { PrimitiveAttributes } from "../../primitives/controlAttributes";
import { formFieldView } from "../form-field";
import { escapeHtml } from "../../../components/format";

export interface ConfirmationInputOptions { id: string; expected: string; label: string; hint: string; value?: string; disabled?: boolean; confirmButtonId?: string; attributes?: PrimitiveAttributes }

export function confirmationInputView(o: ConfirmationInputOptions): string {
  if (!o.expected.trim()) throw new Error("Confirmation requires a non-empty expected name");
  return `<div class="ui-confirmation-input" data-confirmation-input ${o.confirmButtonId ? `data-confirmation-button="${escapeHtml(o.confirmButtonId)}"` : ""}>${formFieldView({ id: o.id, label: o.label, hint: o.hint, disabled: o.disabled, control: binding => textInputView({ ...binding, value: o.value, autoComplete: "off", spellCheck: false, placeholder: "请输入完整名称", attributes: { ...o.attributes, ...binding.attributes, "data-confirmation-expected": o.expected, "data-confirmation-hint": o.hint } }) })}<span class="ui-confirmation-status" aria-live="polite" aria-atomic="true" data-confirmation-status></span></div>`;
}
