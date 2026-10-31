import type { BootstrapResponse } from "../../types";
import { escapeHtml, formatBytes } from "../../components/format";
import { statCardView } from "../../ui/data-display/stat-card";

/** Only display the host and database; never expose credentials or connection query values. */
export function databaseTarget(address: string | null | undefined): string {
  if (!address) return "尚未确认连接目标";
  try { const url = new URL(address); return `${url.host} / ${decodeURIComponent(url.pathname.slice(1)) || "—"}`; }
  catch { return "连接目标不可用"; }
}
export function databaseIdentity(state: BootstrapResponse): string {
  const health = state.data.database_health;
  return `<article class="panel database-identity"><h2>${escapeHtml(health?.database_name ?? "尚未确认数据库")} · ${health ? "已确认连接" : "等待连接"}</h2><p>${health ? "已确认的服务身份；更换连接后以新响应为准。" : "填写连接设置后检查服务身份。"}</p><div class="database-identity-facts">服务器 ${escapeHtml(health?.server_version ?? "—")} · 响应 ${health?.latency_ms ?? "—"} ms · 容量 ${formatBytes(health?.database_size_bytes)}<br>目标 ${escapeHtml(databaseTarget(state.data.database_url))} · 连接身份已脱敏。</div></article>`;
}
export function databaseOverview(state: BootstrapResponse): string {
  const stats = state.data.stats, prefix = stats?.large_counts_are_estimates ? "约 " : "";
  return `${databaseIdentity(state)}<div class="database-metrics">${statCardView({label:"球员",value:`${prefix}${stats?.players ?? "—"}`,note:stats?.large_counts_are_estimates ? "估计总量，不是精确总数" : "完整球员目录"})}${statCardView({label:"比赛",value:`${prefix}${stats?.matches ?? "—"}`,note:`${stats?.active_lineups ?? "—"} 份有效阵容`})}${statCardView({label:"推演",value:`${prefix}${stats?.model_runs ?? "—"}`,note:`${stats?.rule_packages ?? "—"} 个规则包`})}${statCardView({label:"自动升级",value:`${state.data.database_health?.migration_count ?? "—"}`,note:"已完成迁移记录"})}</div><article class="panel database-boundary" id="database-boundary"><h2>已连接不等于所有业务已就绪</h2><p>数据库连接、迁移、统计、模型提供器和业务许可分别校验。</p><p>连接并保存会自动执行迁移与内置资料注册。刷新失败时保留上次已确认数据并标明过期；尚未取得的数据使用“—”，不补成 0。</p><div class="button-row"><button class="secondary" data-action="jump-workspace-anchor" data-anchor-id="database-connection">查看连接设置</button><button class="secondary" data-action="jump-workspace-anchor" data-anchor-id="database-statistics">查看全部统计</button><button class="secondary" data-action="show-database-technical">技术连接信息</button></div></article>`;
}
