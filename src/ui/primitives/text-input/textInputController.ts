import { EventScope } from "../../../app/lifecycle/eventScope";

export function mountTextInput(root: HTMLElement) {
  const input = root.querySelector("input")!, clear = root.querySelector<HTMLButtonElement>("[data-clear-input]");
  const scope = new EventScope();
  const sync = () => { if (clear) { clear.hidden = !input.value; clear.disabled = input.matches(":disabled") || input.readOnly; } };
  scope.listen(input, "input", sync); scope.listen(input, "change", sync);
  if (input.form) scope.listen(input.form, "reset", () => { scope.setTimeout(sync, 0); });
  if (clear) scope.listen(clear, "click", () => {
    if (input.matches(":disabled") || input.readOnly || clear.disabled) return;
    input.value = ""; input.dispatchEvent(new Event("input", { bubbles: true })); input.dispatchEvent(new Event("change", { bubbles: true })); input.focus();
  });
  sync(); return { sync, destroy: () => scope.destroy() };
}
