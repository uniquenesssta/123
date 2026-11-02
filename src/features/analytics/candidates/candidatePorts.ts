import type {AbilityUpdateCandidateRecord,AbilityCandidateDecisionDraft,ParameterTuningCandidateRecord,ParameterTuningDraft,ParameterTuningDecisionDraft,ParameterLifecycleReadiness,ParameterLifecycleReadinessRequest,ParameterShadowValidationRecord,ParameterShadowValidationRequest,ParameterPromotionRequest,ParameterRollbackRequest,ParameterPromotionDecisionRecord} from '../../../types';
export interface CandidateApi {
 decideAbilityCandidate(draft:AbilityCandidateDecisionDraft):Promise<AbilityUpdateCandidateRecord>;
 generateParameterTuningCandidate(draft:ParameterTuningDraft):Promise<ParameterTuningCandidateRecord>;
 decideParameterTuningCandidate(draft:ParameterTuningDecisionDraft):Promise<ParameterTuningCandidateRecord>;
 parameterLifecycleReadiness(request:ParameterLifecycleReadinessRequest):Promise<ParameterLifecycleReadiness>;
 runParameterShadowValidation(request:ParameterShadowValidationRequest):Promise<ParameterShadowValidationRecord>;
 listParameterShadowValidations(id:string):Promise<ParameterShadowValidationRecord[]>;
 promoteParameterCandidate(request:ParameterPromotionRequest):Promise<ParameterPromotionDecisionRecord>;
 rollbackParameterCandidate(request:ParameterRollbackRequest):Promise<ParameterPromotionDecisionRecord>;
 listParameterPromotionDecisions(id:string):Promise<ParameterPromotionDecisionRecord[]>;
}
export interface CandidateFields {competition_id:string;snapshot_type:ParameterTuningDraft['snapshot_type'];target_module:ParameterTuningDraft['target_module'];minimum_sample_size:string;max_relative_change:string}
export interface CandidateDraftState {fields:CandidateFields;notes:Record<string,string>}
export type CandidateOperation='ability-accept'|'ability-reject'|'accept_for_backtest'|'reject'|'shadow'|'promote'|'rollback';
export interface CandidateReceipt {kind:'candidate';key:string;id:string;operation:CandidateOperation;source:Record<string,unknown>;beforeIds:string[];note:string;startedAt:string}
export interface CandidateIntent<Row> {id:string;row:Row;operation:CandidateOperation;note:string;current:()=>boolean}
export const initialCandidateDraft=():CandidateDraftState=>({fields:{competition_id:'',snapshot_type:'T-1h',target_module:'lineup_realization',minimum_sample_size:'100',max_relative_change:'0.05'},notes:{}});
