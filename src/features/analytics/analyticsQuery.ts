import type {AbilityUpdateCandidateRecord,AiAnalysisSuggestionRecord,AnalyticsOverview,BackgroundJob,ParameterTuningCandidateRecord} from '../../types';
import type {AnalyticsRuntime} from './analyticsPorts';
import {requestIdentity,validateJob,validateOverview} from './analyticsIdentity';
const initial=()=>({overview:null as AnalyticsOverview|null,jobs:[] as BackgroundJob[],suggestions:[] as AiAnalysisSuggestionRecord[],abilityCandidates:[] as AbilityUpdateCandidateRecord[],tuningCandidates:[] as ParameterTuningCandidateRecord[],loaded:false,loading:false,error:null as string|null});
/** The displayed directory is one acknowledged read batch; failure retains its prior provenance. */
export class AnalyticsQuery {
 private data=initial();private serial=0;private snapshots=new Map<string,AnalyticsOverview>();private snapshotErrors=new Map<string,string>();
 constructor(private readonly runtime:AnalyticsRuntime){}
 get state():Readonly<ReturnType<typeof initial>>{this.runtime.enter();return this.data;}
 reset(){this.data=initial();this.serial++;this.snapshots.clear();this.snapshotErrors.clear();}
 snapshot(id:string){this.runtime.enter();return this.snapshots.get(id)??null;}
 snapshotError(id:string){return this.snapshotErrors.get(id)??null;}
 async loadSnapshot(id:string,request:Record<string,unknown>):Promise<boolean>{
  const current=this.runtime.current(false),selected=this.runtime.current();
  try{const overview=await this.runtime.api.analyticsOverview(id);if(!current())return false;validateOverview(overview);
   if(overview.source_snapshot?.id!==id||!overview.source_snapshot.request||requestIdentity(overview.source_snapshot.request)!==requestIdentity(request))throw Error('返回的分析快照与原任务范围不一致');
   this.snapshots.set(id,overview);this.snapshotErrors.delete(id);return true;
  }catch(error){if(current())this.snapshotErrors.set(id,this.runtime.formatError(error));return false;}finally{if(selected())this.runtime.changed();}
 }
 navigationChanged(){this.serial++;this.data.loading=false;}
 async refresh():Promise<boolean>{
  this.runtime.enter();const current=this.runtime.current(false),scope=this.runtime.scope(),serial=++this.serial;this.data.loading=true;this.data.error=null;this.runtime.changed();
  try{
   const [overview,jobs,suggestions,abilityCandidates,tuningCandidates]=await Promise.all([this.runtime.api.analyticsOverview(),this.runtime.api.listBackgroundJobs(100),this.runtime.api.listAiAnalysisSuggestions('pending',500),this.runtime.api.listAbilityCandidates('pending',500,null),this.runtime.api.listParameterTuningCandidates(100)]);
   if(!current()||serial!==this.serial)return false;
   validateOverview(overview);if(![jobs,suggestions,abilityCandidates,tuningCandidates].every(Array.isArray))throw Error('分析目录读取不完整');jobs.forEach(validateJob);
   this.data={overview,jobs,suggestions,abilityCandidates,tuningCandidates,loaded:true,loading:false,error:null};return true;
  }catch(error){if(current()&&serial===this.serial)this.data.error='分析刷新未完成，保留上次来源与数据。 '+this.runtime.formatError(error);return false;}
  finally{if(scope()&&serial===this.serial){this.data.loading=false;this.runtime.changed();}}
 }
}
