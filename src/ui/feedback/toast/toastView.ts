import { escapeHtml } from "../../../components/format";
import { iconButtonView } from "../../primitives/icon-button/iconButtonView";
import { feedbackContentView, type FeedbackContentTone } from "../feedback-content/feedbackContentView";

export interface ToastPresentation { readonly message: string; readonly tone: FeedbackContentTone; readonly actionLabel?: string; }
export function toastView(notice: ToastPresentation): string {
  return `<section class="ui-toast" role="group" aria-label="通知"><div class="ui-toast-row"><div class="ui-toast-alignment">${feedbackContentView(notice.message, notice.tone)}</div>${iconButtonView("dismiss-notice", "关闭通知", "close")}</div>${notice.actionLabel ? `<div class="ui-toast-actions"><button type="button" class="ui-button ui-button-secondary" data-action="notice-action">${escapeHtml(notice.actionLabel)}</button></div>` : ""}</section>`;
}
