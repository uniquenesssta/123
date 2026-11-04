import type {ReleaseAcceptanceRequest,ReleaseAcceptanceRun,ReleaseAcceptanceRunSummary} from '../../types';
export type ReleaseSection='overview'|'chain'|'performance'|'security'|'cost'|'history';
export interface ReleaseDraft {performance_window_days:string;cost_window_days:string;daily_cost_budget_usd:string;monthly_cost_budget_usd:string;requested_by:string}
export const initialReleaseDraft=():ReleaseDraft=>({performance_window_days:'30',cost_window_days:'30',daily_cost_budget_usd:'',monthly_cost_budget_usd:'',requested_by:''});
export interface ReleaseReceipt {request:ReleaseAcceptanceRequest;beforeIds:string[];startedAt:string;report?:{id:string;hash:string}}
export interface ReleaseStorageScope {draft():ReleaseDraft;saveDraft(draft:ReleaseDraft):void;pending():ReleaseReceipt|null;savePending(receipt:ReleaseReceipt):void;clearPending():void}
export interface ReleaseStorage {capture():ReleaseStorageScope}
export interface ReleasePorts {
 api:{listReleaseAcceptanceRuns(limit:number):Promise<ReleaseAcceptanceRunSummary[]>;readReleaseAcceptanceRun(id:string):Promise<ReleaseAcceptanceRun>;runReleaseAcceptance(request:ReleaseAcceptanceRequest):Promise<ReleaseAcceptanceRun>};
 storage:ReleaseStorage;scope():()=>boolean;view():()=>boolean;changed():void;formatError(error:unknown):string;
}
export interface ReleaseIntent {request:ReleaseAcceptanceRequest;current():boolean}
