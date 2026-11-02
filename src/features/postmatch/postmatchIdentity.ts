import type {EvidenceScoringItemRecord,PostmatchOverview,PostmatchSettlementRecord} from '../../types';
import type {SettlementTarget} from './postmatchPorts';
export const identity=(value:unknown)=>JSON.stringify(value);
export function validateSettlement(row:PostmatchSettlementRecord,target:SettlementTarget):void {
  if(!row.id||row.match_review_id!==target.reviewId||row.match_id!==target.matchId||!row.settlement_key||!row.result_fingerprint||row.model_run_id!==target.sourceRunId)throw Error('结算记录与原复盘、比赛或正式运行不一致');
}
export function evidenceIdentity(item:EvidenceScoringItemRecord):string {
  return identity([item.id,item.settlement_id,item.evidence_id,item.provider_id,item.field_key,item.retrieved_at,item.data_cutoff_at,item.status,item.decision_id,item.verdict,item.decision_note]);
}
export function validateOverview(value:PostmatchOverview):void {
  if(!value||![value.settlements,value.evidence_queue,value.provider_scores,value.drift_runs].every(Array.isArray)||![value.settlement_count,value.pending_evidence_count,value.scored_evidence_count].every(n=>Number.isSafeInteger(n)&&n>=0))throw Error('赛后总览响应不完整，不能视为零记录');
}
