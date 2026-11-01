import type { PredictionContext } from "../predictionContext";
import type { P4FreezeTaskRecord, P4MatchWorkspace, P4TaskWorkspace, PlanP4HorizonsCommand, ResolveP4ConflictCommand } from "../../../types";
import { decisionRecorded, researchVersion, validateDecision, validateResearchTask, verifiedResearchSnapshot } from "./researchDecision";
import type { ResearchDecisionIntent } from "./researchDecision";
import type { ResearchReceipt, ResearchRecovery } from "./researchRecovery";

export interface ResearchPorts {
  api:{readP4MatchWorkspace(id:string):Promise<P4MatchWorkspace>;readP4TaskWorkspace(id:string):Promise<P4TaskWorkspace>;
    planP4Horizons(command:PlanP4HorizonsCommand):Promise<P4FreezeTaskRecord[]>;
    resolveP4Conflict(command:ResolveP4ConflictCommand):Promise<P4TaskWorkspace>;};
  context:PredictionContext; scope():()=>boolean; view():()=>boolean; changed():void; recovery:ResearchRecovery;
}
/** Research owns query, task selection and mutation receipts; the shared context owns the match. */
export class ResearchFeature {
  private matchValue:P4MatchWorkspace|null=null;
  private taskValue:P4TaskWorkspace|null=null;
  private revision=0;
  private draftRevision=0;
  private dataScope=()=>true;
  private dataContext=()=>true;
  private locked=false;
  loading=false;
  error:string|null=null;
  message:string|null=null;
  constructor(private readonly ports:ResearchPorts){}
  get matchWorkspace():P4MatchWorkspace|null{return this.dataScope()&&this.dataContext()&&this.matchValue?.match_id===this.ports.context.matchId?this.matchValue:null;}
  get taskWorkspace():P4TaskWorkspace|null{return this.matchWorkspace&&this.taskValue?.task.match_id===this.matchWorkspace.match_id?this.taskValue:null;}
  get receipt():ResearchReceipt|null{return this.ports.recovery.read();}
  get busy():boolean{return this.locked;}
  get view(){return {matchId:this.ports.context.matchId,loading:this.loading,busy:this.busy,error:this.dataScope()&&this.dataContext()?this.error:null,message:this.dataScope()&&this.dataContext()?this.message:null,receipt:this.receipt};}
  private capture():()=>boolean {const revision=++this.revision,context=this.ports.context.capture(),view=this.ports.view(),scope=this.ports.scope();return()=>revision===this.revision&&context()&&view()&&scope();}
  async load(matchId:string|null,preferredTaskId:string|null=this.taskWorkspace?.task.id??null):Promise<void>{
    this.ports.context.matchId=matchId;
    const current=this.capture();this.dataScope=this.ports.scope();this.dataContext=this.ports.context.capture();
    this.matchValue=null;this.taskValue=null;this.error=null;this.message=null;this.loading=!!matchId;this.ports.changed();
    if(!matchId)return;
    try{
      const match=await this.ports.api.readP4MatchWorkspace(matchId);if(!current())return;
      if(match.match_id!==matchId||match.tasks.some(t=>t.match_id!==matchId))throw Error("研究比赛返回身份不匹配");
      const taskId=match.tasks.find(t=>t.id===preferredTaskId)?.id??match.tasks[0]?.id;
      const task=taskId?await this.ports.api.readP4TaskWorkspace(taskId):null;if(!current())return;
      if(task)validateResearchTask(task,matchId,taskId!);
      this.matchValue=match;this.taskValue=task;
    }catch(error){if(!current())return;this.error=String(error);throw error;}
    finally{if(current()){this.loading=false;this.ports.changed();}}
  }
  async openTask(taskId:string):Promise<void>{
    const match=this.matchWorkspace;if(!match||!match.tasks.some(t=>t.id===taskId))throw Error("任务不属于当前比赛，请刷新目录");
    const current=this.capture();this.taskValue=null;this.loading=true;this.error=null;this.ports.changed();
    try{const task=await this.ports.api.readP4TaskWorkspace(taskId);if(!current())return;validateResearchTask(task,match.match_id,taskId);this.taskValue=task;}
    catch(error){if(!current())return;this.error=String(error);throw error;}
    finally{if(current()){this.loading=false;this.ports.changed();}}
  }
  prepareDecision(conflictId:string,acceptUnknown:boolean,evidenceId:string|null,note:string|null):ResearchDecisionIntent {
    if(this.locked||this.receipt)throw Error("有研究操作尚未核实，请先查回原任务");
    const task=this.taskWorkspace;if(!task)throw Error("请先选择研究任务");
    const command:ResolveP4ConflictCommand={task_id:task.task.id,conflict_id:conflictId,decision_kind:acceptUnknown?"accept_unknown":"select_evidence",selected_evidence_ids:acceptUnknown?[]:evidenceId?[evidenceId]:[],note:note?.trim()||null};
    validateDecision(task,command);
    return {matchId:task.task.match_id,taskId:task.task.id,version:researchVersion(task),command};
  }
  draftChanged():void{this.draftRevision++;}
  captureDraft():()=>boolean{const revision=this.draftRevision,current=this.ports.view();return()=>revision===this.draftRevision&&current();}
  private available():void{if(this.locked||this.receipt)throw Error("研究操作正在处理或结果尚未核实，禁止重复提交");}
  async decide(intent:ResearchDecisionIntent,confirmationCurrent:()=>boolean):Promise<boolean>{
    this.available();const selected=this.taskWorkspace;
    if(!selected||selected.task.id!==intent.taskId||selected.task.match_id!==intent.matchId||researchVersion(selected)!==intent.version||!confirmationCurrent())throw Error("确认已失效，请重新核对原任务");
    const current=this.capture(),scope=this.ports.scope();this.locked=true;this.error=null;this.ports.changed();let dispatched=false;
    try{
      const fresh=await this.ports.api.readP4TaskWorkspace(intent.taskId);
      if(!current()||!confirmationCurrent())return false;
      validateResearchTask(fresh,intent.matchId,intent.taskId);
      if(researchVersion(fresh)!==intent.version)throw Error("任务版本或证据已变化，请刷新并重新确认");
      validateDecision(fresh,intent.command);
      this.ports.recovery.save({kind:"decision",matchId:intent.matchId,intent:structuredClone(intent)});dispatched=true;
      const result=await this.ports.api.resolveP4Conflict(intent.command);
      validateResearchTask(result,intent.matchId,intent.taskId);
      if(!decisionRecorded(result,intent.command))throw Error("返回记录尚未证实原决策，请查回原任务");
      if(!scope())return false;
      this.ports.recovery.clear();if(!current())return false;
      this.taskValue=result;this.message="决策已记录；冻结状态以实际任务与快照为准。";return true;
    }catch(error){if(!current())return false;this.error=dispatched?"请求结果尚未核实，请先查回原任务。 "+String(error):String(error);throw error;}
    finally{this.locked=false;this.ports.changed();}
  }
  async plan(rulePackageId:string):Promise<boolean>{
    this.available();const match=this.matchWorkspace;
    if(!match||!rulePackageId||!(Date.now()<Date.parse(match.kickoff_at)))throw Error("请选择尚未开赛的实际比赛和有效规则包");
    const current=this.capture(),scope=this.ports.scope();
    const receipt:ResearchReceipt={kind:"plan",matchId:match.match_id,kickoff:match.kickoff_at,command:{match_id:match.match_id,explicit_rule_package_id:rulePackageId,requested_fact_keys:[]}};
    this.locked=true;this.error=null;this.ports.changed();let dispatched=false;
    try{
      this.ports.recovery.save(receipt);dispatched=true;
      await this.ports.api.planP4Horizons(receipt.command);
      if(!scope())return false;
      const actual=await this.ports.api.readP4MatchWorkspace(receipt.matchId);
      if(!scope())return false;
      if(!this.planned(actual,receipt))throw Error("计划窗口尚未完整核实，请查回原比赛");
      this.ports.recovery.clear();if(!current())return false;
      await this.load(receipt.matchId);this.message="三个实际计划窗口已查回；冻结由后台执行。";return true;
    }catch(error){if(!current())return false;this.error=dispatched?"计划请求结果尚未核实，请先查回原任务。 "+String(error):String(error);throw error;}
    finally{this.locked=false;this.ports.changed();}
  }
  private planned(match:P4MatchWorkspace,receipt:Extract<ResearchReceipt,{kind:"plan"}>):boolean{
    return match.match_id===receipt.matchId&&["T-24h","T-6h","T-1h"].every(h=>match.tasks.some(t=>t.match_id===receipt.matchId&&t.horizon===h&&t.rule_package_id===receipt.command.explicit_rule_package_id&&t.kickoff_at===receipt.kickoff));
  }
  async recover():Promise<boolean>{
    if(this.locked)throw Error("研究操作仍在处理，请等待当前请求结束");
    const receipt=this.receipt;if(!receipt)return false;
    this.ports.context.matchId=receipt.matchId;const current=this.capture(),scope=this.ports.scope();this.dataScope=scope;this.dataContext=this.ports.context.capture();
    this.locked=true;this.error=null;this.ports.changed();
    try{
      const match=await this.ports.api.readP4MatchWorkspace(receipt.matchId);if(!scope())return false;
      if(match.match_id!==receipt.matchId)throw Error("原比赛身份不匹配");
      let task:P4TaskWorkspace|null=null,recorded=false;
      if(receipt.kind==="decision"){
        task=await this.ports.api.readP4TaskWorkspace(receipt.intent.taskId);if(!scope())return false;
        validateResearchTask(task,receipt.matchId,receipt.intent.taskId);recorded=decisionRecorded(task,receipt.intent.command);
      }else recorded=this.planned(match,receipt);
      if(recorded)this.ports.recovery.clear();if(!current())return false;
      this.matchValue=match;this.taskValue=task;this.loading=false;
      this.message=recorded?"原操作已查回并核实；研究门禁与冻结结果分别展示。":"尚未查到完整的原操作记录；保留恢复信息，不能据此认定未写入或直接重发。";return recorded;
    }catch(error){if(!current())return false;this.error="查回失败，原请求仍待核实。 "+String(error);throw error;}
    finally{this.locked=false;this.ports.changed();}
  }
  snapshot():Record<string,unknown>{const task=this.taskWorkspace;if(!task||!verifiedResearchSnapshot(task))throw Error("实际冻结状态与快照身份尚未核实");return task.snapshot!;}
}
