import type {PostmatchMonitoringRequest,PostmatchOverview} from '../../types';
import type {PostmatchRuntime} from './postmatchPorts';
import {identity,validateOverview} from './postmatchIdentity';
export interface MonitorFields {competition_id:string;horizon:string;baseline_size:string;current_size:string}
export interface MonitoringIntent {readonly request:PostmatchMonitoringRequest;readonly current:()=>boolean}
export class MonitoringWorkflow {
  private fields:MonitorFields={competition_id:'',horizon:'T-1h',baseline_size:'100',current_size:'50'};
  private result:PostmatchOverview|null=null;private resultIdentity='';private error:string|null=null;
  private locks=new Map<symbol,{key:string;scope:()=>boolean}>();private intents=new WeakSet<MonitoringIntent>();private consumed=new WeakSet<MonitoringIntent>();
  constructor(private readonly runtime:PostmatchRuntime){}
  reset():void{this.fields={competition_id:'',horizon:'T-1h',baseline_size:'100',current_size:'50'};this.result=null;this.resultIdentity='';this.error=null;}
  private key(){return 'monitor:'+this.fields.competition_id+':'+this.fields.horizon;}
  get state(){this.runtime.enter();return {fields:this.fields,result:this.resultIdentity===identity(this.fields)?this.result:null,error:this.error,busy:[...this.locks.values()].some(l=>l.scope()&&l.key===this.key()),pending:this.runtime.recovery.capture().read(this.key())};}
  update(key:keyof MonitorFields,value:string):void{this.runtime.enter();this.fields={...this.fields,[key]:value};this.result=null;this.resultIdentity='';this.error=null;this.runtime.interactionChanged();this.runtime.changed();}
  request():PostmatchMonitoringRequest {
    const f=this.state.fields,baseline=Number(f.baseline_size),current=Number(f.current_size);
    if(!f.competition_id||!['T-N','T-24h','T-6h','T-1h'].includes(f.horizon)||![baseline,current].every(n=>Number.isSafeInteger(n)&&n>=5))throw Error('请选择赛事与正式时点，两个窗口均须为至少5的整数');
    return {competition_id:f.competition_id,horizon:f.horizon,baseline_size:baseline,current_size:current};
  }
  prepare(guard:()=>boolean=()=>true):MonitoringIntent {
    const request=this.request(),state=this.state;if(state.busy||state.pending)throw Error('原监控刷新尚在进行或结果待核实，不能重复发送');
    const current=this.runtime.current(),version=identity(this.fields),ticket={request,current:()=>current()&&guard()&&version===identity(this.fields)};this.intents.add(ticket);return ticket;
  }
  async submit(ticket:MonitoringIntent):Promise<boolean>{
    this.runtime.enter();const key=this.key(),recovery=this.runtime.recovery.capture(),scope=this.runtime.scope(),version=identity(this.fields);
    if(!this.intents.has(ticket)||this.consumed.has(ticket)||!ticket.current()||this.state.busy||recovery.read(key))throw Error('监控确认已变化或重复提交');
    this.consumed.add(ticket);const token=Symbol(key);this.locks.set(token,{key,scope});this.error=null;this.runtime.changed();
    try{
      recovery.save({kind:'monitor',key,request:ticket.request,startedAt:new Date().toISOString()});
      const result=await this.runtime.api.refreshPostmatchMonitoring(ticket.request);validateOverview(result);
      if(!scope())return false;
      if(!result.drift_runs.some(row=>row.competition_id===ticket.request.competition_id&&row.horizon===ticket.request.horizon&&row.partition_key&&row.competition_profile_id&&row.model_version_id&&row.parameter_set_id))throw Error("未读到原赛事与时点的完整分区监控回执");
      recovery.clear(key);
      if(!ticket.current())return false;this.result=result;this.resultIdentity=version;this.runtime.changed();return true;
    }catch(error){if(!ticket.current())return false;this.error='监控可能部分完成，原请求结果待核实；只读查回，禁止自动重复刷新。 '+this.runtime.formatError(error);throw error;}
    finally{this.locks.delete(token);this.runtime.changed();}
  }
  async recover():Promise<boolean>{
    this.runtime.enter();const pending=this.state.pending,current=this.runtime.current();if(pending?.kind!=='monitor')return false;
    const read=await this.runtime.refresh();if(!current())return false;
    this.error=read?'已读回现有评分与漂移报告；评分与漂移分步保存，不能据此证明原请求全部完成。原请求继续保留，需按实际账本核验。':'原监控账本读回失败，继续保留结果未知状态。';this.runtime.changed();return false;
  }
}
