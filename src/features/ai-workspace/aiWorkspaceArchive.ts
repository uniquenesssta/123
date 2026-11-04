import type {AiWorkspacePorts} from './aiWorkspacePorts';
import type {AiWorkspaceState} from './aiWorkspaceState';
import type {AiWorkspaceQuery} from './aiWorkspaceQuery';
export class AiWorkspaceArchive {
 constructor(private readonly ports:AiWorkspacePorts,private readonly state:()=>AiWorkspaceState,private readonly query:AiWorkspaceQuery,private readonly capture:()=>()=>boolean){}
 prepare(id:string){const s=this.state(),current=this.capture(),view=this.ports.view(),revision=s.revision,row=s.sessions.find(x=>x.id===id);
  if(!row)throw Error('会话目录已变化，请刷新');if(s.archives[id])throw Error('原归档请求待核实，不能重复发送');if(s.locked&&(!s.receipt?.sessionId||s.receipt.sessionId===id))throw Error('此会话有未结束或待核实请求，不能归档');let used=false;
  return{id,title:row.title,run:async()=>{if(used||!current()||!view()||s!==this.state()||revision!==s.revision||s.archives[id])throw Error('确认的会话已变化，请重新确认');used=true;s.archives[id]='pending';s.save();if(s.storageError){delete s.archives[id];throw Error('归档恢复记录保存失败，尚未发送');}this.ports.changed();
   try{await this.ports.api.archiveApiWorkspaceSession(id);if(!current())return false;this.complete(s,id);await this.query.list();return true;}
   catch(error){if(current()){s.archives[id]='unknown';s.error='原会话归档结果待核实：'+this.ports.formatError(error);s.save();}return false;}finally{if(current())this.ports.changed();}
  }};
 }
 async reconcile(id:string){const s=this.state(),current=this.capture();if(s.archives[id]!=='unknown')return false;try{const detail=await this.ports.api.readApiWorkspaceSession(id);if(!current())return false;if(detail.session.id!==id)throw Error('归档核验响应身份不匹配');if(detail.session.status!=='archived'){s.error='原会话仍可读，不能据此证明归档未执行；保留原请求，稍后再核验。';this.ports.changed();return false;}this.complete(s,id);await this.query.list();if(current())this.ports.changed();return true;}catch(error){if(current()){s.error=this.ports.formatError(error);this.ports.changed();}return false;}}
 private complete(s:AiWorkspaceState,id:string){delete s.archives[id];s.sessions=s.sessions.filter(x=>x.id!==id);s.details.delete(id);if(s.sessionId===id)s.newSession();s.error=null;s.save();}
}
