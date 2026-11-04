const aiInvoke=window.__TAURI_INTERNALS__.invoke;
window.__ai={details:new Map(['A','B','C'].map(id=>[id,window.aiDetail(id)])),writes:[],cancels:[],archives:[],delaySend:false,delayList:false,slowReads:{},failedRead:null,failedList:false,failSend:null};
window.__TAURI_INTERNALS__.invoke=async(command,args)=>{
 const s=window.__ai,record=()=>window.__calls.push({command,args:structuredClone(args)});
 if(command==='list_openai_profiles'){record();return {active_profile_id:'profile-a',profiles:[{id:'profile-a',name:'演示API-A',has_api_key:true,is_active:true,api_protocol:'responses'},{id:'profile-b',name:'演示API-B',has_api_key:true,is_active:false,api_protocol:'chat_completions'}]};}
 if(command==='list_api_workspace_presets'){record();return [{key:'plain_chat',title:'通用问答',description:'普通文本',category:'通用',requires_match:false,suggested_questions:['解释未知值'],allowed_operation_types:[],web_search_enabled:false}];}
 if(command==='list_api_workspace_sessions'){record();if(s.delayList)await new Promise(resolve=>s.resolveList=resolve);if(s.failedList)throw Error('QA 会话目录读取失败');return [...s.details.values()].filter(d=>d.session.status==='active').map(d=>structuredClone(d.session));}
 if(command==='read_api_workspace_session'){record();const value=structuredClone(s.details.get(args.sessionId));if(s.slowReads[args.sessionId])await new Promise(resolve=>s.slowReads[args.sessionId]=resolve);if(s.failedRead===args.sessionId)throw Error('QA 所选会话读取失败');if(!value)throw Error('QA session missing');return value;}
 if(command==='send_api_workspace_message'){record();const c=structuredClone(args.command);s.writes.push(c);if(s.delaySend)await new Promise(resolve=>s.resolveSend=resolve);if(s.failSend)throw Error(s.failSend);const id=c.session_id??'created-'+s.writes.length,d=structuredClone(s.details.get(id)??window.aiDetail(id));d.messages.push({...structuredClone(d.messages[0]),id:id+'-u-'+s.writes.length,role:'user',content:c.message},{...structuredClone(d.messages[1]),id:id+'-a-'+s.writes.length,role:'assistant',content:'QA 原请求回答 '+c.request_id});d.session.message_count=d.messages.length;s.details.set(id,d);return structuredClone(d);}
 if(command==='cancel_api_workspace_request'){record();s.cancels.push(args.requestId);return true;}
 if(command==='archive_api_workspace_session'){record();s.archives.push(args.sessionId);const d=s.details.get(args.sessionId);if(d)d.session.status='archived';if(s.archiveLost)throw Error('QA 归档回执丢失');return null;}
 return aiInvoke(command,args);
};
