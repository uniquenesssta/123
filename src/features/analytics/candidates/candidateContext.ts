import type {ParameterTuningCandidateRecord,ParameterLifecycleReadiness} from '../../../types';
import type {AnalyticsRuntime} from '../analyticsPorts';
import type {CandidateFields,CandidateIntent,CandidateOperation,CandidateReceipt} from './candidatePorts';
import {initialCandidateDraft} from './candidatePorts';
import {minimumSamples} from './candidateIdentity';
export class CandidateContext {
 private revision=0;private issued=new WeakSet<object>();private consumed=new WeakSet<object>();private locks=new Map<symbol,()=>boolean>();
 private error:string|null=null;private readiness:ParameterLifecycleReadiness|null=null;private lastOutcome:string|null=null;private lastOutcomeTone:"success"|"warning"="success";
 constructor(readonly runtime:AnalyticsRuntime){}
 reset(){this.revision++;this.error=null;this.readiness=null;this.lastOutcome=null;}
 get state(){this.runtime.enter();const store=this.runtime.storage.capture();return {draft:store.candidateDraft?.()??initialCandidateDraft(),error:this.error,readiness:this.readiness,outcome:this.lastOutcome,outcomeTone:this.lastOutcomeTone,busy:[...this.locks.values()].some(s=>s()),pending:store.receipts().filter((r):r is CandidateReceipt=>r.kind==='candidate')};}
 update(field:string,value:string){if(!Object.keys(initialCandidateDraft().fields).includes(field))throw Error('未知候选范围字段');const store=this.runtime.storage.capture(),draft=this.state.draft;const allowed:Record<string,string[]>={snapshot_type:['T-N','T-24h','T-6h','T-1h'],target_module:['lineup_realization','history','state','venue','draw_correction','synergy'],minimum_sample_size:['50','100','200','500'],max_relative_change:['0.02','0.05','0.1']};if(allowed[field]&&!allowed[field].includes(value))throw Error('候选范围值无效');draft.fields={...draft.fields,[field]:value} as CandidateFields;store.saveCandidateDraft(draft);this.revision++;this.readiness=null;this.lastOutcome=null;this.runtime.interactionChanged();}
 note(id:string,value:string){const store=this.runtime.storage.capture(),draft=this.state.draft;draft.notes[id]=value;store.saveCandidateDraft(draft);this.revision++;}
 capture(){const current=this.runtime.current(),revision=this.revision;return()=>current()&&revision===this.revision;}
 issue<Row>(row:Row&{id:string},operation:CandidateOperation,note=''):CandidateIntent<Row>{
  this.runtime.enter();if(this.state.busy||this.state.pending.length)throw Error('原候选操作正在进行或待核实，请先查回原请求');
  const ticket={id:row.id,row:structuredClone(row),operation,note,current:this.capture()};this.issued.add(ticket);return ticket;
 }
 consume(ticket:object,current:()=>boolean){if(!this.issued.has(ticket)||this.consumed.has(ticket)||!current()||this.state.busy||this.state.pending.length)throw Error('候选确认已过期或重复发送');this.consumed.add(ticket);}
 begin(){const lock=Symbol(),scope=this.runtime.scope();this.locks.set(lock,scope);this.error=null;this.lastOutcome=null;this.runtime.changed();return()=>{this.locks.delete(lock);this.runtime.changed();};}
 fail(error:unknown,unknown=false){this.error=(unknown?'原请求结果待核实；禁止重复操作。 ':'')+this.runtime.formatError(error);}
 outcome(value:string,tone:"success"|"warning"="success"){this.lastOutcome=value;this.lastOutcomeTone=tone;this.error=null;}
 receipt(id:string){return this.runtime.storage.capture().read('analysis:candidate:'+id);}
 async gate(row?:ParameterTuningCandidateRecord):Promise<ParameterLifecycleReadiness>{
  const f=this.state.draft.fields,request=row?{competition_id:row.competition_id,snapshot_type:row.snapshot_type as 'T-1h',minimum_sample_size:minimumSamples(row)}:{competition_id:f.competition_id||null,snapshot_type:f.snapshot_type,minimum_sample_size:Number(f.minimum_sample_size)};
  if(!request.competition_id||!Number.isSafeInteger(request.minimum_sample_size)||request.minimum_sample_size<20)throw Error('请先选择具体赛事和有效最小样本');
  const current=this.capture(),gate=await this.runtime.api.parameterLifecycleReadiness(request);
  if(gate.competition_id!==request.competition_id||gate.snapshot_type!==request.snapshot_type||!Array.isArray(gate.blocked_reasons)||![gate.settled_sample_count,gate.eligible_sample_count,gate.minimum_sample_size].every(n=>Number.isSafeInteger(n)&&n>=0)||typeof gate.ready_for_promotion!=='boolean'||typeof gate.ready_for_shadow_validation!=='boolean')throw Error('门禁读回与原赛事或时点不一致');
  if(current()){this.readiness=gate;this.runtime.changed();}return gate;
 }
 async inspect(){const current=this.capture(),done=this.begin();try{await this.gate();return current();}catch(e){if(current())this.fail(e);return false;}finally{done();}}
 async history(id:string){const current=this.capture();try{const [validations,decisions]=await Promise.all([this.runtime.api.listParameterShadowValidations(id),this.runtime.api.listParameterPromotionDecisions(id)]);if(!current())return null;if(![validations,decisions].every(Array.isArray)||[...validations,...decisions].some(r=>r.candidate_id!==id))throw Error('生命周期读回不属于原候选');return {candidate_id:id,shadow_validations:validations,promotion_and_rollback_decisions:decisions};}catch(e){if(!current())return null;throw e;}}
}
