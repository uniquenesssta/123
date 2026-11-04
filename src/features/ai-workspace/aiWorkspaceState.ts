import type {ApiWorkspaceSessionDetail,ApiWorkspaceSessionRecord,ApiWorkspacePreset,OpenAiProfilesState,MatchRecord} from '../../types';
import {emptyAiDraft,type AiDraft,type AiReceipt,type AiStorageScope} from './aiWorkspacePorts';
export class AiWorkspaceState {
 selected='';drafts:Record<string,AiDraft>={};details=new Map<string,ApiWorkspaceSessionDetail>();receipt:AiReceipt|null=null;archives:Record<string,'pending'|'unknown'>={};
 profiles:OpenAiProfilesState|null=null;presets:ApiWorkspacePreset[]=[];sessions:ApiWorkspaceSessionRecord[]=[];matches:MatchRecord[]=[];
 loading=false;listLoading=false;loaded=false;restored=false;detailError:string|null=null;listError:string|null=null;error:string|null=null;storageError:string|null=null;search='';revision=0;
 constructor(readonly storage:AiStorageScope){const saved=storage.read();this.restored=!!saved;if(saved){this.selected=saved.selected;this.drafts=saved.drafts;this.receipt=saved.receipt;this.archives=saved.archives??{};this.search=saved.search??'';}if(!this.selected)this.newSession();if(!this.drafts[this.selected])this.drafts[this.selected]=emptyAiDraft();}
 get sessionId(){return this.selected.startsWith('session:')?this.selected.slice(8):null;}
 get detail(){return this.loading||this.detailError?null:this.sessionId?this.details.get(this.sessionId)??null:null;}
 get draft(){return this.drafts[this.selected]??(this.drafts[this.selected]=emptyAiDraft());}
 get locked(){return !!this.receipt&&!this.receipt.released&&!['completed','cancelled'].includes(this.receipt.phase);}
 save(){try{this.storage.save({selected:this.selected,drafts:this.drafts,receipt:this.receipt,archives:this.archives,search:this.search});this.storageError=null;}catch{this.storageError='草稿恢复记录未能保存，请保留输入并检查本地存储。';}}
 newSession(seed:Partial<AiDraft>={}){this.selected='new:'+crypto.randomUUID();this.drafts[this.selected]={...emptyAiDraft(),...seed};this.loading=false;this.detailError=null;this.revision++;this.save();}
 select(id:string){this.selected='session:'+id;this.drafts[this.selected]??=emptyAiDraft();this.loading=true;this.detailError=null;this.revision++;this.save();}
 adopt(detail:ApiWorkspaceSessionDetail){const id=detail.session.id;if(detail.messages.some(m=>m.session_id!==id))throw Error('会话消息响应身份不匹配');this.details.set(id,detail);const key='session:'+id,d=this.drafts[key]??emptyAiDraft(),m=detail.session.metadata;this.drafts[key]={...d,profileId:detail.session.profile_id,presetKey:detail.session.preset_key,matchId:detail.session.match_id,entityType:m.context_entity_type==='team'||m.context_entity_type==='player'?m.context_entity_type:null,entityId:typeof m.context_entity_id==='string'?m.context_entity_id:null,entityLabel:typeof m.context_entity_label==='string'?m.context_entity_label:null};this.save();}
}
