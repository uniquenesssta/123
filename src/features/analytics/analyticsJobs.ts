import type {BackgroundJob,EnqueueJobDraft} from '../../types';
import type {AnalyticsRuntime} from './analyticsPorts';
import {identity,jobIdentity,requestIdentity,validateJob} from './analyticsIdentity';
export interface AnalysisJobIntent {kind:'enqueue'|'cancel'|'retry';key:string;draft?:EnqueueJobDraft;job?:BackgroundJob;current:()=>boolean}
export const analysisJobTypes=['refresh_analytics','data_quality_scan','query_performance_scan','full_analysis_refresh'];
export class AnalyticsJobs {
 private feedback:string|null=null;private locks=new Map<symbol,{key:string;scope:()=>boolean}>();private issued=new WeakSet<AnalysisJobIntent>();private consumed=new WeakSet<AnalysisJobIntent>();
 constructor(private readonly runtime:AnalyticsRuntime){}
 reset(){this.feedback=null;}
 get state(){this.runtime.enter();return {error:this.feedback,busy:[...this.locks.values()].some(l=>l.scope()),enqueuePending:this.runtime.storage.capture().read('analysis:enqueue')};}
 pending(id:string){return this.runtime.storage.capture().read('analysis:job:'+id);}
 prepareEnqueue(type:EnqueueJobDraft['job_type']):AnalysisJobIntent {
  this.runtime.enter();if(!analysisJobTypes.includes(type))throw Error('不支持的分析任务');
  if(type!=='query_performance_scan'&&!this.runtime.historyReady())throw Error('请先在赛后复盘中完成至少一场正式结算');
  if(type==='data_quality_scan'&&!this.runtime.analysisReady())throw Error('请先完成包含有效样本的完整分析');
  if(this.state.busy||this.state.enqueuePending)throw Error('原入队请求进行中或待核实，不能重新创建任务');
  const payload=['refresh_analytics','full_analysis_refresh'].includes(type)?{...this.runtime.request()}:{},scope=requestIdentity(payload);
  if(this.runtime.jobs().some(j=>j.job_type===type&&['queued','running'].includes(j.status)&&requestIdentity(j.payload)===scope))throw Error('同范围已有排队或运行任务，请查看原任务');
  const draft:EnqueueJobDraft={job_type:type,payload,idempotency_key:'analysis-'+crypto.randomUUID(),priority:type==='full_analysis_refresh'?10:0,max_attempts:3};
  const ticket:AnalysisJobIntent={kind:'enqueue',key:'analysis:enqueue',draft,current:this.runtime.current()};this.issued.add(ticket);return ticket;
 }
 prepareJob(id:string,kind:'cancel'|'retry'):AnalysisJobIntent {
  const job=this.runtime.jobs().find(j=>j.id===id);if(!job||!analysisJobTypes.includes(job.job_type))throw Error('分析任务不在当前目录');
  if(this.state.busy||this.pending(id))throw Error('原任务操作仍待核实');
  if(kind==='cancel'&&(!['queued','running'].includes(job.status)||job.cancellation_requested))throw Error('该任务不能重复请求取消');
  if(kind==='retry'&&(!['failed','cancelled'].includes(job.status)||job.attempts>=job.max_attempts))throw Error('任务不可重试，可能仍在运行或已达到最大尝试次数');
  const ticket:AnalysisJobIntent={kind,key:'analysis:job:'+id,job:structuredClone(job),current:this.runtime.current()};this.issued.add(ticket);return ticket;
 }
 async submit(ticket:AnalysisJobIntent):Promise<BackgroundJob|null>{
  this.runtime.enter();const scope=this.runtime.scope(),storage=this.runtime.storage.capture();
  if(!this.issued.has(ticket)||this.consumed.has(ticket)||!ticket.current()||this.state.busy||storage.read(ticket.key))throw Error('任务确认失效或重复发送');
  this.consumed.add(ticket);const lock=Symbol(ticket.key);this.locks.set(lock,{key:ticket.key,scope});this.feedback=null;this.runtime.changed();let sent=false;
  try{
   const rows=await this.runtime.api.listBackgroundJobs(100);if(!ticket.current())return null;
   if(ticket.kind==='enqueue'){
    if(rows.some(j=>j.job_type===ticket.draft!.job_type&&['queued','running'].includes(j.status)&&requestIdentity(j.payload)===requestIdentity(ticket.draft!.payload)))throw Error('确认后已出现同范围任务，请重新读取');
    storage.save({kind:'enqueue',key:ticket.key,draft:ticket.draft!,startedAt:new Date().toISOString()});
   }else{
    const fresh=rows.find(j=>j.id===ticket.job!.id);if(!fresh||jobIdentity(fresh)!==jobIdentity(ticket.job!))throw Error('原任务状态已变化，请重新确认');
    storage.save({kind:'job',key:ticket.key,operation:ticket.kind,job:ticket.job!,startedAt:new Date().toISOString()});
   }
   sent=true;
   const row=ticket.kind==='enqueue'?await this.runtime.api.enqueueAnalysisJob(ticket.draft!):ticket.kind==='cancel'?await this.runtime.api.cancelBackgroundJob(ticket.job!.id):await this.runtime.api.retryBackgroundJob(ticket.job!.id);
   validateJob(row);if(!scope())return null;
   if(ticket.kind==='enqueue'){
    if(row.job_type!==ticket.draft!.job_type||identity(row.payload)!==identity(ticket.draft!.payload))throw Error('入队回执范围不一致');
   }else if(row.id!==ticket.job!.id||row.job_type!==ticket.job!.job_type||identity(row.payload)!==identity(ticket.job!.payload))throw Error('任务回执与原对象不一致');
   if(ticket.kind==='cancel'&&row.status==='running'&&!row.cancellation_requested)throw Error('尚未读到原任务的取消请求标记');
   storage.clear(ticket.key);if(!ticket.current())return null;await this.runtime.refresh();return ticket.current()?row:null;
  }catch(error){if(!ticket.current())return null;this.feedback=(sent?'原任务请求结果未知，请按原身份核对。 ':'')+this.runtime.formatError(error);throw error;}
  finally{this.locks.delete(lock);this.runtime.changed();}
 }
 async recover(id:string):Promise<boolean>{
  this.runtime.enter();const storage=this.runtime.storage.capture(),receipt=storage.read('analysis:job:'+id),current=this.runtime.current();if(receipt?.kind!=='job')return false;
  try{
   const rows=await this.runtime.api.listBackgroundJobs(100);if(!current())return false;const row=rows.find(j=>j.id===id);if(!row)throw Error('当前目录未读到原任务，不能据此认定失败');validateJob(row);
   if(row.job_type!==receipt.job.job_type||identity(row.payload)!==identity(receipt.job.payload))throw Error('原任务范围变化');
   const verified=receipt.operation==='cancel'?(row.cancellation_requested||['succeeded','failed','cancelled'].includes(row.status)):(row.updated_at!==receipt.job.updated_at&&(!['failed','cancelled'].includes(row.status)||row.attempts>receipt.job.attempts));
   if(!verified)throw Error('尚不能核实原操作是否完成，继续保留未知');
   storage.clear(receipt.key);this.feedback=null;await this.runtime.refresh();return current();
  }catch(error){if(current())this.feedback=this.runtime.formatError(error);return false;}finally{if(current())this.runtime.changed();}
 }
 async recoverEnqueue():Promise<BackgroundJob|null>{
  this.runtime.enter();const storage=this.runtime.storage.capture(),pending=storage.read('analysis:enqueue'),current=this.runtime.current(),scope=this.runtime.scope();if(pending?.kind!=='enqueue')return null;
  if(this.state.busy)throw Error('原任务请求尚在进行');
  const lock=Symbol(pending.key);this.locks.set(lock,{key:pending.key,scope});this.runtime.changed();
  try{
   const row=await this.runtime.api.enqueueAnalysisJob(pending.draft);validateJob(row);if(!scope())return null;
   if(row.job_type!==pending.draft.job_type||identity(row.payload)!==identity(pending.draft.payload))throw Error('原幂等键回执与原范围不一致');
   storage.clear(pending.key);if(!current())return null;await this.runtime.refresh();return current()?row:null;
  }catch(error){if(current())this.feedback='原入队请求仍待核实。 '+this.runtime.formatError(error);return null;}finally{this.locks.delete(lock);this.runtime.changed();}
 }
}
