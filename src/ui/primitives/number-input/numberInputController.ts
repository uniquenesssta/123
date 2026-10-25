import { EventScope } from "../../../app/lifecycle/eventScope";

/** Native decimal stepping and validity own numeric semantics. */
export function mountNumberInput(root: HTMLElement) {
  const input = root.querySelector("input")!, scope = new EventScope();
  const buttons = [...root.querySelectorAll<HTMLButtonElement>("[data-number-step]")];
  const sync = () => { for (const button of buttons) button.disabled = input.matches(":disabled") || input.readOnly || input.step === "any" || (Number.isFinite(input.valueAsNumber) && (button.dataset.numberStep === "up" ? input.max !== "" && input.valueAsNumber >= Number(input.max) : input.min !== "" && input.valueAsNumber <= Number(input.min))); };
  scope.listen(root, "click", event => {
    const button = (event.target as Element).closest<HTMLButtonElement>("[data-number-step]");
    if (!button || !root.contains(button) || button.disabled || input.matches(":disabled") || input.readOnly || input.step === "any") return;
    const before = input.value;
    if (button.dataset.numberStep === "up") input.stepUp(); else input.stepDown();
    if (before !== input.value) { input.dispatchEvent(new Event("input", { bubbles: true })); input.dispatchEvent(new Event("change", { bubbles: true })); }
    sync(); input.focus();
  });
  scope.listen(input, "input", sync); scope.listen(input, "change", sync); sync();
  if (input.form) scope.listen(input.form, "reset", () => { scope.setTimeout(sync, 0); });
  return { sync, destroy: () => scope.destroy() };
}
