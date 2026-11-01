import type { ModelRunListItem } from "../../types";
import type { RunReceipt } from "./predictionRecovery";
const object=(v:unknown):Record<string,unknown>=>v&&typeof v==="object"&&!Array.isArray(v)?v as Record<string,unknown>:{};
/** Read-only evidence: matching input alone never proves which client request created a run. */
export function matchesRun(receipt:RunReceipt,doc:Record<string,unknown>,requireManifest=true):boolean {
 const audit=object(doc.input_audit),model=String(doc.model_key??""),selection=receipt.command.model_family;
 return doc.match_key===receipt.matchKey&&doc.snapshot_type===receipt.command.snapshot_type
  &&(!requireManifest||audit.manifest_sha256===receipt.manifest)&&(model===selection||(!selection.includes("_")&&model.startsWith(selection+"_")))
  &&(!receipt.command.explicit_rule_package_id||doc.rule_package_id===receipt.command.explicit_rule_package_id);
}
export function completeRun(doc:Record<string,unknown>):boolean {
 const audit=object(doc.input_audit);
 return /^[a-f0-9]{64}$/i.test(String(audit.manifest_sha256??""))&&typeof doc.id==="string"&&doc.id.trim().length>0&&Object.keys(object(doc.output)).length>0&&typeof audit.feature_snapshot_id==="string"&&audit.feature_snapshot_id.trim().length>0;
}
export async function findExistingRuns(receipt:RunReceipt,api:{listRecentRuns(limit:number):Promise<ModelRunListItem[]>;readRun(id:string):Promise<Record<string,unknown>>}):Promise<Record<string,unknown>[]> {
 if(receipt.knownRunId){const doc=await api.readRun(receipt.knownRunId);if(doc.id!==receipt.knownRunId||!matchesRun(receipt,doc,receipt.actualManifest===true))throw Error("原运行身份、窗口或输入指纹不匹配");return [doc];}
 if(receipt.mode==="shadow")return [];
 const rows=await api.listRecentRuns(500),found:Record<string,unknown>[]=[];
 for(const row of rows.filter(r=>!receipt.baseline.includes(r.id)&&r.match_key===receipt.matchKey&&r.snapshot_type===receipt.command.snapshot_type)){
   const doc=await api.readRun(row.id);if(doc.id===row.id&&matchesRun(receipt,doc,false))found.push(doc);
 }
 return found;
}
