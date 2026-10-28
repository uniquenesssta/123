import { escapeHtml, formatBytes } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import type { BootstrapResponse } from "../types";
import { iconSlot, type UiIcon } from "../ui/icons/iconSlot";
import "./dashboard/dashboard.css";

function metric(label: string, value: string, note: string, tone: string): string {
  return `<article class="metric-card dashboard-metric tone-${tone}"><span>${escapeHtml(label)}</span><strong>${escapeHtml(value)}</strong><small>${escapeHtml(note)}</small></article>`;
}

function actionCard(page: string, icon: UiIcon, title: string, description: string, action: string): string {
  return `<button class="action-card dashboard-action ${page === "prediction" ? "dashboard-action-featured" : ""}" data-page="${page}"><span class="action-icon">${iconSlot(icon, 20)}</span><div><strong>${escapeHtml(title)}</strong><small>${escapeHtml(description)}</small></div><b>${escapeHtml(action)} ${iconSlot("arrow-right", 16)}</b></button>`;
}

export function dashboardPage(state: BootstrapResponse): string {
  const { data } = state;
  const stats = data.stats;
  const health = data.database_health;
  const estimatePrefix = stats?.large_counts_are_estimates ? "约 " : "";
  return `
    <section class="page-heading simple-heading">
      <div>
        <h1>今天要处理什么？</h1>
        <p>常用功能集中在这里。模型、数据库结构和高级参数仍完整保留，但默认不打扰日常操作。</p>
      </div>
      <span class="status-pill ${data.database_configured ? "online" : "offline"}">${data.database_configured ? "可以开始工作" : "请先连接数据库"}</span>
    </section>
    ${!data.database_configured ? inlineDatabaseSetup("连接数据服务以启用工作台", "连接成功后本页的录入、推演、复盘和分析入口会立即可用。", state.connection_error) : ""}
    <section class="quick-actions">
      ${actionCard("lineups", "match-center", "录入比赛与阵容", "创建赛事、选择球队并保存阵容", "开始录入")}
      ${actionCard("prediction", "prediction", "开始赛事推演", "自动选择规则包与模型并输出结论", "开始推演")}
      ${actionCard("review", "history", "完成赛后复盘", "录入赛果与球员评分，审核能力变化", "开始复盘")}
      ${actionCard("analytics", "chart", "查看分析结论", "比较模型、检查漂移和数据质量", "查看结论")}
      ${actionCard("players", "users", "查找或维护球员", "查看能力、伤停、球队和位置", "维护球员")}
    </section>
    <div class="metric-grid compact-metrics">
      ${metric("数据库", health ? "运行正常" : "未连接", health ? `响应 ${health.latency_ms} 毫秒 · ${formatBytes(health.database_size_bytes)}` : "连接后启用全部功能", health ? "success" : "warning")}
      ${metric("球员", stats ? `${estimatePrefix}${stats.players}` : "—", stats ? `${stats.pending_ability_updates} 个待审核能力候选` : "连接后读取统计", "info")}
      ${metric("比赛", stats ? `${estimatePrefix}${stats.matches}` : "—", stats ? `${stats.active_lineups} 份有效阵容` : "连接后读取统计", "info")}
      ${metric("推演", stats ? `${estimatePrefix}${stats.model_runs}` : "—", stats ? `${stats.rule_packages} 个规则包` : "连接后读取统计", "brand")}
    </div>
    <details class="panel disclosure-panel">
      <summary><div><span>系统状态</span><strong>查看模型与数据结构</strong></div><b>展开</b></summary>
      <div class="two-column disclosure-content">
        <article>
          <div class="panel-heading"><div><span>可用模型</span><h2>模型注册中心</h2></div></div>
          ${data.models.map((model) => `
            <div class="model-card">
              <div><strong>${escapeHtml(model.display_name)}</strong><small>${escapeHtml(model.model_id)} · ${escapeHtml(model.engine_version)}</small></div>
              <div class="tag-row">${model.supported_competitions.map((item) => `<span>${escapeHtml(item)}</span>`).join("")}</div>
            </div>`).join("")}
        </article>
        <article>
          <div class="panel-heading"><div><span>数据范围</span><h2>平台已经保存的内容</h2></div></div>
          <div class="domain-list user-domain-list">
            <div><b>赛事</b><span>赛事、赛季、阶段、轮次、比赛与阵容</span></div>
            <div><b>球员</b><span>姓名、位置、球队、伤停与能力历史</span></div>
            <div><b>模型</b><span>规则包、路由、参数、结果与解释</span></div>
            <div><b>复盘</b><span>赛后评价和后续能力更新候选</span></div>
          </div>
        </article>
      </div>
    </details>
  `;
}
