import type { BulkArchiveResult, BulkDeleteResult } from "../types";
import { escapeHtml } from "./format";
export function bulkArchiveSummary(
  result: BulkArchiveResult,
  entityLabel: string,
): string {
  const failed = result.failed.length
    ? `<ul>${result.failed.map((item) => `<li><strong>${escapeHtml(item.label)}</strong>：${escapeHtml(item.reason)}</li>`).join("")}</ul>`
    : "<p>没有归档失败的记录。</p>";
  return `<div class="route-detail"><div class="route-detail-summary"><span>批量归档结果</span><h3>新归档 ${result.archived_ids.length}，已归档 ${result.already_archived_ids.length} / ${result.requested_count} ${entityLabel}</h3><p>归档只改变活动状态，不删除履历、比赛、阵容、观察或审计记录。</p></div><section class="prediction-detail-section"><h3>失败记录</h3>${failed}</section></div>`;
}

export function bulkDeleteSummary(
  result: BulkDeleteResult,
  entityLabel: string,
): string {
  const blocked = result.blocked.length
    ? `<ul>${result.blocked.map((item) => `<li><strong>${escapeHtml(item.label)}</strong>：${escapeHtml(item.reason)}</li>`).join("")}</ul>`
    : "<p>没有被拦截的记录。</p>";
  return `<div class="route-detail"><div class="route-detail-summary"><span>批量删除结果</span><h3>已删除 ${result.deleted_ids.length} / ${result.requested_count} ${entityLabel}</h3><p>未删除的记录会保留原数据，不会出现半删除关联。</p></div><section class="prediction-detail-section"><h3>拦截记录</h3>${blocked}</section></div>`;
}
