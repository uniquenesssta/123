import type {BootstrapResponse} from '../../types';
import type {ReleaseFeature} from './releaseFeature';
import {inlineDatabaseSetup} from '../../components/databaseSetup';
import {taskPageHeader} from '../../ui/patterns/workbench';
import {releaseRequestView} from './releaseRequestView';
import {releaseReportView,releaseHistoryView} from './releaseReportView';
import {action,panel,html,metric,statusLabel,message} from './releaseViewParts';
export function releaseFeaturePage(state:BootstrapResponse,feature:ReleaseFeature,activeSection:string,inspectorCollapsed:boolean):string {
 const header=taskPageHeader({eyebrow:'',title:'全链路验收与发布',description:'公开外壳、真实运行统计与不可变验收账本；报告完成不等于已经发布。'});
 if(!state.data.database_configured)return `<section class="release-page">${header}${inlineDatabaseSetup('连接数据服务以运行发布验收','验收报告必须写入不可变账本。',state.connection_error)}</section>`;
 const s=feature.state,run=s.selected,section=['overview','chain','performance','security','cost','history'].includes(activeSection)?activeSection:'overview',disabled=s.busy||!!s.pending||!!s.validation;
 const metrics=`<div class="release-metrics">${metric('所选报告',run?statusLabel(run.overall_status):s.detailLoading?'读取中':s.detailError?'读取失败':'未运行',run?run.id:'尚无当前报告，不推定通过')}${metric('检查结果',run?run.passed_count+' / '+run.warning_count+' / '+run.blocked_count:null,'通过 / 警告 / 阻断')}${metric('验收历史',s.loaded&&!s.loading&&!s.listError?s.runs.length:null,'最近50条，只读账本')}${metric('报告指纹',run?.report_sha256.slice(0,12)??null,'由后端保存；非现场完整性核验')}</div>`;
 const tabs=`<nav class="release-tabs" aria-label="验收分区">${[['overview','总览'],['chain','全链路'],['performance','性能'],['security','安全'],['cost','成本'],['history','历史']].map(([key,label])=>action(label,'select-workspace-section',false,key===section,{'data-section-id':key,'aria-current':key===section?'page':'false'})).join('')}${action(inspectorCollapsed?'查看报告摘要':'关闭报告摘要','toggle-workspace-pane',!run,false,{'data-pane':'inspector'})}</nav>`;
 const inspector=run&&!inspectorCollapsed?`<aside class="release-inspector">${panel('所选报告摘要',run.id,`<p>${html(statusLabel(run.overall_status))} · 通过 ${run.passed_count} / 警告 ${run.warning_count} / 阻断 ${run.blocked_count}</p><p class="release-hash">报告指纹 ${html(run.report_sha256)}</p>${action('查看完整报告','show-release-acceptance-json',false,false,{'data-run-id':run.id})}`)}</aside>`:'';
 const body=section==='history'?releaseHistoryView(feature):`${section==='overview'?releaseRequestView(feature):''}${releaseReportView(feature,section)}`;
 return `<section class="module-workspace-page release-core-workspace release-page" data-workspace-persist="false">${header}<p>固定 fixture 未随公开源码分发；真实验收按当前服务能力记录限制。</p><div class="release-actions">${action('刷新历史','refresh-release-acceptance',s.loading)}${action('运行发布验收','run-release-acceptance',disabled,!!run)}${action('验收范围','release-scope')}</div>${metrics}${tabs}<div class="module-workspace-stage release-stage ${inspector?'with-inspector':''}"><main class="workspace-module-view active release-viewport" tabindex="0" aria-label="验收内容" data-workspace-scroll-key="release-${section}"><div class="release-body">${section!=="history"&&s.listError?message("历史刷新失败，数量待核实",s.listError):""}${body}</div></main>${inspector}</div></section>`;
}
