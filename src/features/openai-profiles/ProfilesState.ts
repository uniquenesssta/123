import type {OpenAiProfilesState,OpenAiProfileTestResult} from '../../types';
import type {ProfileEntry,ProfileOperation,ProfileSection,ProfilesPorts} from './profileContracts';
import {newProfileDraft,profileDraft} from './profileDrafts';
export class ProfilesState {
 catalog:OpenAiProfilesState|null=null;selected='';drafts:Record<string,ProfileEntry>={};operations:Record<string,ProfileOperation>={};section:ProfileSection='profiles';loading=false;loaded=false;loadError:string|null=null;error:string|null=null;storageError:string|null=null;selection=0;tests=new Map<string,{revision:number;result:OpenAiProfileTestResult|null;message:string}>();
 constructor(private readonly storage:ProfilesPorts['storage']){const saved=storage.read();if(saved){this.selected=saved.selected;this.drafts=saved.drafts;this.operations=saved.operations;this.section=saved.section;}}
 get entry(){return this.drafts[this.selected]??null;}
 get profile(){const id=this.entry?.draft.id;return this.catalog?.profiles.find(p=>p.id===id)??null;}
 get blocked(){const op=this.operations[this.selected];return !!op&&!op.released;}
 save(){try{this.storage.save({selected:this.selected,drafts:this.drafts,operations:this.operations,section:this.section});this.storageError=null;}catch{this.storageError='非敏感草稿/恢复记录未保存，请检查本地存储。';}}
 select(id:string){this.selected='profile:'+id;const profile=this.catalog?.profiles.find(p=>p.id===id);if(profile&&!this.drafts[this.selected])this.drafts[this.selected]={draft:profileDraft(profile),revision:0,dirty:false};this.selection++;this.error=null;this.save();}
 create(){this.selected='new:'+crypto.randomUUID();this.drafts[this.selected]={draft:newProfileDraft(),revision:0,dirty:false};this.selection++;this.section='request';this.error=null;this.save();}
 publish(catalog:OpenAiProfilesState){this.catalog=catalog;for(const p of catalog.profiles){const key='profile:'+p.id,e=this.drafts[key];if(e&&!e.dirty&&!this.operations[key])e.draft=profileDraft(p);}if(!this.selected){const id=catalog.active_profile_id??catalog.profiles[0]?.id;if(id)this.select(id);}this.loaded=true;this.save();}
}
