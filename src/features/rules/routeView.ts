import { escapeHtml } from "../../components/format";
import { competitionKindLabel } from "../../components/competition";
import type { RouteDecision } from "../../types";

export function routeDetailBody(route: RouteDecision, labels: { source: string; model: string }): string {
  return `<div class="route-detail">
    <div class="route-detail-summary"><span>系统判定</span><h3>${escapeHtml(route.package_display_name)}</h3><p>${escapeHtml(labels.source)}，用于${escapeHtml(competitionKindLabel(route.competition_profile.competition_kind))}。</p></div>
    <div class="model-basis-flow route-basis-flow">
      <div><span>1 · 识别赛事</span><strong>${escapeHtml(competitionKindLabel(route.competition_profile.competition_kind))}</strong></div><i>→</i>
      <div><span>2 · 匹配层级</span><strong>${escapeHtml(labels.source)}</strong></div><i>→</i>
      <div><span>3 · 选择规则</span><strong>${escapeHtml(route.package_display_name)}</strong></div><i>→</i>
      <div><span>4 · 运行模型</span><strong>${escapeHtml(labels.model)}</strong></div>
    </div>
    <dl class="detail-facts route-readable-facts"><div><dt>模型版本</dt><dd>${escapeHtml(route.model_version)}</dd></div><div><dt>参数版本</dt><dd>${escapeHtml(route.parameter_version)}</dd></div><div><dt>规则包版本</dt><dd>${escapeHtml(route.package_version)}</dd></div><div><dt>预览边界</dt><dd>仅解释当前路由；正式执行仍需后端重新校验</dd></div></dl>
    <details class="technical-details"><summary>技术追踪信息</summary><dl class="detail-facts compact"><div><dt>规则包编号</dt><dd>${escapeHtml(route.rule_package_id)}</dd></div><div><dt>绑定编号</dt><dd>${escapeHtml(route.binding_id ?? "默认匹配")}</dd></div><div><dt>模型版本编号</dt><dd>${escapeHtml(route.model_version_id)}</dd></div><div><dt>参数集编号</dt><dd>${escapeHtml(route.parameter_set_id)}</dd></div></dl></details>
  </div>`;
}

export function routeCard(route: RouteDecision, labels: { source: string; model: string }): string {
  return `
    <div class="route-card">
      <div class="route-card-main"><span>${escapeHtml(labels.source)}</span><strong>${escapeHtml(route.package_display_name)}</strong><small>规则包版本 ${escapeHtml(route.package_version)}</small></div>
      <div class="route-chain"><div><span>赛事类型</span><b>${escapeHtml(competitionKindLabel(route.competition_profile.competition_kind))}</b></div><div><span>匹配方式</span><b>${escapeHtml(labels.source)}</b></div><div><span>模型</span><b>${escapeHtml(labels.model)}</b></div><div><span>参数版本</span><b>${escapeHtml(route.parameter_version)}</b></div></div>
      <button class="secondary" data-action="show-route-json">查看完整判定</button>
    </div>`;
}
