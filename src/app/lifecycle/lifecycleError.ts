export class LifecycleError extends Error {
  readonly code = "LIFECYCLE_ENDED";

  constructor(message = "生命周期已结束") {
    super(message);
    this.name = "LifecycleError";
  }
}

export function lifecycleErrors(error: unknown): unknown[] {
  return error instanceof AggregateError
    ? error.errors.flatMap(lifecycleErrors)
    : [error];
}
