import { escapeHtml, formatPercent } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { lineupCompletionWorkspace } from "../components/lineupBuilder";
import { taskContextRibbon, taskPageHeader, taskStatusChip } from "../components/taskWorkspace";
import {
  matchReviewWorkflowAllows,
  matchReviewWorkflowBlocker,
  matchReviewWorkflowCompleted,
} from "../app/matchReviewWorkflow";
import type {
  BootstrapResponse,
  LineupRecord,
  MatchReviewDetail,
  MatchReviewSummary,
  MatchReviewPackagePreview,
  MatchReviewPackageSnapshotSummary,
  MatchReviewPackageWorkflowAction,
  MatchReviewPackageWorkflowRecord,
  PostmatchSettlementRecord,
  ReviewableMatch,
  PlayerCatalogReferenceData,
  PlayerListItem,
  LineupBuilderPlayer,
  LineupBuilderFormState,
} from "../types";

function num(value: number | null | undefined, digits = 1): string {
  return value === null || value === undefined || !Number.isFinite(value) ? "—" : value.toFixed(digits);
}

function eventTypeLabel(value: string): string {
  return ({
    substitution: "换人",
    goal: "进球",
    own_goal: "乌龙球",
    assist: "助攻",
    penalty_goal: "点球命中",
    penalty_missed: "点球罚失",
    yellow_card: "黄牌",
    second_yellow_card: "第二张黄牌",
    red_card: "红牌",
    injury: "伤退/伤情",
    var: "VAR",
    formation_change: "阵型变化",
    goalkeeper_change: "门将更换",
    other: "其他事件",
  } as Record<string, string>)[value] ?? value;
}

function eventMinute(minute: number, stoppage: number | null): string {
  return stoppage && stoppage > 0 ? `${minute}+${stoppage}′` : `${minute}′`;
}

function eventStatusLabel(verification: string, revision: string): string {
  if (revision === "cancelled") return "已取消";
  if (revision === "corrected") return "已修订";
  if (verification === "disputed") return "存在争议";
  if (verification === "unverified") return "待核验";
  return "已核验";
}

function matchEventTimeline(detail: MatchReviewDetail): string {
  const summary = detail.event_summary;
  const latestScore = summary.latest_home_score === null || summary.latest_away_score === null
    ? "未逐事件记录"
    : `${summary.latest_home_score}-${summary.latest_away_score}`;
  const timeline = detail.events.length === 0
    ? `<div class="empty-state compact"><strong>尚未录入结构化比赛事件</strong><span>资料包可记录进球、助攻、牌、换人、伤退、VAR 与阵型变化。</span></div>`
    : `<div class="match-event-timeline">${detail.events.map((event) => {
        const score = event.home_score === null || event.away_score === null
          ? ""
          : `<b>${event.home_score}-${event.away_score}</b>`;
        const participants = [event.player_name, event.related_player_name]
          .filter((value): value is string => Boolean(value))
          .join(" → ");
        return `<article class="match-event-row revision-${escapeHtml(event.revision_status)} verification-${escapeHtml(event.verification_status)}">
          <time>${escapeHtml(eventMinute(event.minute, event.stoppage_minute))}</time>
          <div><header><strong>${escapeHtml(eventTypeLabel(event.event_type))}</strong>${score}<em>${escapeHtml(eventStatusLabel(event.verification_status, event.revision_status))}</em></header><p>${escapeHtml(participants || event.team_name || "未关联主体")}${event.description ? ` · ${escapeHtml(event.description)}` : ""}</p><small>${escapeHtml(event.team_name ?? "全场事件")} · 可信度 ${formatPercent(event.confidence)} · 序号 ${event.sequence_no}</small></div>
        </article>`;
      }).join("")}</div>`;
  return `<section class="match-event-panel"><header><div><span>结构化比赛事实</span><h3>事件时间线</h3></div><small>${summary.effective_count} 条有效 · ${summary.verified_count} 条已核验</small></header><div class="match-event-summary"><div><span>当前事件</span><strong>${summary.total_count}</strong></div><div><span>取消/争议</span><strong>${summary.cancelled_count}/${summary.disputed_count}</strong></div><div><span>最后事件分钟</span><strong>${summary.last_event_minute ?? "—"}</strong></div><div><span>最后事件后比分</span><strong>${escapeHtml(latestScore)}</strong></div></div>${timeline}</section>`;
}

function matchLabel(item: ReviewableMatch): string {
  const match = item.match_record;
  const result = item.result ? ` · ${item.result.home_goals_90}-${item.result.away_goals_90}` : "";
  return `${match.home_team_name} vs ${match.away_team_name}${result}`;
}

function selectedLineupCards(lineups: LineupRecord[]): string {
  if (lineups.length === 0) {
    return `<div class="empty-state compact"><strong>这场比赛还没有阵容</strong><span>可在下方直接补录主客队阵容，不需要离开当前页面。</span></div>`;
  }
  return lineups.map((lineup) => `
    <section class="review-team-input">
      <header><div><span>${escapeHtml(lineup.lineup_type === "actual" ? "实际阵容" : lineup.lineup_type === "confirmed" ? "确认阵容" : "预计阵容")}</span><h3>${escapeHtml(lineup.team_name)}</h3></div><small>${lineup.starter_count} 首发 · ${lineup.player_count} 人</small></header>
      <div class="review-player-list">
        ${lineup.players.map((player) => `
          <article class="review-player-row" data-review-player
            data-player-id="${escapeHtml(player.player_id)}"
            data-team-id="${escapeHtml(lineup.team_id)}"
            data-position-code="${escapeHtml(player.position_code ?? "")}"
            data-role-code="${escapeHtml(player.role_code ?? "")}"
            data-started="${player.is_starter ? "true" : "false"}">
            <div class="review-player-name"><strong>${escapeHtml(player.player_name)}</strong><span>${player.is_starter ? "首发" : "替补"}${player.position_code ? ` · ${escapeHtml(player.position_code)}` : ""}${player.role_code ? ` · ${escapeHtml(player.role_code)}` : " · 角色待补"} · ${player.role_origin === "player_position_default" ? `资料继承${player.role_source_position_code ? `（${escapeHtml(player.role_source_position_code)}）` : ""}` : player.role_origin === "lineup_override" ? "本场覆盖" : "角色缺失"}</span></div>
            <label><span>分钟</span><input class="compact-input" data-field="minutes" type="number" min="0" max="150" value="${player.actual_minutes ?? (player.is_starter ? 90 : 0)}"></label>
            <label><span>评分 0–10</span><input class="compact-input" data-field="rating" type="number" min="0" max="10" step="0.1" placeholder="必填"></label>
            <label><span>进球</span><input class="compact-input" data-field="goals" type="number" min="0" step="1" value="0"></label>
            <label><span>助攻</span><input class="compact-input" data-field="assists" type="number" min="0" step="1" value="0"></label>
            <details class="review-player-extra"><summary>补充数据</summary><div class="review-extra-grid">
              <label><span>预期进球（xG）</span><input data-field="expected_goals" type="number" min="0" step="0.01" value="0"></label>
              <label><span>预期助攻（xA）</span><input data-field="expected_assists" type="number" min="0" step="0.01" value="0"></label>
              <label><span>射门</span><input data-field="shots" type="number" min="0" step="1" value="0"></label>
              <label><span>射正</span><input data-field="shots_on_target" type="number" min="0" step="1" value="0"></label>
              <label><span>关键传球</span><input data-field="key_passes" type="number" min="0" step="1" value="0"></label>
              <label><span>推进动作</span><input data-field="progressive_actions" type="number" min="0" step="1" value="0"></label>
              <label><span>抢断</span><input data-field="tackles" type="number" min="0" step="1" value="0"></label>
              <label><span>拦截</span><input data-field="interceptions" type="number" min="0" step="1" value="0"></label>
              <label><span>解围</span><input data-field="clearances" type="number" min="0" step="1" value="0"></label>
              <label><span>封堵</span><input data-field="blocks" type="number" min="0" step="1" value="0"></label>
              <label><span>对抗成功</span><input data-field="duels_won" type="number" min="0" step="1" value="0"></label>
              <label><span>对抗总数</span><input data-field="duels_total" type="number" min="0" step="1" value="0"></label>
              <label><span>犯规</span><input data-field="fouls" type="number" min="0" step="1" value="0"></label>
              <label><span>黄牌</span><input data-field="yellow_cards" type="number" min="0" step="1" value="0"></label>
              <label><span>红牌</span><input data-field="red_cards" type="number" min="0" step="1" value="0"></label>
              <label><span>致险失误</span><input data-field="errors_leading_to_shot" type="number" min="0" step="1" value="0"></label>
            </div></details>
          </article>`).join("")}
      </div>
    </section>`).join("");
}

function reviewResult(detail: MatchReviewDetail): string {
  const evaluation = detail.summary.prediction_evaluation as Record<string, unknown>;
  const available = evaluation.available === true;
  const pendingCandidates = detail.ability_candidates.filter((candidate) => candidate.status === "pending");
  return `
    <section class="panel review-result-panel">
      <div class="panel-heading"><div><span>复盘结果</span><h2>${escapeHtml(detail.summary.home_team_name)} ${detail.result.home_goals_90}-${detail.result.away_goals_90} ${escapeHtml(detail.summary.away_team_name)}</h2></div><button class="secondary" data-action="show-review-json">查看复盘链路</button></div>
      <div class="review-kpi-grid">
        <div><span>数据覆盖</span><strong>${formatPercent(detail.summary.data_coverage)}</strong></div>
        <div><span>结果概率误差</span><strong>${available && typeof evaluation.log_loss === "number" ? evaluation.log_loss.toFixed(3) : "无预测"}</strong></div>
        <div><span>待审核候选</span><strong>${pendingCandidates.length}</strong></div>
        <div><span>复盘版本</span><strong>${escapeHtml(detail.summary.review_version)}</strong></div>
      </div>
      ${matchEventTimeline(detail)}
      <div class="review-team-cards">${detail.team_reviews.map((team) => `
        <article><header><span>球队复盘</span><h3>${escapeHtml(team.team_name)}</h3></header>
          <dl><div><dt>整体配合度</dt><dd>${num(team.chemistry_score)}</dd></div><div><dt>阵容连续性</dt><dd>${formatPercent(team.lineup_continuity)}</dd></div><div><dt>表现协同性</dt><dd>${formatPercent(team.performance_cohesion)}</dd></div><div><dt>替补强度</dt><dd>${num(team.bench_strength)}</dd></div><div><dt>替补影响</dt><dd>${num(team.substitution_impact)}</dd></div><div><dt>球队兑现率</dt><dd>${formatPercent(team.realization_score)}</dd></div></dl>
        </article>`).join("")}</div>
      <div class="table-wrap"><table><thead><tr><th>球员</th><th>角色</th><th>分钟</th><th>预期</th><th>实际</th><th>兑现率</th><th>可信度</th><th>候选</th></tr></thead><tbody>
        ${detail.player_reviews.map((player) => `<tr><td><strong>${escapeHtml(player.player_name)}</strong><small>${escapeHtml(player.team_name)}</small></td><td>${player.entry_type === "starter" ? "首发" : player.entry_type === "substitute" ? "替补登场" : "未登场"}</td><td>${player.minutes_played ?? 0}</td><td>${num(player.expected_performance)}</td><td>${num(player.actual_performance)}</td><td>${formatPercent(player.realization_ratio)}</td><td>${formatPercent(player.confidence)}</td><td>${player.ability_candidate_count}</td></tr>`).join("")}
      </tbody></table></div>
      ${pendingCandidates.length > 0 ? `<div class="candidate-list"><div class="subheading"><div><span>受控回写</span><h3>球员能力更新候选</h3></div><small>接受后才会写入正式能力历史</small></div>${pendingCandidates.map((candidate) => `<article><div><strong>${escapeHtml(candidate.player_name)} · ${escapeHtml(candidate.dimension_name)}</strong><span>${num(candidate.current_value)} → ${num(candidate.proposed_value)} · 可信度 ${formatPercent(candidate.confidence)}</span></div><div class="button-row"><button class="tiny primary" data-action="decide-ability-candidate" data-candidate-id="${escapeHtml(candidate.id)}" data-decision="accept">接受</button><button class="tiny secondary" data-action="decide-ability-candidate" data-candidate-id="${escapeHtml(candidate.id)}" data-decision="reject">拒绝</button><button class="tiny ghost" data-action="show-candidate-json" data-candidate-id="${escapeHtml(candidate.id)}">证据</button></div></article>`).join("")}</div>` : `<div class="empty-state compact"><strong>没有待审核能力候选</strong><span>数据不足或变化幅度低于保护阈值时不会生成候选。</span></div>`}
    </section>`;
}

function diffNames(items: string[]): string {
  return items.length ? items.map((item) => `<span>${escapeHtml(item)}</span>`).join("") : `<em>无</em>`;
}

function reviewSnapshotCard(label: string, snapshot: MatchReviewPackageSnapshotSummary): string {
  const score = snapshot.home_goals_90 === null || snapshot.away_goals_90 === null
    ? "尚未产生"
    : `${snapshot.home_goals_90}-${snapshot.away_goals_90}`;
  return `<article><span>${escapeHtml(label)}</span><strong>${escapeHtml(score)}</strong><dl><div><dt>主队名单 / 首发</dt><dd>${snapshot.home_player_count} / ${snapshot.home_starter_count}</dd></div><div><dt>客队名单 / 首发</dt><dd>${snapshot.away_player_count} / ${snapshot.away_starter_count}</dd></div></dl></article>`;
}

function identityCheck(label: string, matched: boolean): string {
  return `<div class="${matched ? "matched" : "mismatched"}"><span>${matched ? "✓" : "!"}</span><strong>${escapeHtml(label)}</strong><small>${matched ? "匹配" : "不匹配"}</small></div>`;
}

type WorkflowStepState = "done" | "current" | "locked" | "blocked";

interface WorkflowStepView {
  no: number;
  title: string;
  purpose: string;
  status: string;
  completion: string;
  blocked: string;
  next: string;
  state: WorkflowStepState;
  action?: string;
}


function workflowActionLabel(action: MatchReviewPackageWorkflowAction | null): string {
  if (!action) return "无后续动作";
  const labels: Record<MatchReviewPackageWorkflowAction, string> = {
    export_package: "重新导出资料包",
    preview_import: "导入并预检",
    confirm_import: "人工确认",
    commit_facts: "写入赛后事实",
    generate_review: "生成正式复盘",
    inspect_settlement_readiness: "检查结算门禁",
    settle_review: "正式结算",
    open_analytics: "进入分析与历史",
  };
  return labels[action];
}

function stepState(done: boolean, active: boolean, blocked = false): WorkflowStepState {
  if (done) return "done";
  if (blocked) return "blocked";
  return active ? "current" : "locked";
}

function workflowStepStateLabel(state: WorkflowStepState): string {
  return ({ done: "已完成", current: "当前", locked: "锁定", blocked: "阻断" } as const)[state];
}

function workflowStepButton(step: WorkflowStepView, active: boolean): string {
  const summary = step.blocked || step.next;
  return `<button class="review-stage-button state-${step.state} ${active ? "active" : ""}" data-action="select-review-workflow-step" data-review-step="${step.no}" aria-current="${active ? "step" : "false"}"><b>${String(step.no).padStart(2, "0")}</b><div><strong>${escapeHtml(step.title)}</strong><small>${escapeHtml(summary)}</small></div><em>${escapeHtml(workflowStepStateLabel(step.state))}</em></button>`;
}

function workflowStepWorkspace(step: WorkflowStepView): string {
  const tone = step.state === "done" ? "success" : step.state === "blocked" ? "danger" : step.state === "current" ? "accent" : "neutral";
  return `<section class="review-stage-workspace" data-review-active-step="${step.no}"><header><div><span>第 ${String(step.no).padStart(2, "0")} 步 · 本步用途</span><h2>${escapeHtml(step.title)}</h2><p>${escapeHtml(step.purpose)}</p></div>${taskStatusChip({ label: step.status, tone })}</header><dl class="review-stage-facts"><div><dt>当前状态</dt><dd>${escapeHtml(step.status)}</dd></div><div><dt>完成条件</dt><dd>${escapeHtml(step.completion)}</dd></div><div class="${step.blocked ? "is-blocked" : ""}"><dt>阻塞原因</dt><dd>${escapeHtml(step.blocked || "无")}</dd></div><div class="is-next"><dt>下一步动作</dt><dd>${escapeHtml(step.next)}</dd></div></dl><div class="review-stage-actions">${step.action ?? ""}</div></section>`;
}

function reviewPackageWorkspace(
  selected: ReviewableMatch | null,
  workflow: MatchReviewPackageWorkflowRecord | null,
  preview: MatchReviewPackagePreview | null,
  detail: MatchReviewDetail | null,
  settlement: PostmatchSettlementRecord | null,
  requestedStep: number | null,
): string {
  const matchId = selected?.match_record.id ?? null;
  const activeWorkflow = workflow?.match_id === matchId ? workflow : null;
  const activePreview = preview?.match_id === matchId ? preview : null;
  const reviewId = activeWorkflow?.review_id ?? (detail?.summary.match_id === matchId ? detail.summary.id : null);
  const exported = matchReviewWorkflowCompleted(activeWorkflow, "export_package");
  const externalDataCompleted = matchReviewWorkflowCompleted(activeWorkflow, "complete_external_data");
  const previewValid = matchReviewWorkflowCompleted(activeWorkflow, "preview_import");
  const confirmed = matchReviewWorkflowCompleted(activeWorkflow, "confirm_import");
  const factsCommitted = matchReviewWorkflowCompleted(activeWorkflow, "commit_facts");
  const reviewCreated = matchReviewWorkflowCompleted(activeWorkflow, "generate_review");
  const settled = matchReviewWorkflowCompleted(activeWorkflow, "settle_review") || Boolean(settlement && settlement.match_id === matchId);
  const canExport = Boolean(selected) && (!activeWorkflow || matchReviewWorkflowAllows(activeWorkflow, "export_package"));
  const canPreview = matchReviewWorkflowAllows(activeWorkflow, "preview_import");
  const canConfirm = matchReviewWorkflowAllows(activeWorkflow, "confirm_import");
  const canCommitFacts = matchReviewWorkflowAllows(activeWorkflow, "commit_facts");
  const canGenerateReview = matchReviewWorkflowAllows(activeWorkflow, "generate_review");
  const canInspectSettlement = matchReviewWorkflowAllows(activeWorkflow, "inspect_settlement_readiness");
  const canSettle = matchReviewWorkflowAllows(activeWorkflow, "settle_review");
  const canOpenAnalytics = matchReviewWorkflowAllows(activeWorkflow, "open_analytics") || settled;
  const noMatchReason = selected ? "" : "暂无可复盘比赛或尚未载入比赛";
  const noExportReason = selected && !activeWorkflow ? "尚未导出本轮资料包" : "";
  const previewBlocked = Boolean(activePreview && !activePreview.ready);

  const steps: WorkflowStepView[] = [
    { no: 1, title: "选择比赛", purpose: "确定本次复盘唯一比赛身份，锁定主客队与开球时间。", status: selected ? "已选择" : "等待选择", completion: "比赛出现在可复盘列表中，并已载入当前页面。", blocked: noMatchReason, next: selected ? "导出本轮赛后复盘资料包。" : "在上方选择比赛并点击“载入比赛与阵容”。", state: stepState(Boolean(selected), !selected) },
    { no: 2, title: "导出赛后复盘资料包", purpose: "冻结赛前阵容、球员状态、模型身份、参数版本和预测输出。", status: exported ? "本轮资料包已导出" : "等待导出", completion: "后端登记 package_id、导出路径与原始 SHA256。", blocked: noMatchReason, next: exported ? "在外部补充真实比赛事实。" : "导出 .xlsx 资料包。", state: stepState(exported, Boolean(selected)), action: `<button class="primary" data-action="export-match-review-package" ${canExport ? "" : "disabled"}>${exported ? "重新导出并开始新一轮" : "导出赛后复盘资料包"}</button>` },
    { no: 3, title: "在外部补充真实比赛事实和球员量化数据", purpose: "补充比分、实际名单、换人、进球、助攻、牌、VAR、伤退、阵型变化与球员表现。", status: externalDataCompleted ? "已提交填写后的文件" : exported ? "等待外部补充" : "尚未开始", completion: "使用本轮导出的原文件完成填写，并保留 package_id。", blocked: noMatchReason || noExportReason, next: exported ? "选择填写后的文件并启动导入预检。" : "先完成资料包导出。", state: stepState(externalDataCompleted, exported && !externalDataCompleted, Boolean(noMatchReason || noExportReason)) },
    { no: 4, title: "导入并预检", purpose: "比较赛前值、当前数据库值与准备导入值，核验身份、差异、错误和警告。", status: previewBlocked ? "预检有阻断错误" : previewValid ? "预检通过" : canPreview ? "可以选择文件预检" : "等待资料包", completion: "本轮 package_id 匹配、后端复检通过且严重错误为零。", blocked: previewBlocked ? `${activePreview?.errors.length ?? 0} 条阻断错误需要修正` : matchReviewWorkflowBlocker(activeWorkflow, "preview_import") || noMatchReason || noExportReason, next: previewBlocked ? "修正文件后重新预检。" : previewValid ? "进行人工确认。" : "选择填写后的 .xlsx 文件。", state: stepState(previewValid, canPreview, previewBlocked), action: `<button class="secondary" data-action="preview-match-review-package" ${canPreview ? "" : "disabled"}>选择文件并预检</button>` },
    { no: 5, title: "人工确认", purpose: "确认差异、来源和可信度；后端再次读取文件并校验 SHA256。", status: confirmed ? "已人工确认" : canConfirm ? "等待人工确认" : "尚未解锁", completion: "预检通过，并记录确认人、确认说明和确认时间。", blocked: confirmed ? "" : matchReviewWorkflowBlocker(activeWorkflow, "confirm_import") || noExportReason, next: confirmed ? "写入真实赛后事实。" : "核对预检结果并确认。", state: stepState(confirmed, canConfirm), action: `<button class="primary" data-action="confirm-match-review-package" ${canConfirm ? "" : "disabled"}>人工确认本轮资料包</button>` },
    { no: 6, title: "写入真实赛后事实", purpose: "写入实际阵容、正式赛果、结构化比赛事件与球员观察，不覆盖赛前快照。", status: factsCommitted ? "赛后事实已写入" : canCommitFacts ? "等待写入" : "尚未解锁", completion: "实际阵容、赛果、换人、普通事件和球员观察均成功入库。", blocked: factsCommitted ? "" : matchReviewWorkflowBlocker(activeWorkflow, "commit_facts") || "资料包尚未人工确认", next: factsCommitted ? "生成正式复盘。" : "确认写入真实赛后事实。", state: stepState(factsCommitted, canCommitFacts), action: `<button class="primary" data-action="commit-match-review-package-facts" ${canCommitFacts ? "" : "disabled"}>写入真实赛后事实</button>` },
    { no: 7, title: "生成正式复盘", purpose: "基于已冻结的赛前预测与已写入的真实事实生成球队、球员和预测评价。", status: reviewCreated ? "正式复盘已生成" : canGenerateReview ? "等待生成" : "尚未解锁", completion: "生成 finalized 复盘记录并绑定本轮资料包。", blocked: reviewCreated ? "" : matchReviewWorkflowBlocker(activeWorkflow, "generate_review") || "真实赛后事实尚未写入", next: reviewCreated ? "检查正式结算门禁。" : "生成正式复盘。", state: stepState(reviewCreated, canGenerateReview), action: `<button class="primary" data-action="generate-match-review-from-package" ${canGenerateReview ? "" : "disabled"}>生成正式复盘</button>` },
    { no: 8, title: "正式结算", purpose: "通过模型运行、冻结快照、赛事 Profile 与正式时点门禁，建立不可覆盖的结算样本。", status: settled ? "正式结算已完成" : canSettle ? "等待门禁检查与结算" : "尚未解锁", completion: "正式结算记录创建成功，并建立证据评分队列。", blocked: settled ? "" : matchReviewWorkflowBlocker(activeWorkflow, "settle_review") || (!reviewId ? "复盘记录尚未载入" : ""), next: settled ? "进入分析与历史。" : "检查结算门禁，通过后正式结算。", state: stepState(settled, canInspectSettlement || canSettle), action: `<div class="button-row"><button class="secondary" data-action="inspect-postmatch-readiness" data-review-id="${escapeHtml(reviewId ?? "")}" ${reviewId && canInspectSettlement && !settled ? "" : "disabled"}>检查结算门禁</button><button class="primary" data-action="settle-postmatch-review" data-review-id="${escapeHtml(reviewId ?? "")}" ${reviewId && canSettle && !settled ? "" : "disabled"}>正式结算并建立证据队列</button></div>` },
    { no: 9, title: "进入分析与历史", purpose: "查看结算样本、证据评分、漂移监控、复盘历史和受控能力候选。", status: settled ? "可以进入" : "尚未解锁", completion: "正式结算已完成。", blocked: settled ? "" : matchReviewWorkflowBlocker(activeWorkflow, "open_analytics") || "正式结算尚未完成", next: settled ? "打开分析与历史页面。" : "先完成正式结算。", state: stepState(false, canOpenAnalytics), action: `<button class="primary" data-page="analytics" ${canOpenAnalytics ? "" : "disabled"}>打开分析与历史</button>` },
  ];

  const defaultStep = steps.find((step) => step.state === "blocked")?.no
    ?? steps.find((step) => step.state === "current")?.no
    ?? [...steps].reverse().find((step) => step.state === "done")?.no
    ?? 1;
  const activeStepNumber = requestedStep && steps.some((step) => step.no === requestedStep) ? requestedStep : defaultStep;
  const activeStep = steps.find((step) => step.no === activeStepNumber) ?? steps[0];
  const workflowMeta = activeWorkflow
    ? `<div class="review-workflow-meta"><div><span>当前资料包</span><strong>${escapeHtml(activeWorkflow.package_id)}</strong><small>${escapeHtml(activeWorkflow.status)} · 更新于 ${escapeHtml(new Date(activeWorkflow.updated_at).toLocaleString())}</small></div><div><span>文件与校验</span><strong>${escapeHtml(activeWorkflow.import_path ?? activeWorkflow.export_path)}</strong><small>下一动作：${escapeHtml(workflowActionLabel(activeWorkflow.next_action))} · SHA256 ${escapeHtml(activeWorkflow.import_sha256 ? `${activeWorkflow.import_sha256.slice(0, 20)}…` : "尚未生成")}</small></div></div>`
    : `<div class="empty-inline">当前比赛尚未建立资料包工作流。选择比赛后从第 2 步开始。</div>`;
  const previewMarkup = activePreview ? `<details class="review-preview-shell" ${activeStepNumber === 4 || activeStepNumber === 5 ? "open" : ""}><summary><div><span>资料包预检证据</span><strong>${activePreview.ready ? "预检通过，可以进入人工确认" : `${activePreview.errors.length} 条阻断错误需要处理`}</strong></div><b>${activePreview.ready ? "通过" : "阻断"}</b></summary><section class="review-package-preview ${activePreview.ready ? "ready" : "blocked"}"><header><div><span>${activePreview.ready ? "预检通过" : "预检未通过"}</span><h3>${escapeHtml(activePreview.source_file_name)}</h3><small>资料包 ${escapeHtml(activePreview.package_id)} · SHA256 ${escapeHtml(activePreview.source_sha256.slice(0, 16))}…</small></div><button class="secondary tiny" data-action="show-match-review-package-json">查看完整预检</button></header><div class="review-package-kpis"><div><span>主队名单/首发</span><strong>${activePreview.home_player_count}/${activePreview.home_starter_count}</strong></div><div><span>客队名单/首发</span><strong>${activePreview.away_player_count}/${activePreview.away_starter_count}</strong></div><div><span>结构化事件</span><strong>${activePreview.events.length}</strong></div><div><span>球员观察</span><strong>${activePreview.observation_count}</strong></div></div><section class="review-package-comparison"><header><span>三方值对照</span><strong>赛前快照、当前数据库与准备导入值</strong></header><div>${reviewSnapshotCard("赛前值", activePreview.comparison.pre_match)}${reviewSnapshotCard("当前数据库值", activePreview.comparison.current_database)}${reviewSnapshotCard("准备导入值", activePreview.comparison.proposed_import)}</div></section><section class="review-package-identity"><header><span>身份匹配情况</span><strong>全部匹配后才能人工确认</strong></header><div>${identityCheck("当前导出 package_id", activePreview.comparison.identity.package_id_matches_current_export)}${identityCheck("当前选择比赛 ID", activePreview.comparison.identity.match_id_matches_selection)}${identityCheck("数据库比赛标识", activePreview.comparison.identity.match_key_matches_database)}${identityCheck("主客队身份", activePreview.comparison.identity.team_identity_matches_database)}</div></section><div class="review-package-diff"><article><strong>主队首发新增</strong><div>${diffNames(activePreview.diff.home_added_starters)}</div></article><article><strong>主队首发移出</strong><div>${diffNames(activePreview.diff.home_removed_starters)}</div></article><article><strong>客队首发新增</strong><div>${diffNames(activePreview.diff.away_added_starters)}</div></article><article><strong>客队首发移出</strong><div>${diffNames(activePreview.diff.away_removed_starters)}</div></article><article><strong>新增比赛名单</strong><div>${diffNames(activePreview.diff.added_matchday_players)}</div></article><article><strong>移出比赛名单</strong><div>${diffNames(activePreview.diff.removed_matchday_players)}</div></article></div>${activePreview.errors.length ? `<div class="review-package-messages errors"><strong>阻断错误</strong>${activePreview.errors.map((item) => `<p>${escapeHtml(item)}</p>`).join("")}</div>` : ""}${activePreview.warnings.length ? `<div class="review-package-messages warnings"><strong>需要人工核对</strong>${activePreview.warnings.map((item) => `<p>${escapeHtml(item)}</p>`).join("")}</div>` : ""}${activePreview.ready && canConfirm ? `<div class="review-confirmation-fields"><label class="field"><span>确认人（可选）</span><input id="review-package-confirmed-by" placeholder="例如：本地用户"></label><label class="field"><span>确认说明（建议填写）</span><input id="review-package-confirmation-note" placeholder="说明数据来源、异常或人工修订依据"></label></div>` : ""}</section></details>` : "";

  return `<section class="review-package-workspace"><div class="review-workflow-heading"><div><span>固定链路</span><h2>赛后事实修正、复盘与结算</h2><p>九个步骤始终保留在左侧；右侧只展开当前任务，减少纵向卡片堆叠。</p></div></div>${workflowMeta}<div class="review-command-center"><nav class="review-stage-rail" aria-label="赛后复盘步骤"><div class="review-stage-rail-header"><span>九步工作流</span><strong>选择步骤查看详情</strong><small>业务权限由后端工作流统一返回；前端只负责展示当前步骤。</small></div>${steps.map((step) => workflowStepButton(step, step.no === activeStep.no)).join("")}</nav>${workflowStepWorkspace(activeStep)}</div>${previewMarkup}</section>`;
}

function recentReviewList(reviews: MatchReviewSummary[]): string {
  if (reviews.length === 0) return `<div class="empty-state compact"><strong>暂无复盘记录</strong><span>完成第一场赛后复盘后会显示在这里。</span></div>`;
  return `<div class="review-history">${reviews.slice(0, 20).map((review) => `<button data-action="open-match-review" data-review-id="${escapeHtml(review.id)}"><div><strong>${escapeHtml(review.home_team_name)} vs ${escapeHtml(review.away_team_name)}</strong><span>${escapeHtml(review.review_version)} · ${new Date(review.created_at).toLocaleString()}</span></div><b>${formatPercent(review.data_coverage)}</b></button>`).join("")}</div>`;
}

export function reviewPage(
  state: BootstrapResponse,
  matches: ReviewableMatch[],
  selectedMatchId: string | null,
  lineups: LineupRecord[],
  detail: MatchReviewDetail | null,
  packagePreview: MatchReviewPackagePreview | null,
  packageWorkflow: MatchReviewPackageWorkflowRecord | null,
  settlement: PostmatchSettlementRecord | null,
  recentReviews: MatchReviewSummary[],
  references: PlayerCatalogReferenceData | null,
  playerCandidates: PlayerListItem[],
  lineupBuilderPlayers: LineupBuilderPlayer[],
  lineupBuilderForm: LineupBuilderFormState,
  activeWorkflowStep: number | null,
): string {
  if (!state.data.database_configured) {
    return `<section class="task-empty-workspace">${taskPageHeader({ eyebrow: "赛后复盘", title: "赛后事实、复盘与正式结算", description: "连接成功后九步复盘链路会在当前页面自动恢复，不需要前往其他设置页面。", status: { label: "等待数据库", tone: "warning" } })}${taskContextRibbon([{ label: "当前状态", value: "数据库未连接", note: "连接后可选择比赛并恢复资料包工作流", tone: "warning" }])}${inlineDatabaseSetup("连接数据服务以开始复盘", "连接成功后可直接选择比赛、补录阵容并生成复盘。", state.connection_error)}</section>`;
  }
  const selected = matches.find((item) => item.match_record.id === selectedMatchId) ?? null;
  const pickerMatchId = selectedMatchId ?? matches[0]?.match_record.id ?? null;
  const now = new Date();
  const defaultFinalized = new Date(now.getTime() - now.getTimezoneOffset() * 60_000).toISOString().slice(0, 16);
  const workflowForSelection = packageWorkflow?.match_id === selected?.match_record.id ? packageWorkflow : null;
  const settlementComplete = Boolean(settlement && settlement.match_id === selected?.match_record.id);
  const contextRibbon = taskContextRibbon([
    { label: "当前比赛", value: selected ? `${selected.match_record.home_team_name} vs ${selected.match_record.away_team_name}` : "尚未选择", note: selected ? `${selected.match_record.competition_name ?? "自定义赛事"} · ${new Date(selected.match_record.kickoff_time).toLocaleString()}` : "选择已开始或已结束比赛", tone: selected ? "accent" : "neutral" },
    { label: "资料包状态", value: workflowForSelection?.status ?? "尚未建立", note: workflowForSelection ? `package_id ${workflowForSelection.package_id}` : "导出后建立本轮工作流" },
    { label: "下一动作", value: workflowActionLabel(workflowForSelection?.next_action ?? null), note: workflowForSelection?.blocking_reasons[0]?.reason ?? "按九步链路继续操作", tone: workflowForSelection?.blocking_reasons.length ? "warning" : workflowForSelection ? "success" : "neutral" },
    { label: "正式结算", value: settlementComplete ? "已完成" : "尚未完成", note: settlementComplete ? "可以进入分析与历史" : "完成正式复盘后检查结算门禁", tone: settlementComplete ? "success" : "neutral" },
  ]);
  return `
    ${taskPageHeader({ eyebrow: "赛后复盘", title: "赛后事实、复盘与正式结算", description: "完整链路始终可见；当前步骤集中展开，真实事实、预检证据和结算状态保持同一上下文。", status: { label: settlementComplete ? "本场已正式结算" : selected ? "复盘上下文已载入" : "等待选择比赛", tone: settlementComplete ? "success" : selected ? "accent" : "neutral" }, actions: `<button class="secondary" data-action="refresh-review">刷新</button>` })}
    ${contextRibbon}
    <section class="panel review-workflow review-match-selection">
      <div class="panel-heading"><div><span>第 1 步</span><h2>选择比赛</h2></div></div>
      <div class="review-match-picker"><label class="field"><span>需要复盘的比赛</span><select id="review-match-id" ${matches.length === 0 ? "disabled" : ""}>${matches.length === 0 ? `<option value="">暂无可复盘比赛</option>` : matches.map((item) => `<option value="${escapeHtml(item.match_record.id)}" ${item.match_record.id === pickerMatchId ? "selected" : ""}>${escapeHtml(matchLabel(item))}</option>`).join("")}</select></label><button class="primary" data-action="load-review-match" ${matches.length === 0 ? "disabled" : ""}>载入比赛与阵容</button></div>
      ${matches.length === 0 ? `<div class="empty-state compact"><strong>暂无可复盘比赛</strong><span>比赛需已开赛、已结束或已有正式赛果后才会出现在这里。</span></div>` : ""}
      ${selected ? `<div class="selected-match-summary"><div><span>${escapeHtml(selected.match_record.competition_name ?? "自定义赛事")}</span><strong>${escapeHtml(selected.match_record.home_team_name)} vs ${escapeHtml(selected.match_record.away_team_name)}</strong><small>${new Date(selected.match_record.kickoff_time).toLocaleString()}</small></div><div><span>实际阵容</span><strong>${selected.actual_lineup_count}</strong><small>${selected.latest_review ? `最近复盘 ${escapeHtml(selected.latest_review.review_version)}` : "尚未复盘"}</small></div></div>` : ""}
    </section>
    ${reviewPackageWorkspace(selected, packageWorkflow, packagePreview, detail, settlement, activeWorkflowStep)}
    ${selected ? `<details class="manual-review-fallback"><summary>备用：在客户端手动录入赛果和球员评分</summary><section class="panel review-input-panel"><div class="panel-heading"><div><span>第 2 步</span><h2>正式赛果与球员表现</h2></div><small>评分采用 0–10；详细事件数据按需展开。</small></div>
      <div class="review-result-form">
        <label class="field"><span>${escapeHtml(selected.match_record.home_team_name)} 90 分钟进球</span><input id="review-home-goals" type="number" min="0" step="1" value="${selected.result?.home_goals_90 ?? 0}"></label>
        <label class="field"><span>${escapeHtml(selected.match_record.away_team_name)} 90 分钟进球</span><input id="review-away-goals" type="number" min="0" step="1" value="${selected.result?.away_goals_90 ?? 0}"></label>
        <label class="field"><span>数据覆盖率</span><input id="review-data-coverage" type="number" min="0" max="1" step="0.01" value="1"></label>
        <label class="field"><span>赛果确认时间</span><input id="review-finalized-at" type="datetime-local" value="${defaultFinalized}"></label>
      </div>
      ${selectedLineupCards(lineups)}
      ${lineups.length < 2 ? lineupCompletionWorkspace(
        references,
        playerCandidates,
        lineupBuilderPlayers,
        lineupBuilderForm,
        selected.match_record.id,
        [selected.match_record.home_team_id, selected.match_record.away_team_id],
      ) : ""}
      <details class="advanced-panel"><summary>复盘备注与版本</summary><div class="form-grid two"><label class="field"><span>复盘版本（留空自动生成）</span><input id="review-version" placeholder="例如 data-provider-v1"></label><label class="field"><span>备注</span><input id="review-notes" placeholder="数据来源或特殊情况"></label></div></details>
      <div class="review-submit"><div><strong>系统将自动计算</strong><span>球员实际表现、兑现率、球队配合度、替补能力、预测误差和能力更新候选。</span></div><button class="primary" data-action="generate-match-review" ${lineups.length < 2 ? "disabled" : ""}>生成复盘</button></div>
    </section></details>` : ""}
    ${detail ? reviewResult(detail) : ""}
    <section class="panel"><div class="panel-heading"><div><span>复盘记录</span><h2>最近复盘</h2></div></div>${recentReviewList(recentReviews)}</section>`;
}
