import type {AnalyticsOverview,BackgroundJob} from '../../types';
import type {AnalysisRequest} from './analyticsPorts';
export const identity=(value:unknown):string=>JSON.stringify(value,(_key,item)=>item&&typeof item==="object"&&!Array.isArray(item)?Object.fromEntries(Object.entries(item).sort(([a],[b])=>a.localeCompare(b))):item);
export function analysisRequest(payload:Record<string,unknown>={}):AnalysisRequest {
 const request={competition_id:payload.competition_id??null,window_start:payload.window_start??null,window_end:payload.window_end??null,bucket_count:payload.bucket_count??10,baseline_size:payload.baseline_size??100,current_size:payload.current_size??50};
 if(request.competition_id!==null&&(typeof request.competition_id!=='string'||!request.competition_id))throw Error('分析赛事身份无效');
 for(const field of ['window_start','window_end'] as const)if(request[field]!==null&&(typeof request[field]!=='string'||!Number.isFinite(Date.parse(request[field] as string))))throw Error('分析时间范围无效');
 if(request.window_start&&request.window_end&&Date.parse(String(request.window_start))>Date.parse(String(request.window_end)))throw Error('分析时间范围倒置');
 if(![request.bucket_count,request.baseline_size,request.current_size].every(n=>typeof n==='number'&&Number.isSafeInteger(n)&&n>0))throw Error('分析窗口参数无效');
 return request as AnalysisRequest;
}
/** Compare RFC3339 instants without discarding sub-millisecond scope precision. */
export function timestampIdentity(value:string):bigint {
 const parts=value.match(/^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d)(?:\.(\d{1,9}))?(Z|[+-]\d\d:\d\d)$/);
 if(!parts)throw Error('时间身份不是可核实的RFC3339格式');
 const seconds=Date.parse(parts[1]+parts[3]);if(!Number.isFinite(seconds))throw Error('时间身份无效');
 return BigInt(seconds)*1000000n+BigInt((parts[2]??'').padEnd(9,'0'));
}
export const requestIdentity=(payload:Record<string,unknown>)=>{const request=analysisRequest(payload);return identity({...request,window_start:request.window_start?timestampIdentity(request.window_start).toString():null,window_end:request.window_end?timestampIdentity(request.window_end).toString():null});};
export function validateOverview(value:AnalyticsOverview):void {
 if(!value||!Number.isSafeInteger(value.sample_size)||value.sample_size<0||![value.comparisons,value.calibration,value.drift,value.data_quality?.findings,value.query_performance?.tables].every(Array.isArray))throw Error('分析读回不完整，不能补成空结果');
}
export function validateJob(job:BackgroundJob):void {
 if(!job?.id||!['queued','running','succeeded','failed','cancelled'].includes(job.status)||!job.payload||!Number.isFinite(job.progress)||job.progress<0||job.progress>100)throw Error('任务回执不完整');
}
export const jobIdentity=(job:BackgroundJob)=>identity([job.id,job.job_type,job.payload,job.status,job.attempts,job.max_attempts,job.cancellation_requested,job.updated_at]);
export function matchingJobOverview(latest:AnalyticsOverview|null,job:BackgroundJob|null):AnalyticsOverview|null {
 if(!job)return latest;
 if(!['refresh_analytics','full_analysis_refresh'].includes(job.job_type))return latest?.source_snapshot?.request&&!latest.source_snapshot.competition_id?latest:null;
 if(job.status!=='succeeded'||!job.result?.source_snapshot)return null;
 const source=job.result.source_snapshot as AnalyticsOverview['source_snapshot'];
 if(!source?.id||!source.request||requestIdentity(source.request)!==requestIdentity(job.payload))return null;
 if(latest?.source_snapshot?.id===source.id&&latest.source_snapshot.request&&requestIdentity(latest.source_snapshot.request)===requestIdentity(job.payload))return latest;
 if(Array.isArray(job.result.comparisons)){
  const result=job.result as unknown as AnalyticsOverview;validateOverview(result);return result;
 }
 return latest?.source_snapshot?.id===source.id&&latest.source_snapshot.request&&requestIdentity(latest.source_snapshot.request)===requestIdentity(job.payload)?latest:null;
}
export function globalSnapshot(overview:AnalyticsOverview|null):boolean {
 const source=overview?.source_snapshot,request=source?.request;
 return !!source?.id&&!!request&&source.competition_id===null&&request.competition_id===null&&request.window_start===null&&request.window_end===null;
}
