import type {ReleaseAcceptanceRequest} from '../../types';
import type {ReleaseDraft} from './releasePorts';
export function releaseRequest(draft:ReleaseDraft):ReleaseAcceptanceRequest {
 const window=(raw:string,label:string)=>{const value=Number(raw);if(!raw.trim()||!Number.isInteger(value)||value<1||value>365)throw Error(label+'必须为1至365的整数');return value;};
 const budget=(raw:string,label:string)=>{if(!raw.trim())return null;const value=Number(raw);if(!Number.isFinite(value)||value<0)throw Error(label+'必须为非负有限数值');return value;};
 return {performance_window_days:window(draft.performance_window_days,'性能窗口'),cost_window_days:window(draft.cost_window_days,'成本窗口'),daily_cost_budget_usd:budget(draft.daily_cost_budget_usd,'单日预算'),monthly_cost_budget_usd:budget(draft.monthly_cost_budget_usd,'周期预算'),requested_by:draft.requested_by.trim()||null};
}
export function releaseDraftError(draft:ReleaseDraft):string|null {try{releaseRequest(draft);return null;}catch(error){return (error as Error).message;}}
