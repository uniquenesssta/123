import type {ProfilesFeature} from './ProfilesFeature';
import type {ProfileControls} from './ProfileControls';
import type {ProfileSection} from './profileContracts';
interface ProfileActionPorts {confirm(title:string,description:string,facts:Array<[string,string]>,label:string,run:()=>Promise<void>):void;show(title:string,message:string):void}
export class ProfileActions {
 constructor(private readonly feature:ProfilesFeature,private readonly controls:ProfileControls,private readonly ports:ProfileActionPorts){}
 private async launch(task:Promise<unknown>,key:string){if(!this.feature.state.operations[key]||this.feature.state.operations[key].phase!=='pending'){await task;return;}void task.catch(error=>{this.feature.state.error=this.feature.ports.formatError(error);this.feature.changed();});}
 async handle(button:HTMLElement):Promise<boolean>{const f=this.feature,s=f.state,c=this.controls,action=button.dataset.action;
  switch(action){
   case 'new-openai-profile':c.clear();f.create();break;
   case 'select-openai-profile':if(button.dataset.profileId&&s.entry?.draft.id!==button.dataset.profileId){c.clear();f.select(button.dataset.profileId);}break;
   case 'refresh-openai-profiles':await f.load();break;
   case 'set-openai-section':if(['profiles','request','models','tests'].includes(button.dataset.profileSection??'')){c.example.close();f.section(button.dataset.profileSection as ProfileSection);}break;
   case 'openai-example-open':c.example.close();f.section('example');document.getElementById('openai-api-example')?.focus();break;
   case 'openai-example-parse':await c.example.parse();break;
   case 'openai-example-apply':c.example.apply();document.querySelector<HTMLElement>('[data-action=openai-example-open]')?.focus();break;
   case 'openai-example-close':c.example.close();f.section('request');document.querySelector<HTMLElement>('[data-action=openai-example-open]')?.focus();break;
   case 'toggle-openai-key-visibility':{const input=document.querySelector<HTMLInputElement>('#openai-api-key');if(input){input.type=input.type==='password'?'text':'password';button.textContent=input.type==='password'?'显示本次输入':'隐藏本次输入';}break;}
   case 'save-openai-profile':case 'test-openai-profile':{if(c.example.unapplied())throw Error('请先明确应用或关闭示例，再保存配置。');const key=s.selected,e=s.entry,revision=e?.revision,selection=s.selection,view=f.ports.view(),test=action==='test-openai-profile';if(!e)throw Error('请选择配置。');if(f.validation())throw Error(f.validation()!);this.ports.confirm(test?'保存配置并测试连接':'保存兼容 API 配置',test?'先保存当前草稿，再向当前端点发送可能计费的最小请求。连接成功不代表正式研究能力完整通过。':'只保存当前配置与本次输入的密钥；留空保留旧密钥，不会自动启用其他配置。',[['配置',e.draft.name],['协议',e.draft.api_protocol],['密钥',c.secretValue()?'本次输入将替换保存':'留空，保留原密钥']],test?'保存并测试':'保存配置',async()=>{if(!view()||s.selected!==key||s.entry!==e||e.revision!==revision||s.selection!==selection)throw Error('确认的配置、草稿或页面已变化，请重新确认。');await this.launch(f.operations.save(c.secretValue(),()=>c.clearSecret(),test),key);});break;}
   case 'activate-openai-profile':case 'request-clear-openai-key':case 'request-delete-openai-profile':{const kind=action==='activate-openai-profile'?'activate':action==='request-clear-openai-key'?'clear':'delete',ticket=f.operations.prepare(kind);const title=kind==='activate'?'设为当前配置':kind==='clear'?'移除配置密钥':'删除兼容 API 配置';this.ports.confirm(title,kind==='activate'?'启用已保存配置；密钥存在不代表连接或正式研究已通过。':kind==='clear'?'只移除此配置在 Windows 凭据管理器中的密钥，保留参数；已有保存结果与测试状态分别核验。':'移除此配置参数与对应密钥；历史会话保留原身份，不会自动绑定其他配置。',[['配置',ticket.name],['ID',ticket.id]],title,async()=>{c.clearSecret();const key=s.selected;await this.launch(ticket.run(),key);});break;}
   case 'reconcile-openai-operation':await f.operations.reconcile();break;
   case 'release-openai-operation':{const t=f.operations.prepareRelease();this.ports.confirm('保留未知记录并允许后续操作','已读取原配置，但可能无法确定跨存储的全部结果。此操作不重发、不撤销写入，也不声明密钥已删除。',[['原操作',t.id]],'已核对，允许后续操作',t.run);break;}
   case 'openai-security':this.ports.show('密钥与连接测试边界','密钥只在本次临时输入与凭据调用中存在，不写入草稿、日志或工作区恢复。已保存密钥不会回传。保存、密钥存在、启用、最小连接请求成功和正式研究能力分别判断；清除与删除无撤销。');break;
   default:return false;
  }return true;
 }
}
