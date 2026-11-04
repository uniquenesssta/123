import type {ReleaseFeature} from './releaseFeature';
import type {ReleaseDraft} from './releasePorts';
import {formFieldView} from '../../ui/forms/form-field';
import {numberInputView} from '../../ui/primitives/number-input';
import {textInputView} from '../../ui/primitives/text-input';
import {action,panel,html,message} from './releaseViewParts';
export function releaseRequestView(feature:ReleaseFeature):string {
 const s=feature.state,disabled=s.busy||!!s.pending;
 const field=(key:keyof ReleaseDraft,label:string,window=false)=>formFieldView({id:'release-'+key,label,control:({id,label})=>numberInputView({label,value:s.draft[key],disabled,min:window?1:0,max:window?365:undefined,step:window?1:0.01,attributes:{id,'data-release-field':key}})});
 const form=`<div class="release-request-fields">${field('performance_window_days','性能窗口（天）',true)}${field('cost_window_days','成本窗口（天）',true)}${field('daily_cost_budget_usd','单日预算（USD，可空）')}${field('monthly_cost_budget_usd','周期预算（USD，可空）')}</div>${formFieldView({id:'release-requested-by',label:'操作人（审计记录，可空）',control:({id,label})=>textInputView({label,value:s.draft.requested_by,disabled,attributes:{id,'data-release-field':'requested_by'}})})}${s.validation?message('请求参数需修正',s.validation):''}${s.busy?'<p role="status">正在等待验收结果，进度百分比未知；当前接口没有真实取消入口。</p>':''}<div class="release-actions">${action(s.busy?'验收请求处理中…':'确认运行新一轮验收','run-release-acceptance',disabled||!!s.validation,true)}</div>`;
 const recovery=s.pending?message('原验收结果待核实','不自动重发。'+(s.pending.report?'原报告 '+s.pending.report.id:'原请求未返回报告ID。'))+`<div class="release-actions">${action('核对原验收请求','recover-release-acceptance',s.busy)}${s.recoveryIds.map(id=>action('查看新报告 '+id,'open-release-acceptance-run',false,false,{'data-run-id':id})).join('')}</div>`:'';
 const feedback=s.error?message(s.phase==='unknown'?'请求结果未知':'本次请求未完成',s.error):s.result?`<p role="status">本次验收报告 ${html(s.result.id)}：${html(s.result.overall_status==='pass'?'通过':s.result.overall_status==='warning'?'存在警告':'存在阻断')}。报告完成不等于已经发布。</p>`:'';
 return recovery+feedback+panel('新一轮验收参数','新请求草稿与所选历史报告分开；留空预算允许运行，但成本项产生警告。',form);
}
