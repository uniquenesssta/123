export class LoggedInvokeError extends Error {
  readonly issueLogged = true;
  constructor(message: string, cause?: unknown) {
    super(message, { cause });
    this.name = "LoggedInvokeError";
  }
}

/** Cancels local result delivery; it does not claim to undo an already dispatched native command. */
export class InvokeCancelledError extends Error {
  readonly code = "REQUEST_CANCELLED";
  constructor(readonly requestId: string) {
    super("请求已取消");
    this.name = "InvokeCancelledError";
  }
}

export function invokeErrorMessage(error: unknown): string { return error instanceof Error ? error.message : String(error); }
export function isInvokeCancelled(error: unknown): error is InvokeCancelledError { return error instanceof InvokeCancelledError; }
export function issueWasLogged(error: unknown): boolean {
  return error instanceof LoggedInvokeError || error instanceof Error && "issueLogged" in error && (error as Error & { issueLogged?: boolean }).issueLogged === true;
}
