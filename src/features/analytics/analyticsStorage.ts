import type {PreferenceStore} from '../../platform/storage';
import type {AnalysisExport,AnalysisReceipt,AnalyticsStorage} from './analyticsPorts';
/** Only durable recovery references and acknowledged exports are stored, separately for each database. */
export function analyticsStorage(store:PreferenceStore,key:string,database:()=>string|null):AnalyticsStorage {
 type Data=Record<string,{receipts:Record<string,AnalysisReceipt>;exported:AnalysisExport|null}>;
 return {capture(){
  const db=database(),all=()=>store.readObject<Data>(key)??{};
  const update=(fn:(entry:Data[string])=>void)=>{
   if(!db)throw Error('数据库身份未核实，不能记录分析操作');
   const data=all(),entry=data[db]??{receipts:{},exported:null};fn(entry);store.writeJson(key,{...data,[db]:entry});
  };
  return {
   read:id=>{const r=db?all()[db]?.receipts[id]:null;return r?.key===id?structuredClone(r):null;},
   receipts:()=>db?structuredClone(Object.values(all()[db]?.receipts??{})):[],
   save:receipt=>update(entry=>{entry.receipts[receipt.key]=structuredClone(receipt);}),
   clear:id=>{if(db)update(entry=>{delete entry.receipts[id];});},
   exported:()=>{const value=db?all()[db]?.exported:null;return value?structuredClone(value):null;},
   saveExport:value=>update(entry=>{entry.exported=structuredClone(value);}),
  };
 }};
}
