import { escapeHtml } from "../../../components/format";
import { feedbackContentView } from "../feedback-content/feedbackContentView";
import { buttonView, type ButtonOptions } from "../../primitives/button";
import { operationStateView } from "../operation-state";
export type BlockingState="unavailable"|"blocked"|"error"|"retrying"|"fatal";
export interface BlockingMessageOptions { state: BlockingState; title: string; message: string; id?: string; primary?: ButtonOptions; secondary?: ButtonOptions }
/** Actions are supplied by the real read/check owner; no implicit retry of writes. */
export function blockingMessageView(options: BlockingMessageOptions|string, logsAction?: string): string {
  const o: BlockingMessageOptions=typeof options==="string"?{state:"error",title:"本页数据未能载入",message:"请检查连接后重新加载。若问题持续，请查看问题日志；不要重复提交写入操作。",id:"shell-load-error-title",primary:{label:"重新加载",variant:"primary",attributes:{"data-action":options}},secondary:{label:"查看问题日志",attributes:{"data-action":logsAction??"open-logs"}}}:options;
  const busy=o.state==="retrying",id=o.id??"blocking-message-title";
  const tone=o.state==="blocked"||o.state==="unavailable"?"warning":"danger";
  return `<section class="ui-blocking-message" data-state="${o.state}" role="${busy?"status":"alert"}" aria-labelledby="${escapeHtml(id)}" aria-busy="${busy}"><div id="${escapeHtml(id)}">${feedbackContentView(o.message,tone,o.title)}</div>${busy?operationStateView({state:"loading",message:"正在重新检查，请稍候"}):""}${o.primary||o.secondary?`<div class="blocking-message-actions">${o.secondary?buttonView({...o.secondary,disabled:busy||o.secondary.disabled}):""}${o.primary?buttonView({...o.primary,loading:busy}):""}</div>`:""}</section>`;
}
