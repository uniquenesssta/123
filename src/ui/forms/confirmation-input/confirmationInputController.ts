import { EventScope } from "../../../app/lifecycle";
import { setFieldMessage } from "../form-field";

export interface ConfirmationGateOptions { showMismatch?: boolean; canConfirm?: boolean }

/** Presentation gate only. Host/backend must revalidate before any command. */
export function syncConfirmationInput(input: HTMLInputElement, button: HTMLButtonElement, o: ConfirmationGateOptions = {}): boolean {
  const expected = input.dataset.confirmationExpected ?? "";
  const matched = !!expected.trim() && input.value.trim() === expected;
  const valid = matched && !input.matches(":disabled") && button.getAttribute("aria-busy") !== "true" && o.canConfirm !== false;
  button.disabled = !valid;
  const field = input.closest<HTMLElement>("[data-ui-form-field]"), host = input.closest<HTMLElement>("[data-confirmation-input]");
  const error = o.showMismatch && !matched && input.value ? `名称不匹配，请完整输入 ${expected}` : null;
  if (field) setFieldMessage(field, error, input.dataset.confirmationHint ?? "");
  const status = host?.querySelector<HTMLElement>("[data-confirmation-status]");
  if (status) status.textContent = matched ? "名称已匹配" : error ?? "";
  return valid;
}

export function mountConfirmationInput(input: HTMLInputElement, button: HTMLButtonElement, canConfirm: () => boolean = () => true) {
  const scope = new EventScope(); let composing = false, touched = false;
  const sync = () => { if (!scope.signal.aborted) syncConfirmationInput(input, button, { showMismatch: touched && !composing, canConfirm: canConfirm() && !composing }); };
  scope.listen(input, "compositionstart", () => { composing = true; sync(); });
  scope.listen(input, "compositionend", () => { composing = false; sync(); });
  scope.listen(input, "input", sync);
  scope.listen(input, "blur", () => { touched = true; sync(); });
  if (input.form) scope.listen(input.form, "reset", () => { scope.setTimeout(() => { touched = false; composing = false; sync(); }); });
  sync();
  return { sync, destroy() { scope.destroy(); } };
}
