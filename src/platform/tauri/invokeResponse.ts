import type { InvokeRequestLease } from "./cancellationRegistry";
import { summarizeLogValue } from "./traceSummary";

/** Native results keep their original value and identity; only delivery eligibility is checked. */
export function deliverInvokeResponse<T>(request: InvokeRequestLease, value: unknown): T {
  request.assertCurrent();
  return value as T;
}

/** Diagnostics describe response metadata and never transform a successful business result. */
export function summarizeResult(value: unknown): unknown {
  try {
    if (value === null || value === undefined) return { type: "empty" };
    if (Array.isArray(value)) return { type: "array", length: value.length };
    if (typeof value === "object") {
      const entries = Object.entries(value as Record<string, unknown>).slice(0, 40);
      const summary: Record<string, unknown> = { type: "object", keys: entries.map(([key]) => key) };
      for (const [key, childValue] of entries) {
        if (childValue === null || ["string", "number", "boolean", "bigint"].includes(typeof childValue) || Array.isArray(childValue)) summary[key] = summarizeLogValue(childValue, key, 1);
      }
      return summary;
    }
    if (typeof value === "string") return { type: "text", chars: [...value].length };
    return { type: typeof value, value };
  } catch { return { type: "unavailable" }; }
}
