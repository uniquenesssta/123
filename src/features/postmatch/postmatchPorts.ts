import type { EvidenceScoringDecisionDraft, EvidenceScoringItemRecord, MatchReviewDetail, MatchReviewPackageWorkflowRecord, PostmatchMonitoringRequest, PostmatchOverview, PostmatchSettlementDraft, PostmatchSettlementReadiness, PostmatchSettlementRecord } from '../../types';
export interface PostmatchApi {
  postmatchSettlementReadiness(id: string): Promise<PostmatchSettlementReadiness>;
  settlePostmatchReview(draft: PostmatchSettlementDraft): Promise<PostmatchSettlementRecord>;
  listPostmatchSettlements(limit: number): Promise<PostmatchSettlementRecord[]>;
  readMatchReview(id: string): Promise<MatchReviewDetail>;
  readMatchReviewPackageWorkflow(matchId: string): Promise<MatchReviewPackageWorkflowRecord | null>;
  listEvidenceScoringItems(status: string | null, limit: number): Promise<EvidenceScoringItemRecord[]>;
  decideEvidenceScoringItem(draft: EvidenceScoringDecisionDraft): Promise<EvidenceScoringItemRecord>;
  refreshPostmatchMonitoring(request: PostmatchMonitoringRequest): Promise<PostmatchOverview>;
  postmatchOverview(limit: number): Promise<PostmatchOverview>;
}
export interface SettlementTarget { reviewId: string; matchId: string; reviewVersion: string; sourceRunId: string | null; finalizedAt: string | null; result: string }
export function settlementTarget(detail: MatchReviewDetail): SettlementTarget {
  const s=detail.summary;
  return {reviewId:s.id,matchId:s.match_id,reviewVersion:s.review_version,sourceRunId:s.source_run_id,finalizedAt:s.finalized_at,result:JSON.stringify(detail.result)};
}
export type PostmatchReceipt =
  | {kind:'settle'; key:string; target:SettlementTarget; startedAt:string}
  | {kind:'evidence'; key:string; itemId:string; settlementId:string; verdict:EvidenceScoringDecisionDraft['verdict']; note:string; startedAt:string}
  | {kind:'monitor'; key:string; request:PostmatchMonitoringRequest; startedAt:string};
export interface RecoveryScope { read(key:string):PostmatchReceipt|null; save(receipt:PostmatchReceipt):void; clear(key:string):void }
export interface PostmatchRecovery { capture():RecoveryScope }
export interface PostmatchPorts {
  api:PostmatchApi; recovery:PostmatchRecovery; scope():()=>boolean; view():()=>boolean;
  changed():void; formatError(error:unknown):string;
}
/** Explicit dependencies shared by the three independent postmatch workflows. */
export interface PostmatchRuntime extends PostmatchPorts {
  enter():void; current():()=>boolean; interactionChanged():void;
  overview():PostmatchOverview|null; refresh():Promise<boolean>;
}
