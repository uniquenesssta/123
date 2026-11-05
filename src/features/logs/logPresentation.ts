import {redactLogText} from '../../platform/diagnostics';
import type {IssueLogEntry,IssueSeverity} from '../../types';

export const safeLogText=(value:string)=>redactLogText(value);
export function logTime(value:string):string{const date=new Date(value);return Number.isNaN(date.getTime())?safeLogText(value):new Intl.DateTimeFormat('zh-CN',{year:'numeric',month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit',second:'2-digit',hour12:false}).format(date);}
export const severityLabel=(severity:IssueSeverity)=>({warning:'警告',error:'错误',critical:'严重错误'}[severity]??'问题');
const sourceLabels:Record<string,string>={backend:'后台业务或数据库',frontend:'界面操作',startup:'平台启动',global:'客户端未捕获异常',background_job:'后台分析任务'};
export const sourceLabel=(source:string)=>sourceLabels[source]??safeLogText(source);
export function friendlyMessage(entry:IssueLogEntry):string{const message=safeLogText(entry.user_message||entry.technical_message);if(/column .* does not exist|relation .* does not exist|数据库迁移失败/i.test(message))return '数据库结构与当前客户端不一致';if(/PostgreSQL 连接或查询失败|connection refused|pool timed out/i.test(message))return '数据库连接或查询失败';return message||'未提供问题描述';}
const operations:Record<string,string>={bootstrap:'平台初始化',configure_database:'连接数据库',execute_prediction:'执行赛事推演',execute_prediction_from_match:'使用数据库比赛正式推演',execute_shadow_prediction_from_match:'使用数据库比赛影子推演',preview_route:'查看模型判定',player_catalog_reference_data:'读取球员目录',list_players:'加载球员列表',read_player:'读取球员详情',create_player:'创建球员',update_player:'更新球员',delete_player:'删除球员',create_match:'创建比赛',delete_match:'删除比赛',create_lineup:'保存阵容',list_lineups:'读取阵容',generate_match_review:'生成赛后复盘',register_rule_package:'注册规则包',create_competition_binding:'建立赛事模型路由',analytics_overview:'读取分析中心',export_issue_logs:'导出问题日志'};
export const operationLabel=(value:string)=>operations[value]??safeLogText(value);
