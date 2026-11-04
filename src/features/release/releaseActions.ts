import type {ReleaseFeature} from './releaseFeature';
export interface ReleaseActionPorts {confirm(title:string,body:string,facts:Array<[string,string]>,label:string,submit:()=>Promise<void>):void;show(title:string,value:unknown):void;notify(text:string,tone:'success'|'normal'|'error'):void}
export class ReleaseActions {
 constructor(private feature:ReleaseFeature,private ports:ReleaseActionPorts){}
 run(){const ticket=this.feature.prepare(),r=ticket.request;this.ports.confirm('运行新一轮发布验收','保存新的不可变报告。公开模型不可用将如实产生警告；本操作不会部署或发布Git。',[['性能 / 成本窗口',r.performance_window_days+' / '+r.cost_window_days+' 天'],['单日 / 周期预算',(r.daily_cost_budget_usd===null?'未设置':r.daily_cost_budget_usd)+' / '+(r.monthly_cost_budget_usd===null?'未设置':r.monthly_cost_budget_usd)+' USD'],['操作人',r.requested_by??'未填写']],'确认运行',async()=>{if(await this.feature.run(ticket)){const result=this.feature.state.result!;this.ports.notify('验收报告已保存：'+result.id,result.overall_status==='pass'?'success':result.overall_status==='blocked'?'error':'normal');}});}
 report(id?:string){const report=this.feature.report();if(id&&report.id!==id)throw Error('报告选择已经变化');this.ports.show('接入点 J 完整验收报告 · '+report.id,report);}
 evidence(id:string,runId?:string){const evidence=this.feature.evidence(id);if(runId&&evidence.report_id!==runId)throw Error('证据报告已经变化');this.ports.show('验收证据 · '+evidence.report_id,evidence);}
 scope(){this.ports.show('应用内验收范围',{mode:'public_shell_and_runtime',provider:'公开模型运行时警告必须保留；固定fixture版本字段不证明执行。',reports:'报告只读。当前报告DTO不包含完整原始性能窗口，不能用新草稿补写历史参数。',recovery:'没有客户端幂等键；未知请求不自动重发，新历史记录不自动归属原请求。',release:'应用内报告完成不等于R18软件发布，不创建Git标签或部署。'});}
}
