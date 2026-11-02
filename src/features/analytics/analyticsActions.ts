import type {EnqueueJobDraft} from '../../types';
import type {AnalyticsFeature} from './analyticsFeature';
import {identity} from './analyticsIdentity';
export interface AnalyticsActionPorts {
 confirm(title:string,description:string,facts:Array<[string,string]>,label:string,run:()=>Promise<void>):void;
 show(title:string,value:unknown):void;notify(text:string):void;
}
/** Translate protected UI intentions; operation owners keep their own identities and recovery receipts. */
export class AnalyticsActions {
 constructor(private readonly feature:AnalyticsFeature,private readonly ports:AnalyticsActionPorts){}
 enqueue(type:EnqueueJobDraft['job_type']){
  const ticket=this.feature.jobs.prepareEnqueue(type),request=ticket.draft!.payload;
  this.ports.confirm('确认创建分析任务','入队只产生后台任务；任务成功与有效快照分别核实，不自动改参。',[
   ['任务类型',type],['赛事',String(request.competition_id??'全库范围')],['原幂等键',ticket.draft!.idempotency_key!],
  ],'确认入队',async()=>{const job=await this.feature.jobs.submit(ticket);if(job)this.ports.notify('任务已入队：'+job.id);});
 }
 job(id:string,operation:'cancel'|'retry'){
  const ticket=this.feature.jobs.prepareJob(id,operation);
  this.ports.confirm(operation==='cancel'?'请求取消 '+id:'重试 '+id,operation==='cancel'?'取消请求不等于已取消；读回实际终态后再判断重试。':'只重试原任务，不更换赛事、时间范围或输入；必须仍有可用尝试次数。',[
   ['原任务',id],['实际状态',ticket.job!.status],['范围',JSON.stringify(ticket.job!.payload)],
  ],operation==='cancel'?'确认请求取消':'确认重试',async()=>{const job=await this.feature.jobs.submit(ticket);if(job)this.ports.notify(operation==='cancel'?(job.status==='cancelled'?'任务已取消':'取消请求已核实；实际状态为 '+job.status):'原任务状态已读回：'+job.status);});
 }
 recoverEnqueue(){const current=this.feature.capture();this.ports.confirm('核对原入队请求','将使用保存的原幂等键核对原任务，不生成新键。',[['原请求','保留的任务类型与输入范围']],'确认按原键核对',async()=>{if(!current())throw Error('查看上下文已变化');const row=await this.feature.jobs.recoverEnqueue();if(row)this.ports.notify('原任务已核实：'+row.id);});}
 exportPackage(){
  const current=this.feature.capture(),signature=identity(this.feature.overview?.source_snapshot);
  this.ports.confirm('导出外部智能分析资料','导出只读资料，绑定当前已核验分析快照；快照变化会阻止写文件。',[
   ['原快照',this.feature.overview?.source_snapshot?.id??'尚未核实'],['当前范围','全库分析与全库质量扫描分别核实'],
  ],'确认选择导出位置',async()=>{if(!current()||signature!==identity(this.feature.overview?.source_snapshot))throw Error('分析来源已变化');if(await this.feature.exchange.exportPackage(current))this.ports.notify('分析包已导出并登记来源');});
 }
 template(){const exported=this.feature.exchange.state.exported,current=this.feature.capture();if(!exported)throw Error('请先导出本轮智能分析资料包');this.ports.confirm('生成建议填写模板','模板绑定原分析包，不产生建议或修改参数。',[['分析包',exported.summary.package_id]],'确认选择保存位置',async()=>{if(!current()||this.feature.exchange.state.exported?.summary.package_id!==exported.summary.package_id)throw Error('分析包已变化');if(await this.feature.exchange.template())this.ports.notify('建议模板已保存');});}
 importResponse(){const ticket=this.feature.exchange.prepareImport();this.ports.confirm('确认导入分析建议','仅登记待审建议；不直接执行SQL或写入能力历史。后端会核对已确认的完整预检内容。',[['回包',ticket.preview.manifest.response_id],['来源分析包',ticket.preview.manifest.source_package_id??'未记录'],['建议条数',String(ticket.preview.suggestions.length)],['原文件',ticket.path]],'确认导入建议',async()=>{if(await this.feature.exchange.importResponse(ticket))this.ports.notify('回包建议已进入待审目录');});}
 suggestion(id:string,decision:'accept'|'reject'){
  const ticket=this.feature.reviews.prepareSuggestion(id,decision);if(ticket.kind!=='suggestion')return;
  this.ports.confirm(decision==='accept'?'接受建议':'拒绝建议',decision==='accept'?(ticket.row.suggestion_type==='ability_update'?'接受后只形成待审能力候选，仍需另行审核写入。':'仅登记接受记录，不直接修改参数或正式模型。'):'记录对原建议的拒绝，不修改来源资料。',[
   ['建议',ticket.row.id],['回包',ticket.row.response_id],['标题',ticket.row.title],
  ],'确认决定',async()=>{if(await this.feature.reviews.submit(ticket))this.ports.notify(decision==='accept'?'接受决定已核实；后续候选需分别审核':'拒绝决定已核实');});
 }
 quality(id:string,decision:'resolve'|'ignore'){
  const ticket=this.feature.reviews.prepareQuality(id,decision);if(ticket.kind!=='quality')return;
  this.ports.confirm(decision==='resolve'?'确认标记已处理':'确认忽略问题','只登记当前扫描的问题决定，不自动修复业务数据。新扫描仍可能发现同一问题。',[
   ['问题',ticket.row.id],['扫描',ticket.row.scan_id],['内容',ticket.row.message],
  ],'确认决定',async()=>{if(await this.feature.reviews.submit(ticket))this.ports.notify('问题决定已核实，请按当前扫描核对门禁');});
 }
 showJob(id:string){const row=this.feature.state.jobs.find(j=>j.id===id);if(!row)throw Error('任务未在当前目录');this.ports.show('分析任务 '+id,row);}
 showScope(){this.ports.show('当前分析范围',{snapshot:this.feature.overview?.source_snapshot??null,selectedJob:this.feature.selectedJob,quality:{scope:'全库诊断，不提供H分区许可',scan:this.feature.overview?.data_quality.scan_id??null},history:'正式结算总数与评估样本口径不同',binding:'H/I仍需按赛事、Profile、模型、参数与时点重新核对'});}
}
