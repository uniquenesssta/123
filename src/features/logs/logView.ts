import {escapeHtml} from '../../components/format';
import {buttonView} from '../../ui/primitives/button';
import {statCardView} from '../../ui/data-display/stat-card';
import {codeBlockView} from '../../ui/data-display/code-block';
import {emptyStateView} from '../../ui/data-display/empty-state';
import type {IssueLogEntry} from '../../types';
import type {LogState} from './logContracts';
import {friendlyMessage,logTime,operationLabel,safeLogText,severityLabel,sourceLabel} from './logPresentation';

function issueCard(entry:IssueLogEntry,state:LogState):string{
  const opened=state.expanded.has(entry.id),id='issue-details-'+entry.id;
  const button=buttonView({label:opened?'收起技术详情':'查看技术详情',attributes:{'data-action':'toggle-issue-detail','data-issue-id':entry.id,'aria-expanded':opened,'aria-controls':id}});
  return `<article class="issue-card severity-${escapeHtml(entry.severity)}" data-issue-id="${escapeHtml(entry.id)}">
    <header class="issue-card-heading"><small>${escapeHtml(safeLogText(entry.id))} · 客户端 v${escapeHtml(safeLogText(entry.app_version))}</small><h2>${escapeHtml(friendlyMessage(entry))}</h2></header>
    <div class="issue-summary-line"><span class="issue-severity">${escapeHtml(severityLabel(entry.severity))}</span><span>${entry.occurrence_count} 次发生 · 已合并 ${Math.max(0,entry.occurrence_count-1)} 次重复</span></div>
    <div class="issue-meta-grid"><span>来源：${escapeHtml(sourceLabel(entry.source))}</span><span>首次：${escapeHtml(logTime(entry.first_seen_at))}</span><span>最近：${escapeHtml(logTime(entry.last_seen_at))}</span></div>
    <div class="issue-operations"><span>涉及操作：${escapeHtml(entry.operations.length?entry.operations.map(operationLabel).join('、'):'未记录具体操作')}</span>${button}</div>
    <div id="${escapeHtml(id)}" class="issue-details" ${opened?'':'hidden'}>${opened?codeBlockView({label:'原始技术信息（已脱敏）',text:safeLogText(entry.technical_message||entry.user_message||'未记录技术信息')}):''}${opened?`<p>请求 / 任务尝试关联：${escapeHtml(entry.occurrence_keys?.length?entry.occurrence_keys.map(safeLogText).join('、'):'未记录关联 ID；不根据操作名称推断请求身份。')}</p>`:''}</div>
  </article>`;
}

function processing():string{return '<section id="issue-processing" class="issue-processing"><header><small>本机问题记录</small><h2>同类历史如何聚合</h2></header><ol><li>捕获界面、请求与后台任务问题，复用同一诊断入口。</li><li>来源与归一化后的技术信息形成指纹；技术信息为空时使用用户提示。</li><li>同类新发生会更新次数与最近时间，严重度只升级；历史记录不是故障已解决的证明。</li><li>导出当前保留记录的脱敏文本快照，不上传、修复或重新执行原操作。</li></ol></section><section class="issue-processing"><header><small>本机问题记录</small><h2>保留窗口与重复读取</h2></header><p>最多保留最近 500 个独立问题；消息最多 4,000 字符，操作名称 160 字符，每个问题最多保留 8 个不同操作。<br>同一个已保留的请求或任务尝试标识再次扫描不增加次数；每条最多记住 100 个标识，不能宣称永久去重。<br>清空后再次刷新仍可能重新扫描到失败任务并开始新一轮记录；这不等于重复执行原业务。</p></section>';}
export function logsPage(state:LogState):string{
  const entries=state.entries,total=entries?.reduce((sum,item)=>sum+item.occurrence_count,0),critical=entries?.filter(item=>item.severity==='critical').length;
  const reliable=entries!==null&&!state.readError&&!state.loading,locked=state.operation!==null||state.loading;
  const tabs=([['overview','问题概览'],['processing','聚合机制'],['records','问题记录']]as const).map(([key,label])=>buttonView({label,variant:state.section===key?'primary':'secondary',attributes:{'data-action':'set-logs-section','data-log-section':key,'aria-pressed':state.section===key}})).join('');
  const metrics=[['独立问题',entries?.length,'按指纹归并后的记录数','info'],['总发生次数',total,'包含重复发生','accent'],['已聚合重复',total===undefined?undefined:Math.max(0,total-(entries?.length??0)),'总发生次数 − 独立问题','info'],['严重问题',critical,'严重级别的问题条数','danger']]as const;
  let body=state.section==='processing'?processing():entries?.length?`<section id="issue-records" class="issue-list">${entries.map(entry=>issueCard(entry,state)).join('')}</section>`:emptyStateView({title:state.loading?'正在读取本机问题记录':state.readError?'问题记录暂时无法读取':entries===null?'尚未取得问题日志快照':'目前没有记录到问题',description:state.readError?'原文件及错误说明保留；恢复读取前不导出空报告或清空未知集合。'+state.readError:entries===null||state.loading?'请等待可靠读取；未知状态不会显示为 0。':'本机当前保留的集合为空，不代表系统已完全健康。',size:'Default'});
  const notices=[state.storageError,state.status,state.unknownClear?'上次清空结果未知。请先刷新核对；再次操作必须重新确认当前集合，不自动重放。':null,state.readError&&entries?'读取失败：'+state.readError+(entries?'。以下为上次成功快照，当前状态未确认。':'。不能把失败当作没有问题。'):null].filter(Boolean);
  return `<section class="module-workspace-page management-module-workspace logs-module-workspace" data-logs-section="${state.section}">
    <header class="logs-page-heading"><h1>问题历史与重复聚合</h1><p>记录用于排查历史问题；保留卡片不代表故障仍在发生，清空记录也不会修复故障。</p></header>
    <div class="core-local-navigation logs-toolbar">${tabs}<span class="logs-toolbar-spacer"></span>${buttonView({label:state.loading?'读取中':'刷新',disabled:locked,attributes:{'data-action':'refresh-issue-logs'}})}${buttonView({label:'导出报告',disabled:locked||!reliable||!entries?.length,attributes:{'data-action':'export-issue-logs'}})}${buttonView({label:'清空日志',variant:'danger',disabled:locked||!reliable||!entries?.length||state.unknownClear&&!state.checkedUnknown,attributes:{'data-action':'request-clear-issue-logs'}})}${buttonView({label:'使用边界',attributes:{'data-action':'issue-log-scope'}})}</div>
    <div id="issue-summary" class="issue-metrics">${metrics.map(([label,value,note,tone])=>`<div class="issue-metric-${tone}">${statCardView({label,value:value??null,note:entries===null?'尚未取得可靠快照':note})}</div>`).join('')}</div>
    <div class="logs-snapshot" role="status">${state.loading?(entries?'正在刷新；保留上次成功快照。':'正在取得可靠快照…'):state.readError?(entries?'上次成功快照 · 当前读取失败':'读取失败；问题数量未知'):state.readAt?'本机问题快照 · '+escapeHtml(logTime(state.readAt)):'尚未读取本机历史。'}</div>
    ${notices.length?`<div class="logs-notices" role="status">${notices.map(text=>`<p>${escapeHtml(text!)}</p>`).join('')}</div>`:''}
    <div class="management-module-stage logs-body-viewport" data-workspace-scroll-key="logs-stage-${state.section}">${body}</div>
  </section>`;
}
