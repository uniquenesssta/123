import type {ReleaseAcceptanceRun,ReleaseAcceptanceRequest} from '../../types';
/** Validate report identity and transport shape; the backend exclusively decides business gates. */
export function assertReleaseReport(run:ReleaseAcceptanceRun,id=run?.id):void {
 if(!run||!id||run.id!==id||!['pass','warning','blocked'].includes(run.overall_status)||!run.report_sha256||!Array.isArray(run.checks)||!Array.isArray(run.category_summaries)||!run.cost||!run.performance)throw Error('验收报告身份或结构尚未核实');
 if(run.checks.some(c=>c.run_id!==id)||new Set(run.checks.map(c=>c.id)).size!==run.checks.length)throw Error('检查证据不属于所选报告');
 if(![run.passed_count,run.warning_count,run.blocked_count].every(n=>Number.isSafeInteger(n)&&n>=0)||run.passed_count+run.warning_count+run.blocked_count!==run.checks.length)throw Error('验收报告计数与检查记录不一致');
}
export function assertReleaseRequest(run:ReleaseAcceptanceRun,request:ReleaseAcceptanceRequest):void {
 assertReleaseReport(run);
 if(run.cost.window_days!==request.cost_window_days||run.cost.daily_budget_usd!==request.daily_cost_budget_usd||run.cost.monthly_budget_usd!==request.monthly_cost_budget_usd||run.requested_by!==request.requested_by)throw Error('验收回执与原请求不一致，保留待核实状态');
}
/** The ledger stores chrono timestamps as PostgreSQL microseconds; preserve every stored digit. */
function ledgerTimestamp(value:string):string {const match=/^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})(?:\.(\d{1,9}))?Z$/.exec(value);return match?match[1]+'.'+(match[2]??'').padEnd(6,'0').slice(0,6)+'Z':value;}
function canonical(value:unknown):unknown {if(Array.isArray(value))return value.map(canonical);if(value&&typeof value==='object')return Object.fromEntries(Object.entries(value).sort(([a],[b])=>a.localeCompare(b)).map(([key,item])=>[key,canonical(item)]));return value;}
function ledgerIdentity(run:ReleaseAcceptanceRun):string {return JSON.stringify(canonical({...run,started_at:ledgerTimestamp(run.started_at),completed_at:ledgerTimestamp(run.completed_at)}));}
export function sameReleaseReport(a:ReleaseAcceptanceRun,b:ReleaseAcceptanceRun):boolean {return a.id===b.id&&a.report_sha256===b.report_sha256&&ledgerIdentity(a)===ledgerIdentity(b);}
