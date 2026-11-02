import type {MatchReviewPackageWorkflowRecord,PostmatchSettlementReadiness,PostmatchSettlementRecord} from '../../types';
import {matchReviewWorkflowAllows} from '../../app/matchReviewWorkflow';
import type {PostmatchRuntime,SettlementTarget} from './postmatchPorts';
import {settlementTarget} from './postmatchPorts';
import {identity,validateSettlement} from './postmatchIdentity';
export interface SettlementIntent {readonly target:SettlementTarget;readonly signature:string;readonly current:()=>boolean;readonly resume:boolean}
/** The settlement record and workflow registration are read and reported independently. */
export class SettlementWorkflow {
  private target:SettlementTarget|null=null;
  private readiness:PostmatchSettlementReadiness|null=null;
  private workflow:MatchReviewPackageWorkflowRecord|null=null;
  private record:PostmatchSettlementRecord|null=null;
  private error:string|null=null;
  private revision=0;private reads=0;private loading=false;
  private locks=new Map<symbol,{key:string;scope:()=>boolean}>();
  private intents=new WeakSet<SettlementIntent>();private consumed=new WeakSet<SettlementIntent>();
  constructor(private readonly runtime:PostmatchRuntime){}
  reset():void {this.target=null;this.readiness=null;this.workflow=null;this.record=null;this.error=null;this.loading=false;this.revision++;this.reads++;}
  private key(target:SettlementTarget){return 'settle:'+target.reviewId;}
  get state(){this.runtime.enter();return {target:this.target,readiness:this.readiness,workflow:this.workflow,record:this.record,error:this.error,loading:this.loading,pending:this.target?this.runtime.recovery.capture().read(this.key(this.target)):null,busy:this.target?[...this.locks.values()].some(l=>l.key===this.key(this.target!)&&l.scope()):false};}
  recordFor(reviewId:string|null,matchId:string|null){this.runtime.enter();return this.record?.match_review_id===reviewId&&this.record.match_id===matchId?this.record:null;}
  private activate(target:SettlementTarget):void {this.runtime.enter();if(identity(this.target)!==identity(target)){this.reset();this.target=structuredClone(target);}}
  private capture(guard:()=>boolean){const current=this.runtime.current(),revision=this.revision;return()=>current()&&guard()&&revision===this.revision;}
  async load(target:SettlementTarget,guard:()=>boolean=()=>true):Promise<boolean>{
    this.activate(target);const current=this.capture(guard),scope=this.runtime.scope(),serial=++this.reads;this.loading=true;this.error=null;
    try{const rows=await this.runtime.api.listPostmatchSettlements(500);if(!current()||serial!==this.reads)return false;const row=rows.find(r=>r.match_review_id===target.reviewId)??null;if(row)validateSettlement(row,target);this.record=row;return true;}
    catch(error){if(current()&&serial===this.reads)this.error='结算记录尚未核实。 '+this.runtime.formatError(error);return false;}
    finally{if(scope()&&serial===this.reads){this.loading=false;this.runtime.changed();}}
  }
  private async readGate(target:SettlementTarget){
    const [readiness,workflow,detail]=await Promise.all([this.runtime.api.postmatchSettlementReadiness(target.reviewId),this.runtime.api.readMatchReviewPackageWorkflow(target.matchId),this.runtime.api.readMatchReview(target.reviewId)]);
    if(readiness.match_review_id!==target.reviewId||readiness.match_id!==target.matchId||identity(settlementTarget(detail))!==identity(target))throw Error('复盘、正式运行或赛果身份已变化，请重新载入');
    if(workflow&&(workflow.match_id!==target.matchId||workflow.review_id!==target.reviewId))throw Error('资料包工作流与原复盘不一致');
    return {readiness,workflow};
  }
  async inspect(target:SettlementTarget,guard:()=>boolean=()=>true):Promise<boolean>{
    this.activate(target);const current=this.capture(guard),scope=this.runtime.scope(),serial=++this.reads;this.loading=true;this.error=null;this.readiness=null;this.runtime.changed();
    try{const gate=await this.readGate(target);if(!current()||serial!==this.reads)return false;Object.assign(this,{readiness:gate.readiness,workflow:gate.workflow});return true;}
    catch(error){if(current()&&serial===this.reads)this.error=this.runtime.formatError(error);return false;}
    finally{if(scope()&&serial===this.reads){this.loading=false;this.runtime.changed();}}
  }
  canSubmit():boolean {
    const s=this.state;
    return !!s.target&&!s.loading&&!s.busy&&!s.pending&&s.readiness?.ready===true&&(!s.workflow||matchReviewWorkflowAllows(s.workflow,'settle_review'))&&!s.record;
  }
  prepare(guard:()=>boolean,resume=false):SettlementIntent {
    const s=this.state,mayResume=resume&&s.pending?.kind==='settle'&&!!s.record&&s.readiness?.existing_settlement_id===s.record.id&&(!s.workflow||s.workflow.status!=='settled');
    if(s.pending?.kind==='settle'&&identity(s.pending.target)!==identity(this.target))throw Error('原结算请求绑定其他复盘版本或赛果，需核验原对象，不能补写当前内容');
    if(!this.target||(!this.canSubmit()&&!mayResume)||s.busy||s.loading||s.readiness?.ready!==true)throw Error('请先核对原结算门禁；结果未知时只能查回，不能盲目重发');
    const ticket={target:structuredClone(this.target),signature:identity([this.target,this.readiness,this.workflow]),current:this.capture(guard),resume};this.intents.add(ticket);return ticket;
  }
  async submit(ticket:SettlementIntent):Promise<boolean>{
    this.runtime.enter();const {target}=ticket,key=this.key(target),scope=this.runtime.scope(),recovery=this.runtime.recovery.capture();
    if(!this.intents.has(ticket)||this.consumed.has(ticket)||!ticket.current()||[...this.locks.values()].some(l=>l.key===key&&l.scope())||recovery.read(key)&&!ticket.resume)throw Error('结算确认已失效或原请求仍待核实');
    this.consumed.add(ticket);const token=Symbol(key);this.locks.set(token,{key,scope});this.error=null;this.runtime.changed();let sent=false;
    try{
      const gate=await this.readGate(target);if(!ticket.current())return false;
      if(identity([target,gate.readiness,gate.workflow])!==ticket.signature||!gate.readiness.ready||gate.workflow&&gate.workflow.status!=='settled'&&!matchReviewWorkflowAllows(gate.workflow,'settle_review'))throw Error('结算许可已变化，请重新检查并确认');
      recovery.save({kind:'settle',key,target,startedAt:new Date().toISOString()});sent=true;
      const record=await this.runtime.api.settlePostmatchReview({match_review_id:target.reviewId,settled_by:null,settlement_note:ticket.resume?'用户核实原账本后补全同一结算工作流登记':'用户确认正式结算'});
      validateSettlement(record,target);if(!scope())return false;
      return await this.readBack(target,ticket.current,recovery);
    }catch(error){if(!ticket.current())return false;this.error=(sent?'结算请求结果待核实；只读查回原复盘，不自动重发。 ':'')+this.runtime.formatError(error);throw error;}
    finally{this.locks.delete(token);this.runtime.changed();}
  }
  private async readBack(target:SettlementTarget,current:()=>boolean,recovery:ReturnType<PostmatchRuntime['recovery']['capture']>):Promise<boolean>{
    const [rows,gate]=await Promise.all([this.runtime.api.listPostmatchSettlements(500),this.readGate(target)]);
    if(!current())return false;
    const record=rows.find(row=>row.match_review_id===target.reviewId);
    if(!record)throw Error('当前目录未读到原结算，不能据此判定未写入');
    validateSettlement(record,target);
    if(gate.readiness.existing_settlement_id!==record.id)throw Error('结算门禁与原账本读回不一致');
    this.record=record;this.readiness=gate.readiness;this.workflow=gate.workflow;
    if(gate.workflow&&gate.workflow.status!=='settled'){this.error='原正式结算账本已记录，工作流登记尚未核实；可明确确认后补全同一结算登记。';return false;}
    recovery.clear(this.key(target));this.error=null;this.runtime.changed();return true;
  }
  async recover(guard:()=>boolean=()=>true):Promise<boolean>{
    const s=this.state;if(!s.target)return false;const current=this.capture(guard),recovery=this.runtime.recovery.capture();
    if(s.pending?.kind==='settle'&&identity(s.pending.target)!==identity(s.target)){this.error='原结算请求绑定其他复盘版本或赛果，需核验原对象；继续保留结果未知。';this.runtime.changed();return false;}
    if(s.busy)throw Error('结算请求尚在进行，请等待回执');
    try{return await this.readBack(s.target,current,recovery);}catch(error){if(current())this.error='原结算仍待核实。 '+this.runtime.formatError(error);return false;}finally{if(current())this.runtime.changed();}
  }
}
