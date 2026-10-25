import { escapeHtml } from "../../../components/format";
import { iconSlot } from "../../icons/iconSlot";
import { statusBadgeView } from "../status-badge";
import { emptyStateView } from "../empty-state";

export interface TimelineItem {
  id: string; time: string; title: string; description: string; metadata?: string; score?: string;
  verification: "Verified" | "Unverified" | "Disputed";
  revision: "Original" | "Corrected" | "Cancelled" | "Superseded";
}
export interface TimelineOptions { label: string; title?: string; summary?: string; items: readonly TimelineItem[]; emptyTitle?: string; emptyDescription?: string }

/** Keeps supplied order and both independent status axes. No score/count calculation. */
export function timelineView(o: TimelineOptions): string {
  if (!o.label.trim()) throw new Error("Timeline requires an accessible name");
  const ids = o.items.map(item => item.id);
  if (ids.some(id => !id) || new Set(ids).size !== ids.length) throw new Error("Timeline item ids must be unique");
  const rows = o.items.map((item, index) => {
    const verified = item.verification === "Verified", disputed = item.verification === "Disputed", cancelled = item.revision === "Cancelled";
    const status = verified ? "已核验" : disputed ? "有争议" : "待核验";
    const revision = item.revision === "Corrected" ? "已修订" : cancelled ? "已取消" : item.revision === "Superseded" ? "已被替代" : "";
    return `<li class="ui-timeline-item" data-event-key="${escapeHtml(item.id)}" data-verification="${item.verification}" data-revision="${item.revision}"><span class="ui-timeline-time">${escapeHtml(item.time)}</span><span class="ui-timeline-rail" aria-hidden="true">${iconSlot(cancelled ? "close" : verified ? "check" : "alert-circle", 16)}${index < o.items.length - 1 ? '<span class="ui-timeline-connector"></span>' : ""}</span><div class="ui-timeline-content"><div class="ui-timeline-heading"><strong>${escapeHtml(item.title)}</strong>${item.score === undefined ? "" : `<span class="ui-timeline-score">${escapeHtml(item.score)}</span>`}</div><div class="ui-timeline-statuses">${statusBadgeView({label:status,tone:verified ? "Success" : disputed ? "Warning" : "Neutral",style:"Outline"})}${revision ? statusBadgeView({label:revision,tone:item.revision === "Corrected" ? "Info" : "Neutral",style:"Outline"}) : ""}</div><p class="ui-timeline-description">${escapeHtml(item.description)}</p>${item.metadata === undefined ? "" : `<p class="ui-timeline-metadata">${escapeHtml(item.metadata)}</p>`}</div></li>`;
  }).join("");
  return `<section class="ui-timeline" data-ui-display="timeline" aria-label="${escapeHtml(o.label)}">${o.title ? `<header><h3>${escapeHtml(o.title)}</h3>${o.summary === undefined ? "" : `<p>${escapeHtml(o.summary)}</p>`}</header>` : ""}${o.items.length ? `<ol>${rows}</ol>` : emptyStateView({title:o.emptyTitle ?? "暂无事件",description:o.emptyDescription})}</section>`;
}
