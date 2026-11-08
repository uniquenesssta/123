import type { PreferenceStore } from "../../platform/storage";
import type { StoredMatchPredictionCommand, BootstrapResponse } from "../../types";
export type PredictionMode = "formal" | "shadow";
export interface RunReceipt {
  command:StoredMatchPredictionCommand;mode:PredictionMode;manifest:string;matchKey:string;
  baseline:string[];knownRunId?:string;actualManifest?:boolean;
}
export interface PredictionRecovery {read():RunReceipt|null;save(value:RunReceipt):void;clear():void;}
export function predictionRecovery(store:PreferenceStore,key:string,snapshot:()=>BootstrapResponse|null):PredictionRecovery {
  const identity=()=>{const s=snapshot();if(!s?.data.database_configured||!s.data.database_url)return null;const u=new URL(s.data.database_url);return JSON.stringify([u.protocol,u.hostname,u.port,u.pathname,u.username]);};
  const all=()=>store.readObject<{receipts:Record<string,RunReceipt>}>(key)?.receipts??{};
  return {
    read(){const id=identity();if(!id)return null;const r=all()[id];return r&&r.command?.match_id&&typeof r.manifest==="string"&&typeof r.matchKey==="string"&&Array.isArray(r.baseline)&&["formal","shadow"].includes(r.mode)?structuredClone(r):null;},
    save(value){const id=identity();if(!id)throw Error("数据库身份未核实，不能安全记录运行恢复信息");store.writeJson(key,{receipts:{...all(),[id]:structuredClone(value)}});},
    clear(){const id=identity(),receipts={...all()};if(id&&receipts[id]){delete receipts[id];if(Object.keys(receipts).length)store.writeJson(key,{receipts});else store.removeItem(key);}}
  };
}

/** The native adapter rejects these inputs before save_successful_run; no receipt may already have a run ID. */
export function rejectedNativeModelInput(error: unknown): boolean {
  const message = error instanceof Error ? error.message : String(error);
  return /^(?:Error: )?模型执行失败：模型(?:输入|参数)无效：/.test(message);
}
