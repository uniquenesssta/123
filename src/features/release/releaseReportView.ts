import type {ReleaseAcceptanceRun,ReleaseAcceptanceCheck} from '../../types';
import type {ReleaseFeature} from './releaseFeature';
import {action,panel,html,empty,message,statusBadge,statusLabel} from './releaseViewParts';
const labels:Record<string,string>={chain:'全链路',performance:'性能',security:'安全',cost:'成本',release:'发布'};
export function releaseChecks(checks:readonly ReleaseAcceptanceCheck[]):string {
 return checks.length?checks.map(c=>panel(String(c.sequence_no).padStart(2,'0')+' · '+c.title,(labels[c.category]??c.category)+' · '+c.check_code,`${statusBadge(c.status)}<p>${html(c.summary)}</p>${c.remediation?message('处理建议',c.remediation):''}<div class="release-actions">${action('查看证据','show-release-check-evidence',false,false,{'data-check-id':c.id,'data-run-id':c.run_id})}</div>`)).join(''):empty('尚无检查结果','未取得所选报告的此类检查，不推定通过。');
}
export function releaseReportView(feature:ReleaseFeature,section:string):string {
 const s=feature.state,run=s.selected;
 if(s.detailLoading)return message('正在读取报告 '+s.selectedId,'不显示另一份历史报告的内容。');
 if(s.detailError)return message('原报告读取失败',s.detailError)+action('重新读取原报告','open-release-acceptance-run',false,false,{'data-run-id':s.selectedId??''});
 if(!run)return empty('尚未选择验收报告','运行新一轮验收或从历史中选择报告；公开引擎限制会如实记录。');
 const heading=panel(run.id+' · '+statusLabel(run.overall_status),run.checks.length+'项检查属于该报告快照，修改新请求参数不会重算旧报告。',`<p>版本 ${html(run.app_version)} · 契约 ${html(run.contract_version)} · ${html(run.completed_at)}</p><p>当前公开验收包含运行时边界；不得把“0阻断”标记为全部通过。</p>`);
 return heading+(section==='overview'?releaseChecks(run.checks):section==='cost'?costView(run)+releaseChecks(run.checks.filter(c=>c.category==='cost')):section==='performance'?performanceView(run)+releaseChecks(run.checks.filter(c=>c.category==='performance')):releaseChecks(run.checks.filter(c=>c.category===section)));
}
function performanceView(run:ReleaseAcceptanceRun){const p=run.performance;return panel('数据库与近期推演性能','只展示这份报告记录的测量值。',`<dl class="release-facts"><div><dt>数据库延迟</dt><dd>${p.database_latency_ms} ms</dd></div><div><dt>近期推演</dt><dd>${p.recent_model_run_count}</dd></div><div><dt>推演 P95</dt><dd>${p.recent_model_run_p95_ms===null?'未取得':p.recent_model_run_p95_ms+' ms'}</dd></div><div><dt>查询警告</dt><dd>${p.query_warning_count}</dd></div></dl>`);}
function costView(run:ReleaseAcceptanceRun){const c=run.cost,budget=(value:number|null)=>value===null?'未设置':value+' USD';return panel('已保存的成本与预算','周期指所选成本窗口；最近用量日不一定是今天。',`<dl class="release-facts">${Object.entries({'成本窗口':c.window_days+' 天','估算成本':c.estimated_cost_usd+' USD','最近用量日成本':c.latest_day_cost_usd+' USD','单日预算':budget(c.daily_budget_usd),'周期预算':budget(c.monthly_budget_usd),'完成 / 失败':c.completed_requests+' / '+c.failed_requests,'搜索调用':c.search_calls}).map(([k,v])=>`<div><dt>${html(k)}</dt><dd>${html(String(v))}</dd></div>`).join('')}</dl>`);}
export function releaseHistoryView(feature:ReleaseFeature):string {const s=feature.state;return (s.loading?message('正在读取历史','刷新只读目录，不创建新的验收。'):'')+(s.listError?message('历史刷新失败，保留原列表',s.listError):'')+(s.runs.length?`<div class="release-history">${s.runs.map(run=>panel(run.id+' · '+statusLabel(run.overall_status),run.completed_at,`${statusBadge(run.overall_status)}<p>通过 ${run.passed_count} / 警告 ${run.warning_count} / 阻断 ${run.blocked_count} · ${html(run.report_sha256.slice(0,12))}</p>${action('读取报告','open-release-acceptance-run',false,false,{'data-run-id':run.id})}`)).join('')}</div>`:s.loaded?empty('尚无历史报告','首份报告会在实际执行后保存为不可变记录。'):empty('历史尚未取得','刷新目录后显示实际记录。'));}
