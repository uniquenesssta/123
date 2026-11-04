import type {SendApiWorkspaceCommand} from '../../types';
import type {AiReceipt,AiWorkspacePorts} from './aiWorkspacePorts';
import type {AiWorkspaceState} from './aiWorkspaceState';
import type {AiWorkspaceQuery} from './aiWorkspaceQuery';
interface ActiveRequest {receipt:AiReceipt;state:AiWorkspaceState;command:SendApiWorkspaceCommand;scope:()=>boolean}
/** A single send lock spans all conversations; cancelling never releases it before actual completion. */
export class AiWorkspaceRequests {
 private active:ActiveRequest|null=null;private cancelInFlight=new Set<string>();
 constructor(private readonly ports:AiWorkspacePorts,private readonly state:()=>AiWorkspaceState,private readonly query:AiWorkspaceQuery,private readonly current:()=>()=>boolean){}
 get sending(){return this.active!==null;}
 get pending(){const a=this.active;return a&&a.scope()?{origin:a.receipt.origin,content:a.command.message,started_at:a.receipt.startedAt,session_id:a.command.session_id}:null;}
 async send():Promise<boolean>{const s=this.state(),scope=this.current();if(this.active||s.locked)throw Error('原 AI 请求仍在处理或结果待核实，不能重复发送');if(!scope())throw Error('请先连接数据服务');if(s.loading||s.detailError)throw Error('请先读取当前会话');const d=structuredClone(s.draft),id=s.sessionId;
  if(id&&!s.detail)throw Error('请先读取当前会话');if(s.detail?.session.status==='archived'||id&&s.archives[id])throw Error('此会话已归档或归档结果待核实，不能发送');if(!d.message.trim())throw Error('请输入问题');if(!s.profiles?.profiles.some(p=>p.id===d.profileId&&p.has_api_key))throw Error('原 API 配置或密钥不可用，请先维护配置');
  const preset=s.presets.find(p=>p.key===d.presetKey);if(!preset||d.presetKey==='file_structuring')throw Error('历史结构化会话只读，请新建普通文本会话');if(preset.requires_match&&!d.matchId)throw Error('此问答类型需要比赛上下文');if(d.includeContext&&!d.matchId&&!d.entityId)throw Error('只读上下文不可用');
  const receipt:AiReceipt={id:crypto.randomUUID(),origin:s.selected,sessionId:id,startedAt:new Date().toISOString(),phase:'sending',error:null,released:false,checked:false};
  const command:SendApiWorkspaceCommand={session_id:id,profile_id:d.profileId,preset_key:d.presetKey,title:null,match_id:d.matchId,context_entity_type:d.entityType,context_entity_id:d.entityId,context_entity_label:d.entityLabel,include_context:d.includeContext,request_id:receipt.id,message:d.message.trim(),attachments:[]};
  s.receipt=receipt;s.error=null;s.save();if(s.storageError){s.receipt=null;throw Error('请求恢复记录保存失败，尚未发送；请检查本地存储。');}
  const active=this.active={receipt,state:s,command,scope};s.draft.message='';s.save();this.ports.changed();
  try{const detail=await this.ports.api.sendApiWorkspaceMessage(command);if(id&&detail.session.id!==id||detail.session.profile_id!==command.profile_id||detail.session.preset_key!==command.preset_key||detail.session.match_id!==command.match_id)throw Error('发送结果会话身份不匹配');s.adopt(detail);receipt.sessionId=detail.session.id;receipt.phase='completed';s.save();if(!scope())return false;this.query.completedWrite(detail.session.id);
   // The write belongs to its origin. A catalog await cannot authorize a later view mutation.
   await this.query.list();if(!scope())return false;
   if(s===this.state()&&s.selected===receipt.origin){if(!id){const next='session:'+detail.session.id;s.drafts[next]={...s.drafts[next],...s.drafts[receipt.origin]};delete s.drafts[receipt.origin];s.selected=next;s.adopt(detail);}s.loading=false;s.detailError=null;}s.save();return true;
  }catch(error){const text=this.ports.formatError(error);receipt.error=text;
   // Only the backend's explicit cancellation reply establishes cancellation. A local delivery abort does not.
   receipt.phase=text==='OpenAI研究任务已取消'?'cancelled':'unknown';
   const origin=s.drafts[receipt.origin];if(origin&&!origin.message)origin.message=d.message;s.save();if(!scope())return false;
   await this.query.list();if(!scope())return false;s.save();return false;
  }finally{if(this.active===active)this.active=null;s.save();this.ports.changed();}
 }
 async cancel(id:string):Promise<boolean>{const a=this.active;if(!a||a.receipt.id!==id||!a.scope()||a.receipt.phase==='cancelling'||this.cancelInFlight.has(id))return false;this.cancelInFlight.add(id);a.receipt.phase='cancelling';a.state.save();this.ports.changed();
  try{const accepted=await this.ports.api.cancelApiWorkspaceRequest(id);if(!a.scope()||this.active!==a)return accepted;if(!accepted)a.receipt.error='取消未被受理；等待原请求的实际结果。';return accepted;
  }catch(error){if(a.scope()&&this.active===a)a.receipt.error=this.ports.formatError(error);return false;}finally{this.cancelInFlight.delete(id);if(a.scope()){a.state.save();this.ports.changed();}}
 }
 async reconcile():Promise<boolean>{const s=this.state(),r=s.receipt,scope=this.current();if(!r||r.phase!=='unknown'||r.released)return false;r.checked=false;const view=this.ports.view(),latest=r.sessionId?this.query.beginOriginRead(r.sessionId):()=>true,current=()=>scope()&&latest()&&s.receipt===r&&r.phase==='unknown'&&!r.released;
  try{if(r.sessionId){const detail=await this.ports.api.readApiWorkspaceSession(r.sessionId);if(!current())return false;if(detail.session.id!==r.sessionId)throw Error('原会话核验身份不匹配');s.adopt(detail);this.query.finishOriginRead(r.sessionId);}else{if(!await this.query.list(view))return false;}
   if(!current())return false;r.checked=true;r.error='原会话/目录已读取。现有接口不能按请求ID确认终态；已有用户消息或部分写入不代表回答完整保存。';s.save();this.ports.changed();return true;
  }catch(error){if(current()){r.error=this.ports.formatError(error);s.save();this.ports.changed();}return false;}
 }
 prepareRelease(){const s=this.state(),r=s.receipt,scope=this.current(),view=this.ports.view(),revision=s.revision;if(!r||r.phase!=='unknown'||!r.checked||this.active)throw Error('请先读取原会话或目录核验结果');let used=false;return{id:r.id,current:()=>!used&&r.checked&&scope()&&view()&&s===this.state()&&s.revision===revision&&s.receipt===r,release:()=>{if(used||!r.checked||r.released||!scope()||!view()||s!==this.state()||s.revision!==revision||s.receipt!==r)throw Error('原请求或会话已变化，请重新核验');used=true;r.released=true;s.save();this.ports.changed();}};}
}
