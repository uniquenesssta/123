import type {PostmatchSettlementRecord} from '../../types';
import type {SettlementTarget} from './postmatchPorts';
import type {PostmatchFeature} from './postmatchFeature';
import {html,button,message,panel} from './postmatchViewParts';
import {badgeView} from '../../ui/primitives/badge';
function settlementIdentity(row:PostmatchSettlementRecord):string{return `${row.competition_name} · ${row.horizon} · Profile ${row.competition_profile_id} · 模型 ${row.model_version} · 参数 ${row.parameter_version}`;}
export function postmatchSettlementView(feature:PostmatchFeature,target:SettlementTarget|null):string {
  if(!target)return '';
  const state=feature.settlement.state;if(state.target?.reviewId!==target.reviewId||state.target.matchId!==target.matchId)return panel('独立结算门禁','复盘生成不代表正式结算。','<p>尚未检查本场正式运行、冻结快照、赛事 Profile 与时点。</p>','settlement');
  const gate=state.readiness,checks=gate?[
    ['正式赛果',gate.result_ready],['最终复盘',gate.finalized_review_ready],['成功正式运行',gate.successful_run_ready],['冻结快照',gate.frozen_snapshot_ready],['快照身份',gate.snapshot_identity_ready],['真实证据快照',gate.real_evidence_snapshot_ready],['赛事 Profile',gate.competition_profile_ready],['正式时点',gate.formal_horizon_ready],
  ] as const:[];
  const feedback=(state.loading?message('正在核验原结算','核验完成前不开放写入。','info'):'')+(state.error?message('结算尚未完成核验',state.error,'danger'):'')+(state.pending?message('原结算请求结果待核实','只读查询原复盘和账本，不自动重发。'):'');
  const record=state.record?message(state.workflow&&state.workflow.status!=='settled'?'账本已记录，工作流登记待核验':'正式结算已核实',`${state.record.id} · ${settlementIdentity(state.record)}。证据评分与能力候选继续分别核对。`,state.pending?'warning':'success')+`<dl class="postmatch-identity"><div><dt>结算键</dt><dd>${html(state.record.settlement_key)}</dd></div><div><dt>正式运行 / 冻结快照</dt><dd>${html(state.record.model_run_id)} / ${html(state.record.feature_snapshot_id)}</dd></div><div><dt>赛果指纹</dt><dd>${html(state.record.result_fingerprint)}</dd></div></dl>`:'';
  return `<div class="postmatch-workbench" data-workspace-persist="false">${feedback}${panel('独立结算门禁','所有结果读取后端；当前复盘、运行和快照身份必须一致。',`${checks.length?`<div class="postmatch-table"><table><thead><tr><th>检查项</th><th>当前状态</th></tr></thead><tbody>${checks.map(([label,ok])=>`<tr><th>${label}</th><td>${badgeView(ok?'已核实':'未满足',ok?'Success':'Warning','Outline')}</td></tr>`).join('')}</tbody></table></div>`:'<p>尚未获得本场门禁结果。</p>'}${gate?.blocked_reasons.length?message('后端阻断原因',gate.blocked_reasons.join('；')):''}${record}<div class="postmatch-actions">${state.pending?button('核对原结算','recover-postmatch-settlement',state.busy):''}${state.pending&&state.record&&state.workflow?.status!=='settled'?button('补全同一结算登记','resume-postmatch-settlement',state.busy,false,{'data-review-id':target.reviewId}):''}</div>`,'settlement')}</div>`;
}
