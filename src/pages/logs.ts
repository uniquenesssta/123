import { escapeHtml } from "../components/format";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import { workspaceTaskAnchorNavigation } from "../components/workspace";
import type { IssueLogEntry, IssueSeverity } from "../types";

function formatTime(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
  }).format(date);
}

function severityLabel(value: IssueSeverity): string {
  const labels: Record<IssueSeverity, string> = {
    warning: "警告",
    error: "错误",
    critical: "严重错误",
  };
  return labels[value];
}

function sourceLabel(value: string): string {
  const labels: Record<string, string> = {
    backend: "后台业务或数据库",
    frontend: "界面操作",
    startup: "平台启动",
    global: "客户端未捕获异常",
    background_job: "后台分析任务",
  };
  return labels[value] ?? value;
}

function friendlyMessage(entry: IssueLogEntry): string {
  const message = entry.user_message || entry.technical_message;
  if (/column .* does not exist|relation .* does not exist|数据库迁移失败/i.test(message)) {
    return "数据库结构与当前客户端不一致";
  }
  if (/PostgreSQL 连接或查询失败|connection refused|pool timed out/i.test(message)) {
    return "数据库连接或查询失败";
  }
  return message || "未提供问题描述";
}

function operationLabel(value: string): string {
  const labels: Record<string, string> = {
    bootstrap: "平台初始化",
    configure_database: "连接数据库",
    execute_prediction: "执行赛事推演",
    execute_prediction_from_match: "使用数据库比赛正式推演",
    execute_shadow_prediction_from_match: "使用数据库比赛影子推演",
    preview_route: "查看模型判定",
    player_catalog_reference_data: "读取球员目录",
    list_players: "加载球员列表",
    read_player: "读取球员详情",
    create_player: "创建球员",
    update_player: "更新球员",
    delete_player: "删除球员",
    create_match: "创建比赛",
    delete_match: "删除比赛",
    create_lineup: "保存阵容",
    list_lineups: "读取阵容",
    generate_match_review: "生成赛后复盘",
    register_rule_package: "注册规则包",
    create_competition_binding: "建立赛事模型路由",
    analytics_overview: "读取分析中心",
    export_issue_logs: "导出问题日志",
  };
  return labels[value] ?? value;
}

function issueCard(entry: IssueLogEntry): string {
  const repeated = Math.max(0, entry.occurrence_count - 1);
  return `
    <article class="issue-card severity-${entry.severity}">
      <div class="issue-card-top">
        <div class="issue-title">
          <span class="issue-severity">${escapeHtml(severityLabel(entry.severity))}</span>
          <h2>${escapeHtml(friendlyMessage(entry))}</h2>
          <small>${escapeHtml(entry.id)} · 客户端 v${escapeHtml(entry.app_version)}</small>
        </div>
        <div class="issue-count">
          <strong>${entry.occurrence_count}</strong>
          <span>发生次数</span>
          ${repeated > 0 ? `<small>已合并 ${repeated} 次重复</small>` : ""}
        </div>
      </div>
      <div class="issue-meta-grid">
        <div><span>来源</span><strong>${escapeHtml(sourceLabel(entry.source))}</strong></div>
        <div><span>首次出现</span><strong>${escapeHtml(formatTime(entry.first_seen_at))}</strong></div>
        <div><span>最近出现</span><strong>${escapeHtml(formatTime(entry.last_seen_at))}</strong></div>
      </div>
      <div class="issue-operations">
        <span>涉及操作</span>
        <div>${entry.operations.length > 0
          ? entry.operations.map((operation) => `<b>${escapeHtml(operationLabel(operation))}</b>`).join("")
          : "<b>未记录具体操作</b>"}</div>
      </div>
      <details class="issue-details">
        <summary>查看技术详情</summary>
        <div>
          <span>原始错误</span>
          <p>${escapeHtml(entry.technical_message || entry.user_message || "未记录")}</p>
        </div>
      </details>
    </article>`;
}

export function logsPage(entries: IssueLogEntry[]): string {
  const totalOccurrences = entries.reduce((sum, item) => sum + item.occurrence_count, 0);
  const repeated = Math.max(0, totalOccurrences - entries.length);
  const critical = entries.filter((item) => item.severity === "critical").length;
  const latest = entries[0]?.last_seen_at;

  const navigation = workspaceTaskAnchorNavigation([
    { id: "issue-summary", index: "01", label: "问题概览", description: "数量、严重度与最近时间", badge: `${entries.length}` },
    { id: "issue-processing", index: "02", label: "聚合机制", description: "捕获、指纹、合并与导出" },
    { id: "issue-records", index: "03", label: "问题记录", description: "技术详情与涉及操作", badge: `${totalOccurrences}` },
  ]);
  return `<section class="module-workspace-page management-module-workspace">
    ${taskPageHeader({ eyebrow: "问题日志", title: "问题自动聚合，不让重复错误刷屏", description: "历史问题按指纹合并并保留首次、最近时间与涉及操作；旧卡片不代表当前仍在异常。", status: { label: critical > 0 ? `${critical} 个严重问题` : entries.length > 0 ? `${entries.length} 个独立问题` : "暂无问题", tone: critical > 0 ? "danger" : entries.length > 0 ? "warning" : "success" }, actions: `<button class="secondary" data-action="refresh-issue-logs">刷新</button><button class="secondary" data-action="export-issue-logs" ${entries.length === 0 ? "disabled" : ""}>导出报告</button><button class="secondary danger-quiet" data-action="request-clear-issue-logs" ${entries.length === 0 ? "disabled" : ""}>清空日志</button>` })}
    ${taskContextRibbon([
      { label: "独立问题", value: `${entries.length}`, note: "按问题指纹去重", tone: entries.length > 0 ? "warning" : "success" },
      { label: "总发生次数", value: `${totalOccurrences}`, note: `${repeated} 次重复已合并`, tone: repeated > 0 ? "accent" : "neutral" },
      { label: "严重问题", value: `${critical}`, note: critical > 0 ? "需要优先处理" : "当前无严重记录", tone: critical > 0 ? "danger" : "success" },
      { label: "最近记录", value: latest ? formatTime(latest) : "无", note: "日志为历史证据", tone: latest ? "neutral" : "success" },
    ])}
    <div class="core-local-navigation">${navigation}</div>
    <div class="management-module-stage" data-workspace-scroll-key="logs-stage">
    <section id="issue-summary" class="management-section workspace-anchor-target"><div class="metric-grid issue-metrics">
      <article class="metric-card"><span>独立问题</span><strong>${entries.length}</strong><small>按问题指纹去重</small></article>
      <article class="metric-card"><span>总发生次数</span><strong>${totalOccurrences}</strong><small>包含重复触发</small></article>
      <article class="metric-card"><span>已聚合重复</span><strong>${repeated}</strong><small>不会生成重复卡片</small></article>
      <article class="metric-card"><span>严重问题</span><strong>${critical}</strong><small>${latest ? `最近 ${escapeHtml(formatTime(latest))}` : "暂无记录"}</small></article>
    </div></section>

    <section id="issue-processing" class="issue-explainer panel compact management-section workspace-anchor-target">
      <div><b>1</b><span>捕获问题</span><small>记录用户可见错误和原始技术信息</small></div>
      <i>→</i>
      <div><b>2</b><span>生成指纹</span><small>忽略时间、流水号等易变数字</small></div>
      <i>→</i>
      <div><b>3</b><span>聚合重复</span><small>同类问题只更新次数和最近时间</small></div>
      <i>→</i>
      <div><b>4</b><span>导出沟通</span><small>一键生成可直接发送的问题报告</small></div>
    </section>

    <section id="issue-records" class="issue-list management-section workspace-anchor-target">
      ${entries.length > 0
        ? entries.map(issueCard).join("")
        : `<div class="empty-state panel"><strong>目前没有记录到问题</strong><p>后续发生的启动、界面、后台或数据库问题会自动出现在这里。</p></div>`}
    </section>
    </div>
  </section>`;
}
