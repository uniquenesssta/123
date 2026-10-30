import { escapeHtml } from "../../components/format";
import { competitionKindLabel } from "../../components/competition";
import type { BootstrapResponse } from "../../types";

/** Directory candidates are explanatory projections, never a replacement for preview_route. */
export function competitionPathBody(state: BootstrapResponse, id: string): string {
  const competition = state.data.competitions.find(item => item.id === id);
  if (!competition) throw new Error("赛事不存在");
  const bindings = state.data.competition_bindings.filter(item => item.is_active && (item.competition_id === id || !item.competition_id && item.competition_kind === competition.competition_kind)).sort((a, b) => b.priority - a.priority);
  const scope = (item: typeof bindings[number]): string => item.stage_id ? "阶段绑定" : item.season_id ? "赛季绑定" : item.competition_id ? "赛事绑定" : "类型默认";
  return `<div class="route-detail">
    <div class="route-detail-summary"><span>目录级说明 · 非执行判定</span><h3>${escapeHtml(competition.name)}</h3><p>${escapeHtml(competitionKindLabel(competition.competition_kind))}。下列是当前已登记的候选绑定；未包含具体开球时间、比赛及模型上下文，不表示规则已获执行许可。</p></div>
    <section class="prediction-detail-section"><div class="section-heading"><div><span>候选规则</span><h3>已登记的作用范围与匹配顺序</h3></div><small>正式路由由后端按本场上下文重新判定</small></div>${bindings.length ? `<div class="route-option-list">${bindings.map((item, index) => `<article><span>${index + 1}</span><div><strong>${escapeHtml(item.rule_package_name)}</strong><small>${escapeHtml(scope(item))} · 模型 ${escapeHtml(item.model_id)} · 匹配顺序 ${item.priority}</small></div><b>候选</b></article>`).join("")}</div>` : `<div class="empty-state compact"><strong>当前没有赛事绑定</strong><span>后端会按上下文检查可用类型默认规则；目录不推断最终结果。</span></div>`}</section>
    <details class="technical-details"><summary>技术追踪信息</summary><dl class="detail-facts compact"><div><dt>赛事编号</dt><dd>${escapeHtml(id)}</dd></div></dl></details>
  </div>`;
}
