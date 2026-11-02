import {postmatchSettlementView,settlementTarget,type PostmatchFeature} from '../postmatch/index';
import {defaultReviewStep} from './reviewSteps';
import type { BootstrapResponse } from '../../types';
import { inlineDatabaseSetup } from '../../components/databaseSetup';
import type { ReviewFeature } from './reviewFeature';
import type { ReviewFacts } from './reviewFacts';
import { reviewManualView, reviewManualFooter, type ReviewLineupSupport } from './reviewManualView';
import { reviewWorkflowView, reviewRail } from './reviewWorkflowView';
import { reviewResultView } from './reviewResultView';
import { selectView } from '../../ui/primitives/select';
import { html, reviewPanel, reviewButton, reviewAlert, reviewPercent } from './reviewViewParts';
export function reviewPage(bootstrap: BootstrapResponse, feature: ReviewFeature, facts: ReviewFacts, section: string | null, support: ReviewLineupSupport, postmatch:PostmatchFeature): string {
    if (!bootstrap.data.database_configured)
        return `<div class="review-page">${inlineDatabaseSetup('连接数据服务以开始复盘', '连接后选择比赛，恢复对应资料包工作流。', bootstrap.connection_error)}</div>`;
    const state = feature.state, selected = feature.selected, view = ['manual', 'results', 'history'].includes(section ?? '') ? section! : 'workflow';
    const step = section?.startsWith('step-') ? Number(section.slice(5)) : null;
    const header = `<header class="review-page-heading"><h1>赛后事实、复盘与正式结算</h1><p>完整链路始终可见；当前步骤集中展开，真实事实、预检证据和结算状态保持同一上下文。</p></header><p class="review-context">当前比赛：${selected ? html(selected.match_record.home_team_name + ' vs ' + selected.match_record.away_team_name) : state.detail ? html(state.detail.summary.home_team_name + ' vs ' + state.detail.summary.away_team_name) : '尚未载入'} · 资料包：${html(state.workflow?.status ?? '尚未建立')} · 正式结算：${feature.settlement ? '已完成' : '尚未完成'}</p>`;
    const picker = `<div class="review-match-picker" data-workspace-persist="false"><label class="review-field" data-review-picker><span>需要复盘的比赛</span>${selectView({ id: 'review-match-id', label: '需要复盘的比赛', attributes: { 'data-native-select': 'true' }, value: state.choiceId ?? '', placeholder: state.matches.length ? '请选择比赛' : '暂无可复盘比赛', disabled: state.listLoading || !state.matches.length, options: state.matches.map(row => ({ value: row.match_record.id, label: row.match_record.home_team_name + ' vs ' + row.match_record.away_team_name + (row.result ? ` · ${row.result.home_goals_90}-${row.result.away_goals_90}` : '') })) })}</label>${reviewButton('载入比赛与阵容', 'load-review-match', !state.choiceId || state.loading)}</div>`;
    const views = `<nav class="review-view-nav" aria-label="复盘工作区">${[['workflow', '九步复盘'], ['manual', '备用手动录入'], ['results', '复盘结果'], ['history', '最近复盘']].map(([key, label]) => reviewButton(label, 'review-view', false, view === key, { 'data-review-view': key, 'aria-pressed': String(view === key) })).join('')}${reviewButton(state.listLoading ? '刷新中…' : '刷新', 'refresh-review', state.listLoading)}</nav>`;
    const feedback = (state.loading ? reviewAlert('正在载入本场复盘', '本场资料尚未返回，暂不开放写入。', 'info') : '') + (state.error ? reviewAlert('操作未完成', state.error, 'error') : '') + (state.notice ? reviewAlert('本步骤已核实', state.notice, 'info') : '') + (feature.pending ? reviewAlert('原步骤结果待核实', `比赛 ${feature.pending.matchId} · 资料包 ${feature.pending.packageId ?? '等待导出回执'} · ${feature.pending.kind}。刷新只读查回，禁止自动重发。`) : '');
    const recoveryAction = feature.pending && ['export', 'preview'].includes(feature.pending.kind) && state.workflow ? reviewButton('核对当前登记文件', 'review-adopt-file', feature.busy) : '';
    const history = () => reviewPanel('最近复盘', state.reviews.length ? `已载入 ${state.reviews.length} 项复盘记录。` : '完成第一场赛后复盘后会显示在这里。', `<div class="review-history">${state.reviews.map(row => reviewButton(`${row.home_team_name} vs ${row.away_team_name} · ${row.review_version} · 数据覆盖 ${reviewPercent(row.data_coverage)}`, 'open-match-review', false, false, { 'data-review-id': row.id })).join('')}</div>`);
    const content = view === 'manual' ? reviewManualView(feature, facts, support) : view === 'results' ? reviewResultView(feature) : view === 'history' ? history() : reviewWorkflowView(feature, facts, step,postmatch);
    return `<div class="review-page" data-review-match-id="${html(state.selectedMatchId ?? '')}" data-review-view="${view}">${header}${picker}${views}<div class="review-workarea">${view === 'history' ? '' : reviewRail(feature, step)}<div class="review-workspace-viewport" data-scroll-key="review-${view}" tabindex="-1">${feedback}${recoveryAction}${content}${view==='workflow'&&[8,9].includes(step??defaultReviewStep(feature))?postmatchSettlementView(postmatch,state.detail?settlementTarget(state.detail):null):''}</div></div>${view === 'manual' ? reviewManualFooter(feature, facts) : ''}</div>`;
}
