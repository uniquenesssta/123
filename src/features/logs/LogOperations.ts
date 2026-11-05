import {redactLogText} from '../../platform/diagnostics';
import {logSnapshot} from './logContracts';
import type {LogsFeature} from './LogsFeature';

const errorText=(error:unknown)=>redactLogText(error instanceof Error?error.message:String(error));
const collectionChanged=(error:unknown)=>errorText(error).includes('问题日志集合已变化');

/** Every write binds to the read collection. A read-back never proves an unacknowledged clear succeeded. */
export class LogOperations {
  constructor(private readonly feature:LogsFeature){}
  private available(){const s=this.feature.state;if(s.operation||s.loading||s.readError||!s.entries)throw Error('请先读取可靠的问题日志快照。');}
  prepareClear(){
    this.available();const f=this.feature,s=f.state,entries=structuredClone(s.entries!),snapshot=logSnapshot(entries),view=f.ports.view(),priorUnknown=s.unknownClear,priorId=s.operationId;let used=false;
    if(s.unknownClear&&!s.checkedUnknown)throw Error('请先重新读取并核对上次清空的未知结果。');
    return {entries,unknown:s.unknownClear,run:async()=>{
      if(used||s.operation||s.loading||!f.alive()||!view()||!s.entries||logSnapshot(s.entries)!==snapshot)throw Error('请重新核对已变化的日志集合或页面。');
      used=true;s.operation='checking';s.status='正在核对确认的本机日志集合…';f.changed();
      try {
        if(!await f.refresh(view)||!view()||!f.alive())return;
        if(logSnapshot(s.entries!)!==snapshot){s.status='日志集合已变化，本次没有清空。请关闭确认并重新核对。';return;}
        s.operationId=crypto.randomUUID();s.unknownClear=true;s.checkedUnknown=false;
        if(!f.persist()){s.unknownClear=priorUnknown;s.operationId=priorId;s.status="恢复记录未保存，本次没有提交清空。";return;}
        f.invalidateReads();s.operation='clearing';s.status='正在清空确认的日志集合；这不会修复业务故障。';f.changed();
        try {
          await f.ports.busy(()=>f.ports.api.clearIssueLogs(entries));
          if(!f.alive())return;
          s.unknownClear=false;s.checkedUnknown=false;f.persist();
          s.status='本次清空已确认；后台扫描或后续错误仍可产生新记录。';
          // Read the real collection: new arrivals must never be overwritten by an invented empty array.
          await f.refresh(f.ports.view());
        } catch(error){
          if(!f.alive())return;
          if(collectionChanged(error)){s.unknownClear=priorUnknown;s.operationId=priorId;s.status='日志集合已变化，本次没有清空；请刷新后重新确认。';f.persist();}
          else{s.unknownClear=true;s.checkedUnknown=false;s.status='清空失败或回执未确认，请先重新读取核对；不会自动重试。'+errorText(error);f.persist();}
        }
      } finally {s.operation=null;f.changed();}
    }};
  }
  async export(){
    this.available();const f=this.feature,s=f.state,entries=structuredClone(s.entries!),snapshot=logSnapshot(entries),view=f.ports.view();
    s.operation='choosing';s.status='请选择问题日志报告的保存位置。';f.changed();
    let outputPath:string|null=null;
    try {
      outputPath=await f.ports.api.chooseIssueLogExportFile('足球赛事模型平台_问题日志报告.txt');
      if(!f.alive())return;
      if(!outputPath||!view()){s.status='导出已取消，没有发出文件写入请求。';return;}
      if(!s.entries||logSnapshot(s.entries)!==snapshot){s.status='选择位置期间日志快照已变化，本次未导出；请重新核对。';return;}
      if(!await f.refresh(view)||!view()||!f.alive())return;
      if(logSnapshot(s.entries!)!==snapshot){s.status='日志快照已变化，本次未导出；请重新核对。';return;}
      s.operation='exporting';s.status='正在导出已确认的日志快照…';f.changed();
      await f.ports.busy(()=>f.ports.api.exportIssueLogs(outputPath!,entries));
      if(f.alive())s.status='问题日志报告已导出。报告是本次快照，不会随新记录自动更新。';
    } catch(error){if(f.alive())s.status=collectionChanged(error)?'日志快照已变化，本次未导出；请重新核对。':'导出失败或结果未确认，请到所选位置核对后再操作。'+errorText(error);}
    finally{outputPath=null;s.operation=null;f.changed();}
  }
}
