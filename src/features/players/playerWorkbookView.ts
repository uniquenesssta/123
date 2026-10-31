import { escapeHtml } from "../../components/format";
import type {
  SpreadsheetImportPreview
} from "../../types";
import { fileWorkflowView } from "../../ui/patterns";
export function spreadsheetStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    ready_add: "待新增",
    ready_update: "待更新",
    ready_end_previous: "新增并结束旧履历",
    conflict: "冲突",
    error: "错误",
    skip: "跳过",
    imported: "已导入",
  };
  return labels[status] ?? status;
}
export function spreadsheetEntityLabel(type: string): string {
  const labels: Record<string, string> = {
    team: "球队",
    player: "球员",
    player_name: "球员名称",
    player_position: "球员位置",
    player_team_period: "球员所属球队",
    player_ability: "球员能力",
    player_availability: "球员状态",
    player_dynamic_tag: "球员动态标签",
    external_entity_id: "外部数据关联",
    match: "比赛",
    lineup: "阵容",
    lineup_player: "阵容球员",
  };
  return labels[type] ?? "其他数据";
}
export function spreadsheetPanel(preview: SpreadsheetImportPreview | null): string {
  const counts = preview?.counts;
  const blocking = (counts?.conflict ?? 0) + (counts?.error ?? 0);
  const rows = preview?.rows.slice(0, 80) ?? [];
  return fileWorkflowView({className:"",heading:`
    <div class="panel-heading"><div><span>球员月度工作包</span><h2>身份、履历、位置、状态与能力观察</h2></div></div>`,entry:`
    <div class="spreadsheet-actions">
      <article><strong>空白模板</strong><span>包含说明、参考字典、数据缺口和月度维护字段。</span><button class="secondary" data-action="export-player-template">导出模板</button></article>
      <article><strong>月度数据包</strong><span>导出球员、名称、履历、位置、可用性、能力观察和动态标签。</span><button class="secondary" data-action="export-player-data">导出现有数据</button></article>
      <article><strong>导入并预检</strong><span>仅接受球员月度或球员目录工作包；球队月度文件会被识别并阻止误导入。</span><label class="field"><span>导入模式</span><select id="spreadsheet-import-mode"><option value="add_and_update">新增并更新</option><option value="add_only">仅新增</option></select></label><button class="primary" data-action="preview-player-import">选择球员文件并预检</button><button class="ghost" data-page="workbooks">查看全部工作包入口</button></article>
    </div>
`,preview:preview ? `<div class="spreadsheet-preview">
      <div class="preview-summary">
        <div><span>文件</span><strong>${escapeHtml(preview.source_file_name)}</strong></div>
        <div class="preview-count add"><span>待新增</span><strong>${counts?.ready_add ?? 0}</strong></div>
        <div class="preview-count update"><span>待更新</span><strong>${counts?.ready_update ?? 0}</strong></div>
        <div class="preview-count update"><span>结束旧履历</span><strong>${counts?.ready_end_previous ?? 0}</strong></div>
        <div class="preview-count conflict"><span>冲突</span><strong>${counts?.conflict ?? 0}</strong></div>
        <div class="preview-count error"><span>错误</span><strong>${counts?.error ?? 0}</strong></div>
        <div class="preview-count skip"><span>跳过</span><strong>${counts?.skipped ?? 0}</strong></div>
      </div>
      <div class="spreadsheet-table-wrap"><table class="spreadsheet-table"><thead><tr><th>工作表</th><th>行</th><th>类型</th><th>状态</th><th>说明</th></tr></thead><tbody>${rows.map((row) => `<tr class="status-${escapeHtml(row.status)}"><td>${escapeHtml(row.sheet_name)}</td><td>${row.row_number}</td><td>${escapeHtml(spreadsheetEntityLabel(row.entity_type))}</td><td>${escapeHtml(spreadsheetStatusLabel(row.status))}</td><td>${escapeHtml(row.message ?? "")}${row.conflict_candidates.length ? `<div class="candidate-list">${row.conflict_candidates.map((candidate) => `<button class="candidate-choice" data-action="resolve-import-conflict" data-row-id="${escapeHtml(row.id)}" data-entity-id="${escapeHtml(candidate.entity_id)}"><strong>${escapeHtml(candidate.display_name)}</strong>${candidate.detail ? `<small>${escapeHtml(candidate.detail)}</small>` : ""}</button>`).join("")}<button class="candidate-skip" data-action="skip-import-conflict" data-row-id="${escapeHtml(row.id)}">跳过这一行</button></div>` : ""}</td></tr>`).join("")}</tbody></table></div>
      ${preview.rows.length > rows.length ? `<p class="field-note">仅显示前 ${rows.length} 行，完整结果保存在导入批次中。</p>` : ""}
      <div class="button-row"><button class="primary" data-action="commit-player-import" ${blocking > 0 || (counts?.ready_add ?? 0) + (counts?.ready_update ?? 0) + (counts?.ready_end_previous ?? 0) === 0 ? "disabled" : ""}>确认写入数据库</button><button class="secondary" data-action="show-import-preview-json">查看完整预检结果</button>${blocking > 0 ? `<span class="blocking-note">冲突可在上方直接选择正确记录；格式错误需修正表格后重新预检。</span>` : ""}</div>
    </div>` : null,empty:`<div class="empty-inline">尚未选择球员月度工作簿。导入前会显示新增、更新、冲突和错误明细。</div>`});
}
