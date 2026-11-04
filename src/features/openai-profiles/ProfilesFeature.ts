import {AsyncScope} from '../../app/lifecycle';
import type {ProfileDraft,ProfileSection,ProfilesPorts} from './profileContracts';
import {ProfilesState} from './ProfilesState';
import {ProfileOperations} from './ProfileOperations';
import {profileValidation} from './profileDrafts';
export class ProfilesFeature {
 readonly state:ProfilesState;readonly operations:ProfileOperations;private readonly reads=new AsyncScope();private closed=false;
 constructor(readonly ports:ProfilesPorts){this.state=new ProfilesState(ports.storage);this.operations=new ProfileOperations(this);}
 alive=()=>!this.closed;
 changed(){if(!this.closed)this.ports.changed();}
 async load(view=this.ports.view()){const r=this.reads.begin(),s=this.state;s.loading=true;s.loadError=null;this.changed();try{const result=await this.ports.api.listOpenAiProfiles();if(!r.isCurrent()||!view()||this.closed)return false;s.publish(result);return true;}catch(error){if(r.isCurrent()&&view()&&!this.closed)s.loadError=this.ports.formatError(error);return false;}finally{if(r.isCurrent()){s.loading=false;this.changed();}r.finish();}}
 select(id:string){if(this.state.section==="example")this.state.section="request";this.state.select(id);this.changed();}
 create(){this.state.create();this.changed();}
 section(section:ProfileSection){this.state.section=section;this.state.save();this.changed();}
 update(name:keyof ProfileDraft,value:ProfileDraft[keyof ProfileDraft]){const e=this.state.entry;if(!e||name==='id')return;(e.draft as unknown as Record<string,unknown>)[name]=value;if(['api_protocol','api_endpoint','api_base_url','research_model','extraction_model','max_output_tokens','token_limit_field'].includes(name))e.draft.api_example_template=null;e.revision++;e.dirty=true;this.state.error=null;this.state.save();}
 validation(){const e=this.state.entry;if(e?.draft.id&&this.state.catalog&&!this.state.catalog.profiles.some(p=>p.id===e.draft.id))return '原配置已不可用，请重新读取或新建独立配置。';return e?profileValidation(e.draft,this.state.catalog?.profiles??[]):'请选择或新建配置。';}
 invalidateReads(){this.reads.invalidate();this.state.loading=false;}
 navigationChanged(){this.reads.invalidate();this.state.loading=false;this.state.selection++;}
 stop(){this.closed=true;this.reads.destroy();}
}
