import type { BootstrapResponse, ModelRunListItem, Theme } from "../../types";
import { AsyncScope, LifecycleError, MountHandle } from "../lifecycle";
import { AppSessionState, type AppSessionSnapshot } from "./appSessionState";
import { BootstrapState } from "./bootstrapState";
import { SessionPersistence } from "./sessionPersistence";
import { ThemeState } from "./themeState";

export interface AppSessionAdapters {
  loadBootstrap(): Promise<BootstrapResponse>;
  loadRecentRuns(): Promise<ModelRunListItem[]>;
  applyTheme(theme: Theme): void;
}

/** Coordinates application summary and preferences; no route, workspace or feature state. */
export class AppSession {
  private readonly sessionState = new AppSessionState();
  private readonly bootstrapState = new BootstrapState();
  private readonly themeState: ThemeState;
  private readonly bootstrapRequests = new AsyncScope();
  private readonly recentRunRequests = new AsyncScope();
  private readonly lifecycle: MountHandle;
  private bootstrapTask: Promise<boolean> | null = null;
  private recentCommitRevision = 0;

  constructor(private readonly adapters: AppSessionAdapters, private readonly persistence = new SessionPersistence()) {
    this.themeState = new ThemeState(persistence.readTheme());
    // Preserve the existing initial theme before the first application render.
    adapters.applyTheme(this.themeState.current);
    this.lifecycle = new MountHandle({
      closedMessage: "应用会话生命周期已结束",
      start: async () => {
        this.sessionState.starting();
        try {
          await this.refresh();
          if (!this.lifecycle.signal.aborted) this.sessionState.ready();
        } catch (error: unknown) {
          this.sessionState.failed(error);
          throw error;
        }
      },
      stop: () => {
        this.sessionState.stop();
        this.bootstrapRequests.destroy();
        this.recentRunRequests.destroy();
        this.bootstrapTask = null;
      },
      destroy: () => this.bootstrapState.clear(),
    });
  }

  get bootstrap(): BootstrapResponse | null { return this.bootstrapState.snapshot; }
  get theme(): Theme { return this.themeState.current; }
  get state(): AppSessionSnapshot { return this.sessionState.snapshot; }

  start(): Promise<void> { return this.lifecycle.start(); }
  stop(): void { this.lifecycle.stop(); }
  destroy(): Promise<void> { return this.lifecycle.destroy(); }

  refresh(): Promise<boolean> {
    if (this.lifecycle.signal.aborted) return Promise.resolve(false);
    // A startup caller must wait for actual initial data, even if refresh is requested again.
    if (this.state.phase === "starting" && this.bootstrapTask) return this.bootstrapTask;
    const recentRevision = this.recentCommitRevision;
    this.recentRunRequests.invalidate();
    this.bootstrapTask = this.bootstrapRequests.run(() => this.adapters.loadBootstrap(), result => {
      const current = this.bootstrap;
      const snapshot = recentRevision !== this.recentCommitRevision && current
        ? { ...result, data: { ...result.data, recent_runs: current.data.recent_runs } }
        : result;
      this.bootstrapState.replace(snapshot);
    });
    return this.bootstrapTask;
  }

  refreshRecentRuns(): Promise<boolean> {
    if (this.lifecycle.signal.aborted || !this.bootstrap) return Promise.resolve(false);
    return this.recentRunRequests.run(() => this.adapters.loadRecentRuns(), rows => {
      this.bootstrapState.setRecentRuns(rows);
      this.recentCommitRevision++;
    });
  }

  toggleTheme(): Theme {
    if (this.lifecycle.signal.aborted) throw new LifecycleError("应用会话生命周期已结束");
    const next: Theme = this.theme === "dark" ? "light" : "dark";
    this.adapters.applyTheme(next);
    this.themeState.set(next);
    this.persistence.saveTheme(next);
    return next;
  }
}
