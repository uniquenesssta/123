import type { WorkspaceControlSnapshot, WorkspaceModuleState, WorkspaceStateDocument, WorkspaceTabState } from "./workspaceState";

export const WORKSPACE_SCHEMA_VERSION = 1;
export const WORKSPACE_UI_REVISION = 5;
export const FORBIDDEN_CONTROL_PATTERN = /(api[-_]?key|password|secret|credential|database[-_]?url|attachment|file[-_]?content)/i;
const reserved = new Set(["__proto__", "prototype", "constructor"]);
const record = (value: unknown): Record<string, unknown> => value !== null && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : {};
const keyAllowed = (key: string): boolean => !!key && !reserved.has(key) && !FORBIDDEN_CONTROL_PATTERN.test(key);
const strings = (value: unknown): string[] => Array.isArray(value) ? [...new Set(value.filter((item): item is string => typeof item === "string"))] : [];
const nullableString = (value: unknown): string | null => typeof value === "string" ? value : null;
const position = (value: unknown): number => typeof value === "number" && Number.isFinite(value) ? Math.max(0, value) : 0;

export function emptyWorkspaceDocument<PageKey extends string>(): WorkspaceStateDocument<PageKey> {
  return { schema_version: 1, global: { sidebar_collapsed: false, ui_revision: 5 }, modules: {} };
}

export function normalizeWorkspaceModule(raw: unknown): WorkspaceModuleState {
  const value = record(raw);
  const controls: Record<string, WorkspaceControlSnapshot> = {};
  for (const [key, rawControl] of Object.entries(record(value.controls))) {
    if (!keyAllowed(key)) continue;
    const control = record(rawControl);
    controls[key] = {
      ...(typeof control.value === "string" ? { value: control.value } : {}),
      ...(typeof control.checked === "boolean" ? { checked: control.checked } : {}),
      ...(Array.isArray(control.selected_values) ? { selected_values: strings(control.selected_values) } : {}),
    };
  }
  const internal_scrolls: WorkspaceModuleState["internal_scrolls"] = {};
  for (const [key, rawScroll] of Object.entries(record(value.internal_scrolls))) {
    if (!keyAllowed(key)) continue;
    const scroll = record(rawScroll);
    internal_scrolls[key] = { left: position(scroll.left), top: position(scroll.top) };
  }
  const panel_widths: Record<string, number> = {};
  for (const [key, width] of Object.entries(record(value.panel_widths))) {
    if (keyAllowed(key) && typeof width === "number" && Number.isFinite(width) && width > 0) panel_widths[key] = width;
  }
  const tabs: WorkspaceTabState[] = [];
  for (const rawTab of Array.isArray(value.tabs) ? value.tabs : []) {
    const tab = record(rawTab);
    if (typeof tab.id !== "string" || !tab.id || typeof tab.label !== "string" || tabs.some(item => item.id === tab.id)) continue;
    tabs.push({ id: tab.id, label: tab.label, ...(typeof tab.pinned === "boolean" ? { pinned: tab.pinned } : {}) });
    if (tabs.length === 6) break;
  }
  return {
    scroll_x: position(value.scroll_x), scroll_y: position(value.scroll_y),
    active_element_id: typeof value.active_element_id === "string" && keyAllowed(value.active_element_id) ? value.active_element_id : null,
    controls, open_details: strings(value.open_details), internal_scrolls,
    selected_object_ids: strings(value.selected_object_ids), tabs,
    active_tab_id: nullableString(value.active_tab_id),
    layout_mode: value.layout_mode === "compare" || value.layout_mode === "cards" ? value.layout_mode : "detail",
    module_sidebar_collapsed: value.module_sidebar_collapsed === true,
    inspector_collapsed: value.inspector_collapsed !== false,
    panel_widths, active_section: nullableString(value.active_section),
  };
}

/** Validate the boundary before recovering; never clear or downgrade an unreadable/future file. */
export function normalizeWorkspaceDocument<PageKey extends string>(raw: unknown): { document: WorkspaceStateDocument<PageKey>; migrated: boolean } {
  const value = record(raw), global = record(value.global);
  if (value.schema_version !== WORKSPACE_SCHEMA_VERSION || (typeof global.ui_revision === "number" && global.ui_revision > WORKSPACE_UI_REVISION)) {
    throw new Error("工作区状态版本与客户端不兼容，原状态文件已保留");
  }
  if (value.global === null || typeof value.global !== "object" || Array.isArray(value.global) || value.modules === null || typeof value.modules !== "object" || Array.isArray(value.modules)) {
    throw new Error("工作区状态文件格式无效，原状态文件已保留");
  }
  const needsUiMigration = global.ui_revision !== WORKSPACE_UI_REVISION;
  const modules: Partial<Record<PageKey, WorkspaceModuleState>> = {};
  for (const [key, rawModule] of Object.entries(record(value.modules))) {
    if (!keyAllowed(key)) continue;
    const module = normalizeWorkspaceModule(rawModule);
    modules[key as PageKey] = needsUiMigration ? { ...module, inspector_collapsed: true, panel_widths: {}, active_section: null } : module;
  }
  return {
    document: { schema_version: 1, global: { sidebar_collapsed: needsUiMigration ? false : global.sidebar_collapsed === true, ui_revision: 5 }, modules },
    migrated: needsUiMigration,
  };
}
