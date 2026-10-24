import type { IssueLogDraft } from "../../types";
import type { CommandName } from "./commandName";
import { invokeErrorMessage } from "./invokeError";
import { summarizeLogValue, redactLogText } from "./traceSummary";
import { summarizeResult } from "./invokeResponse";

export type RuntimeOperationPhase = "started" | "completed" | "failed" | "ui_action" | "navigation" | "diagnostic";
interface RuntimeOperationDraft { phase: RuntimeOperationPhase; operation: string; traceId: string | null; durationMs: number | null; context: unknown }
export interface TraceAdapter { runtime(draft: Record<string, unknown>): Promise<unknown>; issue(draft: IssueLogDraft): Promise<unknown> }

export function createTraceId(prefix: string): string {
  const random = typeof crypto !== "undefined" && typeof crypto.randomUUID === "function" ? crypto.randomUUID() : `${Date.now()}-${Math.random().toString(16).slice(2)}`;
  return `${prefix}-${random}`;
}

export class InvokeTrace {
  private acceptUi = true;
  constructor(private readonly adapter: TraceAdapter) {}
  async record(draft: RuntimeOperationDraft): Promise<void> {
    try {
      await this.adapter.runtime({ phase: draft.phase, operation: draft.operation, trace_id: draft.traceId, duration_ms: draft.durationMs, context: summarizeLogValue(draft.context) });
    } catch { /* Logging is best effort; retain the original command outcome. */ }
  }
  async issue(command: CommandName, message: string): Promise<void> {
    if (command === "record_issue" || command === "bootstrap") return;
    try { await this.adapter.issue({ severity: "error", source: "backend", operation: command, user_message: redactLogText(message), technical_message: redactLogText(message) }); }
    catch { /* Preserve the original error, including when the log sink fails. */ }
  }
  request(command: CommandName, traceId: string): InvokeTraceLifecycle { return new InvokeTraceLifecycle(this, command, traceId); }
  ui(phase: "ui_action" | "navigation", operation: string, context?: Record<string, unknown>): string {
    const traceId = createTraceId("ui");
    if (this.acceptUi) void this.record({ phase, operation, traceId, durationMs: null, context: context ?? null });
    return traceId;
  }
  uiFailure(operation: string, error: unknown, traceId: string, context?: Record<string, unknown>): void {
    if (this.acceptUi) void this.record({ phase: "failed", operation, traceId, durationMs: null, context: { ...(context ?? {}), error: invokeErrorMessage(error) } });
  }
  diagnostic(operation: string, context?: Record<string, unknown>): void {
    if (this.acceptUi) void this.record({ phase: "diagnostic", operation, traceId: createTraceId("diag"), durationMs: null, context: context ?? null });
  }
  stopUi(): void { this.acceptUi = false; }
}

/** Start precedes its single terminal record, including a cancellation during a slow log write. */
class InvokeTraceLifecycle {
  private terminal = false;
  private queue = Promise.resolve();
  private readonly startedAt = performance.now();
  constructor(private readonly trace: InvokeTrace, private readonly command: CommandName, private readonly id: string) {}
  started(args?: Record<string, unknown>): Promise<void> { return this.write("started", { args: args ?? null }); }
  completed(result: unknown): Promise<void> { return this.end("completed", { result: summarizeResult(result) }); }
  failed(error: unknown, cancelled = false): Promise<void> { return this.end("failed", { error: invokeErrorMessage(error), ...(cancelled ? { cancelled: true } : {}) }); }
  private end(phase: "completed" | "failed", context: unknown): Promise<void> {
    if (this.terminal) return this.queue;
    this.terminal = true;
    return this.write(phase, context);
  }
  private write(phase: "started" | "completed" | "failed", context: unknown): Promise<void> {
    const durationMs = phase === "started" ? null : Math.max(0, Math.round(performance.now() - this.startedAt));
    this.queue = this.queue.then(() => this.trace.record({ phase, operation: this.command, traceId: this.id, durationMs, context }));
    return this.queue;
  }
}
