import type { ModelRunListItem } from "../../types";
export const readiness = (value: unknown): {
    label: string;
    tone: string;
} => ({
    formal_ready: { label: "正式就绪", tone: "passed" }, ready_with_warnings: { label: "带警告", tone: "warning" }, shadow_only: { label: "影子输入", tone: "info" }, blocked: { label: "已阻断", tone: "blocked" }, not_assessed: { label: "未评估", tone: "neutral" }, legacy_unknown: { label: "旧记录", tone: "neutral" }
}[String(value)] ?? { label: "输入状态未记录", tone: "neutral" });
export const recorded = (value: unknown, fallback = "未记录"): string => typeof value === "string" && value.trim() ? value : fallback;
export const finite = (value: unknown): number | null => typeof value === "number" && Number.isFinite(value) ? value : null;
export const probability = (value: unknown): string => finite(value) === null ? "—" : new Intl.NumberFormat("zh-CN", { style: "percent", maximumFractionDigits: 1 }).format(value as number);
export function matchName(row: Pick<ModelRunListItem, "home_team_name" | "away_team_name" | "match_key">): string { return row.home_team_name && row.away_team_name ? `${row.home_team_name} vs ${row.away_team_name}` : recorded(row.match_key, "未命名比赛"); }
export function time(value: unknown): {
    short: string;
    full: string;
} | null {
    if (typeof value !== "string" || !Number.isFinite(Date.parse(value)))
        return null;
    const date = new Date(value), parts = Object.fromEntries(new Intl.DateTimeFormat("zh-CN", { month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", hour12: false }).formatToParts(date).map(p => [p.type, p.value]));
    return { short: `${parts.month}-${parts.day} ${parts.hour}:${parts.minute}`, full: date.toLocaleString("zh-CN") };
}
