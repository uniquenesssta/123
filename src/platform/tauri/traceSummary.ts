const MAX_LOG_OBJECT_KEYS = 40;
const MAX_LOG_ARRAY_ITEMS = 8;
const MAX_LOG_DEPTH = 4;
const MAX_LOG_STRING_CHARS = 512;
const sensitiveLogKey = (key: string): boolean => /authorization|api[_-]?key|access[_-]?token|refresh[_-]?token|password|secret|credential/i.test(key);
const contentLogKey = (key: string): boolean => /prompt|message|content|body|schema|api[_-]?example|encrypted[_-]?content/i.test(key);

export function redactLogText(value: string): string {
  return value.replace(/((?:postgres(?:ql)?|https?):\/\/)[^\s/@]+(?::[^\s/@]*)?@/gi, "$1[REDACTED]@")
    .replace(/\bBearer\s+[^\s,;"'<>]+/gi, "Bearer [REDACTED]")
    .replace(/\bsk-[a-z0-9_-]{8,}/gi, "[REDACTED]")
    .replace(/((?:api[_-]?key|access[_-]?token|refresh[_-]?token|password|secret|credential|token)["']?\s*[:=]\s*["']?)[^\s,;"'}]+/gi, "$1[REDACTED]");
}

/** Bounded, non-throwing log projection; business arguments/results are never mutated. */
export function summarizeLogValue(value: unknown, key = "", depth = 0): unknown {
  try {
    if (sensitiveLogKey(key)) return "[REDACTED]";
    if (value === null || value === undefined) return value ?? null;
    if (typeof value === "boolean" || typeof value === "number") return value;
    if (typeof value === "bigint") return value.toString();
    if (typeof value === "string") {
      if (contentLogKey(key)) return { type: "text", chars: [...value].length, contentOmitted: true };
      const chars = [...redactLogText(value)];
      return chars.length <= MAX_LOG_STRING_CHARS ? chars.join("") : { type: "text", chars: [...value].length, preview: chars.slice(0, MAX_LOG_STRING_CHARS).join(""), truncated: true };
    }
    if (depth >= MAX_LOG_DEPTH) {
      if (Array.isArray(value)) return { type: "array", length: value.length };
      if (typeof value === "object") return { type: "object", keys: Object.keys(value as Record<string, unknown>).slice(0, MAX_LOG_OBJECT_KEYS) };
      return String(value);
    }
    if (Array.isArray(value)) return { type: "array", length: value.length, sample: value.slice(0, MAX_LOG_ARRAY_ITEMS).map(item => summarizeLogValue(item, key, depth + 1)), truncated: value.length > MAX_LOG_ARRAY_ITEMS };
    if (typeof value === "object") return Object.fromEntries(Object.entries(value as Record<string, unknown>).slice(0, MAX_LOG_OBJECT_KEYS).map(([childKey, childValue]) => [childKey, summarizeLogValue(childValue, childKey, depth + 1)]));
    return String(value);
  } catch { return "[UNAVAILABLE]"; }
}
