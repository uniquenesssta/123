import { escapeHtml } from "../../../components/format";
import { spinnerView } from "../../primitives/spinner";
import { inlineMessageView } from "../inline-message";
export interface OperationStateOptions { state: "loading"|"error"|"success"; message: string; title?: string; id?: string; actions?: string }
export function operationStateView(o: OperationStateOptions): string {
  if(o.state!=="loading")return inlineMessageView({...o,tone:o.state==="error"?"danger":"success",live:true});
  return `<div class="ui-operation-loading" role="status" aria-live="polite" ${o.id?`id="${escapeHtml(o.id)}"`:""}>${spinnerView(16)}<span>${escapeHtml(o.message)}</span></div>`;
}
