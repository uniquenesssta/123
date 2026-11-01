export function formatTime(value: string | null): string {
  if (!value) return "—";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

export function formatValue(value: unknown): string {
  if (value === null || value === undefined) return "未知";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

export function safeSourceUrl(value: string | null): string | null {
  if (!value) return null;
  try {
    const url = new URL(value);
    return url.protocol === "https:" || url.protocol === "http:" ? url.toString() : null;
  } catch {
    return null;
  }
}

export function taskStateLabel(state: string): string {
  const labels: Record<string, string> = {
    PLANNED: "已计划",
    RESEARCH_QUEUED: "研究排队",
    RESEARCH_RUNNING: "研究进行中",
    RESEARCH_SUCCEEDED: "研究完成",
    RESEARCH_PARTIAL: "等待冲突处理",
    READY_TO_FREEZE: "等待冻结",
    FREEZING: "正在冻结",
    FROZEN: "已冻结",
    BLOCKED: "已阻断",
    MISSED: "已错过时点",
    FAILED: "执行失败",
    CANCELLED: "已取消",
  };
  return labels[state] ?? state;
}

export function verificationLabel(value: string): string {
  const labels: Record<string, string> = {
    CONFIRMED: "已确认",
    PROBABLE: "较可信",
    UNVERIFIED: "未验证",
    NOT_FOUND: "未找到",
    CONFLICT: "冲突",
    STALE: "已过期",
    NOT_APPLICABLE: "不适用",
  };
  return labels[value] ?? value;
}

export function taskTone(state: string): string {
  if (state === "FROZEN" || state === "READY_TO_FREEZE" || state === "RESEARCH_SUCCEEDED") return "online";
  if (["BLOCKED", "FAILED", "MISSED"].includes(state)) return "offline";
  return "";
}

export function matchStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    scheduled: "已排期",
    live: "进行中",
    finished: "已结束",
    postponed: "已延期",
    cancelled: "已取消",
  };
  return labels[status] ?? status;
}

export function routeStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    routed: "已路由",
    missing: "明确未知",
    blocked_entity: "实体未确认",
    blocked_time: "时间门禁阻断",
    blocked_conflict: "来源冲突",
    blocked_unregistered_field: "字段未登记",
    ignored_non_model_fact: "非模型字段",
  };
  return labels[status] ?? status;
}

export function manualDecisionLabel(value: string | null): string {
  if (value === "select_evidence") return "已采用证据";
  if (value === "accept_unknown") return "已接受未知";
  return value ?? "已处理";
}
