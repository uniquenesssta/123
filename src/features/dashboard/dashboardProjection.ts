import { formatBytes } from "../../components/format";
import type { BootstrapResponse, ModelDescriptor } from "../../types";

interface DashboardMetric {
  readonly label: string;
  readonly value: string;
  readonly note: string;
  readonly tone: "success" | "warning" | "info" | "brand";
}

export interface DashboardProjection {
  readonly databaseConfigured: boolean;
  readonly connectionError: string | null;
  readonly models: readonly ModelDescriptor[];
  readonly metrics: readonly DashboardMetric[];
}

/** Derive presentation from the current Session snapshot; no retained state or side effects. */
export function projectDashboard(state: BootstrapResponse): DashboardProjection {
  const { data } = state;
  const stats = data.stats;
  const health = data.database_health;
  const estimatePrefix = stats?.large_counts_are_estimates ? "约 " : "";
  return {
    databaseConfigured: data.database_configured,
    connectionError: state.connection_error,
    models: data.models,
    metrics: [
      { label: "数据库", value: health ? "运行正常" : "未连接", note: health ? "响应 " + health.latency_ms + " 毫秒 · " + formatBytes(health.database_size_bytes) : "连接后启用全部功能", tone: health ? "success" : "warning" },
      { label: "球员", value: stats ? estimatePrefix + stats.players : "—", note: stats ? stats.pending_ability_updates + " 个待审核能力候选" : "连接后读取统计", tone: "info" },
      { label: "比赛", value: stats ? estimatePrefix + stats.matches : "—", note: stats ? stats.active_lineups + " 份有效阵容" : "连接后读取统计", tone: "info" },
      { label: "推演", value: stats ? estimatePrefix + stats.model_runs : "—", note: stats ? stats.rule_packages + " 个规则包" : "连接后读取统计", tone: "brand" },
    ],
  };
}
