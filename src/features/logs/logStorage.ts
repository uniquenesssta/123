import type {PreferenceStore} from '../../platform/storage';
import type {LogRecovery,LogSection} from './logContracts';

/** Only navigation and unresolved local operation IDs persist. No log text, export paths or credentials. */
export function logStorage(store:PreferenceStore,key:string) {
  return {
    read():LogRecovery {
      const fallback:LogRecovery={section:'overview',expanded:[],unknownClear:false,operationId:null};
      try {
        const raw=store.getItem(key);if(raw===null)return fallback;
        const value:unknown=JSON.parse(raw);
        if(!value||typeof value!=='object'||Array.isArray(value))throw Error('Invalid recovery');
        const saved=value as Partial<LogRecovery>;
        if(typeof saved.unknownClear!=='boolean')throw Error('Unknown recovery');
        return {section:['overview','processing','records'].includes(saved.section??'')?saved.section as LogSection:'overview',expanded:Array.isArray(saved.expanded)?saved.expanded.filter((id):id is string=>typeof id==='string'&&id.startsWith('ISSUE-')).slice(0,500):[],unknownClear:saved.unknownClear,operationId:typeof saved.operationId==='string'?saved.operationId:null};
      } catch {return {...fallback,unknownClear:true,operationId:'recovery-unreadable'};}
    },
    save(value:LogRecovery){store.writeJson(key,{section:value.section,expanded:value.expanded.filter(id=>id.startsWith('ISSUE-')).slice(0,500),unknownClear:value.unknownClear,operationId:value.operationId});},
  };
}
