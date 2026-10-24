import { InvokeCancelledError } from "./invokeError";

export interface InvokeRequestLease {
  readonly id: string;
  assertCurrent(): void;
  wait<T>(operation: Promise<T>): Promise<T>;
  finish(): void;
}
interface Registration { cancel(): void; release(): void }

/** The only active IPC-result registry. Each exit releases its AbortSignal listener and registration. */
export class CancellationRegistry {
  private readonly entries = new Map<string, Registration>();
  private closed = false;
  get size(): number { return this.entries.size; }

  begin(id: string, signal?: AbortSignal): InvokeRequestLease {
    if (this.closed || signal?.aborted) throw new InvokeCancelledError(id);
    if (this.entries.has(id)) throw new Error("请求 ID 已在使用");
    let cancelled = false;
    let released = false;
    let rejectCancellation: (reason: InvokeCancelledError) => void;
    const cancellation = new Promise<never>((_resolve, reject) => { rejectCancellation = reject; });
    const release = (): void => {
      if (released) return;
      released = true;
      signal?.removeEventListener("abort", onAbort);
      if (this.entries.get(id) === registration) this.entries.delete(id);
    };
    const onAbort = (): void => {
      if (this.entries.get(id) !== registration) return;
      cancelled = true; release(); rejectCancellation(new InvokeCancelledError(id));
    };
    const registration: Registration = { cancel: onAbort, release };
    this.entries.set(id, registration);
    signal?.addEventListener("abort", onAbort, { once: true });
    return {
      id,
      assertCurrent: () => { if (cancelled || this.closed || this.entries.get(id) !== registration) throw new InvokeCancelledError(id); },
      wait: operation => Promise.race([operation, cancellation]),
      finish: release,
    };
  }
  stop(): void {
    if (this.closed) return;
    this.closed = true;
    for (const entry of [...this.entries.values()]) entry.cancel();
  }
}
