import type {ReleaseAcceptanceRun} from '../../types';
import type {ReleaseIntent,ReleasePorts} from './releasePorts';
import {ReleaseQuery} from './releaseQuery';
import {releaseRequest,releaseDraftError} from './releaseDraft';
import {assertReleaseRequest,assertReleaseReport,sameReleaseReport} from './releaseReport';
export class ReleaseFeature {
 readonly query:ReleaseQuery;private valid=()=>false;private navigation=0;private revision=0;
 private issued=new WeakSet<object>();private consumed=new WeakSet<object>();private locks=new Map<symbol,()=>boolean>();
 private error:string|null=null;private phase:'idle'|'running'|'failed'|'unknown'|'complete'='idle';private result:ReleaseAcceptanceRun|null=null;private recoveryIds:string[]=[];
 constructor(readonly ports:ReleasePorts){this.query=new ReleaseQuery(ports,()=>this.capture(false));}
 private enter(){if(this.valid())return;this.valid=this.ports.scope();this.revision++;this.error=null;this.phase='idle';this.result=null;this.recoveryIds=[];this.query.reset();}
 capture(interaction=true){this.enter();const scope=this.ports.scope(),view=this.ports.view(),nav=this.navigation,rev=this.revision;return()=>scope()&&view()&&nav===this.navigation&&(!interaction||rev===this.revision);}
 navigationChanged(){this.navigation++;}
 get state(){this.enter();const storage=this.ports.storage.capture(),draft=storage.draft(),pending=storage.pending();return {...this.query.state,draft,pending,error:this.error,phase:pending&&!this.busy?'unknown':this.phase,result:this.result,recoveryIds:this.recoveryIds,busy:this.busy,validation:releaseDraftError(draft)};}
 private get busy(){return [...this.locks.values()].some(current=>current());}
 update(field:string,value:string){this.enter();const storage=this.ports.storage.capture(),draft=storage.draft();if(!(field in draft))throw Error('未知验收字段');if(this.busy)throw Error('原验收请求正在处理');storage.saveDraft({...draft,[field]:value});this.revision++;}
 prepare():ReleaseIntent {const s=this.state;if(s.busy||s.pending)throw Error('原验收请求正在处理或结果未知，请先查回');const ticket={request:releaseRequest(s.draft),current:this.capture()};this.issued.add(ticket);return ticket;}
 async run(ticket:ReleaseIntent):Promise<boolean>{
  if(!this.issued.has(ticket)||this.consumed.has(ticket)||!ticket.current()||this.state.busy||this.state.pending)throw Error('验收确认已过期或重复发送');this.consumed.add(ticket);
  const storage=this.ports.storage.capture(),scope=this.ports.scope(),selection=this.query.selection,lock=Symbol();this.locks.set(lock,scope);this.phase='running';this.error=null;this.result=null;this.recoveryIds=[];this.ports.changed();let sent=false;
  try{const before=await this.ports.api.listReleaseAcceptanceRuns(50);if(!ticket.current())return false;if(!Array.isArray(before))throw Error('原验收目录未核实，不发送请求');
   const receipt={request:structuredClone(ticket.request),beforeIds:before.map(row=>row.id),startedAt:new Date().toISOString()};storage.savePending(receipt);sent=true;
   const report=await this.ports.api.runReleaseAcceptance(structuredClone(ticket.request));assertReleaseRequest(report,ticket.request);if(receipt.beforeIds.includes(report.id))throw Error('新请求返回了旧历史报告，结果待核实');
   storage.savePending({...receipt,report:{id:report.id,hash:report.report_sha256}});
   if(!scope()||!ticket.current())return false;
   const readback=await this.ports.api.readReleaseAcceptanceRun(report.id);if(!ticket.current())return false;assertReleaseReport(readback,report.id);if(!sameReleaseReport(report,readback))throw Error('验收账本读回与原回执不一致');
   this.query.publish(readback,selection);this.result=structuredClone(readback);storage.clearPending();this.phase='complete';await this.query.list();return ticket.current();
  }catch(error){if(ticket.current()){this.error=this.ports.formatError(error);this.phase=sent?'unknown':'failed';}return false;}finally{this.locks.delete(lock);if(scope())this.ports.changed();}
 }
 async recover():Promise<boolean>{const state=this.state;if(state.busy||!state.pending)return false;const current=this.capture(false),receipt=state.pending,selection=this.query.selection,storage=this.ports.storage.capture();this.error=null;
  try{if(receipt.report){const run=await this.ports.api.readReleaseAcceptanceRun(receipt.report.id);if(!current())return false;assertReleaseRequest(run,receipt.request);if(run.report_sha256!==receipt.report.hash)throw Error('原报告指纹尚未核实');this.query.publish(run,selection);this.result=structuredClone(run);storage.clearPending();this.phase='complete';await this.query.list();return current();}
   const rows=await this.ports.api.listReleaseAcceptanceRuns(50);if(!current())return false;this.recoveryIds=rows.filter(row=>!receipt.beforeIds.includes(row.id)&&Date.parse(row.completed_at)>=Date.parse(receipt.startedAt)).map(row=>row.id);this.error='原请求未返回报告ID；可查看新报告，但现有接口不能证明其属于本次请求，保持未知且不重发。';return false;
  }catch(error){if(current())this.error=this.ports.formatError(error);return false;}finally{if(current())this.ports.changed();}
 }
 async refresh(){this.enter();return this.query.list();}
 async select(id:string){this.enter();return this.query.select(id);}
 report(){const run=this.state.selected;if(!run||this.state.detailLoading||this.state.detailError)throw Error('请先核实所选报告');return structuredClone(run);}
 evidence(id:string){const run=this.report(),item=run.checks.find(row=>row.id===id);if(!item||item.run_id!==run.id)throw Error('检查证据与报告身份不一致');return {report_id:run.id,report_sha256:run.report_sha256,check:structuredClone(item)};}
}
