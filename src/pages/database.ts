import { escapeHtml, formatBytes } from "../components/format";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import { workspaceTaskAnchorNavigation } from "../components/workspace";
import type { BootstrapResponse } from "../types";

export function databasePage(state: BootstrapResponse): string {
  const health = state.data.database_health;
  const stats = state.data.stats;
  const prefix = stats?.large_counts_are_estimates ? "约 " : "";
  const connected = Boolean(health);
  const navigation = workspaceTaskAnchorNavigation([
    { id: "database-overview", index: "01", label: "运行状态", description: "健康、容量和核心数量", badge: connected ? `${health?.latency_ms ?? 0}ms` : "未连接" },
    { id: "database-connection", index: "02", label: "连接设置", description: "地址、连接池与迁移" },
    { id: "database-statistics", index: "03", label: "数据统计", description: "核心表与业务数量" },
    { id: "database-danger", index: "04", label: "危险操作", description: "彻底清空当前数据库" },
  ]);
  return `<section class="module-workspace-page management-module-workspace">
    ${taskPageHeader({ eyebrow: "数据库", title: connected ? "数据服务运行正常" : "连接数据服务", description: connected ? "日常使用不需要修改这里；连接、迁移和数据统计会自动维护。" : "完成一次连接后，赛事、球员、阵容和推演功能即可使用。", status: { label: connected ? `响应 ${health?.latency_ms ?? 0} 毫秒` : "未连接", tone: connected ? "success" : "warning" } })}
    ${taskContextRibbon([
      { label: "连接状态", value: connected ? health?.database_name ?? "已连接" : "未连接", note: health?.server_version ?? "等待配置数据库地址", tone: connected ? "success" : "warning" },
      { label: "数据库大小", value: formatBytes(health?.database_size_bytes), note: `${health?.migration_count ?? 0} 项自动升级记录`, tone: connected ? "accent" : "neutral" },
      { label: "业务数据", value: `${prefix}${stats?.players ?? 0} 名球员 · ${prefix}${stats?.matches ?? 0} 场比赛`, note: `${stats?.active_lineups ?? 0} 份有效阵容`, tone: connected ? "accent" : "neutral" },
      { label: "模型账本", value: `${prefix}${stats?.model_runs ?? 0} 次推演`, note: `${stats?.rule_packages ?? 0} 个规则包`, tone: connected ? "accent" : "neutral" },
    ])}
    <div class="core-local-navigation">${navigation}</div>
    <div class="management-module-stage" data-workspace-scroll-key="database-stage">
      <section id="database-overview" class="management-section workspace-anchor-target">
        <div class="database-overview"><article class="panel database-hero ${connected ? "connected" : ""}"><div class="database-status-icon">${connected ? "✓" : "!"}</div><div><span>${connected ? "已连接" : "需要配置"}</span><h2>${escapeHtml(health?.database_name ?? "数据服务")}</h2><p>${connected ? `${escapeHtml(health?.server_version ?? "")} · ${formatBytes(health?.database_size_bytes)}` : "请输入数据库地址并保存。"}</p></div></article><div class="metric-grid compact-metrics database-metrics"><article class="metric-card"><span>球员</span><strong>${prefix}${stats?.players ?? 0}</strong><small>完整球员目录</small></article><article class="metric-card"><span>比赛</span><strong>${prefix}${stats?.matches ?? 0}</strong><small>${stats?.active_lineups ?? 0} 份有效阵容</small></article><article class="metric-card"><span>推演</span><strong>${prefix}${stats?.model_runs ?? 0}</strong><small>${stats?.rule_packages ?? 0} 个规则包</small></article></div></div>
      </section>
      <details id="database-connection" class="panel disclosure-panel management-section workspace-anchor-target" ${connected ? "" : "open"}><summary><div><span>连接设置</span><strong>${connected ? "查看或更换数据库连接" : "配置数据库连接"}</strong></div><b>展开</b></summary><div class="disclosure-content two-column database-layout"><article class="subpanel"><label class="field"><span>连接地址</span><input id="database-url" type="password" autocomplete="off" placeholder="postgres://football_app:password@localhost:5432/football_model" /></label><div class="field-row"><label class="field"><span>最大连接数</span><input id="max-connections" type="number" min="1" max="100" value="10" /></label><label class="field"><span>连接超时</span><input id="connect-timeout" type="number" min="1" max="120" value="10" /></label></div><div class="button-row"><button class="primary" data-action="connect-database">连接并保存</button><button class="secondary danger-quiet" data-action="disconnect-database" ${state.data.database_configured ? "" : "disabled"}>清除连接</button></div><p class="field-note">连接后自动执行未完成的数据库迁移。正式业务数据保存在数据库服务器，本机只保存受保护的连接凭据。</p></article><article class="subpanel"><div class="panel-heading"><div><span>连接详情</span><h2>当前数据库</h2></div></div><dl class="detail-list"><div><dt>服务器版本</dt><dd>${escapeHtml(health?.server_version ?? "—")}</dd></div><div><dt>数据库大小</dt><dd>${formatBytes(health?.database_size_bytes)}</dd></div><div><dt>自动升级记录</dt><dd>${health?.migration_count ?? 0} 项</dd></div><div><dt>正式数据位置</dt><dd>数据库服务器</dd></div></dl><details class="inline-details technical-details"><summary>查看技术连接信息</summary><dl class="detail-list"><div><dt>本机配置位置</dt><dd>${escapeHtml(state.config_path)}</dd></div><div><dt>连接地址</dt><dd>${escapeHtml(state.data.database_url ?? "—")}</dd></div></dl></details></article></div></details>
      <details id="database-statistics" class="panel disclosure-panel management-section workspace-anchor-target"><summary><div><span>数据统计</span><strong>查看全部核心表数量</strong></div><b>展开</b></summary><div class="disclosure-content stats-table"><div><span>赛事</span><strong>${stats?.competitions ?? 0}</strong></div><div><span>球队</span><strong>${prefix}${stats?.teams ?? 0}</strong></div><div><span>球员</span><strong>${prefix}${stats?.players ?? 0}</strong></div><div><span>比赛</span><strong>${prefix}${stats?.matches ?? 0}</strong></div><div><span>外部数据来源</span><strong>${stats?.data_providers ?? 0}</strong></div><div><span>状态记录</span><strong>${prefix}${stats?.availability_records ?? 0}</strong></div><div><span>球员能力记录</span><strong>${prefix}${stats?.ability_observations ?? 0}</strong></div><div><span>有效阵容</span><strong>${stats?.active_lineups ?? 0}</strong></div><div><span>规则包</span><strong>${stats?.rule_packages ?? 0}</strong></div><div><span>有效模型规则</span><strong>${stats?.route_bindings ?? 0}</strong></div><div><span>推演记录</span><strong>${prefix}${stats?.model_runs ?? 0}</strong></div><div><span>待审核更新</span><strong>${stats?.pending_ability_updates ?? 0}</strong></div></div></details>
      <section id="database-danger" class="panel database-danger-zone management-section workspace-anchor-target" aria-labelledby="database-reset-title"><div class="database-danger-copy"><span>危险操作</span><h2 id="database-reset-title">彻底清空当前数据库</h2><p>删除当前数据库中的全部球队、球员、比赛、阵容、推演、P4 快照、规则、导入批次、AI 会话、审核记录、任务、审计和设置数据，并重新建立一套空白数据库结构。</p><small>数据库本身和本机连接配置会保留。此操作不可撤销，无法从客户端恢复。</small></div><button class="primary danger-action" data-action="request-reset-database" ${connected ? "" : "disabled"}>彻底清空数据库</button></section>
    </div>
  </section>`;
}
