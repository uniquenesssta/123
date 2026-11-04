import {AsyncScope} from '../../app/lifecycle';
import type {AiWorkspacePorts} from './aiWorkspacePorts';
import type {AiWorkspaceState} from './aiWorkspaceState';
/** Catalog and selected detail have independent latest-wins channels. */
export class AiWorkspaceQuery {
 private readonly catalog=new AsyncScope();private readonly selection=new AsyncScope();private readonly detailRevisions=new Map<string,number>();
 constructor(private readonly ports:AiWorkspacePorts,private readonly state:()=>AiWorkspaceState,private readonly current:()=>()=>boolean){}
 invalidate(){this.catalog.invalidate();this.selection.invalidate();}
 beginOriginRead(id:string){const n=(this.detailRevisions.get(id)??0)+1;this.detailRevisions.set(id,n);return()=>this.detailRevisions.get(id)===n;}
 finishOriginRead(id:string){const s=this.state();if(s.sessionId===id){this.selection.invalidate();s.loading=false;s.detailError=null;}}
 completedWrite(id:string){this.beginOriginRead(id);this.finishOriginRead(id);}
 async list(view:()=>boolean=this.ports.view()):Promise<boolean>{const s=this.state(),scope=this.current(),r=this.catalog.begin();const current=()=>scope()&&view()&&r.isCurrent()&&s===this.state();s.listLoading=true;s.listError=null;this.ports.changed();
  try{const [profiles,presets]=await Promise.all([this.ports.api.listOpenAiProfiles(),this.ports.api.listApiWorkspacePresets()]);if(!current())return false;s.profiles=profiles;s.presets=presets;
   const [sessions,refs]=await Promise.all([this.ports.api.listApiWorkspaceSessions(100),this.ports.api.playerCatalogReferenceData()]);if(!current())return false;s.sessions=sessions;s.matches=refs.managed_matches;s.loaded=true;
   if(!s.sessionId&&!s.draft.profileId)s.draft.profileId=profiles.active_profile_id??profiles.profiles.find(p=>p.has_api_key)?.id??'';s.save();return true;
  }catch(error){if(current())s.listError=this.ports.formatError(error);return false;}finally{if(current()){s.listLoading=false;this.ports.changed();}r.finish();}
 }
 async select(id:string,view:()=>boolean=this.ports.view()):Promise<boolean>{const s=this.state(),scope=this.current(),r=this.selection.begin(),latest=this.beginOriginRead(id);s.select(id);const key=s.selected;const current=()=>scope()&&view()&&r.isCurrent()&&latest()&&s===this.state()&&s.selected===key;this.ports.changed();
  try{const detail=await this.ports.api.readApiWorkspaceSession(id);if(!current())return false;if(detail.session.id!==id)throw Error('读取会话响应身份不匹配');s.adopt(detail);return true;
  }catch(error){if(current())s.detailError=this.ports.formatError(error);return false;}finally{if(current()){s.loading=false;this.ports.changed();}r.finish();}
 }
 stop(){this.catalog.destroy();this.selection.destroy();}
}
