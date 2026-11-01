import type { SpreadsheetImportPreview } from "../../types";
import { previewPhase, type ImportPhase } from "../../app/imports/spreadsheetBatch";
import { escapeHtml } from "../../components/format";
export const importPhaseLabel: Record<ImportPhase,string> = {
 idle:"尚未预检",choosing:"选择文件中",previewing:"正在预检",ready:"可以提交",blocked:"存在冲突或错误",
 no_changes:"没有待提交变更",writing:"正在写入",unknown:"结果未核实",imported:"已处理，不可重复提交",error:"预检失败",
};
export function workbookPreview(preview:SpreadsheetImportPreview|null,phase=previewPhase(preview),error:string|null=null,category="team"):string {
  const status=importPhaseLabel[phase],counts=preview?.counts;
  const prefix=category==="team"?"team-import":category==="player"?"import":"match-import";
  const labels:Record<string,string>={ready_add:"待新增",ready_update:"待更新",ready_end_previous:"新增并结束旧记录",conflict:"冲突",error:"错误",skip:"跳过",imported:"已导入"};
  return `<section class="panel workbook-preview"><header><h2>${escapeHtml(preview?.source_file_name??"预检结果")}</h2><p>${preview?`批次 ${escapeHtml(preview.batch_id)} · ${preview.import_mode==="add_only"?"仅新增":"新增并更新"}`:"选择工作簿后核对真实批次与行明细。"}</p></header><div class="workbook-state-message" role="status" data-import-phase="${phase}">${escapeHtml(status)}${error?`<p class="danger">${escapeHtml(error)}</p>`:""}</div>
    ${counts&&preview?`<p class="workbook-counts">新增 ${counts.ready_add}　更新 ${counts.ready_update}　结束旧记录 ${counts.ready_end_previous}　<span class="${counts.conflict?"warning-text":""}">冲突 ${counts.conflict}</span>　<span class="${counts.error?"danger":""}">错误 ${counts.error}</span>　已导入 ${counts.imported}　跳过 ${counts.skipped??0}</p>
    <div class="workbook-table-wrap"><table class="workbook-table"><colgroup><col class="workbook-sheet-col"><col class="workbook-row-col"><col class="workbook-object-col"><col class="workbook-status-col"><col></colgroup><thead><tr><th>工作表</th><th>行</th><th>对象 / 操作</th><th>状态</th><th>说明</th></tr></thead><tbody>${preview.rows.slice(0,80).map(row=>`<tr><td>${escapeHtml(row.sheet_name)}</td><td>${row.row_number}</td><td>${escapeHtml(String(row.payload.canonical_name??row.payload.name??row.entity_type))} · ${escapeHtml(row.action)}</td><td>${escapeHtml(labels[row.status]??row.status)}</td><td>${escapeHtml(row.message??"—")}${row.status==="conflict"?`<div class="candidate-list">${row.conflict_candidates.map(candidate=>`<button class="secondary" data-action="resolve-${prefix}-conflict" data-row-id="${escapeHtml(row.id)}" data-entity-id="${escapeHtml(candidate.entity_id)}">${escapeHtml(candidate.display_name)}${candidate.detail?` · ${escapeHtml(candidate.detail)}`:""}</button>`).join("")}<button class="secondary" data-action="skip-${prefix}-conflict" data-row-id="${escapeHtml(row.id)}">跳过这一行</button></div>`:""}</td></tr>`).join("")}</tbody></table></div>
    <p class="field-note">${preview.rows.length>80?`显示前80行，共${preview.rows.length}行；完整结果见批次明细。`:"完整批次与写入结果以实际账本读回为准。"}${phase==="imported"?" 当前显示原批次快照，不是新的提交资格。":""}</p>`:`<p class="workbook-empty-copy">导出模板并填写后选择文件。预检不写入业务数据；未知值不补零。</p>`}</section>`;
}
