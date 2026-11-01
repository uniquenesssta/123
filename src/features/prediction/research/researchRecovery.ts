import type { BootstrapResponse, PlanP4HorizonsCommand } from "../../../types";
import type { PreferenceStore } from "../../../platform/storage";
import type { ResearchDecisionIntent } from "./researchDecision";

export type ResearchReceipt =
  | {kind:"plan";matchId:string;kickoff:string;command:PlanP4HorizonsCommand}
  | {kind:"decision";matchId:string;intent:ResearchDecisionIntent};
export interface ResearchRecovery {read():ResearchReceipt|null;save(value:ResearchReceipt):void;clear():void;}
/** Receipts contain public task identifiers only, partitioned by actual database identity. */
export function researchRecovery(store:PreferenceStore,key:string,snapshot:()=>BootstrapResponse|null):ResearchRecovery {
  const identity=()=>{const s=snapshot();if(!s?.data.database_configured||!s.data.database_url)return null;const u=new URL(s.data.database_url);return JSON.stringify([u.protocol,u.hostname,u.port,u.pathname,u.username]);};
  const all=()=>store.readObject<{receipts:Record<string,ResearchReceipt>}>(key)?.receipts??{};
  return {
    read(){const id=identity(),value=id?all()[id]:null;return value&&value.matchId&&["plan","decision"].includes(value.kind)?structuredClone(value):null;},
    save(value){const id=identity();if(!id)throw Error("数据库身份未核实，不能安全保存研究恢复信息");store.writeJson(key,{receipts:{...all(),[id]:structuredClone(value)}});},
    clear(){const id=identity(),receipts={...all()};if(id&&receipts[id]){delete receipts[id];if(Object.keys(receipts).length)store.writeJson(key,{receipts});else store.removeItem(key);}}
  };
}
