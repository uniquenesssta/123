import type {AiWorkspaceFeature} from './aiWorkspaceFeature';
import type {PlayerDetail,TeamDetail} from '../../types';
interface ActionsPorts {
 confirm(title:string,description:string,facts:readonly (readonly [string,string])[],label:string,run:()=>Promise<void>):void;
 show(title:string,message:string):void;copy(value:string):Promise<void>;notify(message:string):void;section(value:'chat'|'history'):void;render():void;navigate():Promise<void>;
 source:{readTeam(id:string):Promise<TeamDetail>;readPlayer(id:string):Promise<PlayerDetail>;team():TeamDetail|null;player():PlayerDetail|null;captureTeam():()=>boolean;capturePlayer():()=>boolean};
}
export class AiWorkspaceActions {
 constructor(private readonly feature:AiWorkspaceFeature,private readonly ports:ActionsPorts){}
 async handle(button:HTMLElement):Promise<boolean>{const action=button.dataset.action,f=this.feature,s=f.state;
  switch(action){
   case 'new-api-workspace-session':f.newSession();this.ports.section('chat');this.ports.render();break;
   case 'refresh-api-workspace':await f.load();break;
   case 'select-api-workspace-session':if(button.dataset.sessionId){this.ports.section('chat');await f.select(button.dataset.sessionId);}break;
   case 'retry-api-workspace-session':if(s.sessionId)await f.select(s.sessionId);break;
   case 'archive-api-workspace-session':{const t=f.prepareArchive(button.dataset.sessionId??'');this.ports.confirm('归档 AI 问答会话','移出常用目录，历史消息、旧提案和历史文件继续保留。',[['会话',t.title],['ID',t.id]],'确认归档',async()=>{await t.run();});break;}
   case 'reconcile-api-workspace-archive':await f.archives.reconcile(button.dataset.sessionId??'');break;
   case 'use-api-workspace-prompt':f.update('message',button.dataset.prompt??'');this.ports.render();document.querySelector<HTMLTextAreaElement>('#api-workspace-message')?.focus();break;
   case 'clear-api-workspace-draft':f.update('message','');this.ports.render();document.querySelector<HTMLTextAreaElement>('#api-workspace-message')?.focus();break;
   case 'copy-api-workspace-message':{const m=s.detail?.messages.find(m=>m.id===button.dataset.messageId);if(m){await this.ports.copy(m.content);this.ports.notify('AI 回答已复制');}break;}
   case 'cancel-api-workspace-request':await f.requests.cancel(button.dataset.requestId??'');break;
   case 'send-api-workspace-message':await f.requests.send();break;
   case 'reconcile-api-workspace-request':await f.requests.reconcile();break;
   case 'release-api-workspace-request':{const t=f.requests.prepareRelease();this.ports.confirm('保留原请求记录并允许新请求','原请求结果仍未知；用户消息或回答可能已经保存。已读取原会话/目录，请人工核对。此操作只解除客户端发送锁，不撤销写入，也不会重发原消息。',[['原请求 ID',t.id]],'已核对，允许新请求',async()=>{t.release();});break;}
   case 'api-workspace-scope':this.ports.show('AI 问答使用边界','客户端保存普通文本会话；AI 不联网、不生成文件、不执行工具或修改业务资料。上下文须主动勾选，发送瞬间的会话、配置和上下文身份固定。取消只针对指定请求，已保存的消息不会回滚。');break;
   default:return false;
  }return true;
 }
 async openTeam(id:string|null){const source=this.ports.source,current=source.captureTeam();const team=id?(source.team()?.team.id===id?source.team():await source.readTeam(id)):null;if(!current())return;if(id&&team?.team.id!==id)throw Error('球队上下文响应身份不匹配');this.feature.newSession({presetKey:'team_profile_completion',entityType:team?'team':null,entityId:team?.team.id??null,entityLabel:team?.team.canonical_name??null,includeContext:false,message:team?`请基于客户端只读资料，概括“${team.team.canonical_name}”当前已知信息、明显缺口和下一步应通过球队月度 Excel 核验的字段。不要联网，不要生成文件，不要提出数据库写入操作。`:'请说明球队资料月度维护时应优先核验哪些字段，以及如何区分缺失、过期和冲突数据。'});this.ports.section('chat');await this.ports.navigate();}
 async openPlayer(id:string){const source=this.ports.source,current=source.capturePlayer();const player=source.player()?.player.id===id?source.player():await source.readPlayer(id);if(!current())return;if(player?.player.id!==id)throw Error('球员上下文响应身份不匹配');this.feature.newSession({presetKey:'player_profile_completion',entityType:'player',entityId:id,entityLabel:player.player.canonical_name,includeContext:false,message:`请基于客户端只读资料，概括球员“${player.player.canonical_name}”当前已知信息、明显缺口和下一步应通过球员月度 Excel 核验的字段。不要联网，不要生成文件，不要提出数据库写入操作。`});this.ports.section('chat');await this.ports.navigate();}
  input(target:HTMLElement){const f=this.feature;if(target instanceof HTMLTextAreaElement&&target.id==='api-workspace-message'){f.update('message',target.value);const send=document.querySelector<HTMLButtonElement>('[data-action=send-api-workspace-message]');if(send)send.disabled=!target.value.trim()||!f.canSend();return true;}
  if(target instanceof HTMLInputElement&&target.id==='api-workspace-session-search'){f.search(target.value);return true;}return false;}
 change(target:HTMLElement){const fields:Record<string,'profileId'|'presetKey'|'matchId'>={'api-workspace-profile':'profileId','api-workspace-preset':'presetKey','api-workspace-match':'matchId'},id=target.dataset.aiField??target.id;if(target instanceof HTMLSelectElement&&fields[id]){this.feature.update(fields[id],fields[id]==='matchId'?target.value||null:target.value);this.ports.render();return true;}if(target instanceof HTMLInputElement&&target.id==='api-workspace-include-context'){this.feature.update('includeContext',target.checked);this.ports.render();return true;}return false;}
}
