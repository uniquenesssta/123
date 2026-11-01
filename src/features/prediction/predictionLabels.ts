import type { RouteDecision } from "../../types";
export function routeSourceLabel(source: RouteDecision["source"]): string {
  const labels: Record<RouteDecision["source"], string> = {
    explicit_rule_package: "本场规则包覆盖",
    stage_binding: "赛事阶段绑定",
    season_binding: "赛季绑定",
    competition_binding: "具体赛事绑定",
    competition_kind_default: "赛事类型默认",
  };
  return labels[source];
}

export function modelDisplayName(modelId: string): string {
  const labels: Record<string, string> = {
    p4: "P4 通用低比分概率模型",
    p4_league: "P4 联赛低比分概率模型",
    p4_group_stage: "P4 小组赛低比分概率模型",
    p4_knockout_90: "P4 单回合淘汰赛模型",
    p4_knockout_two_leg_90: "P4 两回合淘汰赛模型",
    p4_friendly: "P4 友谊赛低比分概率模型",
    p7: "P7 通用函数曲线模型",
    p7_league: "P7 联赛 90 分钟模型",
    p7_group_stage: "P7 小组赛 90 分钟模型",
    p7_knockout_90: "P7 单回合淘汰赛 90 分钟模型",
    p7_knockout_two_leg_90: "P7 两回合淘汰赛 90 分钟模型",
    p7_friendly: "P7 友谊赛 90 分钟模型",
  };
  return labels[modelId] ?? modelId;
}
