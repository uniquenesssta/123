import { LifecycleError, lifecycleErrors } from "./lifecycleError";

export interface MountLifecycle {
  start(): void | Promise<void>;
  destroy(): void | Promise<void>;
  /** Synchronously invalidate callbacks; asynchronous release follows start settlement. */
  stop?(): void;
  readonly closedMessage?: string;
  readonly destroyFailureMessage?: string;
  startFailureMessage?(): string;
}

/** One start attempt and one cleanup, including partial-start failure and teardown races. */
export class MountHandle {
  private readonly controller = new AbortController();
  private startTask: Promise<void> | null = null;
  private destroyTask: Promise<void> | null = null;
  private cleanupTask: Promise<unknown[]> | null = null;
  private readonly stopErrors: unknown[] = [];

  constructor(private readonly lifecycle: MountLifecycle) {}

  get signal(): AbortSignal { return this.controller.signal; }

  start(): Promise<void> {
    if (this.signal.aborted) {
      return Promise.reject(new LifecycleError(this.lifecycle.closedMessage));
    }
    if (!this.startTask) {
      this.startTask = Promise.resolve().then(() => {
        // destroy() can be called before this microtask gets to start.
        if (!this.signal.aborted) return this.lifecycle.start();
      }).catch(async (error: unknown) => {
        this.stop();
        const errors = await this.cleanup();
        if (errors.length) {
          throw new AggregateError([error, ...errors], this.lifecycle.startFailureMessage?.() ?? "挂载启动失败");
        }
        throw error;
      });
    }
    return this.startTask;
  }

  destroy(): Promise<void> {
    if (this.destroyTask) return this.destroyTask;
    this.destroyTask = Promise.resolve().then(async () => {
      try { await this.startTask; } catch { /* start() reports the original failure. */ }
      const errors = await this.cleanup();
      if (errors.length) {
        throw new AggregateError(errors, this.lifecycle.destroyFailureMessage ?? "挂载销毁失败");
      }
    });
    // Publish the task before abort listeners or stop() can reenter destroy().
    this.stop();
    return this.destroyTask;
  }

  stop(): void {
    if (this.signal.aborted) return;
    this.controller.abort();
    try { this.lifecycle.stop?.(); } catch (error: unknown) {
      this.stopErrors.push(...lifecycleErrors(error));
    }
  }

  private cleanup(): Promise<unknown[]> {
    if (!this.cleanupTask) {
      this.cleanupTask = Promise.resolve().then(async () => {
        const errors = [...this.stopErrors];
        try { await this.lifecycle.destroy(); } catch (error: unknown) {
          errors.push(...lifecycleErrors(error));
        }
        return errors;
      });
    }
    return this.cleanupTask;
  }
}
