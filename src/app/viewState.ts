export type WorkspaceLayoutMode = "detail" | "compare" | "cards";

export interface WorkspaceTabState {
  readonly id: string;
  readonly label: string;
  readonly pinned?: boolean;
}

interface ControlSnapshot {
  readonly value?: string;
  readonly checked?: boolean;
}

export interface WorkspaceModuleState {
  readonly scroll_x: number;
  readonly scroll_y: number;
  readonly active_element_id: string | null;
  readonly controls: Record<string, ControlSnapshot>;
  readonly open_details: string[];
  readonly internal_scrolls: Record<string, { readonly left: number; readonly top: number }>;
  readonly selected_object_ids: string[];
  readonly tabs: WorkspaceTabState[];
  readonly active_tab_id: string | null;
  readonly layout_mode: WorkspaceLayoutMode;
  readonly module_sidebar_collapsed: boolean;
  readonly inspector_collapsed: boolean;
  readonly panel_widths: Record<string, number>;
  readonly active_section: string | null;
}

export interface WorkspaceStateDocument<PageKey extends string> {
  readonly schema_version: 1;
  readonly global: { readonly sidebar_collapsed: boolean; readonly ui_revision?: number };
  readonly modules: Partial<Record<PageKey, WorkspaceModuleState>>;
}

interface WorkspaceStateAdapter<PageKey extends string> {
  read(): Promise<WorkspaceStateDocument<PageKey>>;
  save(document: WorkspaceStateDocument<PageKey>): Promise<void>;
  clear(): Promise<WorkspaceStateDocument<PageKey>>;
}

const FORBIDDEN_CONTROL_PATTERN = /(api[-_]?key|password|secret|credential|database[-_]?url|attachment|file[-_]?content)/i;

function compactWorkspaceViewport(): boolean {
  return typeof window !== "undefined" && window.matchMedia("(max-width: 900px)").matches;
}

function defaultModuleState(): WorkspaceModuleState {
  return {
    scroll_x: 0,
    scroll_y: 0,
    active_element_id: null,
    controls: {},
    open_details: [],
    internal_scrolls: {},
    selected_object_ids: [],
    tabs: [],
    active_tab_id: null,
    layout_mode: "detail",
    module_sidebar_collapsed: compactWorkspaceViewport(),
    inspector_collapsed: true,
    panel_widths: {},
    active_section: null,
  };
}

function emptyDocument<PageKey extends string>(): WorkspaceStateDocument<PageKey> {
  return { schema_version: 1, global: { sidebar_collapsed: false, ui_revision: 5 }, modules: {} };
}

function detailsKey(element: HTMLDetailsElement, index: number): string {
  return element.id || element.dataset.workspaceKey || element.dataset.viewKey || `details:${index}`;
}

function safeControl(control: HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement): boolean {
  if (!control.id || control.disabled || control.closest('[data-workspace-persist="false"]')) return false;
  if (FORBIDDEN_CONTROL_PATTERN.test(control.id) || FORBIDDEN_CONTROL_PATTERN.test(control.name)) return false;
  return !(control instanceof HTMLInputElement && (control.type === "file" || control.type === "password"));
}

function normalizeModuleState(value: Partial<WorkspaceModuleState> | undefined): WorkspaceModuleState {
  return {
    ...defaultModuleState(),
    ...value,
    controls: { ...(value?.controls ?? {}) },
    open_details: [...(value?.open_details ?? [])],
    internal_scrolls: { ...(value?.internal_scrolls ?? {}) },
    selected_object_ids: [...(value?.selected_object_ids ?? [])],
    tabs: [...(value?.tabs ?? [])].slice(0, 6),
    panel_widths: { ...(value?.panel_widths ?? {}) },
  };
}

export class WorkspaceStateStore<PageKey extends string> {
  private document: WorkspaceStateDocument<PageKey> = emptyDocument<PageKey>();
  private restoreSequence = 0;
  private saveTimer: number | null = null;
  private initialized = false;

  constructor(private readonly adapter: WorkspaceStateAdapter<PageKey>) {}

  async initialize(): Promise<void> {
    try {
      const loaded = await this.adapter.read();
      if (loaded.schema_version !== 1) {
        this.document = emptyDocument<PageKey>();
        await this.adapter.save(this.document);
      } else {
        const needsUiMigration = loaded.global?.ui_revision !== 5;
        const modules = Object.fromEntries(
          Object.entries(loaded.modules ?? {}).map(([key, rawValue]) => {
            const value = rawValue as Partial<WorkspaceModuleState> | undefined;
            return [
              key,
              needsUiMigration
                ? {
                    ...(value ?? {}),
                    module_sidebar_collapsed: compactWorkspaceViewport()
                      ? true
                      : value?.module_sidebar_collapsed === true,
                    inspector_collapsed: true,
                    panel_widths: {},
                    active_section: null,
                  }
                : value,
            ];
          }),
        ) as Partial<Record<PageKey, WorkspaceModuleState>>;
        this.document = {
          schema_version: 1,
          global: {
            sidebar_collapsed: needsUiMigration ? false : loaded.global?.sidebar_collapsed === true,
            ui_revision: 5,
          },
          modules,
        };
        if (needsUiMigration) await this.adapter.save(this.document);
      }
    } catch {
      this.document = await this.adapter.clear();
    }
    this.initialized = true;
  }

  capture(page: PageKey, root: ParentNode, includeControls: boolean): void {
    const previous = this.module(page);
    const controls: Record<string, ControlSnapshot> = includeControls ? {} : previous.controls;
    if (includeControls) {
      root.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>("input[id], select[id], textarea[id]").forEach((control) => {
        if (!safeControl(control)) return;
        controls[control.id] = control instanceof HTMLInputElement && (control.type === "checkbox" || control.type === "radio")
          ? { checked: control.checked, value: control.value }
          : { value: control.value };
      });
    }

    const openDetails: string[] = [];
    root.querySelectorAll<HTMLDetailsElement>('details:not([data-workspace-persist="false"])').forEach((element, index) => {
      if (element.open) openDetails.push(detailsKey(element, index));
    });
    const internalScrolls: Record<string, { left: number; top: number }> = {};
    root.querySelectorAll<HTMLElement>("[data-workspace-scroll-key]").forEach((element) => {
      const key = element.dataset.workspaceScrollKey;
      if (key) internalScrolls[key] = { left: element.scrollLeft, top: element.scrollTop };
    });
    const panelWidths: Record<string, number> = {};
    root.querySelectorAll<HTMLElement>("[data-workspace-panel]").forEach((element) => {
      const key = element.dataset.workspacePanel;
      if (key) panelWidths[key] = Math.round(element.getBoundingClientRect().width);
    });
    const active = document.activeElement;
    this.setModule(page, {
      ...previous,
      scroll_x: window.scrollX,
      scroll_y: window.scrollY,
      active_element_id: active instanceof HTMLElement && active.id ? active.id : null,
      controls,
      open_details: openDetails,
      internal_scrolls: internalScrolls,
      panel_widths: panelWidths,
    });
  }

  restore(page: PageKey, root: ParentNode): void {
    const sequence = ++this.restoreSequence;
    const snapshot = this.module(page);
    for (const [id, item] of Object.entries(snapshot.controls)) {
      const control = root.querySelector<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>(`#${CSS.escape(id)}`);
      if (!control || !safeControl(control)) continue;
      if (control instanceof HTMLInputElement && (control.type === "checkbox" || control.type === "radio")) {
        if (typeof item.checked === "boolean") control.checked = item.checked;
      } else if (item.value !== undefined) {
        if (control instanceof HTMLSelectElement && !Array.from(control.options).some((option) => option.value === item.value)) continue;
        control.value = item.value;
      }
    }
    root.querySelectorAll<HTMLDetailsElement>('details:not([data-workspace-persist="false"])').forEach((element, index) => {
      element.open = snapshot.open_details.includes(detailsKey(element, index));
    });
    for (const [key, width] of Object.entries(snapshot.panel_widths)) {
      const element = root.querySelector<HTMLElement>(`[data-workspace-panel="${CSS.escape(key)}"]`);
      const grid = element?.closest<HTMLElement>(".workspace-grid");
      if (!grid || !Number.isFinite(width)) continue;
      const clamped = Math.min(460, Math.max(240, width));
      grid.style.setProperty(key.includes("inspector") ? "--workspace-inspector-width" : "--module-sidebar-width", `${clamped}px`);
    }
    window.requestAnimationFrame(() => {
      if (sequence !== this.restoreSequence) return;
      for (const [key, position] of Object.entries(snapshot.internal_scrolls)) {
        root.querySelector<HTMLElement>(`[data-workspace-scroll-key="${CSS.escape(key)}"]`)?.scrollTo(position.left, position.top);
      }
      window.scrollTo(snapshot.scroll_x, snapshot.scroll_y);
      if (snapshot.active_element_id) root.querySelector<HTMLElement>(`#${CSS.escape(snapshot.active_element_id)}`)?.focus({ preventScroll: true });
    });
  }

  module(page: PageKey): WorkspaceModuleState { return normalizeModuleState(this.document.modules[page]); }
  patchModule(page: PageKey, patch: Partial<WorkspaceModuleState>): void { this.setModule(page, { ...this.module(page), ...patch }); }
  sidebarCollapsed(): boolean { return this.document.global.sidebar_collapsed; }
  setSidebarCollapsed(collapsed: boolean): void {
    this.document = { ...this.document, global: { ...this.document.global, sidebar_collapsed: collapsed, ui_revision: 5 } };
    this.scheduleSave();
  }
  clear(page: PageKey): void {
    const modules = { ...this.document.modules };
    delete modules[page];
    this.document = { ...this.document, modules };
    this.scheduleSave();
  }
  async clearAll(): Promise<void> { this.restoreSequence += 1; this.document = await this.adapter.clear(); }
  async flush(): Promise<void> {
    if (!this.initialized) return;
    if (this.saveTimer !== null) { window.clearTimeout(this.saveTimer); this.saveTimer = null; }
    await this.adapter.save(this.document);
  }
  async destroy(): Promise<void> {
    this.restoreSequence += 1;
    await this.flush();
    this.initialized = false;
  }

  private setModule(page: PageKey, state: WorkspaceModuleState): void {
    this.document = { ...this.document, modules: { ...this.document.modules, [page]: normalizeModuleState(state) } };
    this.scheduleSave();
  }
  private scheduleSave(): void {
    if (!this.initialized) return;
    if (this.saveTimer !== null) window.clearTimeout(this.saveTimer);
    this.saveTimer = window.setTimeout(() => { this.saveTimer = null; void this.adapter.save(this.document); }, 250);
  }
}
