import type {AbilityUpdateCandidateRecord} from '../../../types';import type {CandidateIntent,CandidateReceipt} from './candidatePorts';import {abilitySource,same} from './candidateIdentity';import type {CandidateContext} from './candidateContext';
export class AbilityDecisions {
 constructor(private readonly context:CandidateContext){}
 prepare(id:string,decision:'accept'|'reject'):CandidateIntent<AbilityUpdateCandidateRecord>{const row=this.context.runtime.abilities().find(r=>r.id===id);if(!row||row.status!=='pending'||!['accept','reject'].includes(decision))throw Error('能力候选已变化或不在待审队列');return this.context.issue(row,decision==='accept'?'ability-accept':'ability-reject');}
 async submit(ticket:CandidateIntent<AbilityUpdateCandidateRecord>):Promise<boolean>{
  const c=this.context,r=c.runtime,storage=r.storage.capture(),scope=r.scope();c.consume(ticket,ticket.current);const done=c.begin(),key='analysis:candidate:'+ticket.id;let sent=false;
  try{const fresh=(await r.api.listAbilityCandidates(null,500,null)).find(row=>row.id===ticket.id);if(!ticket.current())return false;if(!fresh||!same(fresh,ticket.row))throw Error('能力候选内容或状态已变化，请重新确认');
   storage.save({kind:'candidate',key,id:ticket.id,operation:ticket.operation,source:abilitySource(ticket.row),note:'用户核对原能力候选后决定',beforeIds:[],startedAt:new Date().toISOString()});sent=true;
   const accepted=ticket.operation==='ability-accept',row=await r.api.decideAbilityCandidate({candidate_id:ticket.id,decision:accepted?'accept':'reject',decided_by:'local_user',decision_note:'用户核对原能力候选后'+(accepted?'接受':'拒绝')});if(!scope())return false;
   if(!same(abilitySource(row),abilitySource(ticket.row))||row.status!==(accepted?'accepted':'rejected')||accepted&&!row.accepted_observation_id)throw Error('尚未核实原能力候选的完整决定回执');storage.clear(key);if(!ticket.current())return false;await r.refresh();c.outcome(accepted?'原能力候选已接受，能力历史记录已核实':'原能力候选已拒绝');return true;
  }catch(e){if(!ticket.current())return false;c.fail(e,sent);throw e;}finally{done();}
 }
 async recover(receipt:CandidateReceipt):Promise<boolean>{const c=this.context,r=c.runtime,current=c.capture(),store=r.storage.capture(),rows=await r.api.listAbilityCandidates(null,500,null);if(!current())return false;const row=rows.find(row=>row.id===receipt.id),accepted=receipt.operation==='ability-accept';if(!row||!same(abilitySource(row),receipt.source)||row.status!==(accepted?'accepted':'rejected')||accepted&&!row.accepted_observation_id)throw Error('原能力决定仍待核实，未出现不能证明未写入');store.clear(receipt.key);await r.refresh();return current();}
}
