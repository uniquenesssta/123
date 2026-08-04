import { escapeHtml } from "../components/format";
import { icon } from "../components/icons";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { workspacePaneToggle, workspaceSectionNavigation } from "../components/workspace";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import type {
  BootstrapResponse,
  ReleaseAcceptanceCheck,
  ReleaseAcceptanceRun,
  ReleaseAcceptanceRunSummary,
  ReleaseAcceptanceStatus,
} from "../types";

const categoryLabels: Record<string, string> = {
  chain: "全链路",
  performance: "性能",
  security: "安全",
  cost: "成本",
  release: "发布",
};

function statusLabel(status: ReleaseAcceptanceStatus): string {
  return status === "pass" ? "通过" : status === "warning" ? "警告" : "阻断";
}

function statusClass(status: ReleaseAcceptanceStatus): string {
  return status === "pass" ? "success" : status === "warning" ? "warning" : "danger";
}

function checksMarkup(checks: readonly ReleaseAcceptanceCheck[]): string {
  if (checks.length === 0) {
    return '<div class="empty-state compact"><strong>尚无检查结果</strong><span>运行一次发布验收后显示逐项证据。</span></div>';
  }
  return `<div class="release-check-list">${checks.map((check) => `
    <article class="release-check ${statusClass(check.status)}">
      <div class="release-check-status"><span>${String(check.sequence_no).padStart(2, "0")}</span><b>${statusLabel(check.status)}</b></div>
      <div class="release-check-copy"><span>${escapeHtml(categoryLabels[check.category] ?? check.category)}</span><h3>${escapeHtml(check.title)}</h3><p>${escapeHtml(check.summary)}</p>${check.remediation ? `<div class="blocking-note">处理：${escapeHtml(check.remediation)}</div>` : ""}</div>
      <button class="ghost tiny" data-action="show-release-check-evidence" data-check-id="${escapeHtml(check.id)}">查看证据</button>
    </article>`).join("")}</div>`;
}

function historyMarkup(runs: readonly ReleaseAcceptanceRunSummary[], selectedId: string | null): string {
  if (runs.length === 0) return '<div class="empty-state compact"><strong>尚未运行验收</strong><span>首份报告会在执行后保存为不可变记录。</span></div>';
  return `<div class="release-history-list">${runs.map((run) => `<button class="history-row history-button ${run.id === selectedId ? "active" : ""}" data-action="open-release-acceptance-run" data-run-id="${escapeHtml(run.id)}"><strong>v${escapeHtml(run.app_version)} · ${statusLabel(run.overall_status)}</strong><span>${escapeHtml(new Date(run.completed_at).toLocaleString())}</span><span>通过 ${run.passed_count} / 警告 ${run.warning_count} / 阻断 ${run.blocked_count}</span><b>${escapeHtml(run.report_sha256.slice(0, 12))}</b></button>`).join("")}</div>`;
}

export function releasePage(
  state: BootstrapResponse,
  runs: readonly ReleaseAcceptanceRunSummary[],
  selected: ReleaseAcceptanceRun | null,
  moduleSidebarCollapsed: boolean,
  inspectorCollapsed: boolean,
  activeSection: string,
): string {
  if (!state.data.database_configured) {
    return `<section class="task-empty-workspace">${taskPageHeader({ eyebrow: "发布验收", title: "全链路验收与发布", description: "连接数据库后执行固定 fixture 与真实运行事实联合验收。", status: { label: "等待数据库", tone: "warning" } })}${taskContextRibbon([{ label: "验收账本", value: "数据库未连接", note: "报告必须写入不可变账本", tone: "warning" }])}${inlineDatabaseSetup("连接数据服务以运行发布验收", "验收报告必须写入不可变账本，因此不能在离线模式下伪造通过。", state.connection_error)}</section>`;
  }
  const section = ["overview", "chain", "performance", "security", "cost", "history"].includes(activeSection) ? activeSection : "overview";
  const categories = selected?.category_summaries ?? [];
  const category = (key: string) => categories.find((item) => item.category === key);
  const sectionNav = workspaceSectionNavigation([
    { id: "overview", index: "01", label: "发布总览", description: "执行验收并查看闸门", badge: selected ? statusLabel(selected.overall_status) : "未运行" },
    { id: "chain", index: "02", label: "全链路", description: "A–I、fixture 与账本", badge: `${category("chain")?.blocked ?? 0}` },
    { id: "performance", index: "03", label: "性能", description: "数据库与推演 P95", badge: `${selected?.performance.recent_model_run_count ?? 0}` },
    { id: "security", index: "04", label: "安全", description: "不可变与密钥边界", badge: `${category("security")?.blocked ?? 0}` },
    { id: "cost", index: "05", label: "成本", description: "调用、搜索与预算", badge: selected ? `$${selected.cost.estimated_cost_usd.toFixed(2)}` : "—" },
    { id: "history", index: "06", label: "历史报告", description: "读取不可变验收记录", badge: `${runs.length}` },
  ], section);
  const selectedChecks = (categoryName: string) => selected?.checks.filter((check) => check.category === categoryName) ?? [];
  return `<section class="module-workspace-page release-core-workspace" data-legacy-module-sidebar-state="${moduleSidebarCollapsed ? "collapsed" : "expanded"}">
    ${taskPageHeader({
      eyebrow: "发布验收",
      title: "全链路验收与发布",
      description: "固定 P4 fixture、真实数据库状态、性能、安全和 API 成本在当前页面完成，不再依赖额外左侧目录。",
      status: { label: selected ? statusLabel(selected.overall_status) : "尚未运行", tone: selected?.overall_status === "pass" ? "success" : selected?.overall_status === "warning" ? "warning" : selected?.overall_status === "blocked" ? "danger" : "neutral" },
      actions: `<button class="secondary" data-action="refresh-release-acceptance">${icon("refresh")}<span>刷新历史</span></button><button class="primary" data-action="run-release-acceptance">运行发布验收</button>`,
    })}
    ${taskContextRibbon([
      { label: "总体结论", value: selected ? statusLabel(selected.overall_status) : "未运行", note: selected ? `完成于 ${new Date(selected.completed_at).toLocaleString()}` : "运行后写入不可变账本", tone: selected?.overall_status === "pass" ? "success" : selected?.overall_status === "blocked" ? "danger" : selected?.overall_status === "warning" ? "warning" : "neutral" },
      { label: "通过 / 警告 / 阻断", value: selected ? `${selected.passed_count} / ${selected.warning_count} / ${selected.blocked_count}` : "—", note: "阻断项禁止发布", tone: (selected?.blocked_count ?? 0) > 0 ? "danger" : selected ? "accent" : "neutral" },
      { label: "历史报告", value: `${runs.length} 份`, note: "只能读取，不能覆盖或删除", tone: runs.length > 0 ? "accent" : "neutral" },
      { label: "报告指纹", value: selected ? selected.report_sha256.slice(0, 16) : "尚未生成", note: "用于确认报告完整性", tone: selected ? "success" : "neutral" },
    ])}
    <div class="core-local-navigation">${sectionNav}</div>
    <div class="module-workspace-stage">
      <section class="release-core-layout ${inspectorCollapsed ? "inspector-collapsed" : ""}">
        <main class="release-core-main" data-workspace-scroll-key="release-main">
          <section class="workspace-module-view ${section === "overview" ? "active" : ""}" data-workspace-section="overview">
            <div class="workspace-section-heading"><div><span>发布总览</span><h2>执行一次完整验收</h2><p>预算属于显式发布闸门；留空时给出警告，不擅自假定额度。</p></div><button class="icon-button ${inspectorCollapsed ? "" : "active"}" data-action="toggle-workspace-pane" data-pane="inspector" title="${inspectorCollapsed ? "打开" : "关闭"}验收检查器">${icon("panel-right")}</button></div>
            <section class="panel release-run-panel"><div class="form-grid two-column-form clean-form"><label class="field"><span>性能窗口（天）</span><input id="release-performance-window" type="number" min="1" max="365" value="30"></label><label class="field"><span>成本窗口（天）</span><input id="release-cost-window" type="number" min="1" max="365" value="30"></label><label class="field"><span>单日成本预算（USD）</span><input id="release-daily-budget" type="number" min="0" step="0.01" placeholder="留空则警告"></label><label class="field"><span>周期成本预算（USD）</span><input id="release-monthly-budget" type="number" min="0" step="0.01" placeholder="留空则警告"></label><label class="field field-wide"><span>验收执行人</span><input id="release-requested-by" placeholder="可选，用于审计"></label></div><div class="workflow-actions"><button class="primary large" data-action="run-release-acceptance">执行固定 fixture + 真实运行验收</button></div></section>
            ${selected ? `<div class="release-scoreboard"><article class="${statusClass(selected.overall_status)}"><span>总体结论</span><strong>${statusLabel(selected.overall_status)}</strong><small>${escapeHtml(new Date(selected.completed_at).toLocaleString())}</small></article><article><span>通过</span><strong>${selected.passed_count}</strong><small>可直接保留</small></article><article><span>警告</span><strong>${selected.warning_count}</strong><small>需明确处理</small></article><article><span>阻断</span><strong>${selected.blocked_count}</strong><small>禁止发布</small></article></div>${checksMarkup(selected.checks)}` : '<section class="panel empty-state"><strong>尚未执行发布验收</strong><span>点击上方按钮后，报告会写入不可变发布账本。</span></section>'}
          </section>
          <section class="workspace-module-view ${section === "chain" ? "active" : ""}" data-workspace-section="chain"><div class="workspace-section-heading"><div><span>全链路</span><h2>A–I 契约、P4 fixture 与真实账本</h2><p>同时验证概率矩阵、确定性、H/I 样本可见性和数据库迁移。</p></div></div>${checksMarkup(selectedChecks("chain"))}</section>
          <section class="workspace-module-view ${section === "performance" ? "active" : ""}" data-workspace-section="performance"><div class="workspace-section-heading"><div><span>性能</span><h2>数据库和近期推演性能</h2><p>运行环境没有样本时明确标记，不用固定 fixture 冒充真实 P95。</p></div></div>${selected ? `<div class="release-scoreboard"><article><span>数据库延迟</span><strong>${selected.performance.database_latency_ms} ms</strong></article><article><span>近期推演</span><strong>${selected.performance.recent_model_run_count}</strong></article><article><span>推演 P95</span><strong>${selected.performance.recent_model_run_p95_ms?.toFixed(1) ?? "—"} ms</strong></article><article><span>查询警告</span><strong>${selected.performance.query_warning_count}</strong></article></div>` : ""}${checksMarkup(selectedChecks("performance"))}</section>
          <section class="workspace-module-view ${section === "security" ? "active" : ""}" data-workspace-section="security"><div class="workspace-section-heading"><div><span>安全</span><h2>不可变账本与秘密边界</h2><p>验证验收报告、历史快照、H/I 决策和 API Key 的责任边界。</p></div></div>${checksMarkup(selectedChecks("security"))}</section>
          <section class="workspace-module-view ${section === "cost" ? "active" : ""}" data-workspace-section="cost"><div class="workspace-section-heading"><div><span>成本</span><h2>兼容 API 用量与显式预算</h2><p>只读取已保存的调用元数据，不读取密钥或消息正文。</p></div></div>${selected ? `<div class="release-scoreboard"><article><span>窗口</span><strong>${selected.cost.window_days} 天</strong></article><article><span>估算成本</span><strong>$${selected.cost.estimated_cost_usd.toFixed(4)}</strong></article><article><span>完成 / 失败</span><strong>${selected.cost.completed_requests} / ${selected.cost.failed_requests}</strong></article><article><span>搜索调用</span><strong>${selected.cost.search_calls}</strong></article></div>` : ""}${checksMarkup(selectedChecks("cost"))}</section>
          <section class="workspace-module-view ${section === "history" ? "active" : ""}" data-workspace-section="history"><div class="workspace-section-heading"><div><span>历史报告</span><h2>不可变发布验收记录</h2><p>历史报告只能读取，不能覆盖或删除。</p></div><button class="secondary" data-action="refresh-release-acceptance">刷新</button></div><section class="panel">${historyMarkup(runs, selected?.id ?? null)}</section></section>
        </main>
        <aside class="panel workspace-inspector" data-workspace-panel="release-inspector">${workspacePaneToggle("inspector", inspectorCollapsed)}<div class="panel-heading"><div><span>验收检查器</span><h2>${selected ? statusLabel(selected.overall_status) : "未运行"}</h2></div></div>${selected ? `<div class="inspector-kpis"><div><span>通过</span><strong>${selected.passed_count}</strong></div><div><span>警告</span><strong>${selected.warning_count}</strong></div><div><span>阻断</span><strong>${selected.blocked_count}</strong></div></div><p>报告指纹：<code>${escapeHtml(selected.report_sha256.slice(0, 20))}</code></p><button class="secondary" data-action="show-release-acceptance-json">查看完整报告</button>` : '<p>运行后显示报告摘要、指纹和原始证据。</p>'}</aside>
      </section>
    </div>
  </section>`;
}
