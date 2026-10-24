import { EventScope } from "../lifecycle";
import { FORBIDDEN_CONTROL_PATTERN } from "./workspaceVersion";
import type { WorkspaceControlSnapshot, WorkspaceModuleState } from "./workspaceState";

type Control = HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement;
const detailsSelector = 'details:not([data-workspace-persist="false"])';
const excluded = (element: Element): boolean => !!element.closest('[data-workspace-persist="false"]');
const detailsKey = (element: HTMLDetailsElement, index: number): string => element.id || element.dataset.workspaceKey || element.dataset.viewKey || `details:${index}`;
function safeControl(control: Control): boolean {
  if (!control.id || control.disabled || excluded(control)) return false;
  if (FORBIDDEN_CONTROL_PATTERN.test(`${control.id} ${control.name}`)) return false;
  return !(control instanceof HTMLInputElement && (control.type === "file" || control.type === "password"));
}
function safeFocus(element: HTMLElement): boolean {
  if (excluded(element) || FORBIDDEN_CONTROL_PATTERN.test(element.id)) return false;
  return element instanceof HTMLInputElement || element instanceof HTMLSelectElement || element instanceof HTMLTextAreaElement ? safeControl(element) : true;
}

/** DOM snapshots and deferred layout restoration; owns no persistent document. */
export class WorkspaceRecovery {
  private readonly events = new EventScope();
  private sequence = 0;
  private frame: number | null = null;
  private activityTimer: number | null = null;
  private bound = false;

  bind(host: HTMLElement, pageRoot: () => ParentNode | null, capture: () => void): void {
    if (this.bound || this.events.signal.aborted) return;
    this.bound = true;
    const activity = (event: Event): void => {
      const root = pageRoot();
      if (!(event.target instanceof Node) || !root?.contains(event.target)) return;
      if (this.activityTimer !== null) this.events.clearTimeout(this.activityTimer);
      this.activityTimer = this.events.setTimeout(() => { this.activityTimer = null; capture(); }, 100);
    };
    for (const type of ["input", "change", "scroll", "toggle", "focusout"]) this.events.listen(host, type, activity, { capture: true });
  }

  capture(root: ParentNode, previous: WorkspaceModuleState, includeControls: boolean): Partial<WorkspaceModuleState> | null {
    if (this.events.signal.aborted || root instanceof HTMLElement && root.dataset.workspaceCapture === "suspended") return null;
    const controls: Record<string, WorkspaceControlSnapshot> = includeControls ? {} : previous.controls;
    if (includeControls) root.querySelectorAll<Control>("input[id], select[id], textarea[id]").forEach(control => {
      if (!safeControl(control)) return;
      controls[control.id] = control instanceof HTMLInputElement && (control.type === "checkbox" || control.type === "radio")
        ? { checked: control.checked, value: control.value }
        : control instanceof HTMLSelectElement && control.multiple
          ? { selected_values: Array.from(control.selectedOptions).map(option => option.value) }
          : { value: control.value };
    });
    const open_details: string[] = [];
    root.querySelectorAll<HTMLDetailsElement>(detailsSelector).forEach((element, index) => { if (!excluded(element) && element.open) open_details.push(detailsKey(element, index)); });
    const internal_scrolls: WorkspaceModuleState["internal_scrolls"] = {};
    const regions = Array.from(root.querySelectorAll<HTMLElement>("[data-workspace-scroll-key]"));
    if (root instanceof HTMLElement && root.hasAttribute("data-workspace-scroll-key")) regions.unshift(root);
    for (const region of regions) {
      const key = region.dataset.workspaceScrollKey;
      if (key && !excluded(region)) internal_scrolls[key] = { left: region.scrollLeft, top: region.scrollTop };
    }
    const panel_widths: Record<string, number> = {};
    root.querySelectorAll<HTMLElement>("[data-workspace-panel]").forEach(element => {
      const key = element.dataset.workspacePanel;
      if (key && !excluded(element)) panel_widths[key] = Math.round(element.getBoundingClientRect().width);
    });
    const active = document.activeElement;
    const active_element_id = active instanceof HTMLElement && root.contains(active) && safeFocus(active) ? active.id || null : null;
    return { scroll_x: window.scrollX, scroll_y: window.scrollY, active_element_id, controls, open_details, internal_scrolls, panel_widths };
  }

  restore(root: ParentNode, snapshot: WorkspaceModuleState): void {
    if (this.events.signal.aborted) return;
    this.invalidate();
    if (root instanceof HTMLElement && root.dataset.workspaceCapture === "suspended") return;
    const sequence = this.sequence;
    for (const [id, item] of Object.entries(snapshot.controls)) {
      const control = root.querySelector<Control>(`#${CSS.escape(id)}`);
      if (!control || !safeControl(control)) continue;
      if (control instanceof HTMLInputElement && (control.type === "checkbox" || control.type === "radio")) {
        if (typeof item.checked === "boolean") control.checked = item.checked;
      } else if (control instanceof HTMLSelectElement && control.multiple && item.selected_values) {
        const selected = new Set(item.selected_values);
        Array.from(control.options).forEach(option => { option.selected = selected.has(option.value); });
      } else if (item.value !== undefined) {
        if (control instanceof HTMLSelectElement && !Array.from(control.options).some(option => option.value === item.value)) continue;
        control.value = item.value;
      }
    }
    root.querySelectorAll<HTMLDetailsElement>(detailsSelector).forEach((element, index) => {
      if (!excluded(element)) element.open = snapshot.open_details.includes(detailsKey(element, index));
    });
    for (const [key, width] of Object.entries(snapshot.panel_widths)) {
      const element = root.querySelector<HTMLElement>(`[data-workspace-panel="${CSS.escape(key)}"]`);
      const grid = element?.closest<HTMLElement>(".workspace-grid");
      if (!grid || !Number.isFinite(width)) continue;
      const clamped = Math.min(460, Math.max(240, width));
      grid.style.setProperty(key.includes("inspector") ? "--workspace-inspector-width" : "--module-sidebar-width", `${clamped}px`);
    }
    this.frame = this.events.requestAnimationFrame(() => {
      this.frame = null;
      if (sequence !== this.sequence || root instanceof HTMLElement && !root.isConnected) return;
      for (const [key, position] of Object.entries(snapshot.internal_scrolls)) {
        const region = root instanceof HTMLElement && root.dataset.workspaceScrollKey === key ? root : root.querySelector<HTMLElement>(`[data-workspace-scroll-key="${CSS.escape(key)}"]`);
        if (region && !excluded(region)) region.scrollTo(position.left, position.top);
      }
      window.scrollTo(snapshot.scroll_x, snapshot.scroll_y);
      const active = document.activeElement;
      // A Shell/dialog action focused after render owns focus; delayed restoration must not steal it.
      if (active && active !== document.body && active !== document.documentElement && !root.contains(active)) return;
      const focus = snapshot.active_element_id ? root.querySelector<HTMLElement>(`#${CSS.escape(snapshot.active_element_id)}`) : null;
      if (focus && safeFocus(focus) && !focus.closest('[hidden], [inert], [aria-hidden="true"]') && !focus.matches(":disabled")) focus.focus({ preventScroll: true });
    });
  }
  invalidate(): void {
    this.sequence++;
    if (this.frame !== null) this.events.cancelAnimationFrame(this.frame);
    this.frame = null;
  }
  destroy(): void { this.invalidate(); this.events.destroy(); }
}
