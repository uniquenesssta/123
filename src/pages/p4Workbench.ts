import { escapeHtml } from "../components/format";
import type {
  P4ConflictWorkspaceRecord,
  P4EvidenceWorkspaceRecord,
  P4MatchWorkspace,
  P4TaskWorkspace,
  RulePackageSummary,
  MatchRecord,
} from "../types";

function formatTime(value: string | null): string {
  if (!value) return "—";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return "未知";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

function safeSourceUrl(value: string | null): string | null {
  if (!value) return null;
  try {
    const url = new URL(value);
    return url.protocol === "https:" || url.protocol === "http:" ? url.toString() : null;
  } catch {
    return null;
  }
}

function taskStateLabel(state: string): string {
  const labels: Record<string, string> = {
    PLANNED: "已计划",
    RESEARCH_QUEUED: "研究排队",
    RESEARCH_RUNNING: "研究进行中",
    RESEARCH_SUCCEEDED: "研究完成",
    RESEARCH_PARTIAL: "等待冲突处理",
    READY_TO_FREEZE: "等待冻结",
    FREEZING: "正在冻结",
    FROZEN: "已冻结",
    BLOCKED: "已阻断",
    MISSED: "已错过时点",
    FAILED: "执行失败",
    CANCELLED: "已取消",
  };
  return labels[state] ?? state;
}

function verificationLabel(value: string): string {
  const labels: Record<string, string> = {
    CONFIRMED: "已确认",
    PROBABLE: "较可信",
    UNVERIFIED: "未验证",
    NOT_FOUND: "未找到",
    CONFLICT: "冲突",
    STALE: "已过期",
    NOT_APPLICABLE: "不适用",
  };
  return labels[value] ?? value;
}

function taskTone(state: string): string {
  if (state === "FROZEN" || state === "READY_TO_FREEZE" || state === "RESEARCH_SUCCEEDED") return "online";
  if (["BLOCKED", "FAILED", "MISSED"].includes(state)) return "offline";
  return "";
}

function matchStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    scheduled: "已排期",
    live: "进行中",
    finished: "已结束",
    postponed: "已延期",
    cancelled: "已取消",
  };
  return labels[status] ?? status;
}

function routeStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    routed: "已路由",
    missing: "明确未知",
    blocked_entity: "实体未确认",
    blocked_time: "时间门禁阻断",
    blocked_conflict: "来源冲突",
    blocked_unregistered_field: "字段未登记",
    ignored_non_model_fact: "非模型字段",
  };
  return labels[status] ?? status;
}

function manualDecisionLabel(value: string | null): string {
  if (value === "select_evidence") return "已采用证据";
  if (value === "accept_unknown") return "已接受未知";
  return value ?? "已处理";
}

function evidenceCard(evidence: P4EvidenceWorkspaceRecord, conflictId: string, disabled: boolean): string {
  const sourceUrl = safeSourceUrl(evidence.source_url);
  const sourceName = evidence.source_title ?? evidence.source_domain ?? "未命名来源";
  return `
    <label class="p4-evidence-choice ${disabled ? "disabled" : ""}">
      <input type="radio" name="p4-conflict-${escapeHtml(conflictId)}" value="${escapeHtml(evidence.id)}" ${disabled ? "disabled" : ""}>
      <span class="p4-evidence-body">
        <span class="p4-evidence-heading"><b>${escapeHtml(formatValue(evidence.value))}</b><em>${escapeHtml(verificationLabel(evidence.verification_state))} · ${escapeHtml(evidence.source_tier)}</em></span>
        <span>${sourceUrl ? `<a href="${escapeHtml(sourceUrl)}" target="_blank" rel="noreferrer">${escapeHtml(sourceName)}</a>` : escapeHtml(sourceName)}</span>
        <small>发布 ${escapeHtml(formatTime(evidence.published_at))} · 获取 ${escapeHtml(formatTime(evidence.retrieved_at))}</small>
      </span>
    </label>`;
}

function conflictCard(
  conflict: P4ConflictWorkspaceRecord,
  evidence: P4EvidenceWorkspaceRecord[],
  canResolve: boolean,
): string {
  const members = evidence.filter((item) => conflict.evidence_ids.includes(item.id));
  const resolved = conflict.manual_decision_kind !== null;
  const actionable = canResolve && !resolved && conflict.evaluation_status === "manual_required";
  return `
    <article class="p4-conflict-card ${resolved ? "resolved" : ""}">
      <div class="p4-conflict-heading">
        <div><span>${escapeHtml(conflict.field_key)}</span><h4>${resolved ? "已追加人工决策" : "事实来源相互冲突"}</h4></div>
        <span class="status-pill ${resolved ? "online" : "offline"}">${resolved ? escapeHtml(manualDecisionLabel(conflict.manual_decision_kind)) : "需要处理"}</span>
      </div>
      <p>${resolved
        ? `原证据未被改写；决策时间 ${escapeHtml(formatTime(conflict.manual_decision_at))}${conflict.manual_decision_note ? `，说明：${escapeHtml(conflict.manual_decision_note)}` : ""}`
        : "选择你确认采用的事实来源，或明确接受当前字段未知。人工选择会以“较可信”状态进入正式路由，避免把人工判断伪装成已确认事实。"}</p>
      <div class="p4-evidence-list">${members.length > 0 ? members.map((item) => evidenceCard(item, conflict.id, !actionable)).join("") : `<div class="empty-state compact"><span>该冲突没有可显示的证据成员。</span></div>`}</div>
      ${resolved ? "" : `
        <label class="field"><span>处理说明（可选）</span><input id="p4-conflict-note-${escapeHtml(conflict.id)}" placeholder="记录选择依据，写入不可变决策账本" ${actionable ? "" : "disabled"}></label>
        <div class="button-row">
          <button class="primary" data-action="resolve-p4-conflict-select" data-conflict-id="${escapeHtml(conflict.id)}" ${actionable && members.length > 0 ? "" : "disabled"}>采用所选证据</button>
          <button class="secondary" data-action="resolve-p4-conflict-unknown" data-conflict-id="${escapeHtml(conflict.id)}" ${actionable ? "" : "disabled"}>接受当前未知</button>
        </div>`}
    </article>`;
}

function taskWorkspaceMarkup(workspace: P4TaskWorkspace, rulePackageLabel: string): string {
  const task = workspace.task;
  const canResolve = ["RESEARCH_PARTIAL", "BLOCKED"].includes(task.state)
    && Date.now() < new Date(task.data_cutoff_at).getTime();
  const manualConflicts = workspace.conflicts.filter((item) =>
    item.evaluation_status === "manual_required" || item.manual_decision_kind !== null,
  );
  const sourceRows = workspace.evidence.map((item) => {
    const url = safeSourceUrl(item.source_url);
    return `<tr><td><strong>${escapeHtml(item.field_key)}</strong><small>${escapeHtml(item.entity_type)}</small></td><td>${escapeHtml(formatValue(item.value))}</td><td>${escapeHtml(verificationLabel(item.verification_state))}</td><td>${url ? `<a href="${escapeHtml(url)}" target="_blank" rel="noreferrer">${escapeHtml(item.source_title ?? item.source_domain ?? "打开来源")}</a>` : escapeHtml(item.source_title ?? item.source_domain ?? "无链接")}</td><td>${escapeHtml(formatTime(item.retrieved_at))}</td></tr>`;
  }).join("");
  const routeRows = workspace.routes.map((route) => `<tr><td><strong>${escapeHtml(route.field_key)}</strong><small>${escapeHtml(route.target_module)} / ${escapeHtml(route.target_slot)}</small></td><td>${escapeHtml(routeStatusLabel(route.route_status))}</td><td>${escapeHtml(verificationLabel(route.verification_state))}</td><td>${escapeHtml(formatValue(route.selected_value))}</td><td>${escapeHtml(route.reason)}</td></tr>`).join("");
  const events = workspace.events.slice().reverse().map((event) => `<li><span>${escapeHtml(formatTime(event.occurred_at))}</span><b>${escapeHtml(event.from_state ?? "创建")} → ${escapeHtml(event.to_state)}</b><p>${escapeHtml(event.reason)}</p></li>`).join("");

  return `
    <div class="p4-task-detail">
      <div class="p4-task-title">
        <div><span>${escapeHtml(task.horizon)} · ${escapeHtml(rulePackageLabel)} · 数据截止 ${escapeHtml(formatTime(task.data_cutoff_at))}</span><h3>${escapeHtml(taskStateLabel(task.state))}</h3><p>研究截止 ${escapeHtml(formatTime(task.research_due_at))} · 冻结截止 ${escapeHtml(formatTime(task.freeze_deadline_at))}</p></div>
        <span class="status-pill ${taskTone(task.state)}">${escapeHtml(taskStateLabel(task.state))}</span>
      </div>
      <div class="metrics-grid compact-metrics p4-readiness-grid">
        <article class="metric-card"><span>正式字段</span><strong>${workspace.readiness.requested_fact_count}</strong><small>本时点要求</small></article>
        <article class="metric-card"><span>已路由</span><strong>${workspace.readiness.routed_fact_count}</strong><small>可进入模型</small></article>
        <article class="metric-card"><span>未找到</span><strong>${workspace.readiness.missing_fact_count}</strong><small>已明确记录未知</small></article>
        <article class="metric-card"><span>阻断</span><strong>${workspace.readiness.blocked_fact_count}</strong><small>${workspace.readiness.ready ? "门禁已通过" : "尚未允许冻结"}</small></article>
      </div>
      ${workspace.readiness.blockers.length > 0 ? `<div class="alert error"><b>当前阻断</b>${workspace.readiness.blockers.map((item) => `<span>${escapeHtml(item)}</span>`).join("")}</div>` : `<div class="alert success"><b>冻结门禁</b><span>${workspace.readiness.ready ? "全部正式字段已形成可追溯路由。" : "正在等待研究任务或冻结执行。"}</span></div>`}

      <section class="p4-workbench-section">
        <div class="panel-heading compact"><div><span>冲突处理</span><h3>人工决策只追加，不覆盖来源</h3></div><span class="status-pill ${manualConflicts.some((item) => !item.manual_decision_kind) ? "offline" : "online"}">${manualConflicts.filter((item) => !item.manual_decision_kind).length} 个待处理</span></div>
        ${manualConflicts.length > 0 ? `<div class="p4-conflict-list">${manualConflicts.map((item) => conflictCard(item, workspace.evidence, canResolve)).join("")}</div>` : `<div class="empty-state compact"><strong>没有人工冲突</strong><span>当前研究结果不需要人工选择来源。</span></div>`}
        ${!canResolve && manualConflicts.some((item) => !item.manual_decision_kind) ? `<div class="alert error"><span>${Date.now() >= new Date(task.data_cutoff_at).getTime() ? "数据截止时间已到，正式证据不能再改变。" : "当前任务状态不允许处理冲突。"}</span></div>` : ""}
      </section>

      <section class="p4-workbench-section">
        <div class="panel-heading compact"><div><span>来源证据</span><h3>${workspace.evidence.length} 条研究证据</h3></div></div>
        <div class="table-wrap"><table class="data-table p4-table"><thead><tr><th>字段</th><th>事实值</th><th>验证</th><th>来源</th><th>获取时间</th></tr></thead><tbody>${sourceRows || `<tr><td colspan="5">尚未产生研究证据。</td></tr>`}</tbody></table></div>
      </section>

      <section class="p4-workbench-section">
        <div class="panel-heading compact"><div><span>有效路由</span><h3>${workspace.routes.length} 条字段路由</h3></div></div>
        <div class="table-wrap"><table class="data-table p4-table"><thead><tr><th>字段</th><th>状态</th><th>验证</th><th>有效值</th><th>说明</th></tr></thead><tbody>${routeRows || `<tr><td colspan="5">尚未产生字段路由。</td></tr>`}</tbody></table></div>
      </section>

      <section class="p4-workbench-section">
        <div class="panel-heading compact"><div><span>任务历史</span><h3>${workspace.events.length} 个不可变状态事件</h3></div></div>
        <ol class="p4-event-timeline">${events || `<li><p>尚无状态事件。</p></li>`}</ol>
      </section>

      <section class="p4-workbench-section p4-snapshot-summary">
        <div class="panel-heading compact"><div><span>冻结快照</span><h3>${workspace.snapshot ? "本时点快照已冻结" : "尚未生成冻结快照"}</h3></div>${workspace.snapshot ? `<button class="secondary" data-action="show-p4-snapshot">查看完整快照</button>` : ""}</div>
        <p>${workspace.snapshot ? "冻结结果、公开字段及提供器返回的概率链均可从当前任务追溯。" : "任务通过研究与路由门禁后，由后台 Worker 在冻结窗口内自动写入不可变快照。"}</p>
      </section>
    </div>`;
}

export function p4WorkbenchMarkup(
  databaseConfigured: boolean,
  packages: RulePackageSummary[],
  matches: MatchRecord[],
  workspace: P4MatchWorkspace | null,
  selectedTask: P4TaskWorkspace | null,
): string {
  const activeP4Packages = packages
    .filter((item) => item.status === "active"
      && (item.model_id === "p4" || item.model_id.startsWith("p4_")))
    .sort((left, right) => new Date(right.created_at).getTime() - new Date(left.created_at).getTime());
  const p4Packages = Array.from(new Map(activeP4Packages.map((item) => [item.package_key, item])).values())
    .sort((left, right) => right.priority - left.priority || left.display_name.localeCompare(right.display_name, "zh-CN"));
  const packageOptions = p4Packages.map((item, index) => `<option value="${escapeHtml(item.id)}" ${index === 0 ? "selected" : ""}>${escapeHtml(item.display_name)} · ${escapeHtml(item.version)}</option>`).join("");
  const packageLabels = new Map(packages.map((item) => [item.id, `${item.display_name} · ${item.version}`]));
  const matchOptions = matches.map((item) => `<option value="${escapeHtml(item.id)}" ${workspace?.match_id === item.id ? "selected" : ""}>${escapeHtml(item.home_team_name)} vs ${escapeHtml(item.away_team_name)} · ${escapeHtml(formatTime(item.kickoff_time))} · ${escapeHtml(matchStatusLabel(item.status))}</option>`).join("");
  const selectedMatch = workspace ? matches.find((item) => item.id === workspace.match_id) ?? null : null;
  const canPlan = selectedMatch?.status === "scheduled"
    && new Date(selectedMatch.kickoff_time).getTime() > Date.now();
  const taskCards = workspace?.tasks.map((task) => {
    const rulePackageLabel = packageLabels.get(task.rule_package_id) ?? "历史规则包";
    return `<button class="p4-horizon-card ${selectedTask?.task.id === task.id ? "active" : ""}" data-action="open-p4-task" data-task-id="${escapeHtml(task.id)}"><span>${escapeHtml(task.horizon)}</span><strong>${escapeHtml(taskStateLabel(task.state))}</strong><small>${escapeHtml(rulePackageLabel)}</small><small>截止 ${escapeHtml(formatTime(task.data_cutoff_at))}</small><em class="status-pill ${taskTone(task.state)}">${escapeHtml(taskStateLabel(task.state))}</em></button>`;
  }).join("") ?? "";
  const selectedTaskPackageLabel = selectedTask
    ? packageLabels.get(selectedTask.task.rule_package_id) ?? "历史规则包"
    : "";

  return `
    <section class="panel p4-workbench" id="p4-match-workbench">
      <div class="panel-heading">
        <div><span>接入点 G · 单场研究工作台</span><h2>三个计划窗口、按需 T-N、来源与冻结历史</h2><p>不离开赛事推演页即可完成本场比赛的正式研究全链路。</p></div>
        <button class="secondary" data-action="refresh-p4-workbench" ${databaseConfigured && workspace ? "" : "disabled"}>刷新状态</button>
      </div>
      ${!databaseConfigured ? `<div class="empty-state"><strong>连接数据库后启用</strong><span>工作台依赖接入点 F 的任务账本、联网研究和冻结 Worker。</span></div>` : matches.length === 0 ? `<div class="empty-state"><strong>暂无可用比赛</strong><span>先在阵容与比赛页创建比赛，再回到本页建立计划窗口。</span></div>` : `
        <div class="p4-workbench-picker"><label class="field"><span>工作台比赛</span><select id="p4-workbench-match-id">${matchOptions}</select><small>包含已结束比赛，可回看历史任务、来源、事件与冻结快照。</small></label></div>
        ${!workspace ? `<div class="empty-state"><strong>请选择一场比赛</strong><span>选择后会在本页加载三个计划窗口任务与历史记录。</span></div>` : `
        <div class="p4-match-context"><div><span>${escapeHtml(workspace.competition_name ?? "未绑定赛事")}</span><h3>${escapeHtml(workspace.home_team_name)} vs ${escapeHtml(workspace.away_team_name)}</h3><p>开球 ${escapeHtml(formatTime(workspace.kickoff_at))} · 当前比赛的研究、冲突和冻结记录均在本页完成</p></div><span class="status-pill ${workspace.tasks.length > 0 ? "online" : ""}">${workspace.tasks.length} 个计划窗口任务</span></div>
        <div class="p4-plan-row">
          <label class="field"><span>P4 正式规则包</span><select id="p4-plan-rule-package" ${p4Packages.length === 0 || !canPlan ? "disabled" : ""}>${packageOptions || `<option value="">没有已启用的 P4 规则包</option>`}</select><small>${canPlan ? "一次建立 T-24h、T-6h、T-1h 三个计划研究任务；T-N 由正式推演按需读取。" : "已开球或已结束比赛仅用于历史回看，不能补建正式赛前任务。"}</small></label>
          <button class="primary" data-action="plan-p4-horizons" ${p4Packages.length > 0 && canPlan ? "" : "disabled"}>${canPlan ? "建立 / 校验三个窗口计划" : "历史比赛只读"}</button>
        </div>
        <div class="p4-horizon-grid">${taskCards || `<div class="empty-state compact"><strong>尚未建立计划窗口</strong><span>选择 P4 规则包并建立三个计划窗口。</span></div>`}</div>
        ${selectedTask ? taskWorkspaceMarkup(selectedTask, selectedTaskPackageLabel) : `<div class="empty-state p4-task-empty"><strong>选择一个时点</strong><span>查看研究来源、冲突、状态事件和冻结快照。</span></div>`}
        `}
      `}
    </section>`;
}
