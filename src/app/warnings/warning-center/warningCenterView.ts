import { escapeHtml } from "../../../components/format";
import { iconSlot } from "../../../ui/icons/iconSlot";
import { iconButtonView } from "../../../ui/primitives/icon-button/iconButtonView";
import { feedbackContentView } from "../../../ui/feedback/feedback-content/feedbackContentView";
import { warningDateGroups } from "../warning-store";
import type { WarningRecord } from "../warning-dto";

export function warningEntryView(unread: number): string {
  const count = unread > 99 ? "99+" : String(unread);
  return `<span class="warning-entry-group"><button type="button" class="ui-icon-button" data-action="open-warning-center" aria-label="告警中心，${unread} 条未读" aria-haspopup="dialog" title="告警中心">${iconSlot("alert-circle", 16)}</button>${unread ? `<span class="warning-unread-badge" aria-hidden="true">${count}</span>` : ""}</span>`;
}

export function warningListView(records: readonly WarningRecord[]): string {
  if (!records.length) return '<div class="warning-empty"><strong>暂无告警</strong><p>本次会话的告警会显示在这里。</p></div>';
  return warningDateGroups(records).map(group => `<section class="warning-date-group"><h3>${escapeHtml(group.label)}</h3>${group.records.map(row => {
    const time = new Date(row.lastSeenAt).toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit", hour12: false });
    return `<article class="warning-record"><div class="warning-record-message warning-record-${row.severity === "warning" ? "warning" : "danger"}">${feedbackContentView(row.message, row.severity === "warning" ? "warning" : "danger", row.title)}</div><p class="warning-record-meta">${escapeHtml(time)} · ${escapeHtml(row.source)} · 发生 ${row.occurrences} 次 · ${row.resolved ? "已恢复" : row.read ? "已读" : "未读"}</p></article>`;
  }).join("")}</section>`).join("");
}

export function warningDialogView(records: readonly WarningRecord[], unread: number): string {
  return `<dialog class="warning-center-dialog" aria-modal="true" aria-labelledby="warning-center-title" aria-describedby="warning-center-description"><header class="warning-center-header"><div><h2 id="warning-center-title">告警中心</h2><p id="warning-center-description">本次会话 · ${records.length ? `${unread} 条未读` : "暂无告警"}</p></div>${iconButtonView("close-warning-center", "关闭告警中心", "close")}</header><div class="warning-center-content">${warningListView(records)}</div><footer class="warning-center-actions"><button type="button" class="ui-button ui-button-secondary" data-action="read-all-warnings" ${records.length ? "" : "hidden"} ${unread ? "" : "disabled"}>全部标为已读</button><button type="button" class="ui-button ui-button-secondary" data-action="warning-open-logs">查看问题日志</button><button type="button" class="ui-button ui-button-secondary" data-action="close-warning-center" autofocus>关闭</button></footer></dialog>`;
}
