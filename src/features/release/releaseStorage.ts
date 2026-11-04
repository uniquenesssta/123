import type {PreferenceStore} from '../../platform/storage';
import {initialReleaseDraft,type ReleaseDraft,type ReleaseReceipt,type ReleaseStorage} from './releasePorts';
/** Durable drafts and narrow recovery identity are isolated by the existing database identity. */
export function releaseStorage(store:PreferenceStore,key:string,database:()=>string|null):ReleaseStorage {
 type Entry={draft:ReleaseDraft;pending:ReleaseReceipt|null};type Data=Record<string,Entry>;
 return {capture(){const db=database(),all=()=>store.readObject<Data>(key)??{};
  const entry=()=>db?all()[db]:null;
  const update=(fn:(value:Entry)=>void)=>{if(!db)throw Error('数据库身份未核实，不能保存验收请求');const data=all(),value=data[db]??{draft:initialReleaseDraft(),pending:null};fn(value);store.writeJson(key,{...data,[db]:value});};
  return {draft:()=>structuredClone(entry()?.draft??initialReleaseDraft()),saveDraft:draft=>update(value=>{value.draft=structuredClone(draft);}),pending:()=>structuredClone(entry()?.pending??null),savePending:receipt=>update(value=>{value.pending=structuredClone(receipt);}),clearPending:()=>update(value=>{value.pending=null;})};
 }};
}
