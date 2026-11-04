import type {PreferenceStore} from '../../platform/storage';
import {emptyAiDraft,type AiDraft,type AiSavedState,type AiStorage} from './aiWorkspacePorts';
function normalize(raw:unknown):AiSavedState|null {
 if(!raw||typeof raw!=='object')return null;
 const value=raw as AiSavedState, drafts:Record<string,AiDraft>={};
 if(typeof value.selected!=='string'||!value.drafts||typeof value.drafts!=='object')return null;
 for(const [key,d]of Object.entries(value.drafts)){
  if(!/^(session|new):/.test(key)||!d||typeof d.message!=='string')continue;
  const base=emptyAiDraft();for(const field of ['message','profileId','presetKey'] as const)if(typeof d[field]==='string')base[field]=d[field];
  for(const field of ['matchId','entityId','entityLabel'] as const)if(typeof d[field]==='string')base[field]=d[field];
  base.entityType=d.entityType==='team'||d.entityType==='player'?d.entityType:null;base.includeContext=d.includeContext===true;drafts[key]=base;
 }
 const r=value.receipt,receipt=r&&typeof r.id==='string'&&typeof r.origin==='string'&&typeof r.startedAt==='string'?{id:r.id,origin:r.origin,sessionId:typeof r.sessionId==='string'?r.sessionId:null,startedAt:r.startedAt,phase:(['completed','cancelled','unknown'].includes(r.phase)?r.phase:'unknown') as NonNullable<AiSavedState['receipt']>['phase'],error:null,released:r.released===true,checked:false}:null;
 const archives:AiSavedState['archives']={};for(const [id,phase]of Object.entries(value.archives??{}))if(phase==='pending'||phase==='unknown')archives[id]='unknown';
 return {selected:value.selected,drafts,receipt,archives,search:typeof value.search==='string'?value.search:''};
}
/** The Feature owns drafts; storage persists only its projection, never profiles, keys or database passwords. */
export function aiWorkspaceStorage(store:PreferenceStore,key:string,database:()=>string|null):AiStorage {
 return{capture(){const db=database();return{read:()=>db?normalize(store.readObject<Record<string,AiSavedState>>(key)?.[db]):null,save(value){if(!db)return;const all=store.readObject<Record<string,AiSavedState>>(key)??{};store.writeJson(key,{...all,[db]:{selected:value.selected,drafts:value.drafts,receipt:value.receipt?{...value.receipt,error:null}:null,archives:value.archives,search:value.search}});}};}};
}
