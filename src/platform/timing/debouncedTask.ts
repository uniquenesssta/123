export interface TimerScheduler { setTimeout(callback: () => void, delay: number): number; clearTimeout(id: number): void }

/** One caller-owned trailing callback; scheduling stays in the caller's existing EventScope. */
export class DebouncedTask {
  private timer: number | null = null;
  private revision = 0;
  private closed = false;
  constructor(private readonly scheduler: TimerScheduler) {}
  get pending(): boolean { return this.timer !== null; }
  schedule(callback: () => void, delay: number): void {
    if (this.closed) return;
    this.cancel();
    const revision = this.revision;
    this.timer = this.scheduler.setTimeout(() => {
      if (this.closed || revision !== this.revision) return;
      this.timer = null;
      callback();
    }, delay);
  }
  cancel(): void {
    this.revision++;
    if (this.timer !== null) this.scheduler.clearTimeout(this.timer);
    this.timer = null;
  }
  destroy(): void { this.closed = true; this.cancel(); }
}
