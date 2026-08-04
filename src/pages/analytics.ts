import { escapeHtml, formatBytes, formatPercent } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import { workspaceTaskAnchorNavigation } from "../components/workspace";
import type {
  AbilityUpdateCandidateRecord,
  AiAnalysisResponsePreview,
  AiAnalysisSuggestionRecord,
  AnalyticsOverview,
  BackgroundJob,
  CompetitionRecord,
  ParameterTuningCandidateRecord,
  PostmatchOverview,
  BootstrapResponse,
} from "../types";

function value(value: number | null, digits = 4): string {
  return value === null ? "—" : value.toFixed(digits);
}

function metric(label: string, amount: string, note: string): string {
  return `<article class="metric-card"><span>${escapeHtml(label)}</span><strong>${escapeHtml(amount)}</strong><small>${escapeHtml(note)}</small></article>`;
}

function statusLabel(status: BackgroundJob["status"]): string {
  return ({ queued: "排队中", running: "运行中", succeeded: "已完成", failed: "失败", cancelled: "已取消" } as const)[status];
}

function jobTypeLabel(jobType: string): string {
  return ({
    refresh_analytics: "模型评估",
    data_quality_scan: "数据质量检查",
    query_performance_scan: "数据库性能检查",
    full_analysis_refresh: "完整分析刷新",
  } as Record<string, string>)[jobType] ?? jobType;
}

function moduleLabel(value: string): string {
  return ({
    lineup_realization: "阵容兑现率",
    history: "历史表现",
    state: "近期状态",
    venue: "主场优势",
    draw_correction: "平局修正",
    synergy: "路径协同",
  } as Record<string, string>)[value] ?? "其他模块";
}

function candidateStatusLabel(value: string): string {
  return ({
    pending: "待人工审核",
    accepted_for_backtest: "待影子验证",
    blocked_by_h: "接入点 H 门禁阻断",
    shadow_running: "影子验证中",
    shadow_passed: "影子通过，待人工晋升",
    shadow_failed: "影子验证未通过",
    promoted: "已人工晋升",
    rolled_back: "已按绑定快照回滚",
    rejected: "已拒绝",
    superseded: "已被后续候选替代",
  } as Record<string, string>)[value] ?? "状态未知";
}

function suggestionTypeLabel(value: string): string {
  return ({
    parameter_tuning: "参数调整建议",
    data_quality: "数据质量建议",
    player_ability: "球员能力建议",
    model_review: "模型复核建议",
  } as Record<string, string>)[value] ?? "分析建议";
}

function severityText(value: string): string {
  return ({ info: "提示", warning: "需要关注", critical: "严重" } as Record<string, string>)[value] ?? "提示";
}

function snapshotLabel(value: string): string {
  return ({ "T-N": "任意赛前时间", "T-24h": "开球前 24 小时窗口", "T-6h": "开球前 6 小时窗口", "T-1h": "开球前 1 小时窗口", "T-90m": "历史 90 分钟时点" } as Record<string, string>)[value] ?? value;
}

function modelLabel(value: string): string {
  const normalized = value.toLowerCase();
  if (normalized.includes("p7")) return "第 7 代赛事模型";
  return value.replaceAll("_", " ");
}

function driftMetricLabel(value: string): string {
  return ({ log_loss: "结果概率误差", brier: "整体概率偏差", calibration_error: "可信度校准偏差", data_coverage: "数据覆盖率" } as Record<string, string>)[value] ?? value.replaceAll("_", " ");
}

function modelTable(overview: AnalyticsOverview | null): string {
  const rows = overview?.comparisons.slice(0, 12) ?? [];
  if (rows.length === 0) return `<div class="empty-state compact"><strong>还没有可比较的模型数据</strong><span>至少需要完成带推演结果的赛后复盘。</span></div>`;
  return `<div class="table-wrap"><table><thead><tr><th>排名</th><th>模型</th><th>数据时点</th><th>样本</th><th>结果概率误差</th><th>整体概率偏差</th><th>数据覆盖</th></tr></thead><tbody>${rows.map((row) => `<tr><td><strong>#${row.rank}</strong></td><td>${escapeHtml(modelLabel(row.model_key))}<small>参数版本 ${escapeHtml(row.parameter_version)}</small></td><td>${escapeHtml(snapshotLabel(row.snapshot_type))}</td><td>${row.sample_size}</td><td>${row.average_log_loss.toFixed(4)}</td><td>${row.average_brier.toFixed(4)}</td><td>${formatPercent(row.average_data_coverage)}</td></tr>`).join("")}</tbody></table></div>`;
}

function driftPanel(overview: AnalyticsOverview | null): string {
  const findings = overview?.drift ?? [];
  if (findings.length === 0) return `<div class="empty-state compact"><strong>暂未发现漂移</strong><span>样本不足时系统不会强行给出结论。</span></div>`;
  return `<div class="analysis-card-list">${findings.map((item) => `<article class="analysis-finding ${escapeHtml(item.severity)}"><div><span>${escapeHtml(driftMetricLabel(item.metric_name))}</span><strong>${item.severity === "critical" ? "明显漂移" : item.severity === "warning" ? "需要关注" : "稳定"}</strong></div><p>历史平均 ${item.baseline_mean.toFixed(4)} → 当前平均 ${item.current_mean.toFixed(4)}</p><small>历史样本 ${item.baseline_size} 场 · 当前样本 ${item.current_size} 场</small></article>`).join("")}</div>`;
}

function qualityPanel(overview: AnalyticsOverview | null): string {
  const quality = overview?.data_quality;
  if (!quality || quality.findings.length === 0) return `<div class="empty-state compact"><strong>暂无数据质量问题</strong><span>运行完整分析后会检查比赛、阵容、球员、模型和复盘。</span></div>`;
  return `<div class="quality-list">${quality.findings.slice(0, 30).map((item) => `<article class="quality-row ${escapeHtml(item.severity)}"><span>${item.severity === "critical" ? "严重" : item.severity === "warning" ? "警告" : "提示"}</span><div><strong>${escapeHtml(item.message)}</strong><small>系统检查发现</small></div><div class="button-row compact"><button class="secondary tiny" data-action="decide-quality-finding" data-finding-id="${escapeHtml(item.id)}" data-decision="resolve">已处理</button><button class="ghost tiny" data-action="decide-quality-finding" data-finding-id="${escapeHtml(item.id)}" data-decision="ignore">忽略</button><button class="ghost tiny" data-action="show-quality-json" data-finding-id="${escapeHtml(item.id)}">详情</button></div></article>`).join("")}</div>`;
}

function jobsPanel(jobs: BackgroundJob[]): string {
  if (jobs.length === 0) return `<div class="empty-state compact"><strong>暂无后台任务</strong><span>分析刷新会作为可恢复任务运行。</span></div>`;
  return `<div class="job-list">${jobs.slice(0, 20).map((job) => `<article class="job-row"><div class="job-main"><strong>${escapeHtml(jobTypeLabel(job.job_type))}</strong><span>${escapeHtml(statusLabel(job.status))} · 尝试 ${job.attempts}/${job.max_attempts}</span></div><div class="job-progress"><i style="width:${Math.max(0, Math.min(100, job.progress))}%"></i></div><b>${Math.round(job.progress)}%</b><div class="button-row compact">${job.status === "running" || job.status === "queued" ? `<button class="ghost tiny" data-action="cancel-analysis-job" data-job-id="${escapeHtml(job.id)}">取消</button>` : ""}${job.status === "failed" || job.status === "cancelled" ? `<button class="secondary tiny" data-action="retry-analysis-job" data-job-id="${escapeHtml(job.id)}">重试</button>` : ""}${job.error_message ? `<button class="ghost tiny" data-action="show-job-json" data-job-id="${escapeHtml(job.id)}">错误</button>` : ""}</div></article>`).join("")}</div>`;
}

function responsePreviewPanel(preview: AiAnalysisResponsePreview | null): string {
  if (!preview) return "";
  const blocking = preview.blocking_errors.length;
  return `<section class="panel"><div class="panel-heading"><div><span>智能分析建议</span><h2>导入前检查结果</h2></div><span class="status-pill ${blocking > 0 ? "offline" : "online"}">${blocking > 0 ? `${blocking} 个阻断问题` : `${preview.suggestions.length} 条建议`}</span></div>${preview.blocking_errors.map((item) => `<div class="alert error">${escapeHtml(item)}</div>`).join("")}${preview.warnings.map((item) => `<div class="alert">${escapeHtml(item)}</div>`).join("")}<div class="suggestion-preview">${preview.suggestions.slice(0, 20).map((item) => `<article><span>${escapeHtml(suggestionTypeLabel(item.suggestion_type))}</span><strong>${escapeHtml(item.title)}</strong><p>${escapeHtml(item.summary)}</p></article>`).join("")}</div><div class="button-row"><button class="primary" data-action="import-ai-analysis-response" ${blocking > 0 ? "disabled" : ""}>确认导入这些建议</button><button class="secondary" data-action="show-ai-response-json">查看全部检查结果</button></div></section>`;
}


function candidateLifecycleActions(item: ParameterTuningCandidateRecord): string {
  const candidateId = escapeHtml(item.id);
  const actions: string[] = [];
  if (item.status === "pending") {
    actions.push(`<button class="primary tiny" data-action="decide-parameter-tuning" data-candidate-id="${candidateId}" data-decision="accept_for_backtest">接受候选</button>`);
    actions.push(`<button class="secondary tiny" data-action="decide-parameter-tuning" data-candidate-id="${candidateId}" data-decision="reject">拒绝</button>`);
  }
  if (["accepted_for_backtest", "blocked_by_h", "shadow_failed"].includes(item.status)) {
    actions.push(`<button class="primary tiny" data-action="run-parameter-shadow-validation" data-candidate-id="${candidateId}">运行影子验证</button>`);
  }
  if (item.status === "shadow_passed") {
    actions.push(`<button class="primary tiny" data-action="promote-parameter-candidate" data-candidate-id="${candidateId}">人工晋升</button>`);
  }
  if (item.status === "promoted") {
    actions.push(`<button class="secondary tiny" data-action="rollback-parameter-candidate" data-candidate-id="${candidateId}">按快照回滚</button>`);
  }
  actions.push(`<button class="ghost tiny" data-action="show-parameter-lifecycle-history" data-candidate-id="${candidateId}">生命周期</button>`);
  actions.push(`<button class="ghost tiny" data-action="show-parameter-tuning" data-candidate-id="${candidateId}">诊断</button>`);
  return actions.join("");
}

function tuningPanel(candidates: ParameterTuningCandidateRecord[], competitions: CompetitionRecord[]): string {
  const awaitingAction = candidates.filter((item) => ["pending", "accepted_for_backtest", "shadow_passed", "blocked_by_h"].includes(item.status));
  const recent = candidates.slice(0, 12);
  return `<section class="panel tuning-workbench"><div class="panel-heading"><div><span>接入点 I · 受控参数生命周期</span><h2>候选校准、影子验证、人工晋升与回滚</h2></div><span class="status-pill ${awaitingAction.length > 0 ? "online" : ""}">${awaitingAction.length} 个待处理候选</span></div>
    <div class="tuning-flow"><div><b>1</b><strong>生成不可变候选</strong><span>新建候选模型与参数版本，正式绑定不动</span></div><i>→</i><div><b>2</b><strong>检查 H 门禁</strong><span>赛果、证据队列和漂移契约必须就绪</span></div><i>→</i><div><b>3</b><strong>留出集影子验证</strong><span>按赛事 Profile 与时点隔离，禁止混样本</span></div><i>→</i><div><b>4</b><strong>人工晋升或回滚</strong><span>不自动改写正式模型，绑定变化可追溯</span></div></div>
    <div class="notice-card"><strong>接入点 H 已接通</strong><span>阶段 I 只读取 H 的不可变结算样本、证据评分和正式分区漂移记录；样本不足或分区不一致时仍会阻断。公开仓库不执行参数生成或晋升，相关操作必须由外部模型提供器完成。</span></div>
    <div class="form-grid four-column clean-form"><label class="field"><span>具体赛事</span><select id="tuning-competition-id"><option value="">请选择赛事（禁止跨赛事）</option>${competitions.map((item) => `<option value="${escapeHtml(item.id)}">${escapeHtml(item.name)}</option>`).join("")}</select></label><label class="field"><span>数据窗口</span><select id="tuning-snapshot-type"><option value="T-N">T-N · 任意赛前时间</option><option value="T-24h">T-24h · 24 小时窗口</option><option value="T-6h">T-6h · 6 小时窗口</option><option value="T-1h" selected>T-1h · 1 小时窗口</option></select></label><label class="field"><span>本轮只调一个模块</span><select id="tuning-module"><option value="lineup_realization">阵容兑现率</option><option value="history">历史表现</option><option value="state">近期状态</option><option value="venue">主场优势</option><option value="draw_correction">平局修正</option><option value="synergy">路径协同</option></select></label><label class="field"><span>最小有效样本</span><select id="tuning-min-sample"><option value="50">50 场</option><option value="100" selected>100 场</option><option value="200">200 场</option><option value="500">500 场</option></select></label><label class="field"><span>单次最大变化</span><select id="tuning-max-change"><option value="0.02">2%</option><option value="0.05" selected>5%</option><option value="0.1">10%</option></select></label></div>
    <div class="button-row"><button class="primary" data-action="generate-parameter-tuning">生成不可变候选</button><button class="secondary" data-action="check-parameter-lifecycle-readiness">检查阶段 I 门禁</button><span class="field-note">训练、验证、留出窗口按时间顺序 60% / 20% / 20% 分割；只允许人工晋升。</span></div>
    ${recent.length === 0 ? `<div class="empty-state compact"><strong>暂无参数候选</strong><span>需要同一赛事、同一 Profile、同一冻结时点和同一基线版本的真实已结算样本。</span></div>` : `<div class="suggestion-list">${recent.map((item) => `<article class="suggestion-card"><div><span>${escapeHtml(item.competition_name ?? "未识别赛事")} · ${escapeHtml(snapshotLabel(item.snapshot_type))} · ${escapeHtml(moduleLabel(item.target_module))} · ${item.sample_size} 场</span><strong>${escapeHtml(item.parameter_version)} → ${escapeHtml(item.candidate_parameter_version ?? "候选版本未生成")}</strong><p>${escapeHtml(item.partition_key ?? "分区未锁定")} · 最大变化 ${formatPercent(Number(item.constraints.maximum_relative_change ?? 0))} · ${escapeHtml(candidateStatusLabel(item.status))}</p></div><div class="button-row compact">${candidateLifecycleActions(item)}</div></article>`).join("")}</div>`}
  </section>`;
}


function postmatchPanel(postmatch: PostmatchOverview, competitions: CompetitionRecord[]): string {
  const pending = postmatch.evidence_queue.filter((item) => item.status === "pending").slice(0, 20);
  const latestDrift = postmatch.drift_runs[0] ?? null;
  const latestScores = postmatch.provider_scores.slice(0, 12);
  return `<section class="panel postmatch-workbench"><div class="panel-heading"><div><span>接入点 H · 赛后闭环</span><h2>正式结算、证据评分与漂移监控</h2></div><span class="status-pill online">已启用</span></div>
    <div class="metric-grid compact-metrics">
      ${metric("正式结算", `${postmatch.settlement_count}`, "仅成功推演 + 冻结快照 + 最终复盘")}
      ${metric("待判定证据", `${postmatch.pending_evidence_count}`, "需要人工核对真实赛后结果")}
      ${metric("已评分证据", `${postmatch.scored_evidence_count}`, "不可验证项不会伪造正确率")}
      ${metric("最新漂移", latestDrift?.status ?? "未计算", latestDrift ? `${latestDrift.competition_name} · ${snapshotLabel(latestDrift.horizon)}` : "选择赛事后运行正式分区监控")}
    </div>
    <div class="notice-card"><strong>统计隔离</strong><span>所有 H 指标严格绑定模型版本 × 赛事 Profile × 参数版本 × 冻结时点；不会跨赛事或跨时点合并。</span></div>
    <div class="form-grid four-column clean-form"><label class="field"><span>监控赛事</span><select id="postmatch-competition-id"><option value="">请选择赛事</option>${competitions.map((item) => `<option value="${escapeHtml(item.id)}">${escapeHtml(item.name)}</option>`).join("")}</select></label><label class="field"><span>数据窗口</span><select id="postmatch-horizon"><option value="T-N">T-N · 任意赛前时间</option><option value="T-24h">T-24h · 24 小时窗口</option><option value="T-6h">T-6h · 6 小时窗口</option><option value="T-1h" selected>T-1h · 1 小时窗口</option></select></label><label class="field"><span>基线窗口</span><input id="postmatch-baseline-size" type="number" min="5" value="100"></label><label class="field"><span>当前窗口</span><input id="postmatch-current-size" type="number" min="5" value="50"></label></div>
    <div class="button-row"><button class="primary" data-action="refresh-postmatch-monitoring">刷新 H 正式监控</button><button class="secondary" data-action="refresh-analysis-page">刷新全部数据</button></div>
    <div class="two-column analysis-columns"><div><div class="subheading"><div><span>证据队列</span><h3>待人工判定</h3></div><small>${pending.length} 项显示中</small></div>${pending.length === 0 ? `<div class="empty-state compact"><strong>没有待判定证据</strong><span>完成正式结算后，冻结快照关联的真实证据会自动进入这里。</span></div>` : `<div class="suggestion-list">${pending.map((item) => `<article class="suggestion-card"><div><span>${escapeHtml(item.provider_name ?? "未绑定供应商")} · ${escapeHtml(item.field_key)} · 时效 ${formatPercent(item.timeliness_score)}</span><strong>${escapeHtml(item.source_title ?? item.source_domain ?? "未命名证据")}</strong><p>${escapeHtml(item.verification_state)} · ${escapeHtml(item.source_tier)}</p></div><button class="primary tiny" data-action="prepare-evidence-decision" data-evidence-item-id="${escapeHtml(item.id)}">人工判定</button></article>`).join("")}</div>`}</div><div><div class="subheading"><div><span>供应商评分</span><h3>最新不可变快照</h3></div></div>${latestScores.length === 0 ? `<div class="empty-state compact"><strong>暂无供应商评分</strong><span>证据完成判定并运行 H 监控后生成。</span></div>` : `<div class="table-wrap"><table><thead><tr><th>供应商</th><th>样本</th><th>准确</th><th>及时</th><th>可靠</th><th>综合</th></tr></thead><tbody>${latestScores.map((item) => `<tr><td>${escapeHtml(item.provider_name)}</td><td>${item.sample_size}</td><td>${formatPercent(item.accuracy_mean)}</td><td>${formatPercent(item.timeliness_mean)}</td><td>${formatPercent(item.reliability_mean)}</td><td><strong>${formatPercent(item.weighted_score)}</strong></td></tr>`).join("")}</tbody></table></div>`}</div></div>
  </section>`;
}

function abilityCandidatesPanel(candidates: AbilityUpdateCandidateRecord[]): string {
  if (candidates.length === 0) return `<div class="empty-state compact"><strong>暂无待审核能力候选</strong><span>赛后复盘或智能分析建议会先进入这里，不会直接覆盖球员能力。</span></div>`;
  return `<div class="suggestion-list">${candidates.slice(0, 50).map((item) => `<article class="suggestion-card"><div><span>${escapeHtml(item.dimension_name)} · 可信度 ${formatPercent(item.confidence)}</span><strong>${escapeHtml(item.player_name)}：${item.current_value === null ? "未设置" : item.current_value.toFixed(2)} → ${item.proposed_value.toFixed(2)}</strong><p>样本 ${item.sample_size} · ${escapeHtml(item.calculation_version)}</p></div><div class="button-row compact"><button class="primary tiny" data-action="decide-ability-candidate" data-candidate-id="${escapeHtml(item.id)}" data-decision="accept">写入能力历史</button><button class="secondary tiny" data-action="decide-ability-candidate" data-candidate-id="${escapeHtml(item.id)}" data-decision="reject">拒绝</button><button class="ghost tiny" data-action="show-candidate-json" data-candidate-id="${escapeHtml(item.id)}">证据</button></div></article>`).join("")}</div>`;
}

function suggestionsPanel(suggestions: AiAnalysisSuggestionRecord[]): string {
  const pending = suggestions.filter((item) => item.status === "pending");
  if (pending.length === 0) return `<div class="empty-state compact"><strong>暂无待审核智能分析建议</strong><span>智能分析建议导入后会先进入审核区，不会直接修改数据库。</span></div>`;
  return `<div class="suggestion-list">${pending.slice(0, 50).map((item) => `<article class="suggestion-card"><div><span>${escapeHtml(suggestionTypeLabel(item.suggestion_type))} · ${escapeHtml(severityText(item.severity || "info"))}</span><strong>${escapeHtml(item.title)}</strong><p>${escapeHtml(item.summary)}</p></div><div class="button-row compact"><button class="primary tiny" data-action="decide-ai-suggestion" data-suggestion-id="${escapeHtml(item.id)}" data-decision="accept">接受</button><button class="secondary tiny" data-action="decide-ai-suggestion" data-suggestion-id="${escapeHtml(item.id)}" data-decision="reject">拒绝</button><button class="ghost tiny" data-action="show-ai-suggestion-json" data-suggestion-id="${escapeHtml(item.id)}">证据</button></div></article>`).join("")}</div>`;
}

function latestAnalysisJob(jobs: BackgroundJob[], jobType: string): BackgroundJob | null {
  return jobs.find((job) => job.job_type === jobType) ?? null;
}

function chainStateClass(completed: boolean, enabled: boolean): string {
  if (completed) return "complete";
  return enabled ? "current" : "locked";
}

function chainStatusText(completed: boolean, enabled: boolean, currentLabel = "待处理"): string {
  if (completed) return "已满足";
  return enabled ? currentLabel : "前置条件未满足";
}

function analysisStepSummary(
  number: number,
  eyebrow: string,
  title: string,
  description: string,
  completed: boolean,
  enabled: boolean,
  currentLabel: string,
): string {
  return `<summary class="analysis-step-summary"><span class="analysis-step-number">${String(number).padStart(2, "0")}</span><div><span>${escapeHtml(eyebrow)}</span><strong>${escapeHtml(title)}</strong><small>${escapeHtml(description)}</small></div><b class="analysis-step-status">${escapeHtml(chainStatusText(completed, enabled, currentLabel))}</b><i aria-hidden="true"></i></summary>`;
}

function lockedStep(reason: string, action = "请先完成上一阶段"): string {
  return `<div class="analysis-step-lock"><div class="analysis-step-lock-icon">锁</div><div><strong>${escapeHtml(action)}</strong><p>${escapeHtml(reason)}</p></div></div>`;
}

export function analyticsPage(
  state: BootstrapResponse,
  overview: AnalyticsOverview | null,
  jobs: BackgroundJob[],
  preview: AiAnalysisResponsePreview | null,
  suggestions: AiAnalysisSuggestionRecord[],
  abilityCandidates: AbilityUpdateCandidateRecord[],
  competitions: CompetitionRecord[],
  tuningCandidates: ParameterTuningCandidateRecord[],
  postmatch: PostmatchOverview,
  lastAnalysisPackageId: string | null,
): string {
  if (!state.data.database_configured) {
    return `<section class="task-empty-workspace">${taskPageHeader({ eyebrow: "分析与历史", title: "从赛后历史到模型校准的受控链路", description: "连接成功后按“历史样本 → 完整分析 → 质量门禁 → 受控校准”的顺序引导操作。", status: { label: "等待数据库", tone: "warning" } })}${taskContextRibbon([{ label: "当前状态", value: "数据库未连接", note: "连接后自动加载结算样本、模型表现和质量门禁", tone: "warning" }])}${inlineDatabaseSetup("连接数据服务以运行分析", "连接成功后本页会自动加载正式结算、模型表现、数据质量和待审核建议。", state.connection_error)}</section>`;
  }

  const quality = overview?.data_quality;
  const queryWarnings = overview?.query_performance.tables.filter((item) => item.severity !== "stable") ?? [];
  const fullJob = latestAnalysisJob(jobs, "full_analysis_refresh");
  const fullJobRunning = fullJob?.status === "queued" || fullJob?.status === "running";
  const historyReady = postmatch.settlement_count > 0;
  const analysisReady = historyReady && Boolean(overview?.generated_at) && (overview?.sample_size ?? 0) > 0;
  const qualityScanned = analysisReady && Boolean(quality?.scan_id);
  const qualityReady = qualityScanned && (quality?.critical ?? 0) === 0;
  const pendingSuggestions = suggestions.filter((item) => item.status === "pending");
  const pendingReviewCount = pendingSuggestions.length + abilityCandidates.length;
  const reviewGateReady = qualityReady && pendingReviewCount === 0;
  const lifecycleCompleted = tuningCandidates.some((item) => item.status === "promoted");
  const completedSteps = [historyReady, analysisReady, reviewGateReady, lifecycleCompleted].filter(Boolean).length;
  const activeStep = !historyReady ? 1 : !analysisReady ? 2 : !reviewGateReady ? 3 : 4;

  const contextRibbon = taskContextRibbon([
    { label: "正式结算样本", value: `${postmatch.settlement_count} 场`, note: `${postmatch.pending_evidence_count} 条证据待判定`, tone: historyReady ? "accent" : "neutral" },
    { label: "完整分析", value: analysisReady ? `${overview?.sample_size ?? 0} 个有效样本` : "尚未完成", note: overview?.generated_at ? `生成于 ${overview.generated_at}` : "先形成正式结算样本", tone: analysisReady ? "success" : historyReady ? "warning" : "neutral" },
    { label: "质量门禁", value: qualityReady ? "严重问题已清零" : `${quality?.critical ?? 0} 条严重问题`, note: `${pendingReviewCount} 项建议或能力候选待审核`, tone: qualityReady ? "success" : analysisReady ? "warning" : "neutral" },
    { label: "参数生命周期", value: lifecycleCompleted ? "已有人工晋升版本" : "尚未完成晋升", note: reviewGateReady ? "可以生成受控候选" : "先通过质量与人工审核门禁", tone: lifecycleCompleted ? "success" : reviewGateReady ? "accent" : "neutral" },
  ]);
  const stepNavigation = workspaceTaskAnchorNavigation([
    { id: "analysis-history-step", index: "01", label: "历史样本", description: "复盘与正式结算", badge: historyReady ? "已就绪" : "待建立" },
    { id: "analysis-model-step", index: "02", label: "完整分析", description: "模型表现与漂移", badge: analysisReady ? "已完成" : fullJobRunning ? "运行中" : "待运行" },
    { id: "analysis-quality-step", index: "03", label: "质量门禁", description: "问题与建议审核", badge: reviewGateReady ? "通过" : `${(quality?.critical ?? 0) + pendingReviewCount}` },
    { id: "analysis-lifecycle-step", index: "04", label: "受控校准", description: "候选、影子与晋升", badge: lifecycleCompleted ? "已晋升" : "未完成" },
  ]);
  return `<section class="module-workspace-page analysis-module-workspace">
    ${taskPageHeader({ eyebrow: "分析与历史", title: "从赛后历史到模型校准的受控链路", description: "每一步都有明确输入、完成条件和下一步动作；前置条件未满足时保持锁定，避免错误结论或错误改参。", status: { label: `${completedSteps}/4 阶段已完成`, tone: completedSteps === 4 ? "success" : completedSteps > 0 ? "accent" : "neutral" }, actions: `<button class="secondary" data-action="refresh-analysis-page">刷新链路状态</button>` })}
    ${contextRibbon}
    <div class="core-local-navigation">${stepNavigation}</div>
    <div class="module-workspace-stage" data-workspace-scroll-key="analytics-stage"><div class="workspace-module-view active">
    <details id="analysis-history-step" class="analysis-chain-step workspace-anchor-target ${chainStateClass(historyReady, true)}" ${activeStep === 1 ? "open" : ""}>
      ${analysisStepSummary(1, "第一步 · 历史样本", "形成历史样本并创建正式结算", "真实赛果、成功推演、阵容与证据形成不可变历史样本。", historyReady, true, "需要建立样本")}
      <div class="analysis-step-content">
      <div class="analysis-step-guide"><article><span>本步输入</span><strong>真实赛果、成功推演、阵容与证据</strong><p>先到“赛后复盘”选择已结束比赛，填写赛果并完成正式结算。</p></article><article><span>完成标准</span><strong>至少 1 条正式结算</strong><p>当前已有 ${postmatch.settlement_count} 条结算，待判定证据 ${postmatch.pending_evidence_count} 条。</p></article><div class="analysis-step-actions"><button class="primary" data-page="review">前往赛后复盘</button><button class="secondary" data-action="refresh-postmatch-monitoring" ${historyReady ? "" : "disabled"}>刷新结算监控</button></div></div>
      ${postmatchPanel(postmatch, competitions)}
      </div>
    </details>

    <details id="analysis-model-step" class="analysis-chain-step workspace-anchor-target ${chainStateClass(analysisReady, historyReady)}" ${activeStep === 2 ? "open" : ""}>
      ${analysisStepSummary(2, "第二步 · 完整分析", "计算模型表现、校准分桶与近期漂移", "只读取已冻结的正式历史，不会自动修改模型或参数。", analysisReady, historyReady, fullJobRunning ? "分析运行中" : "可以运行")}
      <div class="analysis-step-content">
      ${historyReady ? `
        <div class="analysis-step-guide"><article><span>本步输入</span><strong>${postmatch.settlement_count} 条正式结算</strong><p>系统会按模型、参数版本、赛事 Profile 和数据时点分组，禁止混合样本。</p></article><article><span>完成标准</span><strong>生成时间与有效样本均存在</strong><p>${overview?.generated_at ? `最近生成：${escapeHtml(overview.generated_at)}` : "尚未生成分析结果"}；当前有效样本 ${overview?.sample_size ?? 0}。</p></article><div class="analysis-step-actions"><button class="primary" data-action="run-full-analysis" ${fullJobRunning ? "disabled" : ""}>${fullJobRunning ? "完整分析运行中" : analysisReady ? "重新运行完整分析" : "运行完整分析"}</button><button class="secondary" data-action="refresh-analysis-jobs">刷新任务</button></div></div>
        <section class="metric-grid compact-metrics analysis-step-metrics">
          ${metric("有效样本", `${overview?.sample_size ?? 0}`, "带正式复盘和成功推演")}
          ${metric("结果概率误差", value(overview?.average_log_loss ?? null), "越低代表结果概率越准确")}
          ${metric("整体概率偏差", value(overview?.average_brier ?? null), "越低代表胜平负概率越稳定")}
          ${metric("可信度校准偏差", value(overview?.expected_calibration_error ?? null), "越接近 0 越可靠")}
          ${metric("数据问题", `${quality?.open_total ?? 0}`, `${quality?.critical ?? 0} 严重 · ${quality?.warning ?? 0} 警告`)}
          ${metric("数据库", formatBytes(overview?.query_performance.database_size_bytes ?? 0), `${queryWarnings.length} 项性能提醒`)}
        </section>
        <div class="two-column analysis-columns"><section class="panel inset-panel"><div class="panel-heading"><div><span>模型比较</span><h3>模型表现排名</h3></div></div>${modelTable(overview)}</section><section class="panel inset-panel"><div class="panel-heading"><div><span>稳定性检查</span><h3>近期变化</h3></div></div>${driftPanel(overview)}</section></div>
        <section class="panel inset-panel"><div class="panel-heading"><div><span>运行进度</span><h3>分析任务</h3></div><button class="secondary" data-action="refresh-analysis-jobs">刷新任务</button></div>${jobsPanel(jobs)}</section>
      ` : lockedStep("至少完成一场比赛的正式赛后结算后，才能运行完整分析。", "先完成第一步")}
      </div>
    </details>

    <details id="analysis-quality-step" class="analysis-chain-step workspace-anchor-target ${chainStateClass(reviewGateReady, analysisReady)}" ${activeStep === 3 ? "open" : ""}>
      ${analysisStepSummary(3, "第三步 · 质量门禁与建议审核", "处理严重数据问题，再决定是否接受分析建议", "严重问题阻断后续；外部建议必须先预检再人工审核。", reviewGateReady, analysisReady, qualityReady ? `${pendingReviewCount} 项待审核` : "质量门禁未通过")}
      <div class="analysis-step-content">
      ${analysisReady ? `
        <div class="analysis-step-guide"><article><span>本步输入</span><strong>完整分析结果与数据质量扫描</strong><p>当前严重问题 ${quality?.critical ?? 0} 条、警告 ${quality?.warning ?? 0} 条、待审核建议 ${pendingReviewCount} 条。</p></article><article><span>完成标准</span><strong>严重问题为 0，待审核项目为 0</strong><p>外部智能分析是可选分支；导入后必须逐条接受或拒绝，不能直接改写正式数据。</p></article><div class="analysis-step-actions"><button class="primary" data-action="run-quality-scan">${qualityScanned ? "重新检查数据质量" : "运行数据质量检查"}</button><button class="secondary" data-action="export-ai-analysis-package" ${qualityReady ? "" : "disabled"}>导出智能分析资料</button></div></div>
        <section class="panel inset-panel"><div class="panel-heading"><div><span>质量门禁</span><h3>需要处理的数据问题</h3></div><span class="status-pill ${(quality?.critical ?? 0) > 0 ? "offline" : "online"}">${(quality?.critical ?? 0) > 0 ? `${quality?.critical ?? 0} 条严重问题` : "严重问题已清零"}</span></div>${qualityPanel(overview)}</section>
        <section class="analysis-exchange-panel"><div class="analysis-exchange-heading"><div><span>可选分支 · 外部智能分析</span><h3>导出资料 → 生成回包 → 导入预检 → 人工审核</h3><p>只有质量门禁通过后才能导出。建议填写模板必须绑定刚导出的分析包；导入文件必须先通过格式与来源检查。</p></div><span class="status-pill ${lastAnalysisPackageId ? "online" : ""}">${lastAnalysisPackageId ? "分析包已登记" : "尚未导出分析包"}</span></div><div class="analysis-actions chain-actions">
          <button class="action-card" data-action="export-ai-analysis-package" ${qualityReady ? "" : "disabled"}><span class="action-icon">1</span><div><strong>导出智能分析资料</strong><small>整理模型、复盘、球员、球队和数据质量摘要</small></div><b>${qualityReady ? "导出资料 →" : "先通过质量门禁"}</b></button>
          <button class="action-card" data-action="export-ai-response-template" ${lastAnalysisPackageId ? "" : "disabled"}><span class="action-icon">2</span><div><strong>生成建议填写模板</strong><small>绑定已导出的 package_id，提供固定格式和检查清单</small></div><b>${lastAnalysisPackageId ? "生成模板 →" : "先导出资料"}</b></button>
          <button class="action-card" data-action="preview-ai-analysis-response" ${lastAnalysisPackageId ? "" : "disabled"}><span class="action-icon">3</span><div><strong>导入并预检分析回包</strong><small>检查来源、格式和阻断项，通过后才能正式导入</small></div><b>${lastAnalysisPackageId ? "选择回包 →" : "先完成前两步"}</b></button>
        </div></section>
        ${responsePreviewPanel(preview)}
        <div class="two-column analysis-columns"><section class="panel inset-panel"><div class="panel-heading"><div><span>智能分析建议</span><h3>逐条人工审核</h3></div><span class="status-pill">${pendingSuggestions.length} 条待处理</span></div>${suggestionsPanel(suggestions)}</section><section class="panel inset-panel"><div class="panel-heading"><div><span>能力审核</span><h3>待确认的球员能力变化</h3></div><span class="status-pill">${abilityCandidates.length} 条待处理</span></div>${abilityCandidatesPanel(abilityCandidates)}</section></div>
      ` : lockedStep("第二步必须生成包含有效样本的完整分析结果，才能检查质量和处理建议。", "先完成第二步")}
      </div>
    </details>

    <details id="analysis-lifecycle-step" class="analysis-chain-step workspace-anchor-target ${chainStateClass(lifecycleCompleted, reviewGateReady)}" ${activeStep === 4 ? "open" : ""}>
      ${analysisStepSummary(4, "第四步 · 受控参数生命周期", "生成候选、影子验证、人工晋升，并保留回滚能力", "所有候选都保持可追溯，不会自动覆盖正式模型。", lifecycleCompleted, reviewGateReady, "可以生成候选")}
      <div class="analysis-step-content">
      ${reviewGateReady ? tuningPanel(tuningCandidates, competitions) : lockedStep((quality?.critical ?? 0) > 0 ? `仍有 ${quality?.critical ?? 0} 条严重数据问题。` : `仍有 ${pendingReviewCount} 条建议或能力候选未完成人工审核。`, "先通过第三步门禁")}
      </div>
    </details>

    ${analysisReady ? `<details class="panel disclosure-panel analysis-advanced"><summary><div><span>高级历史详情</span><strong>校准分桶与数据库表统计</strong></div><b>展开</b></summary><div class="disclosure-content"><div class="table-wrap"><table><thead><tr><th>结果</th><th>概率区间</th><th>样本</th><th>预测均值</th><th>实际命中</th><th>差值</th></tr></thead><tbody>${(overview?.calibration ?? []).map((item) => `<tr><td>${escapeHtml(item.outcome)}</td><td>${item.lower_bound.toFixed(1)}–${item.upper_bound.toFixed(1)}</td><td>${item.sample_size}</td><td>${formatPercent(item.predicted_mean)}</td><td>${formatPercent(item.actual_rate)}</td><td>${formatPercent(item.absolute_gap)}</td></tr>`).join("")}</tbody></table></div><div class="table-wrap"><table><thead><tr><th>数据表</th><th>估算行数</th><th>大小</th><th>顺序扫描</th><th>索引扫描</th><th>建议</th></tr></thead><tbody>${(overview?.query_performance.tables ?? []).slice(0, 50).map((item) => `<tr><td>${escapeHtml(item.schema_name)}.${escapeHtml(item.table_name)}</td><td>${item.estimated_rows}</td><td>${formatBytes(item.table_size_bytes)}</td><td>${item.sequential_scans}</td><td>${item.index_scans}</td><td>${escapeHtml(item.recommendation ?? "正常")}</td></tr>`).join("")}</tbody></table></div></div></details>` : ""}
    </div></div>
  </section>`;
}
