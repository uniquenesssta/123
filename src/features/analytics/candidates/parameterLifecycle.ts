import type {ParameterTuningCandidateRecord,ParameterShadowValidationRecord,ParameterPromotionDecisionRecord} from '../../../types';
import type {CandidateIntent,CandidateOperation,CandidateReceipt} from './candidatePorts';import type {CandidateContext} from './candidateContext';import {matchingReadiness,parameterSource,same} from './candidateIdentity';
const allowed=(row:ParameterTuningCandidateRecord,op:CandidateOperation)=>op==='accept_for_backtest'||op==='reject'?row.status==='pending':op==='shadow'?['accepted_for_backtest','blocked_by_h','shadow_failed'].includes(row.status):op==='promote'?row.status==='shadow_passed':op==='rollback'?row.status==='promoted':false;
export class ParameterLifecycle {
 constructor(private readonly context:CandidateContext){}
 prepare(id:string,operation:CandidateOperation):CandidateIntent<ParameterTuningCandidateRecord>{
  const row=this.context.runtime.candidates().find(r=>r.id===id);if(!row||!allowed(row,operation))throw Error('参数候选当前状态不允许此操作');
  const note=this.context.state.draft.notes[id+':'+operation]??'';if(['promote','rollback'].includes(operation)&&note.trim().length<8)throw Error('请填写至少8个字符的人工决定说明');return this.context.issue(row,operation,note);
 }
 private async current(ticket:CandidateIntent<ParameterTuningCandidateRecord>){const row=(await this.context.runtime.api.listParameterTuningCandidates(100)).find(r=>r.id===ticket.id);if(!row||!same(row,ticket.row)||!allowed(row,ticket.operation))throw Error('原候选、分区或版本已经变化，旧确认失效');return row;}
 private async validate(row:ParameterTuningCandidateRecord,operation:CandidateOperation){
  if(!['shadow','promote','rollback'].includes(operation))return;
  const c=this.context,gate=await c.gate(row),target=operation==='rollback'?'candidate':'baseline';
  if(!matchingReadiness(row,gate,target))throw Error('当前正式绑定与候选锁定的版本不一致，请重新准入');
  if(operation!=='rollback'&&!(operation==='promote'?gate.ready_for_promotion:gate.ready_for_shadow_validation))throw Error('精确分区门禁阻断：'+gate.blocked_reasons.join('；'));
 }
 async submit(ticket:CandidateIntent<ParameterTuningCandidateRecord>):Promise<boolean>{
  const c=this.context,r=c.runtime,store=r.storage.capture(),scope=r.scope();c.consume(ticket,ticket.current);const done=c.begin(),key='analysis:candidate:'+ticket.id;let sent=false;
  try{
   const row=await this.current(ticket);if(!ticket.current())return false;await this.validate(row,ticket.operation);if(!ticket.current())return false;
   let history:Awaited<ReturnType<CandidateContext['history']>>=null;
   if(['shadow','promote','rollback'].includes(ticket.operation))history=await c.history(ticket.id);if(!ticket.current())return false;
   if(ticket.operation==='promote'&&(!history?.shadow_validations.length||history.shadow_validations[0].status!=='passed'||history.shadow_validations[0].partition_key!==row.partition_key))throw Error('最新影子验证不是本候选当前分区的通过记录');
   if(ticket.operation==='rollback'&&history?.promotion_and_rollback_decisions[0]?.decision!=='promote')throw Error('未核实原晋升决定，不能回滚其他绑定');
   const beforeIds=history?[...history.shadow_validations,...history.promotion_and_rollback_decisions].map(r=>r.id):[];
   const receipt:CandidateReceipt={kind:'candidate',key,id:ticket.id,operation:ticket.operation,source:parameterSource(row),beforeIds,note:ticket.note,startedAt:new Date().toISOString()};store.save(receipt);sent=true;
   let response:ParameterTuningCandidateRecord|ParameterShadowValidationRecord|ParameterPromotionDecisionRecord;
   if(ticket.operation==='accept_for_backtest'||ticket.operation==='reject')response=await r.api.decideParameterTuningCandidate({candidate_id:ticket.id,decision:ticket.operation,decision_note:'用户核对原参数候选后'+(ticket.operation==='reject'?'拒绝':'接受，仅进入影子队列')});
   else if(ticket.operation==='shadow')response=await r.api.runParameterShadowValidation({candidate_id:ticket.id});
   else if(ticket.operation==='promote')response=await r.api.promoteParameterCandidate({candidate_id:ticket.id,decided_by:'local_user',decision_note:ticket.note});
   else response=await r.api.rollbackParameterCandidate({candidate_id:ticket.id,decided_by:'local_user',decision_note:ticket.note});
   if(!scope())return false;
   if(ticket.operation==='accept_for_backtest'||ticket.operation==='reject'){
    const result=response as ParameterTuningCandidateRecord;if(!same(parameterSource(result),receipt.source)||result.status!==(ticket.operation==='reject'?'rejected':'accepted_for_backtest'))throw Error('参数候选决定回执与原对象不一致');
   }else if(ticket.operation==='shadow'){
    const result=response as ParameterShadowValidationRecord;if(result.candidate_id!==ticket.id||result.partition_key!==row.partition_key||!['passed','failed','blocked'].includes(result.status)||beforeIds.includes(result.id))throw Error('影子回执未核实原候选的新报告');
   }else{
    const result=response as ParameterPromotionDecisionRecord;if(result.candidate_id!==ticket.id||result.decision!==ticket.operation||beforeIds.includes(result.id)||result.decision_note!==ticket.note)throw Error('绑定决定回执与原候选不一致');
    const gate=await c.gate(row);if(!scope())return false;if(!matchingReadiness(row,gate,ticket.operation==='promote'?'candidate':'baseline'))throw Error('决定已返回，但当前正式绑定尚未核实；继续保留原请求');
   }
   store.clear(key);if(!ticket.current())return false;await r.refresh();
   c.outcome(ticket.operation==='promote'?'已核实原候选晋升决定及当前正式版本':ticket.operation==='rollback'?'已核实原回滚决定及恢复后的正式版本':ticket.operation==='shadow'?({passed:'影子验证通过，仍需人工晋升',failed:'影子验证未通过，正式绑定未变',blocked:'影子验证被门禁阻断，正式绑定未变'}[(response as ParameterShadowValidationRecord).status]):ticket.operation==='reject'?'原参数候选已拒绝':'候选已接受进入影子队列，尚未晋升',ticket.operation==='shadow'&&(response as ParameterShadowValidationRecord).status!=='passed'?'warning':'success');return true;
  }catch(e){if(!ticket.current())return false;c.fail(e,sent);throw e;}finally{done();}
 }
 async recover(receipt:CandidateReceipt):Promise<boolean>{
  const c=this.context,r=c.runtime,current=c.capture(),store=r.storage.capture(),row=(await r.api.listParameterTuningCandidates(100)).find(row=>row.id===receipt.id);if(!current())return false;
  if(!row||!same(parameterSource(row),receipt.source))throw Error('原候选版本尚未核实，不能推断未执行');
  if(receipt.operation==='accept_for_backtest'||receipt.operation==='reject'){
   if(row.status!==(receipt.operation==='reject'?'rejected':'accepted_for_backtest'))throw Error('原参数决定仍待核实');
  }else{
   const history=await c.history(receipt.id);if(!history||!current())return false;
   if(receipt.operation==='shadow'){
    const report=history.shadow_validations.find(row=>!receipt.beforeIds.includes(row.id)&&row.partition_key===receipt.source.partition_key);if(!report||!['passed','failed','blocked'].includes(report.status))throw Error('尚未查到原请求的新影子报告，不重复验证');
   }else{
    const decision=history.promotion_and_rollback_decisions.find(row=>!receipt.beforeIds.includes(row.id)&&row.decision===receipt.operation&&row.decision_note===receipt.note);if(!decision)throw Error('尚未查到原绑定决定，禁止盲目晋升或回滚');
    const gate=await c.gate(row);if(!current())return false;if(!matchingReadiness(row,gate,receipt.operation==='promote'?'candidate':'baseline'))throw Error('原决定已存在，但当前绑定已变化；保留差异待核实，不覆盖正式版本');
   }
  }
  store.clear(receipt.key);await r.refresh();return current();
 }
}
