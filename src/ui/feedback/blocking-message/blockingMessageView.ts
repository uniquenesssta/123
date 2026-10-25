import { iconSlot } from "../../icons/iconSlot";
import { buttonView } from "../../primitives/button";

/** Approved Error variant only. The Shell binds recovery to its page read, never a write. */
export function blockingMessageView(retryAction: string, logsAction: string): string {
  return `<section class="ui-blocking-message" role="alert" aria-labelledby="shell-load-error-title">
    <div class="blocking-message-content">${iconSlot("alert-circle", 16)}<div><h2 id="shell-load-error-title">本页数据未能载入</h2><p>请检查连接后重新加载。若问题持续，请查看问题日志；不要重复提交写入操作。</p></div></div>
    <div class="blocking-message-actions">${buttonView({label:"查看问题日志",attributes:{"data-action":logsAction}})}${buttonView({label:"重新加载",variant:"primary",attributes:{"data-action":retryAction}})}</div>
  </section>`;
}
