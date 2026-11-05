import type {LogsFeature} from './LogsFeature';
import type {LogSection} from './logContracts';

interface LogActionPorts {
  confirm(title:string,description:string,facts:Array<[string,string]>,label:string,run:()=>Promise<void>):void;
  show(title:string,description:string):void;
}
export class LogActions {
  constructor(private readonly feature:LogsFeature,private readonly ports:LogActionPorts){}
  async handle(button:HTMLElement):Promise<boolean>{
    const f=this.feature,s=f.state,view=f.ports.view();
    switch(button.dataset.action){
      case 'refresh-issue-logs':if(!s.operation)await f.refresh();break;
      case 'export-issue-logs':await f.operations.export();break;
      case 'set-logs-section':if(['overview','processing','records'].includes(button.dataset.logSection??''))f.section(button.dataset.logSection as LogSection);break;
      case 'toggle-issue-detail':if(button.dataset.issueId)f.toggle(button.dataset.issueId);break;
      case 'request-clear-issue-logs':{
        const ticket=f.operations.prepareClear(),total=ticket.entries.reduce((sum,row)=>sum+row.occurrence_count,0);
        this.ports.confirm('清空本机问题记录',(ticket.unknown?'上次清空结果仍未确认。本次在你核对后处理当前新快照，不重放上次请求。':'')+'只清除当前本机问题集合，不删除业务数据库、运行追踪文件、会话或草稿，也不会修复故障。后台扫描仍可产生新记录；集合变化时必须重新确认。',[['独立问题',String(ticket.entries.length)],['总发生次数',String(total)]],'清空日志',ticket.run);break;
      }
      case 'issue-log-scope':this.ports.show('问题日志的使用边界','本页是当前 Windows 用户的本机历史，最多保留 500 个独立问题。按指纹合并，计数、首次/最近时间和来源分别保留；同一保留窗口内的任务尝试或请求 ID 不重复记账。旧问题不代表当前仍异常，空记录也不证明系统健康。刷新在数据库可用时会采集失败任务，扫描失败可以与本机读取成功并存。详情和导出复用同一脱敏边界；未知格式的个人信息仍需分享前复核。导出是文件快照，不上传；清空只影响日志，不删除追踪或修复故障。');break;
      default:return false;
    }if(view()&&["toggle-issue-detail","set-logs-section","refresh-issue-logs"].includes(button.dataset.action??"")){const action=button.dataset.action,id=button.dataset.issueId,section=button.dataset.logSection;document.querySelector<HTMLElement>(`.logs-module-workspace [data-action="${action}"]${id?`[data-issue-id="${CSS.escape(id)}"]`:section?`[data-log-section="${section}"]`:""}`)?.focus({preventScroll:true});}return true;
  }
}
