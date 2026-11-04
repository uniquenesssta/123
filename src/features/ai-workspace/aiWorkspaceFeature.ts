import {AiWorkspaceState} from './aiWorkspaceState';
import {AiWorkspaceQuery} from './aiWorkspaceQuery';
import {AiWorkspaceRequests} from './aiWorkspaceRequests';
import {AiWorkspaceArchive} from './aiWorkspaceArchive';
import type {AiDraft,AiWorkspacePorts} from './aiWorkspacePorts';
export class AiWorkspaceFeature {
 private data:AiWorkspaceState;private scope:()=>boolean;private closed=false;
 readonly query:AiWorkspaceQuery;readonly requests:AiWorkspaceRequests;readonly archives:AiWorkspaceArchive;
 constructor(private readonly ports:AiWorkspacePorts){this.scope=ports.scope();this.data=new AiWorkspaceState(ports.storage.capture());this.query=new AiWorkspaceQuery(ports,()=>this.state,()=>this.capture());this.requests=new AiWorkspaceRequests(ports,()=>this.state,this.query,()=>this.capture());this.archives=new AiWorkspaceArchive(ports,()=>this.state,this.query,()=>this.capture());}
 get state(){if(!this.closed&&!this.scope()){this.query?.invalidate();this.scope=this.ports.scope();this.data=new AiWorkspaceState(this.ports.storage.capture());}return this.data;}
 private capture(){const s=this.state,current=this.scope;return()=>!this.closed&&current()&&s===this.data;}
 async load(isCurrent=this.ports.view()){const s=this.state,key=s.selected;if(!await this.query.list(isCurrent))return false;if(!isCurrent())return false;if(s.selected!==key)return true;return s.sessionId?this.query.select(s.sessionId,isCurrent):true;}
 select(id:string){return this.query.select(id);}
 canSend(){const s=this.state,d=s.draft,preset=s.presets.find(p=>p.key===d.presetKey);return !!s.profiles?.profiles.some(p=>p.id===d.profileId&&p.has_api_key)&&!!preset&&d.presetKey!=='file_structuring'&&!s.loading&&!s.detailError&&(!s.sessionId||!!s.detail)&&s.detail?.session.status!=='archived'&&!s.archives[s.sessionId??'']&&(!preset.requires_match||!!d.matchId)&&(!d.includeContext||!!(d.matchId||d.entityId))&&!s.locked&&!this.requests.sending;}
 restoreLegacyDraft(message:unknown){const s=this.state;if(!s.restored&&!s.loaded&&!s.sessionId&&!s.draft.message&&typeof message==='string'&&message){s.draft.message=message;s.error='已保留旧工作区的未绑定草稿。旧记录未保存会话身份，请核对配置与上下文后再发送。';s.save();}s.restored=true;}
 newSession(seed:Partial<AiDraft>={}){this.query.invalidate();this.state.newSession(seed);this.ports.changed();}
 update(field:keyof AiDraft,value:string|boolean|null){const s=this.state;if(field!=='message'&&field!=='includeContext'&&s.sessionId)return;const d=s.draft;(d as unknown as Record<string,unknown>)[field]=value;if(field==='matchId'&&!d.matchId&&!d.entityId)d.includeContext=false;s.revision++;s.save();}
 search(value:string){this.state.search=value;this.state.save();this.ports.changed();}
 prepareArchive(id:string){return this.archives.prepare(id);}
 navigationChanged(){this.query.invalidate();const s=this.state;s.loading=false;s.listLoading=false;s.revision++;}
 stop(){this.closed=true;this.query.stop();}
}
