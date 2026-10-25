import { EventScope } from "../../../app/lifecycle/eventScope";

/** Native select owns value/form state. Only open, keyboard-active and typeahead are local. */
export function mountSelect(root: HTMLElement) {
  const native = root.querySelector("select")!, trigger = root.querySelector<HTMLButtonElement>("[role=combobox]")!;
  const list = root.querySelector<HTMLElement>("[role=listbox]")!, value = root.querySelector<HTMLElement>(".ui-select-value")!;
  const options = [...list.querySelectorAll<HTMLElement>("[role=option]")], scope = new EventScope();
  const externalInvalid = trigger.getAttribute("aria-invalid") === "true";
  let active = -1, query = "", timer: number | undefined, destroyed = false, reportedInvalid = false;
  const enabled = () => !destroyed && !native.matches(":disabled") && !trigger.matches(":disabled");
  const available = () => options.map((option, index) => option.getAttribute("aria-disabled") !== "true" ? index : -1).filter(index => index >= 0);
  const close = () => { list.hidden = true; trigger.setAttribute("aria-expanded", "false"); trigger.removeAttribute("aria-activedescendant"); for (const option of options) option.removeAttribute("data-active"); active = -1; query = ""; if (timer !== undefined) scope.clearTimeout(timer); timer = undefined; };
  const activate = (index: number) => {
    active = index;
    options.forEach((option, position) => option.toggleAttribute("data-active", position === index));
    if (index < 0) trigger.removeAttribute("aria-activedescendant"); else { trigger.setAttribute("aria-activedescendant", options[index].id); options[index].scrollIntoView({ block: "nearest" }); }
  };
  const sync = () => {
    trigger.disabled = native.matches(":disabled");
    const selected = native.selectedOptions[0]; value.textContent = selected?.textContent ?? "";
    value.dataset.placeholder = String(!native.value);
    for (const option of options) { const item = [...native.options].find(item => item.value === option.dataset.value); option.setAttribute("aria-selected", String(option.dataset.value === native.value)); option.setAttribute("aria-disabled", String(!!item?.disabled)); }
    if (reportedInvalid && !externalInvalid && native.validity.valid) { trigger.removeAttribute("aria-invalid"); reportedInvalid = false; }
    if (trigger.disabled) close();
  };
  const open = (last = false) => {
    if (!enabled()) return;
    list.hidden = false; trigger.setAttribute("aria-expanded", "true");
    const rect = trigger.getBoundingClientRect(), view = root.ownerDocument.defaultView!;
    const below = view.innerHeight - rect.bottom - 4, above = rect.top - 4;
    const useAbove = below < Math.min(list.scrollHeight + 2, 288, view.innerHeight * 0.4) && above > below;
    const height = Math.max(0, Math.min(288, view.innerHeight * 0.4, useAbove ? above : below));
    list.style.maxHeight = `${height}px`;
    list.style.width = `${Math.min(rect.width, view.innerWidth - 8)}px`;
    list.style.left = `${Math.max(4, Math.min(rect.left, view.innerWidth - Math.min(rect.width, view.innerWidth - 8) - 4))}px`;
    list.style.top = `${useAbove ? rect.top - 4 - Math.min(list.scrollHeight + 2, height) : rect.bottom + 4}px`;
    const indices = available(), selected = options.findIndex(option => option.dataset.value === native.value && option.getAttribute("aria-disabled") !== "true");
    activate(selected >= 0 ? selected : indices[last ? indices.length - 1 : 0] ?? -1);
  };
  const commit = (index: number) => {
    if (!enabled() || index < 0 || options[index]?.getAttribute("aria-disabled") === "true") return;
    const before = native.value; native.value = options[index].dataset.value!; sync(); close(); trigger.focus();
    if (before !== native.value) { native.dispatchEvent(new Event("input", { bubbles: true })); native.dispatchEvent(new Event("change", { bubbles: true })); }
  };
  scope.listen(trigger, "click", () => { if (enabled()) { if (list.hidden) open(); else close(); } });
  scope.listen(trigger, "keydown", event => {
    const key = event as KeyboardEvent;
    if (!enabled() || key.isComposing) return;
    if (key.key === "Tab") { close(); return; }
    if (key.key === "Escape") { if (!list.hidden) { key.preventDefault(); close(); } return; }
    if (["ArrowDown", "ArrowUp", "Home", "End"].includes(key.key)) {
      key.preventDefault(); const wasClosed = list.hidden;
      if (wasClosed) open(key.key === "ArrowUp" || key.key === "End");
      const indices = available(), position = indices.indexOf(active);
      if (key.key === "Home") activate(indices[0] ?? -1); else if (key.key === "End") activate(indices.at(-1) ?? -1);
      else if (!wasClosed) activate(indices[(position + (key.key === "ArrowDown" ? 1 : -1) + indices.length) % indices.length] ?? -1);
      return;
    }
    if (key.key === "Enter" || key.key === " ") { key.preventDefault(); if (list.hidden) open(); else commit(active); return; }
    if (key.key.length === 1 && !key.ctrlKey && !key.metaKey && !key.altKey) {
      key.preventDefault(); if (list.hidden) open(); query += key.key.toLocaleLowerCase();
      if (timer !== undefined) scope.clearTimeout(timer);
      timer = scope.setTimeout(() => { query = ""; timer = undefined; }, 700);
      const index = available().find(index => options[index].textContent?.trim().toLocaleLowerCase().startsWith(query));
      if (index !== undefined) activate(index);
    }
  });
  scope.listen(list, "click", event => { const option = (event.target as Element).closest<HTMLElement>("[role=option]"); if (option && list.contains(option)) commit(options.indexOf(option)); });
  scope.listen(root.ownerDocument, "pointerdown", event => { if (!root.contains(event.target as Node)) close(); }, { capture: true });
  scope.listen(root, "focusout", event => { if (!root.contains((event as FocusEvent).relatedTarget as Node | null)) close(); });
  scope.listen(native, "change", sync);
  scope.listen(native, "invalid", event => { event.preventDefault(); reportedInvalid = true; trigger.setAttribute("aria-invalid", "true"); trigger.focus(); });
  if (native.form) scope.listen(native.form, "reset", () => { scope.setTimeout(() => { reportedInvalid = false; if (!externalInvalid) trigger.removeAttribute("aria-invalid"); sync(); close(); }, 0); });
  const view = root.ownerDocument.defaultView;
  if (view) { scope.listen(view, "resize", close); scope.listen(view, "scroll", event => { if (!(event.target instanceof Node) || !list.contains(event.target)) close(); }, { capture: true }); }
  sync();
  return { sync, close, setValue(next: string) { if (destroyed) return; if (next && ![...native.options].some(option => option.value === next)) throw new Error("Unknown select value"); native.value = next; sync(); close(); }, destroy() { if (destroyed) return; close(); scope.destroy(); destroyed = true; } };
}
