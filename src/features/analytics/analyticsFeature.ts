import type {AnalyticsPorts,AnalyticsRuntime,AnalysisStep} from './analyticsPorts';
import {analysisRequest,globalSnapshot,matchingJobOverview,timestampIdentity} from './analyticsIdentity';
import {AnalyticsQuery} from './analyticsQuery';
import {AnalyticsJobs} from './analyticsJobs';
import {AnalyticsExchange} from './analyticsExchange';
import {AnalyticsReviews} from './analyticsReviews';
/** Coordinates view identity; domain writes and snapshots keep their distinct owners. */
export class AnalyticsFeature {
 private valid=()=>false;private navigationRevision=0;private interactionRevision=0;
 private stage:AnalysisStep='history';private jobId:string|null=null;
 readonly query:AnalyticsQuery;readonly jobs:AnalyticsJobs;readonly exchange:AnalyticsExchange;readonly reviews:AnalyticsReviews;
 constructor(private readonly ports:AnalyticsPorts){
  const runtime:AnalyticsRuntime={...ports,enter:()=>this.enter(),current:i=>this.capture(i),interactionChanged:()=>this.interactionChanged(),refresh:()=>this.refresh(),overview:()=>this.overview,jobs:()=>this.state.jobs,suggestions:()=>this.state.suggestions,qualityReady:()=>this.gates.quality,analysisReady:()=>this.gates.analysis,historyReady:()=>this.gates.history,request:()=>this.request};
  this.query=new AnalyticsQuery(runtime);this.jobs=new AnalyticsJobs(runtime);this.exchange=new AnalyticsExchange(runtime);this.reviews=new AnalyticsReviews(runtime);
 }
 private enter(){
  if(this.valid())return;this.valid=this.ports.scope();this.interactionRevision++;this.jobId=null;
  const requested=this.ports.initialStep()?.replace(/^analysis-|\-step$/g,'');
  this.stage=['history','model','quality','lifecycle','exchange','advanced'].includes(requested??'')?requested as AnalysisStep:'history';
  this.query.reset();this.jobs.reset();this.exchange.reset();this.reviews.reset();
 }
 get state(){this.enter();return this.query.state;}
 get step(){this.enter();return this.stage;}
 get selectedJob(){return this.state.jobs.find(j=>j.id===this.jobId)??null;}
 get selectedJobId(){this.enter();return this.jobId;}
 get overview(){
  if(this.jobId&&!this.selectedJob)return null;
  const source=this.selectedJob?.result?.source_snapshot as {id?:string}|undefined;
  return matchingJobOverview(source?.id?this.query.snapshot(source.id)??this.state.overview:this.state.overview,this.selectedJob);
 }
 get request(){const job=this.selectedJob;return analysisRequest(job&&['refresh_analytics','full_analysis_refresh'].includes(job.job_type)?job.payload:{});}
 get pendingReceipts(){this.enter();return this.ports.storage.capture().receipts();}
 get gates(){
  const state=this.state,overview=this.overview,history=this.ports.historyCount(),quality=overview?.data_quality;
  const analysis=history!==null&&history>0&&!!overview?.generated_at&&overview.sample_size>0&&!!overview.source_snapshot?.id&&!!overview.source_snapshot.request;
  let scanCurrent=false;try{scanCurrent=!!quality?.scan_id&&!!quality.generated_at&&!!overview?.generated_at&&timestampIdentity(quality.generated_at)>=timestampIdentity(overview.generated_at);}catch{/* Unverified timestamp never grants a quality gate. */}
  const snapshotId=overview?.source_snapshot?.id;
  const clean=analysis&&state.loaded&&!state.loading&&!state.error&&(!snapshotId||!this.query.snapshotError(snapshotId))&&globalSnapshot(overview)&&scanCurrent&&quality?.critical===0;
  const undecided=this.pendingReceipts.some(r=>['import','suggestion','quality'].includes(r.kind));
  return {history:history!==null&&history>0,analysis,quality:clean,review:clean&&state.loaded&&!undecided&&state.suggestions.length+state.abilityCandidates.length===0,scanCurrent,global:globalSnapshot(overview)};
 }
 capture(interaction=true){const scope=this.ports.scope(),view=this.ports.view(),nav=this.navigationRevision,revision=this.interactionRevision;return()=>scope()&&view()&&nav===this.navigationRevision&&(!interaction||revision===this.interactionRevision);}
 interactionChanged(){this.interactionRevision++;}
 setStep(step:AnalysisStep){this.enter();if(!['history','model','quality','lifecycle','exchange','advanced'].includes(step))throw Error('分析步骤不存在');this.interactionChanged();this.stage=step;this.ports.saveStep(step);this.ports.changed();}
 async selectJob(id:string|null){this.enter();if(id&&!this.state.jobs.some(j=>j.id===id&&['refresh_analytics','full_analysis_refresh','data_quality_scan','query_performance_scan'].includes(j.job_type)))throw Error('任务不属于当前分析目录');this.jobId=id;this.interactionChanged();this.ports.changed();await this.readSelectedSnapshot();}
 navigationChanged(){this.navigationRevision++;this.interactionChanged();this.query.navigationChanged();}
 private async readSelectedSnapshot(){const job=this.selectedJob,source=job?.result?.source_snapshot as {id?:string}|undefined;if(job&&source?.id)await this.query.loadSnapshot(source.id,job.payload);}
 async refresh(){const loaded=await this.query.refresh();if(loaded)await this.readSelectedSnapshot();return loaded;}
}
