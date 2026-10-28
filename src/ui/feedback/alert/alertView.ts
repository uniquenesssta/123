import { escapeHtml } from "../../../components/format";
import { feedbackContentView, type FeedbackContentTone } from "../feedback-content/feedbackContentView";
import { iconButtonView } from "../../primitives/icon-button";

export interface AlertOptions { message: string; title?: string; tone?: FeedbackContentTone; id?: string; dismissAction?: string; actions?: string; live?: boolean }
/** Local presentation only. Dismissal never changes host validation or request state. */
export function alertView(o: AlertOptions): string {
  const tone=o.tone??"info";
  return `<section class="ui-alert" data-tone="${tone}" ${o.id?`id="${escapeHtml(o.id)}"`:""} ${o.live?`role="${tone==="danger"?"alert":"status"}"`:""}><div class="ui-alert-row">${feedbackContentView(o.message,tone,o.title)}${o.dismissAction?iconButtonView({label:"关闭提示",icon:"close",action:o.dismissAction}):""}</div>${o.actions?`<div class="ui-alert-actions">${o.actions}</div>`:""}</section>`;
}
