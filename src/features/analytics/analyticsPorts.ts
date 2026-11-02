import type {AbilityUpdateCandidateRecord,AiAnalysisPackageSummary,AiAnalysisResponsePreview,AiAnalysisSuggestionRecord,AiSuggestionDecisionDraft,AnalyticsOverview,BackgroundJob,DataQualityDecisionDraft,DataQualityFinding,EnqueueJobDraft,ParameterTuningCandidateRecord} from '../../types';
export interface AnalysisRequest {competition_id:string|null;window_start:string|null;window_end:string|null;bucket_count:number;baseline_size:number;current_size:number}
export type AnalysisStep='history'|'model'|'quality'|'lifecycle'|'exchange'|'advanced';
export interface AnalyticsApi {
  analyticsOverview(snapshotId?:string|null):Promise<AnalyticsOverview>;
  listBackgroundJobs(limit:number):Promise<BackgroundJob[]>;
  enqueueAnalysisJob(draft:EnqueueJobDraft):Promise<BackgroundJob>;
  cancelBackgroundJob(id:string):Promise<BackgroundJob>;
  retryBackgroundJob(id:string):Promise<BackgroundJob>;
  listAiAnalysisSuggestions(status:string|null,limit:number):Promise<AiAnalysisSuggestionRecord[]>;
  listAbilityCandidates(status:string|null,limit:number,reviewId:string|null):Promise<AbilityUpdateCandidateRecord[]>;
  listParameterTuningCandidates(limit:number):Promise<ParameterTuningCandidateRecord[]>;
  chooseAiAnalysisExportFile(name:string):Promise<string|null>;
  chooseAiAnalysisResponseTemplateFile(name:string):Promise<string|null>;
  chooseAiAnalysisResponseFile():Promise<string|null>;
  exportAiAnalysisPackage(path:string,expectedSnapshotId?:string|null):Promise<AiAnalysisPackageSummary>;
  exportAiAnalysisResponseTemplate(path:string,id:string|null):Promise<string>;
  previewAiAnalysisResponse(path:string):Promise<AiAnalysisResponsePreview>;
  importAiAnalysisResponse(path:string,expected?:AiAnalysisResponsePreview|null):Promise<AiAnalysisSuggestionRecord[]>;
  decideAiAnalysisSuggestion(draft:AiSuggestionDecisionDraft):Promise<AiAnalysisSuggestionRecord>;
  decideDataQualityFinding(draft:DataQualityDecisionDraft):Promise<DataQualityFinding>;
}
export interface AnalysisExport {summary:AiAnalysisPackageSummary;snapshotId:string}
export type AnalysisReceipt=
 |{kind:'enqueue';key:string;draft:EnqueueJobDraft;startedAt:string}
 |{kind:'job';key:string;operation:'cancel'|'retry';job:BackgroundJob;startedAt:string}
 |{kind:'import';key:string;path:string;preview:AiAnalysisResponsePreview;startedAt:string}
 |{kind:'suggestion';key:string;id:string;responseId:string;decision:'accept'|'reject';startedAt:string}
 |{kind:'quality';key:string;id:string;scanId:string;decision:'resolve'|'ignore';startedAt:string};
export interface AnalyticsStorageScope {read(key:string):AnalysisReceipt|null;receipts():AnalysisReceipt[];save(receipt:AnalysisReceipt):void;clear(key:string):void;exported():AnalysisExport|null;saveExport(value:AnalysisExport):void}
export interface AnalyticsStorage {capture():AnalyticsStorageScope}
export interface AnalyticsPorts {
 api:AnalyticsApi;storage:AnalyticsStorage;scope():()=>boolean;view():()=>boolean;changed():void;formatError(e:unknown):string;
 historyCount():number|null;initialStep():string|null;saveStep(step:AnalysisStep):void;
}
export interface AnalyticsRuntime extends AnalyticsPorts {
 enter():void;current(interaction?:boolean):()=>boolean;interactionChanged():void;
 refresh():Promise<boolean>;overview():AnalyticsOverview|null;jobs():readonly BackgroundJob[];
 suggestions():readonly AiAnalysisSuggestionRecord[];qualityReady():boolean;analysisReady():boolean;historyReady():boolean;
 request():AnalysisRequest;
}
