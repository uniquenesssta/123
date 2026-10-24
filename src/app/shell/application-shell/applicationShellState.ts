import type { BootstrapResponse, Page, Theme } from "../../../types";

/** Read-only render inputs; Session, Router and Workspace remain the sole owners. */
export interface ShellOptions {
  readonly state: { readonly data: Pick<BootstrapResponse["data"], "database_configured" | "app_version"> };
  readonly page: Page;
  readonly theme: Theme;
  readonly content: string;
  readonly busy: boolean;
  readonly navigationPending: boolean;
  readonly navigationFailed?: boolean;
  /** Existing persisted key: only the secondary navigation is collapsed. */
  readonly sidebarCollapsed: boolean;
}

export interface ShellActions {
  navigate(page: Page): void;
  action(action: string, button: HTMLElement): void;
}

/** Adapters expose the existing owners; Shell only coordinates the UI effects. */
export interface ShellOperations {
  sidebarCollapsed(): boolean;
  setSidebarCollapsed(collapsed: boolean): void;
  toggleTheme(): void;
  renderPreservingWorkspace(): void;
  historyBack(): void;
  historyForward(): void;
  resetWorkspace(): void;
  retryPageRead(): void | Promise<void>;
  openLogs(): void | Promise<void>;
}
