import {escapeHtml,formatPercent} from '../../components/format';
import type {AbilityUpdateCandidateRecord,CompetitionRecord,ParameterTuningCandidateRecord} from '../../types';
// Existing candidate actions remain connected for R14-16; this batch only supplies the shared stage.
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

function snapshotLabel(value: string): string {
  return ({ "T-N": "任意赛前时间", "T-24h": "开球前 24 小时窗口", "T-6h": "开球前 6 小时窗口", "T-1h": "开球前 1 小时窗口", "T-90m": "历史 90 分钟时点" } as Record<string, string>)[value] ?? value;
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

export function tuningPanel(candidates: ParameterTuningCandidateRecord[], competitions: CompetitionRecord[]): string {
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

export function abilityCandidatesPanel(candidates: AbilityUpdateCandidateRecord[]): string {
  if (candidates.length === 0) return `<div class="empty-state compact"><strong>暂无待审核能力候选</strong><span>赛后复盘或智能分析建议会先进入这里，不会直接覆盖球员能力。</span></div>`;
  return `<div class="suggestion-list">${candidates.slice(0, 50).map((item) => `<article class="suggestion-card"><div><span>${escapeHtml(item.dimension_name)} · 可信度 ${formatPercent(item.confidence)}</span><strong>${escapeHtml(item.player_name)}：${item.current_value === null ? "未设置" : item.current_value.toFixed(2)} → ${item.proposed_value.toFixed(2)}</strong><p>样本 ${item.sample_size} · ${escapeHtml(item.calculation_version)}</p></div><div class="button-row compact"><button class="primary tiny" data-action="decide-ability-candidate" data-candidate-id="${escapeHtml(item.id)}" data-decision="accept">写入能力历史</button><button class="secondary tiny" data-action="decide-ability-candidate" data-candidate-id="${escapeHtml(item.id)}" data-decision="reject">拒绝</button><button class="ghost tiny" data-action="show-candidate-json" data-candidate-id="${escapeHtml(item.id)}">证据</button></div></article>`).join("")}</div>`;
}
