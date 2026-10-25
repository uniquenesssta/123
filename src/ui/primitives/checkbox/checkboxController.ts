import { EventScope } from "../../../app/lifecycle/eventScope";

export function mountCheckbox(input: HTMLInputElement) {
  const previous = input.indeterminate;
  const initial = input.dataset.indeterminate === "true", scope = new EventScope();
  input.indeterminate = initial;
  if (input.form) scope.listen(input.form, "reset", () => { scope.setTimeout(() => { input.indeterminate = initial; }, 0); });
  let destroyed = false;
  return { setIndeterminate(value: boolean) { if (!destroyed) input.indeterminate = value; }, destroy() { if (destroyed) return; destroyed = true; scope.destroy(); input.indeterminate = previous; } };
}
