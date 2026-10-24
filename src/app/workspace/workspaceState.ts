import { emptyWorkspaceDocument, normalizeWorkspaceModule } from "./workspaceVersion";

export type WorkspaceLayoutMode = "detail" | "compare" | "cards";
export interface WorkspaceTabState { readonly id: string; readonly label: string; readonly pinned?: boolean }
export interface WorkspaceControlSnapshot { readonly value?: string; readonly checked?: boolean; readonly selected_values?: string[] }
export interface WorkspaceModuleState {
  readonly scroll_x: number;
  readonly scroll_y: number;
  readonly active_element_id: string | null;
  readonly controls: Record<string, WorkspaceControlSnapshot>;
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

/** The only owner of the document and its local mutation revisions. No DOM or I/O. */
export class WorkspaceState<PageKey extends string> {
  private document = emptyWorkspaceDocument<PageKey>();
  private currentRevision = 0;
  private readonly pageRevisions = new Map<PageKey, number>();
  private globalRevision = 0;

  get revision(): number { return this.currentRevision; }
  module(page: PageKey): WorkspaceModuleState { return normalizeWorkspaceModule(this.document.modules[page]); }
  snapshot(): WorkspaceStateDocument<PageKey> { return structuredClone(this.document); }
  sidebarCollapsed(): boolean { return this.document.global.sidebar_collapsed; }
  patchModule(page: PageKey, patch: Partial<WorkspaceModuleState>): void {
    this.document = { ...this.document, modules: { ...this.document.modules, [page]: normalizeWorkspaceModule({ ...this.module(page), ...patch }) } };
    this.pageRevisions.set(page, ++this.currentRevision);
  }
  setSidebarCollapsed(collapsed: boolean): void {
    this.document = { ...this.document, global: { sidebar_collapsed: collapsed, ui_revision: 5 } };
    this.globalRevision = ++this.currentRevision;
  }
  clear(page: PageKey): void {
    const modules = { ...this.document.modules };
    delete modules[page];
    this.document = { ...this.document, modules };
    this.pageRevisions.set(page, ++this.currentRevision);
  }
  /** A late read/clear may replace only state untouched since that operation began. */
  recover(loaded: WorkspaceStateDocument<PageKey>, afterRevision = 0): boolean {
    const modules = { ...loaded.modules };
    let localChanges = this.globalRevision > afterRevision;
    for (const [page, revision] of this.pageRevisions) {
      if (revision <= afterRevision) continue;
      localChanges = true;
      if (Object.hasOwn(this.document.modules, page)) modules[page] = this.module(page);
      else delete modules[page];
    }
    this.document = {
      ...loaded,
      global: this.globalRevision > afterRevision ? { ...this.document.global } : { ...loaded.global },
      modules,
    };
    this.currentRevision++;
    return localChanges;
  }
}
