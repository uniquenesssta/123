import type {PostmatchFeature} from '../postmatch/index';
import type { ReviewFeature } from './reviewFeature';
import type { ReviewFacts } from './reviewFacts';
import { reviewSteps, defaultReviewStep, workflowStepStateLabel } from './reviewSteps';
import { html, reviewPanel, reviewButton, reviewText, reviewAlert } from './reviewViewParts';
import { badgeView } from '../../ui/primitives/badge';
import { iconSlot } from '../../ui/icons/iconSlot';
import { matchReviewWorkflowAllows } from '../../app/matchReviewWorkflow';
function stepAction(feature: ReviewFeature, no: number, postmatch?:PostmatchFeature): string {
    const map = { 2: ['导出赛后复盘资料包', 'export-match-review-package', 'export'], 4: ['选择文件并预检', 'preview-match-review-package', 'preview'], 5: ['人工确认本轮资料包', 'confirm-match-review-package', 'confirm'], 6: ['写入真实赛后事实', 'commit-match-review-package-facts', 'facts'], 7: ['生成正式复盘', 'generate-match-review-from-package', 'generate'] } as const;
    if (no in map) {
        const [label, action, kind] = map[no as keyof typeof map];
        return reviewButton(no === 2 && feature.state.workflow ? '重新导出并开始新一轮' : label, action, !feature.allows(kind), no !== 4);
    }
    const id = feature.state.detail?.summary.id ?? feature.state.workflow?.review_id ?? '';
    if (no === 8)
        return reviewButton('检查结算门禁', 'inspect-postmatch-readiness', !id || feature.busy || !matchReviewWorkflowAllows(feature.state.workflow, 'inspect_settlement_readiness'), false, { 'data-review-id': id }) + reviewButton('正式结算并建立证据队列', 'settle-postmatch-review', !id || feature.busy || !postmatch?.settlement.canSubmit(), true, { 'data-review-id': id });
    if (no === 9)
        return reviewButton('打开分析与历史', 'review-open-analytics', !matchReviewWorkflowAllows(feature.state.workflow, 'open_analytics'), true);
    return '';
}
export function reviewRail(feature: ReviewFeature, requested: number | null): string {
    const active = requested ?? defaultReviewStep(feature);
    return `<nav class="review-stage-rail" aria-label="赛后复盘步骤">${reviewSteps(feature).map(step => `<button type="button" class="review-stage-button state-${step.state}" data-action="select-review-workflow-step" data-review-step="${step.no}" aria-current="${active === step.no ? 'step' : 'false'}"><span class="review-step-marker">${step.state === 'done' ? iconSlot('check', 16) : String(step.no).padStart(2, '0')}</span><span class="review-step-copy"><strong>${html(step.title)}</strong><small>${html(step.blocked || step.next)}</small></span>${badgeView(workflowStepStateLabel(step.state), step.state === 'blocked' ? 'Warning' : step.state === 'done' ? 'Success' : 'Neutral', 'Outline')}</button>`).join('')}</nav>`;
}
function previewPanel(feature: ReviewFeature, facts: ReviewFacts): string {
    const preview = feature.preview;
    if (!preview)
        return '';
    const comparison = preview.comparison, snapshots = [comparison.pre_match, comparison.current_database, comparison.proposed_import];
    const rows: [
        string,
        (s: typeof snapshots[number]) => string
    ][] = [['90分钟比分', s => s.home_goals_90 === null || s.away_goals_90 === null ? '未记录' : `${s.home_goals_90}-${s.away_goals_90}`], ['主队名单 / 首发', s => `${s.home_player_count} / ${s.home_starter_count}`], ['客队名单 / 首发', s => `${s.away_player_count} / ${s.away_starter_count}`]];
    const identityLabels: Record<string, string> = { package_id_matches_current_export: '当前导出资料包', match_id_matches_selection: '当前选择比赛', match_key_matches_database: '数据库比赛标识', team_identity_matches_database: '主客队身份' };
    const diffLabels: Record<string, string> = { home_added_starters: '主队首发新增', home_removed_starters: '主队首发移出', away_added_starters: '客队首发新增', away_removed_starters: '客队首发移出', added_matchday_players: '新增比赛名单', removed_matchday_players: '移出比赛名单' };
    const saved = feature.selected ? facts.values(feature.selected, feature.state.lineups, feature.state.detail).fields : null;
    return reviewPanel('资料包预检证据', `${preview.source_file_name} · ${preview.observation_count} 项球员观察 · ${preview.events.length} 项结构化事件`, `${reviewAlert(preview.ready ? '预检通过，请核对差异' : '预检未通过', preview.ready ? '人工确认只冻结本次导入；写入事实需单独执行。' : preview.errors.join('；'), preview.ready ? 'info' : 'error')}<p>身份匹配情况</p><dl class="review-package-identity"><div><dt>资料包</dt><dd>${html(preview.package_id)}</dd></div><div><dt>原文件SHA256</dt><dd>${html(preview.source_sha256)}</dd></div>${Object.entries(comparison.identity).map(([key, value]) => `<div><dt>${identityLabels[key] ?? html(key)}</dt><dd>${badgeView(value ? '匹配' : '不匹配', value ? 'Success' : 'Danger', 'Outline')}</dd></div>`).join('')}</dl><div class="review-table-scroll"><table><thead><tr><th>字段</th><th>赛前值</th><th>当前数据库值</th><th>准备导入值</th></tr></thead><tbody>${rows.map(([name, value]) => `<tr><th>${name}</th>${snapshots.map(snapshot => `<td>${html(value(snapshot))}</td>`).join('')}</tr>`).join('')}</tbody></table></div><dl class="review-diff-grid">${Object.entries(preview.diff).map(([key, values]) => `<div><dt>${diffLabels[key] ?? html(key)}</dt><dd>${values.length ? values.map(html).join('、') : '无'}</dd></div>`).join('')}</dl>${preview.warnings.length ? reviewAlert('需要人工核对', preview.warnings.join('；')) : ''}${preview.ready && saved ? `<fieldset data-workspace-persist="false" ${feature.busy ? 'disabled' : ''}><div class="review-confirmation-fields">${reviewText('确认人（可选）', 'confirmedBy', saved.confirmedBy)}${reviewText('确认说明与来源', 'confirmationNote', saved.confirmationNote)}</div></fieldset>` : ''}<div class="button-row">${reviewButton('查看完整预检', 'show-match-review-package-json')}${stepAction(feature, 5)}</div>`, 'review-preview-panel');
}
export function reviewWorkflowView(feature: ReviewFeature, facts: ReviewFacts, requested: number | null, postmatch:PostmatchFeature): string {
    const steps = reviewSteps(feature), active = steps.find(step => step.no === (requested ?? defaultReviewStep(feature))) ?? steps[0], workflow = feature.state.workflow;
    const meta = workflow ? `<dl class="review-workflow-meta"><div><dt>资料包</dt><dd>${html(workflow.package_id)}</dd></div><div><dt>文件</dt><dd>${html(workflow.import_path ?? workflow.export_path)}</dd></div><div><dt>SHA256</dt><dd>${html(workflow.import_sha256 ?? workflow.export_sha256 ?? '未生成')}</dd></div></dl>` : '';
    return reviewPanel(`第 ${String(active.no).padStart(2, '0')} 步 · ${active.title}`, `本步用途：${active.purpose}`, `<dl class="review-stage-facts"><div><dt>当前状态</dt><dd>${html(active.status)}</dd></div><div><dt>完成条件</dt><dd>${html(active.completion)}</dd></div><div><dt>阻塞原因</dt><dd>${html(active.blocked || '无')}</dd></div><div><dt>下一步动作</dt><dd>${html(active.next)}</dd></div></dl><div class="button-row">${stepAction(feature, active.no,postmatch)}</div>`, `review-task-panel`) + meta + ((active.no === 4 || active.no === 5) ? previewPanel(feature, facts) : '');
}
