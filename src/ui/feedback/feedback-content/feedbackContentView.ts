import { escapeHtml } from "../../../components/format";
import { iconSlot, type UiIcon } from "../../icons/iconSlot";

export type FeedbackContentTone = "info" | "success" | "warning" | "danger";
const toneIcons: Record<FeedbackContentTone, UiIcon> = { info: "info", success: "check", warning: "alert-circle", danger: "alert-circle" };

export function feedbackContentView(message: string, tone: FeedbackContentTone, title?: string): string {
  return `<div class="ui-feedback-content ui-feedback-${tone}">${iconSlot(toneIcons[tone], 16)}<div class="ui-feedback-copy">${title ? `<strong>${escapeHtml(title)}</strong>` : ""}<p>${escapeHtml(message)}</p></div></div>`;
}
