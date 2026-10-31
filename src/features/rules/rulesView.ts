import { competitionCatalogue, competitionStructure } from "../competitions/index";
import { bindingWorkspace } from "./bindingView";
import { competitionKindLabel } from "../../components/competition";
import { escapeHtml } from "../../components/format";
import { inlineDatabaseSetup } from "../../components/databaseSetup";
import { taskContextRibbon, taskPageHeader } from "../../ui/patterns/workbench";
import { workspaceSectionNavigation } from "../../ui/navigation";

import type { BootstrapResponse, RulePackageDraft } from "../../types";

function modelLabel(modelId: string): string {
  const labels: Record<string, string> = {
    p4: "P4 通用函数曲线协同模型",
    p4_league: "P4 联赛 90 分钟模型",
    p4_group_stage: "P4 小组赛 90 分钟模型",
    p4_knockout_90: "P4 单回合淘汰赛 90 分钟模型",
    p4_knockout_two_leg_90: "P4 两回合淘汰赛 90 分钟模型",
    p4_friendly: "P4 友谊赛 90 分钟模型",
    p7: "历史 P7 模型（只读）",
    p7_league: "历史 P7 联赛模型（只读）",
    p7_group_stage: "历史 P7 小组赛模型（只读）",
    p7_knockout_90: "历史 P7 单回合模型（只读）",
    p7_knockout_two_leg_90: "历史 P7 两回合模型（只读）",
    p7_friendly: "历史 P7 友谊赛模型（只读）",
  };
  return labels[modelId] ?? modelId;
}

export function rulesPage(
  state: BootstrapResponse,
  pendingRulePackage: RulePackageDraft | null,
  activeSection: string,
  readingPackage = false,
): string {
  const disabled = state.data.database_configured ? "" : "disabled";
  const activeP4Packages = [...state.data.rule_packages]
    .filter((item) => item.status === "active"
      && (item.model_id === "p4" || item.model_id.startsWith("p4_")))
    .sort((left, right) => new Date(right.created_at).getTime() - new Date(left.created_at).getTime());
  const p4ProductionPackages = Array.from(new Map(activeP4Packages.map((item) => [item.package_key, item])).values())
    .sort((left, right) => right.priority - left.priority || left.display_name.localeCompare(right.display_name, "zh-CN"));
  const packageOptions = p4ProductionPackages.map((item) => `<option value="${escapeHtml(item.id)}">${escapeHtml(item.display_name)} · ${escapeHtml(item.version)}</option>`).join("");
  const p4Bindings = state.data.competition_bindings.filter((binding) => binding.is_active && (binding.model_id === "p4" || binding.model_id.startsWith("p4_")));
  const section = ["catalog", "structure", "routing", "packages"].includes(activeSection) ? activeSection : "catalog";
  const sectionNav = workspaceSectionNavigation([
    { id: "catalog", index: "01", label: "赛事目录", description: "三级分类、搜索与规则路径", badge: `${state.data.competitions.length}` },
    { id: "structure", index: "02", label: "赛事结构", description: "赛事、赛季、阶段与轮次", badge: `${state.data.seasons.length}/${state.data.stages.length}` },
    { id: "routing", index: "03", label: "模型路由", description: "作用范围与生产规则绑定", badge: `${p4Bindings.length}` },
    { id: "packages", index: "04", label: "规则包", description: "导入、预检与注册", badge: `${p4ProductionPackages.length}` },
  ], section);
  const connected = state.data.database_configured;

  return `<section class="module-workspace-page model-rules-workspace">
    ${taskPageHeader({
      eyebrow: "赛事设置",
      title: "赛事目录与自动模型规则",
      description: "目录、赛事层级、路由绑定与规则包分别维护；本页不执行模型。",
      status: { label: connected ? `${state.data.competitions.length} 项赛事 · ${p4ProductionPackages.length} 个 P4 生产规则` : "等待数据库", tone: connected ? "success" : "warning" },
      actions: '<button class="secondary" data-action="refresh-service-state">刷新</button><button class="secondary" data-page="prediction">进入赛事推演</button><button class="secondary" data-action="select-workspace-section" data-section-id="routing">路由优先级</button>',
    })}
    ${taskContextRibbon([
      { label: "赛事目录", value: `${state.data.competitions.length} 项`, note: `${state.data.seasons.length} 个赛季 · ${state.data.stages.length} 个阶段`, tone: connected ? "accent" : "neutral" },
      { label: "生产规则包", value: `${p4ProductionPackages.length} 个`, note: "只统计已启用的 P4 生产规则", tone: p4ProductionPackages.length > 0 ? "success" : "warning" },
      { label: "自动路由", value: `${p4Bindings.length} 条`, note: "本场 → 阶段 → 赛季 → 赛事 → 类型默认", tone: p4Bindings.length > 0 ? "success" : "neutral" },
      { label: "待注册文件", value: pendingRulePackage ? pendingRulePackage.display_name : "无", note: pendingRulePackage ? `版本 ${pendingRulePackage.version}` : "选择 JSON 后先预检再注册", tone: pendingRulePackage ? "accent" : "neutral" },
    ])}
    <div class="core-local-navigation">${sectionNav}</div>
    <div class="module-workspace-stage" data-workspace-scroll-key="rules-stage">
      <section class="workspace-module-view ${section === "catalog" ? "active" : ""}" data-workspace-section="catalog">
        <div class="module-section-stack">
          ${connected ? "" : inlineDatabaseSetup("连接数据服务以管理赛事和规则", "连接成功后可创建赛事、赛季、阶段、轮次和模型绑定。", state.connection_error)}
          <section class="panel rules-directory-panel"><div class="panel-heading"><div><h2>赛事筛选与目录</h2><p class="field-note">按体系、地区和赛制筛选；规则路径是目录级说明。</p></div></div>${competitionCatalogue(state)}</section>
        </div>
      </section>

      ${competitionStructure(state, section, p4Bindings.length)}
      ${bindingWorkspace(state, section, packageOptions)}
      <section class="workspace-module-view ${section === "packages" ? "active" : ""}" data-workspace-section="packages">
        <div class="module-section-stack">
          <div class="module-section-heading"><div><span>规则包</span><h2>导入、预检并注册生产规则</h2><p>客户端只接受标准赛事文档生成的 JSON；注册前显示模型、版本与适用范围。</p></div></div>
          <section class="panel"><div class="panel-heading"><div><span>导入规则包</span><h2>选择标准规则包文件</h2></div><span class="status-pill ${pendingRulePackage ? "online" : ""}">${readingPackage ? "正在读取" : pendingRulePackage ? "等待注册" : "等待文件"}</span></div><div class="file-import-row"><label class="secondary file-button">选择规则包文件<input id="rule-package-file" type="file" accept=".json,application/json" /></label><span class="field-note">先做格式、版本和适用范围校验，不直接覆盖现有生产规则。</span></div>${pendingRulePackage ? `<article class="rule-package-preview"><div><span>待注册规则包</span><strong>${escapeHtml(pendingRulePackage.display_name)}</strong><small>版本 ${escapeHtml(pendingRulePackage.version)}</small></div><div class="route-chain"><div><span>赛事类型</span><b>${escapeHtml(competitionKindLabel(pendingRulePackage.competition_profile.competition_kind))}</b></div><div><span>模型</span><b>${escapeHtml(modelLabel(pendingRulePackage.routing.model_id))}</b></div><div><span>模型版本</span><b>${escapeHtml(pendingRulePackage.routing.model_version)}</b></div><div><span>参数版本</span><b>${escapeHtml(pendingRulePackage.routing.parameter_version)}</b></div></div><div class="button-row"><button class="primary" data-action="register-rule-package" ${readingPackage ? "disabled" : disabled}>校验并注册</button><button class="ghost" data-action="clear-rule-package">取消</button><button class="secondary" data-action="show-pending-rule-package">只读详情</button></div></article>` : `<div class="empty-state compact"><strong>尚未选择文件</strong><span>选择文件后显示名称、适用赛事、模型和参数版本。</span></div>`}</section>
        </div>
      </section>
    </div>
  </section>`;
}
