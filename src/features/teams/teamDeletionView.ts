import { bulkDeleteSummary } from "../../components/entityDeletion";
import { escapeHtml } from "../../components/format";
import type { BulkDeleteResult, EntityDeletionCheck, TeamForceDeletePreview, TeamForceDeleteResult } from "../../types";
const TEAM_REFERENCE_LABELS: Record<string, string> = {
  matches: "比赛",
  lineups: "正式阵容",
  team_lineup_presets: "球队阵容预设",
  player_team_periods: "球员效力履历",
  team_coach_periods: "教练任期",
  team_season_memberships: "赛季注册",
  player_availability: "球员可用性",
  formation_usage: "阵型使用观察",
  team_tactical_observations: "球队战术观察",
  team_ability_observations: "球队能力观察",
  substitutions: "换人记录",
  dynamic_tag_opponents: "对手动态标签",
  team_match_reviews: "球队赛后复盘",
  player_match_reviews: "球员赛后复盘",
  player_match_observations: "球员赛后观察",
};

export function teamDeletionCheckSummary(checks: EntityDeletionCheck[]): string {
  if (checks.length === 0) return "<p>没有需要说明的受保护球队。</p>";
  return `<ul>${checks.map((check) => {
    const references = check.references
      .filter((item) => item.count > 0)
      .map((item) => `${TEAM_REFERENCE_LABELS[item.relation] ?? item.relation} ${item.count}`)
      .join("、");
    const detail = references || check.reason;
    return `<li><strong>${escapeHtml(check.label)}</strong>：${escapeHtml(detail)}</li>`;
  }).join("")}</ul>`;
}

const TEAM_FORCE_DELETE_LABELS: Record<string, string> = {
  teams: "球队主体",
  matches: "相关比赛",
  players: "关联球员主体",
  coaches: "关联教练主体",
  feature_snapshots: "P4 输入快照",
  model_runs: "模型运行",
  research_runs: "研究运行",
  p4_freeze_tasks: "P4 冻结任务",
  postmatch_settlements: "赛后结算",
  match_reviews: "比赛复盘",
  ability_update_candidates: "能力更新候选",
  import_batches: "导入批次",
  team_lineup_presets: "球队阵容预设",
  team_lineup_preset_members: "阵容预设成员",
  player_team_periods: "球员效力关系",
  player_ability_observations: "球员能力观察",
  player_dynamic_tags: "球员动态评分/标签",
  formation_usage_observations: "阵型使用记录",
  team_tactical_observations: "球队战术观察",
  team_ability_observations: "球队能力观察",
};

function forceDeleteReferenceRows(preview: TeamForceDeletePreview): string {
  if (preview.references.length === 0) return "<p>未发现额外关联记录。</p>";
  return `<div class="force-delete-impact-grid">${preview.references
    .map(
      (item) =>
        `<div><span>${escapeHtml(TEAM_FORCE_DELETE_LABELS[item.relation] ?? item.relation)}</span><strong>${item.count}</strong></div>`,
    )
    .join("")}</div>`;
}

export function forceDeleteResultSummary(result: TeamForceDeleteResult): string {
  const counts = Object.entries(result.deleted_counts)
    .filter(([, count]) => count > 0)
    .map(
      ([key, count]) =>
        `<div><span>${escapeHtml(TEAM_FORCE_DELETE_LABELS[key] ?? key)}</span><strong>${count}</strong></div>`,
    )
    .join("");
  return `<div class="route-detail"><div class="route-detail-summary"><span>强制清除完成</span><h3>${escapeHtml(result.label)} 已从当前数据库中移除</h3><p>相关球队、球员、教练、比赛、评分、动态状态、导入批次及直接关联的 P4 历史已在同一事务内清除。</p></div><section class="prediction-detail-section"><h3>清除结果</h3><div class="force-delete-impact-grid">${counts || "<p>没有额外记录。</p>"}</div></section></div>`;
}

export function forceDeleteBody(preview: TeamForceDeletePreview): string { return `<div class="force-delete-warning">
      <strong>这不是归档，也不是普通删除。</strong>
      <p>${escapeHtml(preview.warning)}</p>
      <p>删除成功后，可使用相同球队代码和新版 Excel 重新导入，不会继续读取本次清除的球队、球员、教练、评分、阵型、比赛或 P4 记录。</p>
    </div>
    <section class="prediction-detail-section"><h3>预计清除范围 · ${preview.total_rows} 条</h3>${forceDeleteReferenceRows(preview)}</section>
    <label class="field force-delete-confirmation"><span>请输入完整球队名称确认</span><input id="force-delete-team-confirmation" autocomplete="off" placeholder="${escapeHtml(preview.confirmation_text)}"><small>必须完全输入：${escapeHtml(preview.confirmation_text)}</small></label>`; }
export function teamBulkDeleteSummary(result: BulkDeleteResult): string { return bulkDeleteSummary(result, "支球队"); }
