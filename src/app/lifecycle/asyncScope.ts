import { LifecycleError } from "./lifecycleError";
import { RequestSequence } from "../../platform/timing";

export interface AsyncRequest {
  readonly id: number;
  readonly signal: AbortSignal;
  isCurrent(): boolean;
  commit(apply: () => void): boolean;
  finish(): void;
}

/** One latest-wins channel; separate owners must use separate scopes. */
export class AsyncScope {
  private readonly sequence = new RequestSequence();
  private active: AbortController | null = null;
  private closed = false;

  begin(): AsyncRequest {
    if (this.closed) throw new LifecycleError();
    const previous = this.active;
    const controller = new AbortController();
    const id = this.sequence.next();
    this.active = controller;
    previous?.abort();
    const isCurrent = () => !this.closed && this.active === controller && !controller.signal.aborted;
    return {
      id, signal: controller.signal, isCurrent,
      commit(apply) {
        if (!isCurrent()) return false;
        apply();
        return true;
      },
      finish: () => { if (this.active === controller) this.active = null; },
    };
  }

  invalidate(): void {
    const active = this.active;
    this.active = null;
    active?.abort();
  }

  destroy(): void {
    this.closed = true;
    this.invalidate();
  }

  async run<T>(load: (request: AsyncRequest) => Promise<T>, apply: (value: T) => void): Promise<boolean> {
    const request = this.begin();
    try {
      let result: T;
      try {
        result = await load(request);
      } catch (error: unknown) {
        if (!request.isCurrent()) return false;
        throw error;
      }
      return request.commit(() => apply(result));
    } finally {
      request.finish();
    }
  }
}
