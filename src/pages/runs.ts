import { escapeHtml, formatPercent } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import type { BootstrapResponse, ModelRunListItem } from "../types";


function runReadinessLabel(level: ModelRunListItem["input_readiness_level"]): string {
  const labels: Record<ModelRunListItem["input_readiness_level"], string> = {
    formal_ready: "正式就绪",
    ready_with_warnings: "带警告",
    shadow_only: "影子输入",
    blocked: "已阻断",
    not_assessed: "未评估",
    legacy_unknown: "旧记录",
  };
  return labels[level];
}

function runReadinessTone(level: ModelRunListItem["input_readiness_level"]): string {
  if (level === "formal_ready") return "passed";
  if (level === "ready_with_warnings" || level === "shadow_only") return "warning";
  if (level === "blocked") return "blocked";
  return "neutral";
}

function runMatchName(row: ModelRunListItem): string {
  if (row.home_team_name && row.away_team_name) {
    return `${row.home_team_name} vs ${row.away_team_name}`;
  }
  return row.match_key || "未命名比赛";
}


export function runHistoryMarkup(rows: ModelRunListItem[], compact = false): string {
  if (rows.length === 0) {
    return `<div class="empty-state ${compact ? "compact" : ""}"><strong>暂无推演记录</strong><span>完成一次正式推演后，结果会直接出现在这里。</span></div>`;
  }
  const visible = compact ? rows.slice(0, 6) : rows;
  return `<div class="balanced-run-table-shell ${compact ? "compact" : ""}"><table class="balanced-data-table balanced-run-table">
    <thead><tr><th>比赛</th><th>时间</th><th>模型 / 规则</th><th>窗口</th><th>最可能比分</th><th>胜平负</th><th>操作</th></tr></thead>
    <tbody>${visible.map((row) => `<tr data-context-kind="run" data-run-id="${escapeHtml(row.id)}" data-run-label="${escapeHtml(runMatchName(row))}">
      <td><strong>${escapeHtml(runMatchName(row))}</strong><small>${escapeHtml(row.competition_name ?? "赛事未记录")}</small></td>
      <td>${escapeHtml(row.kickoff_time ? new Date(row.kickoff_time).toLocaleString("zh-CN") : new Date(row.created_at).toLocaleString("zh-CN"))}</td>
      <td><strong>${escapeHtml(row.model_key?.toUpperCase() ?? "模型")}</strong><small>${escapeHtml(row.rule_package_name ?? "系统默认规则")}</small></td>
      <td><strong>${escapeHtml(row.snapshot_type)}</strong><small class="run-input-audit ${runReadinessTone(row.input_readiness_level)}">${escapeHtml(runReadinessLabel(row.input_readiness_level))}${row.input_readiness_score === null ? "" : ` · ${row.input_readiness_score}/100`} · ${escapeHtml(row.input_manifest_sha256.slice(0, 10))}</small></td>
      <td><span class="balanced-score-pill">${escapeHtml(row.top_scoreline ?? "—")}</span><small>${row.top_scoreline_probability === null ? "暂无矩阵" : formatPercent(row.top_scoreline_probability)}</small></td>
      <td>${formatPercent(row.summary.home_win)} / ${formatPercent(row.summary.draw)} / ${formatPercent(row.summary.away_win)}</td>
      <td><div class="balanced-row-actions"><button class="secondary tiny" data-action="open-run" data-run-id="${escapeHtml(row.id)}">查看</button><button class="ghost tiny danger" data-action="request-hide-run-history" data-run-id="${escapeHtml(row.id)}" data-run-label="${escapeHtml(runMatchName(row))}">删除</button></div></td>
    </tr>`).join("")}</tbody>
  </table></div>`;
}

export function runsPage(state: BootstrapResponse): string {
  const rows = state.data.recent_runs;
  if (!state.data.database_configured) {
    return `<section class="page-heading simple-heading"><div><p class="eyebrow">推演记录</p><h1>历史推演结果</h1><p>连接成功后，已保存的比赛预测会自动显示在本页。</p></div></section>${inlineDatabaseSetup("连接数据服务以读取推演记录", "连接成功后无需切换页面。", state.connection_error)}`;
  }
  return `
    <section class="page-heading simple-heading balanced-page-heading">
      <div><p class="eyebrow">推演记录</p><h1>正式推演历史</h1><p>模型、窗口、最可能比分与胜平负保留在列表；技术血缘按需打开。</p></div>
      <button class="secondary" data-action="refresh-runs">刷新记录</button>
    </section>
    <section class="panel run-history-panel balanced-history-panel">
      <div class="panel-heading"><div><span>历史记录</span><h2>最近 ${rows.length} 次正式推演</h2></div><small>点击查看；右键或删除按钮可从列表移除</small></div>
      ${runHistoryMarkup(rows)}
    </section>
  `;
}
