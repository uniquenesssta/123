import { escapeHtml } from "../../components/format";
import type {
  PlayerDetail
} from "../../types";
export const availabilityLabels: Record<string, string> = {
  available: "可用",
  unavailable: "不可出场",
  doubtful: "存疑",
  injured: "伤病",
  suspended: "停赛",
  rested: "轮休",
  returning: "恢复中",
  unknown: "未知",
};
export const footLabels: Record<string, string> = {
  left: "左脚",
  right: "右脚",
  both: "双脚",
  unknown: "未知",
};
export function options(
  items: Array<{ id: string; canonical_name: string }>,
  selected: string | null,
  empty = "全部球队",
): string {
  return `<option value="">${empty}</option>${items
    .map(
      (item) =>
        `<option value="${escapeHtml(item.id)}" ${item.id === selected ? "selected" : ""}>${escapeHtml(item.canonical_name)}</option>`,
    )
    .join("")}`;
}
export function recordString(record: Record<string, unknown> | undefined, key: string): string | null {
  const value = record?.[key];
  return typeof value === "string" && value.trim() ? value.trim() : null;
}
export function recordNumber(record: Record<string, unknown> | undefined, key: string): number | null {
  const value = record?.[key];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}
export function recordBoolean(record: Record<string, unknown> | undefined, key: string): boolean {
  return record?.[key] === true;
}
export function isChineseNameRecord(record: Record<string, unknown>): boolean {
  const language = recordString(record, "language_code")?.toLowerCase() ?? "";
  const name = recordString(record, "name") ?? "";
  return ["zh-cn", "zh-hans", "zh"].includes(language) || /[一-龥]/u.test(name);
}
export function currentLocalizedPlayerName(detail: PlayerDetail | null, today = new Date().toISOString().slice(0, 10)): string {
  if (!detail) return "";
  const records = (detail.names as Array<Record<string, unknown>>).filter(record => isChineseNameRecord(record) && (!recordString(record, "valid_from") || (recordString(record, "valid_from") ?? "") <= today) && (!recordString(record, "valid_to") || (recordString(record, "valid_to") ?? "") >= today));
  records.sort((left, right) => {
    const leftLanguage = recordString(left, "language_code")?.toLowerCase() ?? "";
    const rightLanguage = recordString(right, "language_code")?.toLowerCase() ?? "";
    const languageRank = (language: string) => ["zh-cn", "zh-hans", "zh"].includes(language) ? 1 : 0;
    const byLanguage = languageRank(rightLanguage) - languageRank(leftLanguage);
    if (byLanguage !== 0) return byLanguage;
    return (recordString(right, "valid_from") ?? "").localeCompare(recordString(left, "valid_from") ?? "") ||
      (recordString(right, "id") ?? "").localeCompare(recordString(left, "id") ?? "");
  });
  return recordString(records[0], "name") ?? "";
}
export function dateTimeLocalValue(value: string | null): string {
  if (!value) return "";
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return "";
  const local = new Date(parsed.getTime() - parsed.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 16);
}
export function playerNameHistory(detail: PlayerDetail): string {
  const records = detail.names as Array<Record<string, unknown>>;
  if (records.length === 0) return '<div class="empty-state compact"><strong>暂无名称记录</strong><span>正式姓名仍保存在基础身份中。</span></div>';
  return `<div class="history-list">${records.map((record) => {
    const name = recordString(record, "name") ?? "未命名";
    const language = recordString(record, "language_code") ?? "未指定语言";
    const primary = recordBoolean(record, "is_primary") ? "主名称" : "别名";
    const from = recordString(record, "valid_from");
    const to = recordString(record, "valid_to");
    return `<div class="history-row"><strong>${escapeHtml(name)}</strong><span>${escapeHtml(language)} · ${primary}</span><span>${escapeHtml(from ?? "长期有效")} → ${escapeHtml(to ?? "至今")}</span><b>已保存</b></div>`;
  }).join("")}</div>`;
}
