import type {AnalyticsFeature} from './analyticsFeature';
import {action,empty,formatPercent,html,message,metric,number,panel,table} from './analyticsViewParts';
import {analyticsJobsView} from './analyticsJobsView';
export function analyticsModelView(feature:AnalyticsFeature):string {
 const overview=feature.overview,gates=feature.gates,job=feature.selectedJob,source=overview?.source_snapshot;
 const guide=panel('02 · 完整分析','任务终态与有效分析快照分别核实；不生成预测，也不自动更改正式参数。',`<p>${html(job?'原任务 '+job.id+' · '+(job.payload.competition_id??'全库范围'):'查看最近分析快照；不将全局计数当作某个分区许可。')}</p><div class="analytics-actions">${action('运行完整分析','run-full-analysis',!gates.history||feature.jobs.state.busy,true)}${action('只刷新模型评估','run-model-analysis',!gates.history||feature.jobs.state.busy)}${action('刷新任务','refresh-analysis-jobs',feature.state.loading)}${action('校准分桶与表统计','analysis-advanced',!overview)}</div>`,'model-guide');
 let result=empty('尚未取得本范围的有效分析','任务完成与生成有效样本是两个条件；请选择原任务并读取原快照。');
 if(overview?.generated_at){
  const metadata=source?`快照 ${source.id} · ${source.competition_id?'赛事 '+source.competition_id:'全库'} · ${source.window_start??'无样本起点'} — ${source.window_end??'无样本终点'}`:'旧快照缺少来源身份，数据保留展示；重新运行后再核对门禁。';
  result=panel((source?.id??'历史来源待核实')+' · 分析快照',metadata,`<div class="analytics-metrics">${metric('有效样本',overview.sample_size,'与正式结算总数分开')}${metric('Log loss',number(overview.average_log_loss),'越低越好')}${metric('Brier',number(overview.average_brier),'越低越好')}${metric('校准偏差',number(overview.expected_calibration_error),'统计结论不代替H/I门禁')}</div>${table('模型表现比较',['模型 / 版本','参数版本','时点','样本','Log loss','Brier','覆盖率'],overview.comparisons.map(row=>[row.model_key+' / '+row.model_version,row.parameter_version,row.snapshot_type,row.sample_size,number(row.average_log_loss),number(row.average_brier),formatPercent(row.average_data_coverage)]))}<p>不同参数和时点分别列出；历史比较不能授予晋升许可。</p>`,'model-result');
  if(overview.sample_size===0)result+=empty('分析已返回，但没有有效样本','保留实际0样本；不能按任务成功推断已完成质量门禁。');
 }
 const drift=overview?panel('近期变化','原分析快照中的统计变化；不替代H的完整分区报告。',overview.drift.length?table('统计变化',['指标','基线','当前','变化','样本','状态'],overview.drift.map(row=>[row.metric_name,number(row.baseline_mean),number(row.current_mean),number(row.absolute_delta),row.baseline_size+' / '+row.current_size,({stable:'稳定',warning:'需关注',critical:'严重变化'}[row.severity])])):empty('暂无变化记录','后端未返回报告，不补出稳定结论。'),'model-drift'):'';
 const id=(job?.result?.source_snapshot as {id?:string}|undefined)?.id,error=id?feature.query.snapshotError(id):null;
 return guide+(error?message('原快照读取未完成',error):'')+result+drift+analyticsJobsView(feature);
}
export function analyticsAdvancedView(feature:AnalyticsFeature):string {
 const overview=feature.overview;if(!overview)return empty('没有可核对的分析快照','先返回完整分析，选择并读回原任务快照。');
 return panel('高级历史详情 · 校准分桶',`快照 ${overview.source_snapshot?.id??'来源未核实'}；使用历史样本，不执行模型调用。`,table('校准分桶',['结果','概率区间','样本','预测均值','实际命中','绝对差值'],overview.calibration.map(row=>[({home_win:'主胜',draw:'平局',away_win:'客胜'}[row.outcome]),row.lower_bound+'–'+row.upper_bound,row.sample_size,formatPercent(row.predicted_mean),formatPercent(row.actual_rate),formatPercent(row.absolute_gap)])),'calibration')+panel('数据库表统计','全库诊断单独标注；估算行数不是精确COUNT，也不自动执行SQL。',`<div class="analytics-actions">${action('运行数据库性能检查','run-query-performance-scan',feature.jobs.state.busy)}${action('返回完整分析','analysis-model')}</div>${table('数据库表统计',['数据表','估算行数','大小（字节）','顺序扫描','索引扫描','建议'],overview.query_performance.tables.map(row=>[row.schema_name+'.'+row.table_name,row.estimated_rows,row.table_size_bytes,row.sequential_scans,row.index_scans,row.recommendation??'无建议']))}`,'query-statistics');
}
