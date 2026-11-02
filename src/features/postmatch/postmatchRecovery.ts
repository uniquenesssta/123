import type {PreferenceStore} from '../../platform/storage';
import type {PostmatchReceipt,PostmatchRecovery,RecoveryScope} from './postmatchPorts';
/** A narrow, database-bound recovery reference. Captured handles never switch to a different database. */
export function postmatchRecovery(store:PreferenceStore,key:string,database:()=>string|null):PostmatchRecovery {
  type Records=Record<string,Record<string,PostmatchReceipt>>;
  return {capture():RecoveryScope{
    const db=database();
    const all=()=>store.readObject<Records>(key)??{};
    return {
      read(id){const value=db?all()[db]?.[id]:null;return value&&value.key===id?structuredClone(value):null;},
      save(receipt){if(!db)throw Error('数据库身份未核实，不能发送赛后写入');const data=all();store.writeJson(key,{...data,[db]:{...data[db],[receipt.key]:structuredClone(receipt)}});},
      clear(id){if(!db)return;const data=all(),records={...data[db]};delete records[id];if(Object.keys(records).length)data[db]=records;else delete data[db];if(Object.keys(data).length)store.writeJson(key,data);else store.removeItem(key);},
    };
  }};
}
