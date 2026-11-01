import { PredictionContext } from "./predictionContext";
import { completeRun, findExistingRuns, matchesRun } from "./predictionRunRecovery";
import type { PredictionMode, PredictionRecovery, RunReceipt } from "./predictionRecovery";
import type { MatchLineupChain, MatchPredictionReadiness, ModelRunListItem, PredictionExecution, RouteDecision, StoredMatchPredictionCommand } from "../../types";

export type PredictionPhase="unchecked"|"changed"|"checking"|"ready"|"blocked"|"running"|"failed"|"unavailable"|"unknown"|"result";
export interface PredictionPorts {
  api: {
    inspectMatchPredictionReadiness(c:StoredMatchPredictionCommand):Promise<MatchPredictionReadiness>;
    readMatchLineupChain(id:string,snapshot:string):Promise<MatchLineupChain>;
    executePredictionFromMatch(c:StoredMatchPredictionCommand):Promise<PredictionExecution>;
    executeShadowPredictionFromMatch(c:StoredMatchPredictionCommand):Promise<PredictionExecution>;
    listRecentRuns(limit:number):Promise<ModelRunListItem[]>;
    readRun(id:string):Promise<Record<string,unknown>>;
  };
  scope():()=>boolean; view():()=>boolean; changed():void;
  busy<T>(operation:()=>Promise<T>):Promise<T>; recovery:PredictionRecovery;
}
/** Context, eligibility and execution are single-owner; the backend remains the model authority. */
export class PredictionFeature {
  readonly context:PredictionContext;
  private phaseValue:PredictionPhase="unchecked";
  private locked=false;
  readiness:MatchPredictionReadiness|null=null;
  chain:MatchLineupChain|null=null;
  error:string|null=null;
  private resultValue:unknown=null;
  private dataScope=()=>true;
  get lastResult():unknown{return this.dataScope()?this.resultValue:null;}
  set lastResult(value:unknown){this.resultValue=value;}
  resultMode:"formal"|"shadow"|"temporary"|null=null;
  candidates:Record<string,unknown>[]=[];
  route:RouteDecision|null=null;
  private routeRevision=0;
  private viewGeneration=0;
  constructor(private readonly ports:PredictionPorts) {
    this.context=new PredictionContext(()=>this.invalidate());
  }
  get view(){const valid=this.dataScope();return {context:this.context.command,phase:this.phase,busy:this.busy,readiness:valid?this.readiness:null,chain:valid?this.chain:null,error:valid?this.error:null,lastResult:this.lastResult,resultMode:this.resultMode,candidates:valid?this.candidates:[],receipt:this.receipt,canFormal:this.canFormal,canShadow:this.canShadow,route:valid?this.route:null};}
  initializeContext(command:Partial<StoredMatchPredictionCommand>):void{const first=!this.context.matchId;this.context.update(command);if(first)this.phaseValue="unchecked";}
  get busy():boolean{return this.locked;}
  get receipt():RunReceipt|null{return this.ports.recovery.read();}
  get phase():PredictionPhase{return this.receipt&&!this.locked?"unknown":!this.dataScope()&&!this.locked?"changed":this.phaseValue;}
  get canFormal():boolean{return !this.locked&&!this.receipt&&this.dataScope()&&this.phaseValue!=="result"&&!!this.context.matchId&&(this.readiness?this.readiness.can_run_formal&&!!this.readiness.input_manifest_sha256:this.phaseValue==="unchecked");}
  get canShadow():boolean{return !this.locked&&!this.receipt&&this.dataScope()&&this.phaseValue!=="result"&&this.readiness?.can_run_shadow===true&&!!this.readiness.input_manifest_sha256;}
  private invalidate():void{this.dataScope=this.ports.scope();this.routeRevision++;this.route=null;this.candidates=[];this.readiness=null;this.chain=null;this.lastResult=null;this.resultMode=null;this.error=null;this.phaseValue="changed";this.ports.changed();}
  viewChanged():void{this.viewGeneration++;}
  navigationChanged():void{this.viewChanged();this.context.invalidate();this.readiness=null;this.chain=null;this.lastResult=null;this.resultMode=null;this.phaseValue="unchecked";}
  capturePresentation():()=>boolean{const context=this.context.capture(),view=this.ports.view(),generation=this.viewGeneration;return ()=>generation===this.viewGeneration&&context()&&view();}
  private current():()=>boolean{const current=this.capturePresentation(),scope=this.ports.scope();return ()=>current()&&scope();}
  private validate(c:StoredMatchPredictionCommand,r:MatchPredictionReadiness):void{
    if(r.match_id!==c.match_id||r.snapshot_type!==c.snapshot_type||r.model_family!==c.model_family)throw Error("门禁返回的比赛、窗口或模型与当前输入不匹配");
  }
  async readChain():Promise<MatchLineupChain>{
    const c=this.context.require(),current=this.current(),chain=await this.ports.api.readMatchLineupChain(c.match_id,c.snapshot_type);
    if(!current()||chain.match_record.id!==c.match_id||chain.snapshot_type!==c.snapshot_type)throw Error("原阵容查询已失效");
    this.chain=chain;return chain;
  }
  async check():Promise<boolean>{
    if(this.locked)throw Error("当前推演流程正在处理");
    if(this.receipt)throw Error("原请求结果尚未核实，请先查回已有运行");
    this.dataScope=this.ports.scope();
    const command=this.context.require(),current=this.current();this.locked=true;this.phaseValue="checking";this.error=null;this.lastResult=null;this.ports.changed();
    try{
      const r=await this.ports.busy(()=>this.ports.api.inspectMatchPredictionReadiness(command));
      if(!current())return false;this.validate(command,r);
      this.readiness=structuredClone(r);
      try{const chain=await this.ports.api.readMatchLineupChain(command.match_id,command.snapshot_type);if(!current())return false;if(chain.match_record.id!==command.match_id||chain.snapshot_type!==command.snapshot_type)throw Error("阵容链返回身份不匹配");this.chain=chain;}
      catch(error){if(current())this.error="阵容详情读取失败："+String(error);}
      if(!current())return false;this.phaseValue=(r.can_run_formal||r.can_run_shadow)&&r.input_manifest_sha256?"ready":"blocked";return true;
    }catch(error){if(!current())return false;this.phaseValue="failed";this.error=String(error);throw error;}
    finally{this.locked=false;this.ports.changed();}
  }
  async execute(mode:PredictionMode):Promise<boolean>{
    if(this.locked)throw Error("当前推演流程正在处理，禁止重复提交");
    if(this.receipt)throw Error("原请求结果尚未核实，禁止直接重跑");
    if(this.phaseValue==="result")throw Error("本轮推演已完成，请重新检查完整度后再发起新的运行");
    this.dataScope=this.ports.scope();
    const command=this.context.require(),current=this.current(),scope=this.ports.scope();
    this.locked=true;this.candidates=[];this.error=null;this.lastResult=null;this.resultMode=null;this.phaseValue="checking";this.ports.changed();
    let dispatched=false;
    try{
      const baseline=mode==="formal"?await this.ports.api.listRecentRuns(500):[];
      if(!current())return false;
      const r=await this.ports.busy(()=>this.ports.api.inspectMatchPredictionReadiness(command));
      if(!current())return false;this.validate(command,r);this.readiness=structuredClone(r);
      const allowed=mode==="formal"?r.can_run_formal:r.can_run_shadow;
      if(!allowed||!r.input_manifest_sha256){this.phaseValue="blocked";throw Error(mode==="formal"?"当前门禁未允许正式推演":"当前门禁未允许影子推演");}
      const receipt:RunReceipt={command,mode,manifest:r.input_manifest_sha256,matchKey:r.match_key,baseline:baseline.map(x=>x.id)};
      this.ports.recovery.save(receipt);
      this.phaseValue="running";this.ports.changed();dispatched=true;
      const result=await this.ports.busy(()=>mode==="formal"?this.ports.api.executePredictionFromMatch(command):this.ports.api.executeShadowPredictionFromMatch(command));
      if(!scope())return false;
      if(mode==="formal"&&result.run_id&&result.run_id!=="00000000-0000-0000-0000-000000000000"){receipt.knownRunId=result.run_id;this.ports.recovery.save(receipt);}
      const audit=result.input_audit;
      if(!audit?.input_manifest_sha256||!audit.input_sha256)throw Error("提供器结果缺少实际输入审计，不能宣布完成");
      if(mode==="formal"&&(audit.readiness_level==="shadow_only"||!receipt.knownRunId))throw Error("正式结果身份或准入类型不匹配");
      if(mode==="shadow"&&result.run_id!=="00000000-0000-0000-0000-000000000000")throw Error("影子结果返回了正式运行身份");
      // Native execution re-assesses at its own safe cutoff; preserve its actual manifest, never synthesize a hash.
      receipt.manifest=audit.input_manifest_sha256;receipt.actualManifest=true;this.ports.recovery.save(receipt);
      if(mode==="formal"){
        const doc=await this.ports.api.readRun(result.run_id);
        if(!scope())return false;
        if(doc.id!==result.run_id||!matchesRun(receipt,doc)||!completeRun(doc))throw Error("原运行、快照与输入审计读回尚未完成");
      }
      if(!current())return false;
      this.ports.recovery.clear();this.present(result,mode);this.phaseValue="result";return true;
    }catch(error){
      if(!current()){
        if(scope()&&String(error).includes("模型运行时不可用："))this.ports.recovery.clear();
        return false;
      }
      if(scope()){
        this.error=error instanceof Error?error.message:String(error);
        // This native ModelError is raised before save_successful_run; transport errors have no such proof.
        const unavailable=this.error.includes("模型运行时不可用：");
        if(unavailable)this.ports.recovery.clear();
        this.phaseValue=dispatched&&!unavailable?"unknown":unavailable?"unavailable":this.phaseValue==="blocked"?"blocked":"failed";
      }
      throw error;
    }finally{this.locked=false;this.ports.changed();}
  }
  async inspectRoute(load:()=>Promise<RouteDecision>):Promise<RouteDecision|null>{
    if(this.locked)throw Error("当前推演流程正在处理");
    const current=this.current(),revision=++this.routeRevision;this.route=null;
    const route=await this.ports.busy(load);if(!current()||revision!==this.routeRevision)return null;this.route=route;this.ports.changed();return route;
  }
  present(payload:unknown,mode:"formal"|"shadow"|"temporary"):void{
    this.lastResult=payload&&typeof payload==="object"?{snapshot_type:this.context.snapshot,...payload as Record<string,unknown>,execution_mode:mode}:payload;this.resultMode=mode;const route=payload&&typeof payload==="object"?(payload as Record<string,unknown>).route:null;if(route&&typeof route==="object")this.route=route as RouteDecision;
  }
  async readback():Promise<void>{
    if(this.locked)throw Error("当前请求仍在处理中");
    const receipt=this.receipt;if(!receipt)throw Error("没有待核查的原请求");
    const scope=this.ports.scope(),current=this.current();this.locked=true;this.ports.changed();
    try{
      const docs=await this.ports.busy(()=>findExistingRuns(receipt,this.ports.api));
      if(!scope()||!current())return;this.candidates=docs;this.error=docs.length?"已找到相关运行，请核对实际指纹后打开；未自动认定为原请求成功。":receipt.mode==="shadow"?"影子请求不写入正式历史，当前无法由历史证明结果。":"未找到可证实的原运行；不能据此断言失败或自动重跑。";
      if(receipt.knownRunId&&docs.length===1&&completeRun(docs[0])&&matchesRun(receipt,docs[0],receipt.actualManifest===true)){
        this.context.update(receipt.command);this.ports.recovery.clear();this.present(docs[0],"formal");this.phaseValue="result";this.error=null;
      }
    }finally{this.locked=false;this.ports.changed();}
  }
  async adoptExisting(id:string):Promise<void>{
    if(this.locked)throw Error("当前流程正在处理");
    const receipt=this.receipt;if(!receipt||receipt.mode!=="formal"||!this.candidates.some(r=>r.id===id))throw Error("候选运行已失效，请重新核查");
    const scope=this.ports.scope(),current=this.current(),identity=JSON.stringify(receipt);this.locked=true;
    try{
      const doc=await this.ports.api.readRun(id);if(!scope()||!current())return;
      if(JSON.stringify(this.receipt)!==identity||!matchesRun(receipt,doc,receipt.actualManifest===true)||!completeRun(doc))throw Error("该运行尚不能证明完整结果或原请求已变化");
      this.context.update(receipt.command);this.ports.recovery.clear();this.present(doc,"formal");this.phaseValue="result";this.error="已采用核对过的既有运行；未重新发送推演请求。";
    }finally{this.locked=false;this.ports.changed();}
  }
  retryReadonlyShadow():void{
    if(this.locked||this.receipt?.mode!=="shadow")throw Error("仅可恢复不写正式历史的影子请求");
    this.ports.recovery.clear();this.phaseValue="unchecked";this.error=null;this.ports.changed();
  }
}
