import type {OpenAiProfileSummary} from '../../types';
import type {ProfileDraft} from './profileContracts';
export const newProfileDraft=():ProfileDraft=>({id:null,name:'新的兼容 API 配置',api_protocol:'responses',api_endpoint:'https://api.openai.com/v1/responses',api_base_url:'https://api.openai.com/v1',token_limit_field:'max_output_tokens',api_workspace_web_search_mode:'disabled',api_example_template:null,research_model:'gpt-5.5',extraction_model:'gpt-5.5',fallback_model:null,reasoning_effort:'medium',timeout_seconds:180,max_retries:3,max_concurrency:2,max_output_tokens:12000,max_tool_calls:12,search_context_size:'high'});
export function profileDraft(profile:OpenAiProfileSummary):ProfileDraft {const draft=newProfileDraft();for(const key of Object.keys(draft) as Array<keyof ProfileDraft>)(draft as unknown as Record<string,unknown>)[key]=profile[key];draft.api_workspace_web_search_mode='disabled';return draft;}
export function profileValidation(d:ProfileDraft,profiles:OpenAiProfileSummary[]):string|null {
 if(!d.name.trim()||[...d.name].length>80)return '配置名称须为 1–80 个字符。';
 if(profiles.some(p=>p.id!==d.id&&p.name.trim().toLocaleLowerCase()===d.name.trim().toLocaleLowerCase()))return '配置名称已存在，请使用其他名称。';
 if(!['responses','chat_completions'].includes(d.api_protocol))return '请选择受支持的协议。';
 try {for(const raw of [d.api_endpoint,d.api_base_url]){const url=new URL(raw);if(url.username||url.password||url.search||url.hash)throw Error();if(url.protocol!=='https:'&&!(url.protocol==='http:'&&['localhost','127.0.0.1','[::1]'].includes(url.hostname)))throw Error();}const endpoint=new URL(d.api_endpoint);if(!endpoint.pathname.replace(/\/$/,'').endsWith(d.api_protocol==='responses'?'/responses':'/chat/completions'))return '完整端点路径与协议不一致。';}catch{return '端点须为 HTTPS；仅本机回环地址允许 HTTP，不能包含凭据、查询参数或片段。';}
 for(const model of [d.research_model,d.extraction_model])if(!model||!/^[-\w.:]+$/.test(model)||new TextEncoder().encode(model).length>120)return '模型 ID 须为 1–120 字节，仅使用字母、数字、- _ . :。';
 if(d.fallback_model&&(!/^[-\w.:]+$/.test(d.fallback_model)||new TextEncoder().encode(d.fallback_model).length>120))return '备用模型 ID 格式不正确。';
 for(const [field,min,max] of [['timeout_seconds',10,900],['max_retries',0,10],['max_concurrency',1,16],['max_tool_calls',1,100],['max_output_tokens',1,100000]] as const)if(!Number.isInteger(d[field])||d[field]<min||d[field]>max)return `${field} 超出允许范围 ${min}–${max}。`;
 if(!['none','minimal','low','medium','high','xhigh'].includes(d.reasoning_effort)||!['low','medium','high'].includes(d.search_context_size))return '研究参数选项无效。';
 return null;
}
