/** Application warnings are presentation DTOs; they do not replace backend issues or readiness gates. */
export type WarningSeverity = "warning" | "error" | "critical";
export type FeedbackTone = "info" | "success" | "warning" | "danger";
export type WarningTarget = "database" | "logs";

export interface WarningInput {
  readonly key: string;
  readonly source: string;
  readonly severity: WarningSeverity;
  readonly title: string;
  readonly message: string;
  readonly target?: WarningTarget;
  readonly occurredAt?: string;
}

export interface WarningRecord extends Omit<WarningInput, "occurredAt"> {
  readonly firstSeenAt: string;
  readonly lastSeenAt: string;
  readonly occurrences: number;
  readonly read: boolean;
  readonly resolved: boolean;
}

export interface FeedbackAction { readonly label: string; readonly target: WarningTarget; }
export interface FeedbackNotice {
  readonly id: number;
  readonly message: string;
  readonly tone: FeedbackTone;
  readonly action?: FeedbackAction;
  readonly autoDismiss: boolean;
}

/** Only a bounded user-facing summary enters the center; never raw credentials or a stack trace. */
export function safeWarningText(value: string): string {
  return value
    .replace(/\b(?:postgres(?:ql)?|https?):\/\/[^\s"'<>]+/gi, "[连接地址已隐藏]")
    .replace(/\bBearer\s+[^\s,;]+/gi, "Bearer [已隐藏]")
    .replace(/["']?\b(api[-_ ]?key|authorization|password|token|secret)["']?\s*[:=]\s*(?:"[^"]*"|'[^']*'|[^\s,;]+)/gi, "$1=[已隐藏]")
    .replace(/\bsk-(?:proj-)?[A-Za-z0-9_-]{8,}\b/g, "[API key 已隐藏]")
    .replace(/\r?\n\s*at\s+[^\n]*/g, "")
    .replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f]/g, "")
    .trim().slice(0, 2000);
}

export function normalizedWarning(input: WarningInput, now: number): WarningInput & { occurredAt: string } {
  const supplied = input.occurredAt ? Date.parse(input.occurredAt) : now;
  return Object.freeze({
    key: safeWarningText(input.key).slice(0, 240), source: safeWarningText(input.source).slice(0, 120),
    severity: input.severity, title: safeWarningText(input.title).slice(0, 160),
    message: safeWarningText(input.message), target: input.target,
    occurredAt: new Date(Number.isFinite(supplied) ? supplied : now).toISOString(),
  });
}
