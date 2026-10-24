import { LifecycleError, lifecycleErrors } from "./lifecycleError";

export type Disposer = () => void | Promise<void>;

export interface Disposable {
  dispose(): void | Promise<void>;
}

/** Owns resources in acquisition order and releases every resource in reverse. */
export class DisposableScope implements Disposable {
  private readonly resources: { dispose: Disposer }[] = [];
  private disposeTask: Promise<void> | null = null;
  private closed = false;

  constructor(private readonly failureMessage = "资源销毁失败") {}

  defer(dispose: Disposer): () => void {
    if (this.closed) throw new LifecycleError();
    const entry = { dispose };
    this.resources.push(entry);
    return () => {
      if (this.closed) return;
      const index = this.resources.indexOf(entry);
      if (index >= 0) this.resources.splice(index, 1);
    };
  }

  add(resource: Disposable): () => void {
    return this.defer(() => resource.dispose());
  }

  dispose(): Promise<void> {
    if (this.disposeTask) return this.disposeTask;
    this.closed = true;
    const resources = this.resources.splice(0).reverse();
    this.disposeTask = Promise.resolve().then(async () => {
      const errors: unknown[] = [];
      for (const resource of resources) {
        try { await resource.dispose(); } catch (error: unknown) {
          errors.push(...lifecycleErrors(error));
        }
      }
      if (errors.length) throw new AggregateError(errors, this.failureMessage);
    });
    return this.disposeTask;
  }
}
