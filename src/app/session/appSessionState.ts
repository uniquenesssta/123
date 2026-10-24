export type AppSessionPhase = "created" | "starting" | "ready" | "failed" | "stopped";

export interface AppSessionSnapshot {
  readonly phase: AppSessionPhase;
  readonly error: unknown;
}

/** Observable session phase; MountHandle still owns start/destroy attempts and promises. */
export class AppSessionState {
  private value: AppSessionSnapshot = Object.freeze({ phase: "created", error: null });
  get snapshot(): AppSessionSnapshot { return this.value; }

  starting(): void { this.value = Object.freeze({ phase: "starting", error: null }); }
  ready(): void { if (this.value.phase !== "stopped") this.value = Object.freeze({ phase: "ready", error: null }); }
  failed(error: unknown): void { if (this.value.phase !== "stopped") this.value = Object.freeze({ phase: "failed", error }); }
  stop(): void { this.value = Object.freeze({ phase: "stopped", error: this.value.error }); }
}
