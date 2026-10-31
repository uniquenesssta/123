import { escapeHtml } from "../../components/format";
import { fileWorkflowView } from "../../ui/patterns";

import type {
  TeamPackageImportPreview
} from "../../types";

export function teamPackagePanel(preview: TeamPackageImportPreview | null): string {
  const teamCounts = preview?.team_preview?.counts;
  const playerCounts = preview?.player_preview?.counts;
  const blocking =
    (teamCounts?.conflict ?? 0) +
    (teamCounts?.error ?? 0) +
    (playerCounts?.conflict ?? 0) +
    (playerCounts?.error ?? 0);
  const ready =
    (teamCounts?.ready_add ?? 0) +
    (teamCounts?.ready_update ?? 0) +
    (teamCounts?.ready_end_previous ?? 0) +
    (playerCounts?.ready_add ?? 0) +
    (playerCounts?.ready_update ?? 0) +
    (playerCounts?.ready_end_previous ?? 0);
  const rows = [
    ...(preview?.team_preview?.rows.map((row) => ({ scope: "team", row })) ?? []),
    ...(preview?.player_preview?.rows.map((row) => ({ scope: "player", row })) ?? []),
  ].slice(0, 100);
  const statusLabel: Record<string, string> = {
    ready_add: "待新增",
    ready_update: "待更新",
    ready_end_previous: "新增并结束旧记录",
    conflict: "冲突",
    error: "错误",
    skip: "跳过",
    imported: "已导入",
  };
  const coverage = preview?.coverage;
  const p4Ready = Boolean(coverage && coverage.blockers.length === 0 && coverage.readiness_score >= 70 && blocking === 0);
  const readinessClass = p4Ready ? "ready" : coverage ? "warning" : "idle";
  return fileWorkflowView({className:"team-package-panel",heading:`
    <div class="panel-heading"><div><span>统一导入入口</span><h2>球队完整资料包 → P4 输入链路</h2><p>一份 Excel 同时承载球队、球队多语言名称、全部球员、球员多语言名称、基础能力、动态评分、教练历史和阵型分布。</p></div></div>`,entry:`
    <div class="team-package-entry-grid">
      <article class="team-package-primary-action">
        <span class="package-step">主要操作</span>
        <h3>导入完整资料包</h3>
        <p>仅接受 football.team-package.v1 完整资料包。新模板包含“球队名称”和“球员名称”工作表，可维护中文名、英文名、主显示名与历史名称；球队月度文件请使用右侧“球队月度工作包”。</p>
        <label class="field"><span>导入模式</span><select id="team-package-import-mode"><option value="add_and_update">新增并更新</option><option value="add_only">仅新增</option></select></label>
        <button class="primary" data-action="preview-team-package-import">选择 Excel 并统一预检</button>
      </article>
      <article class="team-package-secondary-action">
        <span class="package-step">辅助</span>
        <h3>标准空白资料包</h3>
        <p>仅在需要新建填报文件时导出。日常操作以导入为主，不再要求分别维护球队表和球员表。</p>
        <button class="secondary" data-action="export-team-package-template">导出标准模板</button>
        <button class="ghost" data-page="workbooks">打开球队月度工作包</button>
      </article>
    </div>
`,preview:preview && coverage ? `<div class="team-package-readiness ${readinessClass}">
      <div class="readiness-score"><span>P4 输入就绪度</span><strong>${coverage.readiness_score}</strong><small>/ 100</small></div>
      <div class="readiness-state"><b>${p4Ready ? "数据结构已达到 P4 输入门槛" : "仍有缺口或预检阻断"}</b><span>${escapeHtml(preview.source_file_name)}</span></div>
      <div class="readiness-metrics">
        <div><span>球队</span><strong>${coverage.team_count}</strong></div>
        <div><span>球员</span><strong>${coverage.player_count}</strong></div>
        <div><span>教练</span><strong>${coverage.coach_count}</strong></div>
        <div><span>阵型记录</span><strong>${coverage.formation_usage_count}</strong></div>
        <div><span>能力观察</span><strong>${coverage.player_ability_count}</strong></div>
        <div><span>动态标签</span><strong>${coverage.player_dynamic_tag_count}</strong></div>
        <div><span>默认角色</span><strong>${coverage.player_role_count}</strong></div>
      </div>
    </div>
    <div class="package-message-grid">
      ${coverage.blockers.length ? `<section class="package-message blocker"><strong>必须处理</strong>${coverage.blockers.map((item) => `<span>${escapeHtml(item)}</span>`).join("")}</section>` : `<section class="package-message ok"><strong>结构阻断</strong><span>没有发现资料包结构性阻断。</span></section>`}
      ${coverage.warnings.length ? `<section class="package-message warning"><strong>建议补全</strong>${coverage.warnings.map((item) => `<span>${escapeHtml(item)}</span>`).join("")}</section>` : `<section class="package-message ok"><strong>完整度提示</strong><span>核心球队、阵容、能力和动态数据均已覆盖。</span></section>`}
    </div>
    <div class="preview-summary"><div><span>待处理总数</span><strong>${ready}</strong></div><div class="preview-count add"><span>球队链新增</span><strong>${teamCounts?.ready_add ?? 0}</strong></div><div class="preview-count update"><span>球员链新增</span><strong>${playerCounts?.ready_add ?? 0}</strong></div><div class="preview-count conflict"><span>冲突</span><strong>${(teamCounts?.conflict ?? 0) + (playerCounts?.conflict ?? 0)}</strong></div><div class="preview-count error"><span>错误</span><strong>${(teamCounts?.error ?? 0) + (playerCounts?.error ?? 0)}</strong></div></div>
    <div class="spreadsheet-table-wrap"><table class="spreadsheet-table"><thead><tr><th>链路</th><th>工作表</th><th>行</th><th>类型</th><th>状态</th><th>说明</th></tr></thead><tbody>${rows.map(({ scope, row }) => `<tr class="status-${escapeHtml(row.status)}"><td>${scope === "team" ? "球队/教练" : "球员/评分"}</td><td>${escapeHtml(row.sheet_name)}</td><td>${row.row_number}</td><td>${escapeHtml(row.entity_type)}</td><td>${escapeHtml(statusLabel[row.status] ?? row.status)}</td><td>${escapeHtml(row.message ?? "")}${row.conflict_candidates.length ? `<div class="candidate-list">${row.conflict_candidates.map((candidate) => `<button class="candidate-choice" data-action="resolve-team-package-conflict" data-package-scope="${scope}" data-row-id="${escapeHtml(row.id)}" data-entity-id="${escapeHtml(candidate.entity_id)}"><strong>${escapeHtml(candidate.display_name)}</strong>${candidate.detail ? `<small>${escapeHtml(candidate.detail)}</small>` : ""}</button>`).join("")}<button class="candidate-skip" data-action="skip-team-package-conflict" data-package-scope="${scope}" data-row-id="${escapeHtml(row.id)}">跳过这一行</button></div>` : ""}</td></tr>`).join("")}</tbody></table></div>
    <div class="button-row"><button class="primary" data-action="commit-team-package-import" ${blocking > 0 || ready === 0 || coverage.blockers.length > 0 ? "disabled" : ""}>确认导入完整资料包</button><button class="secondary" data-action="show-team-package-preview-json">查看完整预检</button><button class="secondary" data-action="export-team-package-preview-json">导出完整预检 JSON</button>${blocking > 0 ? `<span class="blocking-note">仍有 ${blocking} 条冲突或错误；冲突可直接选择，格式错误需修正 Excel 后重新预检。</span>` : ""}</div>
    ` : null,empty:`<div class="empty-state team-package-empty"><strong>尚未导入球队完整资料包</strong><p>优先选择已经整理好的 Excel。预检不会写入数据库，并会先检查 P4 所需的球队、阵容、能力、动态评分、教练与阵型覆盖情况。</p></div>`});
}
