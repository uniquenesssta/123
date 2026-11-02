import type {AiAnalysisSuggestionRecord,DataQualityFinding} from '../../types';
import type {AnalyticsRuntime} from './analyticsPorts';
import {identity} from './analyticsIdentity';
export type AnalysisDecisionIntent={kind:'suggestion';row:AiAnalysisSuggestionRecord;decision:'accept'|'reject';current:()=>boolean}|{kind:'quality';row:DataQualityFinding;decision:'resolve'|'ignore';current:()=>boolean};
export class AnalyticsReviews {
 private error:string|null=null;private locks=new Map<symbol,()=>boolean>();private issued=new WeakSet<AnalysisDecisionIntent>();private consumed=new WeakSet<AnalysisDecisionIntent>();
 constructor(private readonly runtime:AnalyticsRuntime){}
 reset(){this.error=null;}
 get state(){this.runtime.enter();return {error:this.error,busy:[...this.locks.values()].some(s=>s())};}
 pending(id:string){return this.runtime.storage.capture().read('analysis:decision:'+id);}
 prepareSuggestion(id:string,decision:'accept'|'reject'):AnalysisDecisionIntent {
  if(!['accept','reject'].includes(decision))throw Error('建议决定无效');
  const row=this.runtime.suggestions().find(r=>r.id===id);if(!row||row.status!=='pending'||this.pending(id)||this.state.busy)throw Error('建议已变化、已处理或原决定待核实');
  const ticket:AnalysisDecisionIntent={kind:'suggestion',row:structuredClone(row),decision,current:this.runtime.current()};this.issued.add(ticket);return ticket;
 }
 prepareQuality(id:string,decision:'resolve'|'ignore'):AnalysisDecisionIntent {
  if(!['resolve','ignore'].includes(decision))throw Error('质量决定无效');
  const overview=this.runtime.overview(),row=overview?.data_quality.findings.find(r=>r.id===id);
  if(!row||row.status!=='open'||row.scan_id!==overview?.data_quality.scan_id||this.state.busy||this.pending(id))throw Error('质量问题不属于当前扫描，或原决定待核实');
  const ticket:AnalysisDecisionIntent={kind:'quality',row:structuredClone(row),decision,current:this.runtime.current()};this.issued.add(ticket);return ticket;
 }
 async submit(ticket:AnalysisDecisionIntent):Promise<boolean>{
  const current=ticket.current,scope=this.runtime.scope(),storage=this.runtime.storage.capture(),key='analysis:decision:'+ticket.row.id;
  if(!this.issued.has(ticket)||this.consumed.has(ticket)||!current()||this.state.busy||storage.read(key))throw Error('审核确认失效或重复发送');
  this.consumed.add(ticket);const lock=Symbol(key);this.locks.set(lock,scope);this.error=null;this.runtime.changed();let sent=false;
  try{
   if(ticket.kind==='suggestion'){
    const rows=await this.runtime.api.listAiAnalysisSuggestions(null,500);if(!current())return false;
    const fresh=rows.find(r=>r.id===ticket.row.id);if(!fresh||identity(fresh)!==identity(ticket.row))throw Error('建议状态、回包或原始内容已变化');
    storage.save({kind:'suggestion',key,id:ticket.row.id,responseId:ticket.row.response_id,decision:ticket.decision,startedAt:new Date().toISOString()});sent=true;
    const row=await this.runtime.api.decideAiAnalysisSuggestion({suggestion_id:ticket.row.id,decision:ticket.decision,decided_by:'local_user',decision_note:'用户核对原建议后'+(ticket.decision==='accept'?'接受；仅进入受控审核记录或待审候选':'拒绝')});
    if(!scope())return false;
    if(row.id!==ticket.row.id||row.response_id!==ticket.row.response_id||row.status!==(ticket.decision==='accept'?'accepted':'rejected')||ticket.decision==='accept'&&ticket.row.suggestion_type==='ability_update'&&!row.linked_candidate_id)throw Error('建议决定尚未读到原对象完整回执');
   }else{
    const overview=await this.runtime.api.analyticsOverview();if(!current())return false;
    const fresh=overview.data_quality.findings.find(r=>r.id===ticket.row.id);if(overview.data_quality.scan_id!==ticket.row.scan_id||!fresh||identity(fresh)!==identity(ticket.row))throw Error('质量扫描或问题状态已变化');
    storage.save({kind:'quality',key,id:ticket.row.id,scanId:ticket.row.scan_id,decision:ticket.decision,startedAt:new Date().toISOString()});sent=true;
    const row=await this.runtime.api.decideDataQualityFinding({finding_id:ticket.row.id,decision:ticket.decision,resolution_note:ticket.decision==='resolve'?'用户确认已处理；不修改业务事实':'用户确认忽略；不修改业务事实'});
    if(!scope())return false;if(row.id!==ticket.row.id||row.scan_id!==ticket.row.scan_id||row.status!==(ticket.decision==='resolve'?'resolved':'ignored'))throw Error('质量决定回执与原扫描不一致');
   }
   storage.clear(key);if(!current())return false;await this.runtime.refresh();return current();
  }catch(error){if(!current())return false;this.error=(sent?'原审核结果待核实，禁止重复决定。 ':'')+this.runtime.formatError(error);throw error;}
  finally{this.locks.delete(lock);this.runtime.changed();}
 }
 async recover(id:string):Promise<boolean>{
  const current=this.runtime.current(),storage=this.runtime.storage.capture(),receipt=this.pending(id);if(!receipt||!['suggestion','quality'].includes(receipt.kind)||this.state.busy)return false;
  try{
   if(receipt.kind==='suggestion'){
    const rows=await this.runtime.api.listAiAnalysisSuggestions(null,500);if(!current())return false;const row=rows.find(r=>r.id===id);
    if(!row||row.response_id!==receipt.responseId||row.status!==(receipt.decision==='accept'?'accepted':'rejected')||row.suggestion_type==='ability_update'&&receipt.decision==='accept'&&!row.linked_candidate_id)throw Error('原建议决定仍待核实');
   }else if(receipt.kind==='quality'){
    const overview=await this.runtime.api.analyticsOverview();if(!current())return false;const row=overview.data_quality.findings.find(r=>r.id===id&&r.scan_id===receipt.scanId);
    if(!row||row.status!==(receipt.decision==='resolve'?'resolved':'ignored'))throw Error('当前扫描未核实原问题决定，不能推断未写入');
   }
   storage.clear(receipt.key);this.error=null;await this.runtime.refresh();return current();
  }catch(error){if(current())this.error=this.runtime.formatError(error);return false;}finally{if(current())this.runtime.changed();}
 }
}
