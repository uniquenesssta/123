import { buttonView } from "../../primitives/button";
import { iconButtonView } from "../../primitives/icon-button";
import { feedbackContentView, type FeedbackContentTone } from "../feedback-content/feedbackContentView";

export interface ToastPresentation { readonly message: string; readonly tone: FeedbackContentTone; readonly actionLabel?: string; }
export function toastView(notice: ToastPresentation): string {
  return `<section class="ui-toast" role="group" aria-label="通知"><div class="ui-toast-row"><div class="ui-toast-alignment">${feedbackContentView(notice.message, notice.tone)}</div>${iconButtonView({ action: "dismiss-notice", label: "关闭通知", icon: "close"})}</div>${notice.actionLabel ? `<div class="ui-toast-actions">${buttonView({label:notice.actionLabel,attributes:{"data-action":"notice-action"}})}</div>` : ""}</section>`;
}
