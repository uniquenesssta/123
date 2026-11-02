import type {EvidenceScoringDecisionDraft,EvidenceScoringItemRecord} from '../../types';
import type {PostmatchRuntime} from './postmatchPorts';
import {evidenceIdentity} from './postmatchIdentity';
export interface EvidenceIntent {readonly item:EvidenceScoringItemRecord;readonly draft:EvidenceScoringDecisionDraft;readonly current:()=>boolean}
export class EvidenceWorkflow {
  private notes=new Map<string,{note:string;verdict:EvidenceScoringDecisionDraft['verdict']}>();
  private error:string|null=null;private locks=new Map<symbol,{key:string;scope:()=>boolean}>();
  private intents=new WeakSet<EvidenceIntent>();private consumed=new WeakSet<EvidenceIntent>();
  constructor(private readonly runtime:PostmatchRuntime,private readonly selection:()=>string){}
  reset():void{this.notes.clear();this.error=null;}
  get state(){this.runtime.enter();return {error:this.error,busy:[...this.locks.values()].some(l=>l.scope())};}
  item(id:string):EvidenceScoringItemRecord {
    const row=this.runtime.overview()?.evidence_queue.find(r=>r.id===id);
    if(!row||this.selection()&&row.settlement_id!==this.selection())throw Error('证据不属于当前结算目录');
    return row;
  }
  draft(id:string){this.runtime.enter();return this.notes.get(id)??{note:'',verdict:'correct' as const};}
  saveDraft(id:string,note:string,verdict:EvidenceScoringDecisionDraft['verdict']):void{this.item(id);this.notes.set(id,{note,verdict});}
  pending(id:string){this.runtime.enter();return this.runtime.recovery.capture().read('evidence:'+id);}
  prepare(id:string,verdict:EvidenceScoringDecisionDraft['verdict'],note:string,guard:()=>boolean=()=>true):EvidenceIntent {
    this.runtime.enter();const row=this.item(id);
    if(row.status!=='pending'||row.decision_id||this.pending(id))throw Error('证据已判定或原判定待核实，不能覆盖不可变账本');
    if(!['correct','partial','incorrect','not_verifiable'].includes(verdict)||new TextEncoder().encode(note.trim()).length<8)throw Error('判定依据过短，请补充原始来源和核验结果');
    this.saveDraft(id,note,verdict);const current=this.runtime.current(),filter=this.selection();
    const ticket={item:structuredClone(row),draft:{item_id:id,verdict,decided_by:null,decision_note:note.trim()},current:()=>current()&&guard()&&filter===this.selection()};this.intents.add(ticket);return ticket;
  }
  async submit(ticket:EvidenceIntent):Promise<EvidenceScoringItemRecord|null>{
    this.runtime.enter();const id=ticket.item.id,key='evidence:'+id,scope=this.runtime.scope(),recovery=this.runtime.recovery.capture();
    if(!this.intents.has(ticket)||this.consumed.has(ticket)||!ticket.current()||recovery.read(key)||[...this.locks.values()].some(l=>l.key===key&&l.scope()))throw Error('证据确认已变化或重复提交');
    this.consumed.add(ticket);const token=Symbol(key);this.locks.set(token,{key,scope});this.error=null;this.runtime.changed();let sent=false;
    try{
      const rows=await this.runtime.api.listEvidenceScoringItems(null,500);if(!ticket.current())return null;
      const fresh=rows.find(r=>r.id===id);if(!fresh||evidenceIdentity(fresh)!==evidenceIdentity(ticket.item))throw Error('证据状态或结算身份已变化，请重新核对');
      recovery.save({kind:'evidence',key,itemId:id,settlementId:ticket.item.settlement_id,verdict:ticket.draft.verdict,note:ticket.draft.decision_note,startedAt:new Date().toISOString()});sent=true;
      const row=await this.runtime.api.decideEvidenceScoringItem(ticket.draft);
      if(!scope())return null;
      if(row.id!==id||row.settlement_id!==ticket.item.settlement_id||!row.decision_id||row.status==='pending'||row.verdict!==ticket.draft.verdict||row.decision_note!==ticket.draft.decision_note)throw Error('人工判定回执与原证据或确认内容不一致');
      recovery.clear(key);if(!ticket.current())return null;
      await this.runtime.refresh();if(!ticket.current())return null;return row;
    }catch(error){if(!ticket.current())return null;this.error=(sent?'原证据判定结果待核实，禁止自动重发。 ':'')+this.runtime.formatError(error);throw error;}
    finally{this.locks.delete(token);this.runtime.changed();}
  }
  async recover(id:string):Promise<EvidenceScoringItemRecord|null>{
    this.runtime.enter();const recovery=this.runtime.recovery.capture(),pending=recovery.read('evidence:'+id),current=this.runtime.current();
    if(pending?.kind!=='evidence')return null;
    try{
      const rows=await this.runtime.api.listEvidenceScoringItems(null,500);if(!current())return null;const row=rows.find(r=>r.id===id);
      if(!row||row.settlement_id!==pending.settlementId||!row.decision_id||row.status==='pending'||row.verdict!==pending.verdict||row.decision_note!==pending.note)throw Error('原证据判定尚未查回，或存在其他人工决定；不能重写');
      recovery.clear(pending.key);this.error=null;await this.runtime.refresh();return current()?row:null;
    }catch(error){if(current())this.error=this.runtime.formatError(error);return null;}finally{if(current())this.runtime.changed();}
  }
}
