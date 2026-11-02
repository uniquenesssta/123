import type {AiAnalysisResponsePreview} from '../../types';
import type {AnalyticsRuntime} from './analyticsPorts';
import {identity} from './analyticsIdentity';
const suggestionKeys=(rows:readonly {suggestion_type:string;title:string;summary:string;severity?:string;scope:unknown;payload:unknown;evidence:unknown}[])=>rows.map(r=>identity([r.suggestion_type,r.title,r.summary,r.severity?.trim()?r.severity:'info',r.scope,r.payload,r.evidence])).sort();
export interface AnalysisImportIntent {path:string;preview:AiAnalysisResponsePreview;current:()=>boolean}
/** Native file choices, source packages and confirmed semantic previews stay bound to the original database. */
export class AnalyticsExchange {
 private preview:AiAnalysisResponsePreview|null=null;private path:string|null=null;private error:string|null=null;private serial=0;
 private locks=new Map<symbol,()=>boolean>();private issued=new WeakSet<AnalysisImportIntent>();private consumed=new WeakSet<AnalysisImportIntent>();
 constructor(private readonly runtime:AnalyticsRuntime){}
 reset(){this.preview=null;this.path=null;this.error=null;this.serial++;}
 get state(){this.runtime.enter();const storage=this.runtime.storage.capture(),exported=storage.exported();return {preview:this.preview,path:this.path,error:this.error,exported,busy:[...this.locks.values()].some(s=>s()),pending:exported?storage.read('analysis:import:'+exported.summary.package_id):null};}
 private source(preview:AiAnalysisResponsePreview):void {
  const exported=this.state.exported;
  if(!exported||preview.manifest.format_version!=='football.ai-analysis-response.v1'||preview.manifest.source_package_id!==exported.summary.package_id||!preview.manifest.response_id||!preview.manifest.content_sha256)throw Error('回包来源或格式不匹配当前数据库已确认的分析包');
  if(!Array.isArray(preview.suggestions)||!Array.isArray(preview.blocking_errors)||!Array.isArray(preview.warnings))throw Error('回包预检不完整');
 }
 private begin(){if(this.state.busy)throw Error('文件操作正在进行');const key=Symbol(),scope=this.runtime.scope();this.locks.set(key,scope);this.error=null;this.runtime.changed();return()=>{this.locks.delete(key);this.runtime.changed();};}
 async exportPackage(guard:()=>boolean=()=>true):Promise<boolean>{
  this.runtime.enter();if(this.runtime.storage.capture().receipts().some(r=>r.kind==='import'))throw Error('原回包导入结果待核实，先核对原回包再导出下一包');if(!this.runtime.qualityReady())throw Error('请先完成数据质量扫描并清除全部严重问题');
  const source=this.runtime.overview()?.source_snapshot;if(!source)throw Error('分析来源尚未核实');
  const current=this.runtime.current(),storage=this.runtime.storage.capture(),scope=this.runtime.scope(),done=this.begin();
  try{
   const file=await this.runtime.api.chooseAiAnalysisExportFile('足球模型智能分析资料_'+new Date().toISOString().slice(0,10)+'.zip');if(!file||!current()||!guard())return false;
   if(!await this.runtime.refresh()||!current()||!guard())return false;
   if(!this.runtime.qualityReady()||this.runtime.overview()?.source_snapshot?.id!==source.id)throw Error('导出前的分析或质量来源已变化，请重新核对');
   const summary=await this.runtime.api.exportAiAnalysisPackage(file,source.id);if(!scope())return false;
   if(!summary.package_id||!summary.content_sha256||summary.output_path!==file)throw Error('导出回执不完整');
   storage.saveExport({summary,snapshotId:source.id});if(!current())return false;this.preview=null;this.path=null;this.serial++;return true;
  }catch(error){if(current())this.error='导出未核实；如文件已生成，请核对原文件与导出记录。 '+this.runtime.formatError(error);throw error;}finally{done();}
 }
 async template():Promise<boolean>{
  const exported=this.state.exported;if(!exported)throw Error('请先导出本轮智能分析资料包');const current=this.runtime.current(),done=this.begin();
  try{const file=await this.runtime.api.chooseAiAnalysisResponseTemplateFile('足球模型建议填写模板.zip');if(!file||!current())return false;await this.runtime.api.exportAiAnalysisResponseTemplate(file,exported.summary.package_id);return current();}catch(error){if(current())this.error=this.runtime.formatError(error);throw error;}finally{done();}
 }
 async chooseResponse():Promise<boolean>{
  if(!this.state.exported)throw Error('请先导出分析资料并生成对应建议模板');if(this.state.pending)throw Error('原回包导入结果未知，请先核对原回包');
  const current=this.runtime.current(),done=this.begin(),serial=++this.serial;
  try{
   const file=await this.runtime.api.chooseAiAnalysisResponseFile();if(!current()||serial!==this.serial||!file)return false;
   this.path=file;this.preview=null;this.runtime.changed();
   const preview=await this.runtime.api.previewAiAnalysisResponse(file);if(!current()||serial!==this.serial)return false;this.source(preview);this.preview=preview;return true;
  }catch(error){if(current()&&serial===this.serial)this.error=this.runtime.formatError(error);return false;}finally{done();}
 }
 prepareImport():AnalysisImportIntent {
  const state=this.state;if(!state.preview||!state.path||state.busy||state.pending)throw Error('没有可确认导入的回包，或原请求待核实');this.source(state.preview);
  if(state.preview.blocking_errors.length)throw Error('智能分析建议文件仍有必须先处理的问题');
  const revision=this.serial,current=this.runtime.current(),ticket={path:state.path,preview:structuredClone(state.preview),current:()=>current()&&revision===this.serial};this.issued.add(ticket);return ticket;
 }
 async importResponse(ticket:AnalysisImportIntent):Promise<boolean>{
  const state=this.state;if(!this.issued.has(ticket)||this.consumed.has(ticket)||!ticket.current()||state.busy||state.pending)throw Error('回包确认已失效或重复发送');
  this.consumed.add(ticket);const current=ticket.current,scope=this.runtime.scope(),storage=this.runtime.storage.capture(),done=this.begin(),key='analysis:import:'+ticket.preview.manifest.source_package_id;let sent=false;
  try{
   const fresh=await this.runtime.api.previewAiAnalysisResponse(ticket.path);if(!current())return false;this.source(fresh);
   if(identity(fresh)!==identity(ticket.preview))throw Error('回包文件在预检后已变化，请重新预检并确认');
   storage.save({kind:'import',key,path:ticket.path,preview:ticket.preview,startedAt:new Date().toISOString()});sent=true;
   const rows=await this.runtime.api.importAiAnalysisResponse(ticket.path,ticket.preview);if(!scope())return false;
   if(!Array.isArray(rows)||identity(suggestionKeys(rows.filter(r=>r.response_id===ticket.preview.manifest.response_id)))!==identity(suggestionKeys(ticket.preview.suggestions)))throw Error('尚未读到原回包的完整建议目录，结果待核实');
   storage.clear(key);if(!current())return false;this.preview=null;this.path=null;this.serial++;await this.runtime.refresh();return true;
  }catch(error){if(!current())return false;this.error=(sent?'原回包导入结果未知，禁止盲目再导入。 ':'')+this.runtime.formatError(error);throw error;}finally{done();}
 }
 async recoverImport(key?:string):Promise<boolean>{
  const current=this.runtime.current(),storage=this.runtime.storage.capture(),receipt=key?storage.read(key):this.state.pending;if(receipt?.kind!=='import'||this.state.busy)return false;const done=this.begin();
  try{
   const rows=await this.runtime.api.listAiAnalysisSuggestions(null,500);if(!current())return false;
   const original=rows.filter(row=>row.response_id===receipt.preview.manifest.response_id);
   if(!receipt.preview.suggestions.length||identity(suggestionKeys(original))!==identity(suggestionKeys(receipt.preview.suggestions)))throw Error('目录尚不能证明原回包完整导入；继续保留未知，不重复导入');
   storage.clear(receipt.key);this.error=null;this.preview=null;this.path=null;this.serial++;await this.runtime.refresh();return current();
  }catch(error){if(current())this.error=this.runtime.formatError(error);return false;}finally{done();}
 }
}
