import type {OpenAiProfileDraft,OpenAiProfileSummary} from '../../types';
import type {ProfileOperation,ProfileOperationKind} from './profileContracts';
import {profileDraft} from './profileDrafts';
import type {ProfilesFeature} from './ProfilesFeature';
/** Writes retain their original editor identity. A lost reply never authorizes automatic replay. */
export class ProfileOperations {
 private readonly active=new Set<string>();private reconciliation=0;
 constructor(private readonly feature:ProfilesFeature){}
 private start(key:string,kind:ProfileOperationKind):ProfileOperation {
  const s=this.feature.state;if(this.active.has(key)||s.operations[key]&&!s.operations[key].released)throw Error('原配置操作仍在处理或结果待核验，不能重复提交。');
  const op:ProfileOperation={id:crypto.randomUUID(),kind,profileId:s.drafts[key]?.draft.id??null,phase:'pending',checked:false,released:false,message:''};s.operations[key]=op;s.save();
  if(s.storageError){delete s.operations[key];throw Error('恢复记录未能保存，本次尚未提交。');}this.active.add(key);this.feature.invalidateReads();this.feature.changed();return op;
 }
 private failed(key:string,op:ProfileOperation,error:unknown,secret:string|null=null){const s=this.feature.state;op.phase='unknown';op.message=secret?this.feature.ports.formatError(error).split(secret).join('[REDACTED]'):this.feature.ports.formatError(error);op.checked=false;s.save();if(s.selected===key)s.error='原配置操作结果待核验；元数据与系统凭据可能部分完成。请先核验，不要重复提交。';}
 private finish(key:string){this.active.delete(key);this.feature.state.save();this.feature.changed();}
 async save(secret:string|null,clearSecret:()=>void,test=false):Promise<string|null> {
  const f=this.feature,s=f.state,e=s.entry,key=s.selected,validation=f.validation();if(!e||validation)throw Error(validation??'没有可保存的配置。');
  const revision=e.revision,command:OpenAiProfileDraft={...structuredClone(e.draft),api_workspace_web_search_mode:'disabled',api_key:secret?.trim()||null};const op=this.start(key,'save');clearSecret();
  try {
   const saved=await f.ports.api.saveOpenAiProfile(command);if(!f.alive())return null;if(command.id&&saved.id!==command.id)throw Error('保存响应配置身份不匹配。');
   f.invalidateReads();op.profileId=saved.id;const destination='profile:'+saved.id;
   if(s.catalog){s.catalog.profiles=s.catalog.profiles.filter(p=>p.id!==saved.id).concat(saved);s.catalog.active_profile_id=saved.is_active?saved.id:s.catalog.active_profile_id;}
   if(e.revision===revision){e.draft=profileDraft(saved);e.dirty=false;}else e.draft.id=saved.id;
   s.drafts[destination]=e;if(key!==destination)delete s.drafts[key];if(s.selected===key)s.selected=destination;delete s.operations[key];s.save();
   this.active.delete(key);
   if(test&&e.revision===revision)await this.test(saved,e.revision,destination);
   else if(test){s.tests.set(saved.id,{revision,result:null,message:'配置已保存；草稿随后变化，本次没有发送连接测试。'});}
   return saved.id;
  }catch(error){this.failed(key,op,error,command.api_key);return null;}finally{command.api_key=null;this.finish(key);}
 }
 private async test(saved:OpenAiProfileSummary,revision:number,key:string){
  const f=this.feature,s=f.state,op=this.start(key,'test');
  try {const result=await f.ports.api.testOpenAiProfile(saved.id);if(!f.alive())return;if(result.profile_id!==saved.id||result.protocol!==saved.api_protocol||result.endpoint_url!==saved.api_endpoint||result.model_id!==saved.research_model)throw Error('连接测试响应与原配置快照不一致。');
   f.invalidateReads();const row=s.catalog?.profiles.find(p=>p.id===saved.id);if(row?.updated_at===saved.updated_at){row.last_test_status='success';row.last_tested_at=result.tested_at;row.last_test_message='最小连接请求成功';row.updated_at=result.tested_at;}const current=s.drafts[key];s.tests.set(saved.id,{revision,result:current?.revision===revision?result:null,message:current?.revision===revision?'最小连接请求成功；不代表正式研究全部能力已验证。':'原测试已结束，但当前草稿已变化，结果不验证新草稿。'});
   delete s.operations[key];
  }catch(error){if(f.alive()){op.phase='unknown';op.message=f.ports.formatError(error);s.tests.set(saved.id,{revision,result:null,message:'配置已保存；连接测试失败或回执未确认。'+f.ports.formatError(error)});}}
  finally{this.finish(key);}
 }
 prepare(kind:'clear'|'delete'|'activate') {
  const f=this.feature,s=f.state,key=s.selected,e=s.entry,p=s.profile,selection=s.selection,view=f.ports.view();if(!e||!p)throw Error('原配置已不可用，请重新读取。');
  const revision=e.revision,version=p.updated_at,id=p.id;if(s.blocked)throw Error('原配置有未结束操作。');if(kind==='activate'&&e.dirty)throw Error('请先明确保存当前草稿，再设为当前。');let used=false;
  const current=()=>!used&&f.alive()&&view()&&s.selected===key&&s.selection===selection&&s.entry===e&&e.revision===revision&&s.profile?.updated_at===version;
  return {id,name:p.name,run:async()=>{if(!current())throw Error('确认的配置、草稿或页面已变化，请重新确认。');used=true;const op=this.start(key,kind);
   try {const result=await (kind==='clear'?f.ports.api.clearOpenAiProfileKey(id):kind==='delete'?f.ports.api.deleteOpenAiProfile(id):f.ports.api.setActiveOpenAiProfile(id));if(!f.alive())return;f.invalidateReads();
    if(kind==='clear'&&result.profiles.find(p=>p.id===id)?.has_api_key!==false||kind==='delete'&&result.profiles.some(p=>p.id===id)||kind==='activate'&&result.active_profile_id!==id)throw Error('配置操作回执未确认原对象结果。');
    delete s.operations[key];if(kind==='delete'){delete s.drafts[key];s.tests.delete(id);if(s.selected===key)s.selected='';}if(kind==='clear')s.tests.delete(id);s.publish(result);
   }catch(error){this.failed(key,op,error);}finally{this.finish(key);}
  }};
 }
 async reconcile(key=this.feature.state.selected){const f=this.feature,s=f.state,op=s.operations[key];if(!op||op.phase!=='unknown'||this.active.has(key))return false;const generation=++this.reconciliation;op.checked=false;const accepted=await f.load(()=>f.alive());if(!accepted||!f.alive()||s.operations[key]!==op||generation!==this.reconciliation)return false;op.checked=true;op.message='已读取完整配置目录与密钥存在状态；无法据此确认原请求的全部写入或已删除密钥。请核对后决定是否允许后续操作。';s.save();f.changed();return true;}
 prepareRelease(){const f=this.feature,s=f.state,key=s.selected,e=s.entry,op=s.operations[key],selection=s.selection,revision=e?.revision,generation=this.reconciliation,view=f.ports.view();if(!op||op.phase!=='unknown'||!op.checked||this.active.has(key))throw Error('请先核验原配置。');let used=false;return {id:op.id,run:async()=>{if(used||generation!==this.reconciliation||!f.alive()||!view()||s.selected!==key||s.selection!==selection||s.entry?.revision!==revision||s.operations[key]!==op||!op.checked||this.active.has(key))throw Error('核验对象已变化，请重新确认。');used=true;op.released=true;s.error=null;s.save();f.changed();}};}
}
