import {AsyncScope} from '../../app/lifecycle';
import {redactLogText} from '../../platform/diagnostics';
import type {IssueLogEntry} from '../../types';
import type {LogSection,LogState,LogsPorts} from './logContracts';
import {LogOperations} from './LogOperations';

/** Device-local history has one query owner and never depends on the business database. */
export class LogsFeature {
  readonly state:LogState;
  readonly operations:LogOperations;
  private readonly reads=new AsyncScope();
  private closed=false;
  constructor(readonly ports:LogsPorts){
    const saved=ports.storage.read();
    this.state={entries:null,loading:false,readError:null,readAt:null,section:saved.section,expanded:new Set(saved.expanded),operation:null,status:null,unknownClear:saved.unknownClear,checkedUnknown:false,operationId:saved.operationId,storageError:null};
    this.operations=new LogOperations(this);
  }
  alive=()=>!this.closed;
  changed(){if(this.alive())this.ports.changed();}
  persist():boolean {
    const s=this.state;
    try{this.ports.storage.save({section:s.section,expanded:[...s.expanded],unknownClear:s.unknownClear,operationId:s.operationId});s.storageError=null;return true;}
    catch{s.storageError='本地操作恢复记录未能保存。请检查本地存储，未知操作仍需核验。';return false;}
  }
  publish(entries:IssueLogEntry[]){this.state.entries=structuredClone(entries);this.state.readAt=new Date().toISOString();this.state.readError=null;}
  async refresh(view=this.ports.view()):Promise<boolean>{
    const request=this.reads.begin(),s=this.state;s.loading=true;s.readError=null;this.changed();
    try{const entries=await this.ports.api.listIssueLogs(500);if(!request.isCurrent()||!view()||!this.alive())return false;this.publish(entries);if(s.unknownClear)s.checkedUnknown=true;return true;}
    catch(error){if(request.isCurrent()&&view()&&this.alive())s.readError=redactLogText(error instanceof Error?error.message:String(error));return false;}
    finally{if(request.isCurrent()){s.loading=false;this.changed();}request.finish();}
  }
  section(section:LogSection){this.state.section=section;this.persist();this.changed();}
  toggle(id:string){if(!this.state.entries?.some(row=>row.id===id))return;const expanded=this.state.expanded;expanded.has(id)?expanded.delete(id):expanded.add(id);this.persist();this.changed();}
  invalidateReads(){this.reads.invalidate();this.state.loading=false;}
  navigationChanged(){this.invalidateReads();}
  stop(){this.closed=true;this.reads.destroy();}
}
