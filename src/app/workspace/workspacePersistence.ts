import { EventScope } from "../lifecycle";
import type { WorkspaceStateDocument } from "./workspaceState";

export interface WorkspaceStateAdapter<PageKey extends string> {
  read(): Promise<unknown>;
  save(document: WorkspaceStateDocument<PageKey>): Promise<void>;
  clear(): Promise<unknown>;
}
export interface WorkspaceFailure { readonly operation: "read" | "save" | "clear"; readonly error: unknown }

/** One serialized I/O channel; a clear invalidates queued snapshots from the preceding epoch. */
export class WorkspacePersistence<PageKey extends string> {
  private readonly events = new EventScope();
  private timer: number | null = null;
  private queue: Promise<unknown> = Promise.resolve();
  private epoch = 0;
  private savedRevision = 0;
  private enabled = false;

  constructor(private readonly adapter: WorkspaceStateAdapter<PageKey>, private readonly report: (failure: WorkspaceFailure) => void) {}
  read(): Promise<unknown> { return this.enqueue(() => this.adapter.read()); }
  enable(revision: number): void { this.enabled = true; this.savedRevision = revision; }
  get writable(): boolean { return this.enabled; }
  schedule(snapshot: () => { document: WorkspaceStateDocument<PageKey>; revision: number }): void {
    if (!this.enabled || this.events.signal.aborted) return;
    this.cancelTimer();
    this.timer = this.events.setTimeout(() => {
      this.timer = null;
      const current = snapshot();
      void this.save(current.document, current.revision).catch(error => this.report({ operation: "save", error }));
    }, 250);
  }
  save(document: WorkspaceStateDocument<PageKey>, revision: number): Promise<void> {
    this.cancelTimer();
    if (!this.enabled) return Promise.reject(new Error("工作区状态尚未成功读取，原状态文件已保留"));
    const epoch = this.epoch;
    return this.enqueue(async () => {
      if (epoch !== this.epoch || revision <= this.savedRevision) return;
      await this.adapter.save(document);
      if (epoch === this.epoch) this.savedRevision = revision;
    });
  }
  clear(): Promise<unknown> {
    this.cancelTimer();
    this.epoch++;
    return this.enqueue(() => this.adapter.clear());
  }
  stop(): void { this.cancelTimer(); this.events.destroy(); }
  private cancelTimer(): void {
    if (this.timer !== null) this.events.clearTimeout(this.timer);
    this.timer = null;
  }
  private enqueue<T>(operation: () => Promise<T>): Promise<T> {
    const next = this.queue.then(operation);
    this.queue = next.catch(() => {});
    return next;
  }
}
