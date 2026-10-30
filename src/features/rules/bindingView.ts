import { escapeHtml } from "../../components/format";
import { competitionKindLabel } from "../../components/competition";
import type { BootstrapResponse } from "../../types";
import { competitionRegionLabel, hierarchyOptions } from "../competitions/index";
function bindingCatalogue(state: BootstrapResponse): string {
  const p4Bindings = state.data.competition_bindings.filter((binding) => binding.is_active && (binding.model_id === "p4" || binding.model_id.startsWith("p4_")));
  if (p4Bindings.length === 0) return `<div class="empty-state"><strong>暂无 P4 绑定</strong><span>系统会尝试使用 P4 赛事类型默认规则。</span></div>`;
  const grouped = new Map<string, typeof p4Bindings>();
  for (const binding of [...p4Bindings].sort((left, right) => right.priority - left.priority)) {
    const competition = binding.competition_id ? state.data.competitions.find((item) => item.id === binding.competition_id) : null;
    const group = competition ? `${competitionRegionLabel(competition)} / ${competition.name}` : `通用默认 / ${binding.competition_kind ? competitionKindLabel(binding.competition_kind) : "未限定类型"}`;
    grouped.set(group, [...(grouped.get(group) ?? []), binding]);
  }
  return `<div class="binding-catalogue compact-binding-catalogue">${Array.from(grouped.entries()).map(([group, bindings]) => `<details class="binding-group"><summary><div><strong>${escapeHtml(group)}</strong><span>${bindings.length} 条规则</span></div><b>查看</b></summary>${bindings.map((item) => `<div class="list-row"><div><strong>${escapeHtml(item.binding_name)}</strong><small>${escapeHtml(item.rule_package_name)} · ${item.competition_id ? "赛事专属" : "类型默认"}</small></div><span class="priority">${item.stage_id ? "阶段范围" : item.season_id ? "赛季范围" : item.competition_id ? "赛事范围" : "类型默认"} · 顺序 ${item.priority}</span></div>`).join("")}</details>`).join("")}</div>`;
}

export function bindingWorkspace(state: BootstrapResponse, section: string, packageOptions: string): string {
  const disabled = state.data.database_configured ? "" : "disabled";
  const { competitionOptions, seasonOptions, stageOptions } = hierarchyOptions(state);
  const p4Bindings = state.data.competition_bindings.filter(item => item.is_active && (item.model_id === "p4" || item.model_id.startsWith("p4_")));
  return `      <section class="workspace-module-view ${section === "routing" ? "active" : ""}" data-workspace-section="routing">
        <div class="module-section-stack">
          <div class="module-section-heading"><div><span>模型路由</span><h2>指定赛事、赛季或阶段使用的生产规则包</h2><p>更具体的范围优先；同一范围内按匹配顺序从高到低判定。</p></div></div>
          <section class="rules-route-strip"><strong>判定优先级</strong><span>本场覆盖 → 阶段 → 赛季 → 具体赛事 → 赛事类型默认</span></section>
          <section class="panel"><div class="two-column">
            <article class="subpanel"><div class="panel-heading"><div><span>作用范围</span><h2>建立自动匹配规则</h2></div></div><label class="field"><span>赛事</span><select id="binding-competition-id"><option value="">不限定赛事</option>${competitionOptions}</select></label><label class="field"><span>赛季</span><select id="binding-season-id"><option value="">不限定赛季</option>${seasonOptions}</select></label><label class="field"><span>阶段</span><select id="binding-stage-id"><option value="">不限定阶段</option>${stageOptions}</select></label><label class="field"><span>规则包</span><select id="binding-rule-package-id"><option value="">选择规则包</option>${packageOptions}</select></label><div class="field-row"><label class="field"><span>规则名称</span><input id="binding-name" placeholder="例如 2026 淘汰赛规则" /></label><label class="field"><span>匹配顺序</span><input id="binding-priority" type="number" value="100" /><small class="field-note">数字越大越优先。</small></label></div><button class="primary" data-action="create-binding" ${disabled}>保存自动匹配规则</button></article>
            <article class="subpanel table-panel"><div class="panel-heading padded"><div><span>已启用 P4 规则</span><h2>${p4Bindings.length} 条</h2></div></div>${bindingCatalogue(state)}</article>
          </div></section>
        </div>
      </section>

`;
}
