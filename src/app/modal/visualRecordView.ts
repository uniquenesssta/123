import { escapeHtml, formatPercent } from "../../components/format";
import { keyValueListView } from "../../ui/data-display/key-value-list";
const visualLabels: Record<string, string> = {
  id: "记录编号", run_id: "运行编号", match_id: "比赛记录", player_id: "球员记录", team_id: "球队记录",
  status: "状态", name: "名称", title: "标题", display_name: "显示名称", canonical_name: "球员姓名",
  competition_name: "赛事", competition_kind: "赛事类型", season_name: "赛季", stage_name: "阶段", round_name: "轮次",
  home_team_name: "主队", away_team_name: "客队", home_win: "主胜概率", draw: "平局概率", away_win: "客胜概率",
  btts: "双方都进球概率", over_2_5: "总进球大于 2.5 概率", kickoff_time: "开球时间", captured_at: "记录时间",
  created_at: "创建时间", updated_at: "更新时间", valid_from: "生效时间", valid_to: "有效至", finalized_at: "确认时间",
  reason: "原因", confidence: "可信度", priority: "匹配优先级", model_id: "模型", model_version: "模型版本",
  parameter_version: "参数版本", package_display_name: "规则包", package_version: "规则包版本", rule_package_name: "规则包",
  source: "选择来源", message: "说明", summary: "结果摘要", details: "详细内容", evidence: "依据",
  output: "推演结果", input: "输入信息", identity: "模型身份", route: "模型选择路径", metrics: "指标",
  metadata: "补充信息", warnings: "提醒", blocking_errors: "需要先处理的问题", suggestions: "建议",
  sample_size: "样本数量", data_coverage: "数据覆盖率", duration_ms: "计算耗时（毫秒）", snapshot_type: "数据时点",
  lineup_type: "阵容类型", formation: "阵型", player_count: "球员人数", starter_count: "首发人数",
  quality_score: "数据可信度", current_value: "当前值", proposed_value: "建议值", calculation_version: "计算版本",
  error_message: "失败原因", progress: "完成进度", attempts: "已尝试次数", max_attempts: "最多尝试次数",
};

const visualTokenLabels: Record<string, string> = {
  home: "主队", away: "客队", team: "球队", player: "球员", match: "比赛", competition: "赛事", season: "赛季",
  stage: "阶段", round: "轮次", win: "胜率", draw: "平局", goal: "进球", goals: "进球", probability: "概率",
  score: "得分", value: "数值", current: "当前", proposed: "建议", expected: "预期", actual: "实际",
  model: "模型", version: "版本", package: "规则包", rule: "规则", parameter: "参数", source: "来源",
  data: "数据", quality: "质量", coverage: "覆盖率", confidence: "可信度", status: "状态", type: "类型",
  created: "创建", updated: "更新", valid: "有效", from: "开始", to: "结束", time: "时间", at: "时间",
  count: "数量", size: "规模", reason: "原因", message: "说明", summary: "摘要", detail: "详情", details: "详情",
  rank: "排名", priority: "优先级", result: "结果", evaluation: "评估", review: "复盘", lineup: "阵容",
};

function visualLabel(key: string): string {
  const direct = visualLabels[key];
  if (direct) return direct;
  const translated = key.split("_").map((token) => visualTokenLabels[token] ?? "").filter(Boolean).join("");
  return translated || "补充信息";
}

function visualScalar(value: string | number | boolean, key: string): string {
  if (typeof value === "boolean") return value ? "是" : "否";
  if (typeof value === "number" && /(confidence|probability|coverage|ratio|rate)$/.test(key.toLowerCase()) && value >= 0 && value <= 1) {
    return formatPercent(value);
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return Number.isInteger(value) ? String(value) : value.toFixed(4).replace(/0+$/, "").replace(/\.$/, "");
  }
  if (typeof value === "string" && /(_at|_time|valid_from|valid_to)$/.test(key)) {
    const parsed = Date.parse(value);
    if (!Number.isNaN(parsed)) return new Date(parsed).toLocaleString();
  }
  if (typeof value === "string") {
    const labels: Record<string, string> = {
      PASS: "通过",
      CHECK: "需要检查",
      succeeded: "已完成",
      pending: "待处理",
      failed: "失败",
      league: "联赛",
      group_stage: "小组赛",
      knockout_single_leg: "单回合淘汰赛",
      knockout_two_leg: "两回合淘汰赛",
      friendly: "友谊赛",
      custom: "自定义",
    };
    return labels[value] ?? value;
  }
  return String(value);
}

function isTechnicalVisualKey(key: string): boolean {
  return /(^id$|_id$|sha|hash|payload|metadata|binding_id|package_key|model_version_id|parameter_set_id|route_reason)/i.test(key);
}

export function renderVisualValue(value: unknown, key = "value", depth = 0): string {
  if (value === null || value === undefined || value === "") return `<span class="visual-empty">未记录</span>`;
  if (["string", "number", "boolean"].includes(typeof value)) {
    return `<span class="visual-value">${escapeHtml(visualScalar(value as string | number | boolean, key))}</span>`;
  }
  if (Array.isArray(value)) {
    if (value.length === 0) return `<span class="visual-empty">暂无记录</span>`;
    const visible = value.slice(0, 30);
    return `<div class="detail-list">${visible.map((item, index) => `<article><span class="visual-index">${index + 1}</span><div>${renderVisualValue(item, key, depth + 1)}</div></article>`).join("")}${value.length > visible.length ? `<div class="visual-more">另有 ${value.length - visible.length} 条记录未展开</div>` : ""}</div>`;
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    if (entries.length === 0) return `<span class="visual-empty">暂无内容</span>`;
    const normal = entries.filter(([entryKey]) => !isTechnicalVisualKey(entryKey));
    const technical = entries.filter(([entryKey]) => isTechnicalVisualKey(entryKey));
    const scalarEntries = normal.filter(([, entryValue]) => entryValue === null || ["string", "number", "boolean"].includes(typeof entryValue));
    const sectionEntries = normal.filter(([, entryValue]) => entryValue !== null && !["string", "number", "boolean"].includes(typeof entryValue));
    const scalarHtml = scalarEntries.length > 0
      ? keyValueListView(scalarEntries.map(([entryKey, entryValue]) => ({ label: visualLabel(entryKey), renderValue: () => renderVisualValue(entryValue, entryKey, depth + 1) })))
      : "";
    const sectionHtml = sectionEntries.map(([entryKey, entryValue]) => `<section class="detail-section"><h3>${escapeHtml(visualLabel(entryKey))}</h3>${renderVisualValue(entryValue, entryKey, depth + 1)}</section>`).join("");
    const technicalHtml = technical.length > 0
      ? `<details class="technical-details"><summary>技术追踪信息</summary>${keyValueListView(technical.map(([entryKey, entryValue]) => ({ label: visualLabel(entryKey), renderValue: () => renderVisualValue(entryValue, entryKey, depth + 1) })))}</details>`
      : "";
    return `<div class="detail-stack ${depth > 0 ? "nested" : ""}">${scalarHtml}${sectionHtml}${technicalHtml}</div>`;
  }
  return `<span class="visual-value">${escapeHtml(String(value))}</span>`;
}
