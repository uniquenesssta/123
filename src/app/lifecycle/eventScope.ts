import { LifecycleError } from "./lifecycleError";

/** Events, timers and animation frames share one synchronous cancellation boundary. */
export class EventScope {
  private readonly controller = new AbortController();
  private readonly listeners = new Set<() => void>();
  private readonly timeouts = new Set<number>();
  private readonly intervals = new Set<number>();
  private readonly frames = new Set<number>();

  get signal(): AbortSignal { return this.controller.signal; }

  listen(target: EventTarget, type: string, listener: EventListener, options: AddEventListenerOptions = {}): () => void {
    this.assertOpen();
    const { signal: externalSignal, once = false, capture = false } = options;
    if (externalSignal?.aborted) return () => {};
    const scope = this;
    const wrapped: EventListener = function (this: EventTarget, event) {
      if (once) off();
      if (!scope.signal.aborted) listener.call(this, event);
    };
    const off = () => {
      target.removeEventListener(type, wrapped, capture);
      externalSignal?.removeEventListener("abort", off);
      this.listeners.delete(off);
    };
    target.addEventListener(type, wrapped, { ...options, signal: this.signal });
    externalSignal?.addEventListener("abort", off, { once: true });
    this.listeners.add(off);
    return off;
  }

  setTimeout(callback: () => void, delay = 0): number {
    this.assertOpen();
    const id = globalThis.setTimeout(() => {
      if (!this.timeouts.delete(id)) return;
      if (!this.signal.aborted) callback();
    }, delay);
    this.timeouts.add(id);
    return id;
  }

  clearTimeout(id: number): void {
    globalThis.clearTimeout(id);
    this.timeouts.delete(id);
  }

  setInterval(callback: () => void, delay: number): number {
    this.assertOpen();
    const id = globalThis.setInterval(() => { if (this.intervals.has(id) && !this.signal.aborted) callback(); }, delay);
    this.intervals.add(id);
    return id;
  }

  clearInterval(id: number): void {
    globalThis.clearInterval(id);
    this.intervals.delete(id);
  }

  requestAnimationFrame(callback: FrameRequestCallback): number {
    this.assertOpen();
    const id = globalThis.requestAnimationFrame(time => {
      if (!this.frames.delete(id)) return;
      if (!this.signal.aborted) callback(time);
    });
    this.frames.add(id);
    return id;
  }

  cancelAnimationFrame(id: number): void {
    globalThis.cancelAnimationFrame(id);
    this.frames.delete(id);
  }

  destroy(): void {
    if (this.signal.aborted) return;
    this.controller.abort();
    for (const off of this.listeners) off();
    for (const id of this.timeouts) globalThis.clearTimeout(id);
    for (const id of this.intervals) globalThis.clearInterval(id);
    for (const id of this.frames) globalThis.cancelAnimationFrame(id);
    this.timeouts.clear();
    this.intervals.clear();
    this.frames.clear();
  }

  private assertOpen(): void {
    if (this.signal.aborted) throw new LifecycleError();
  }
}
