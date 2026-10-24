import { AsyncScope, LifecycleError } from "../lifecycle";
import { WorkspaceState, type WorkspaceModuleState } from "./workspaceState";
import { normalizeWorkspaceDocument } from "./workspaceVersion";
import { WorkspacePersistence, type WorkspaceFailure, type WorkspaceStateAdapter } from "./workspacePersistence";
import { WorkspaceRecovery } from "./workspaceRecovery";

export interface WorkspaceOptions { readonly onError?: (failure: WorkspaceFailure) => void }

/** Coordinates the sole state owner, version boundary, DOM recovery and persistence channel. */
export class WorkspaceCoordinator<PageKey extends string> {
  private readonly state = new WorkspaceState<PageKey>();
  private readonly persistence: WorkspacePersistence<PageKey>;
  private readonly recovery = new WorkspaceRecovery();
  private readonly initialization = new AsyncScope();
  private initializeTask: Promise<void> | null = null;
  private clearTask: Promise<void> | null = null;
  private destroyTask: Promise<void> | null = null;
  private initialized = false;
  private closed = false;

  constructor(adapter: WorkspaceStateAdapter<PageKey>, private readonly options: WorkspaceOptions = {}) {
    this.persistence = new WorkspacePersistence(adapter, failure => this.report(failure));
  }
  initialize(): Promise<void> {
    if (this.closed) return Promise.reject(new LifecycleError());
    if (this.initialized) return Promise.resolve();
    if (this.initializeTask) return this.initializeTask;
    const request = this.initialization.begin();
    this.initializeTask = (async () => {
      try {
        const loaded = await this.persistence.read();
        if (!request.isCurrent()) return;
        const { document, migrated } = normalizeWorkspaceDocument<PageKey>(loaded);
        const localChanges = this.state.recover(document);
        this.persistence.enable(migrated || localChanges ? 0 : this.state.revision);
        this.initialized = true;
        if (migrated || localChanges) {
          try { await this.flush(); } catch (error) { if (request.isCurrent()) this.report({ operation: "save", error }); }
        }
      } catch (error) {
        if (request.isCurrent()) this.report({ operation: "read", error });
      } finally { request.finish(); this.initializeTask = null; }
    })();
    return this.initializeTask;
  }
  module(page: PageKey): WorkspaceModuleState { return this.state.module(page); }
  start(host: HTMLElement, page: () => PageKey | null, pageRoot: () => ParentNode | null): void {
    if (!this.closed) this.recovery.bind(host, pageRoot, () => {
      const current = page(), root = pageRoot();
      if (current && root) this.capture(current, root, true);
    });
  }
  sidebarCollapsed(): boolean { return this.state.sidebarCollapsed(); }
  patchModule(page: PageKey, patch: Partial<WorkspaceModuleState>): void {
    if (this.closed) return;
    this.state.patchModule(page, patch); this.scheduleSave();
  }
  setSidebarCollapsed(collapsed: boolean): void {
    if (this.closed) return;
    this.state.setSidebarCollapsed(collapsed); this.scheduleSave();
  }
  clear(page: PageKey): void {
    if (this.closed) return;
    this.recovery.invalidate(); this.state.clear(page); this.scheduleSave();
  }
  capture(page: PageKey, root: ParentNode, includeControls: boolean): void {
    if (this.closed) return;
    const patch = this.recovery.capture(root, this.state.module(page), includeControls);
    if (patch) this.patchModule(page, patch);
  }
  restore(page: PageKey, root: ParentNode): void {
    if (!this.closed) this.recovery.restore(root, this.state.module(page));
  }
  clearAll(): Promise<void> {
    if (this.closed) return Promise.reject(new LifecycleError());
    if (this.clearTask) return this.clearTask;
    this.initialization.invalidate(); this.recovery.invalidate();
    const revision = this.state.revision;
    this.clearTask = (async () => {
      try {
        const loaded = await this.persistence.clear();
        if (this.closed) return;
        const { document } = normalizeWorkspaceDocument<PageKey>(loaded);
        const localChanges = this.state.recover(document, revision);
        this.persistence.enable(localChanges ? 0 : this.state.revision);
        this.initialized = true;
      } catch (error) { if (!this.closed) this.report({ operation: "clear", error }); throw error; }
      finally { this.clearTask = null; this.scheduleSave(); }
    })();
    return this.clearTask;
  }
  async flush(): Promise<void> {
    if (this.clearTask) await this.clearTask;
    if (!this.initialized && this.state.revision === 0) return;
    await this.persistence.save(this.state.snapshot(), this.state.revision);
  }
  stop(): void {
    if (this.closed) return;
    this.closed = true; this.initialization.destroy(); this.recovery.destroy(); this.persistence.stop();
  }
  destroy(): Promise<void> {
    if (!this.destroyTask) {
      this.stop();
      this.destroyTask = (this.clearTask ?? (this.initialized ? this.flush() : Promise.resolve())).finally(() => { this.initialized = false; });
    }
    return this.destroyTask;
  }
  private scheduleSave(): void {
    if (!this.closed && !this.clearTask) this.persistence.schedule(() => ({ document: this.state.snapshot(), revision: this.state.revision }));
  }
  private report(failure: WorkspaceFailure): void { if (!this.closed) this.options.onError?.(failure); }
}
