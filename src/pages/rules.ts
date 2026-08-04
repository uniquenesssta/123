import { competitionKindLabel, competitionKindOptions, stageKindOptions } from "../components/competition";
import { escapeHtml } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import { workspaceSectionNavigation, workspaceTaskAnchorNavigation } from "../components/workspace";
import type { BootstrapResponse, CompetitionRecord, RulePackageDraft } from "../types";

type CompetitionScope = "national" | "club" | "other";

function metadataText(item: CompetitionRecord, key: string): string {
  const value = item.metadata?.[key];
  return typeof value === "string" ? value.trim() : "";
}

function competitionScope(item: CompetitionRecord): CompetitionScope {
  const explicit = metadataText(item, "scope").toLowerCase();
  if (explicit === "national" || explicit === "club") return explicit;
  const text = `${item.code} ${item.name} ${metadataText(item, "official_name")}`.toLowerCase();
  if (/world cup|世界杯|euro|欧洲杯|nations|国家联赛|qualif|预选|copa america|美洲杯|asian cup|亚洲杯|afcon|非洲杯|gold cup|金杯|national/.test(text)) return "national";
  if (/league|联赛|cup|杯|champions|libertadores|俱乐部|superliga|premier|bundesliga|serie|liga|ligue/.test(text)) return "club";
  return "other";
}

function scopeLabel(scope: CompetitionScope): string {
  if (scope === "national") return "国家队赛事";
  if (scope === "club") return "俱乐部赛事";
  return "其他与自定义";
}

function regionLabel(item: CompetitionRecord): string {
  const explicit = metadataText(item, "menu_region") || metadataText(item, "region") || metadataText(item, "confederation");
  if (explicit) return explicit;
  const labels: Record<string, string> = {
    FIFA: "世界赛事",
    INT: "世界赛事",
    UEFA: "欧洲洲际",
    CONMEBOL: "南美洲际",
    CONCACAF: "中北美洲际",
    AFC: "亚洲洲际",
    CAF: "非洲洲际",
    OFC: "大洋洲洲际",
    KR: "韩国",
    JP: "日本",
    CN: "中国",
  };
  return labels[item.country_code ?? ""] ?? item.country_code ?? "未分类地区";
}

function regionSortOrder(region: string): number {
  const order = ["世界赛事", "欧洲洲际", "南美洲际", "中北美洲际", "亚洲洲际", "非洲洲际", "大洋洲洲际"];
  const index = order.indexOf(region);
  return index >= 0 ? index : 100;
}

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

function bindingForCompetition(state: BootstrapResponse, item: CompetitionRecord) {
  const direct = state.data.competition_bindings
    .filter((binding) => binding.is_active && (binding.model_id === "p4" || binding.model_id.startsWith("p4_")) && binding.competition_id === item.id)
    .sort((left, right) => right.priority - left.priority)[0];
  const fallback = state.data.competition_bindings
    .filter((binding) => binding.is_active && (binding.model_id === "p4" || binding.model_id.startsWith("p4_")) && !binding.competition_id && binding.competition_kind === item.competition_kind)
    .sort((left, right) => right.priority - left.priority)[0];
  return { binding: direct ?? fallback, direct: Boolean(direct) };
}

function competitionCatalogue(state: BootstrapResponse): string {
  if (state.data.competitions.length === 0) {
    return `<div class="empty-state"><strong>暂无赛事</strong><span>可在下方创建赛事，或通过数据导入恢复赛事目录。</span></div>`;
  }

  const competitions = [...state.data.competitions].sort((left, right) => {
    const scopeDifference = ["national", "club", "other"].indexOf(competitionScope(left)) - ["national", "club", "other"].indexOf(competitionScope(right));
    if (scopeDifference !== 0) return scopeDifference;
    const regionDifference = regionSortOrder(regionLabel(left)) - regionSortOrder(regionLabel(right));
    if (regionDifference !== 0) return regionDifference;
    return Number(left.metadata?.sort_order ?? 9999) - Number(right.metadata?.sort_order ?? 9999) || left.name.localeCompare(right.name, "zh-CN");
  });

  const regions = Array.from(new Set(competitions.map(regionLabel))).sort((left, right) => regionSortOrder(left) - regionSortOrder(right) || left.localeCompare(right, "zh-CN"));
  const regionScopeMap = new Map<string, Set<CompetitionScope>>();
  for (const item of competitions) {
    const region = regionLabel(item);
    const scopes = regionScopeMap.get(region) ?? new Set<CompetitionScope>();
    scopes.add(competitionScope(item));
    regionScopeMap.set(region, scopes);
  }

  const rows = competitions.map((item) => {
    const scope = competitionScope(item);
    const region = regionLabel(item);
    const { binding, direct } = bindingForCompetition(state, item);
    const route = binding ? `${direct ? "赛事专属" : "类型默认"}：${binding.rule_package_name}` : "尚未绑定规则";
    const searchText = `${item.name} ${item.code} ${item.country_code ?? ""} ${region} ${competitionKindLabel(item.competition_kind)} ${metadataText(item, "official_name")}`.toLowerCase();
    const seasonPattern = metadataText(item, "season_pattern");
    return `<tr data-rules-competition-row data-scope="${scope}" data-region="${escapeHtml(region)}" data-kind="${escapeHtml(item.competition_kind)}" data-search="${escapeHtml(searchText)}">
      <td><strong>${escapeHtml(item.name)}</strong><small>${escapeHtml(item.code)}</small></td>
      <td>${escapeHtml(scopeLabel(scope))}</td>
      <td>${escapeHtml(region)}</td>
      <td>${escapeHtml(competitionKindLabel(item.competition_kind))}</td>
      <td><span class="rules-route-text">${escapeHtml(route)}</span>${seasonPattern ? `<small>${escapeHtml(seasonPattern === "cross_year" ? "跨年赛季" : seasonPattern === "calendar" ? "自然年赛季" : "锦标赛赛季")}</small>` : ""}</td>
      <td><div class="rules-row-actions"><button class="secondary tiny" data-action="show-competition-path" data-competition-id="${escapeHtml(item.id)}">规则路径</button><button class="ghost tiny danger-quiet" data-action="request-delete-competition" data-competition-id="${escapeHtml(item.id)}" data-competition-name="${escapeHtml(item.name)}">删除</button></div></td>
    </tr>`;
  }).join("");

  return `<div class="rules-directory" data-rules-directory>
    <div class="rules-directory-toolbar">
      <label class="rules-search"><span>搜索赛事</span><input id="rules-competition-search" type="search" placeholder="赛事、国家、地区或代码"></label>
      <label class="rules-kind-filter"><span>赛制</span><select id="rules-competition-kind"><option value="">全部赛制</option>${competitionKindOptions(null)}</select></label>
      <span class="rules-visible-count"><b id="rules-visible-count">${competitions.length}</b> / ${competitions.length} 项</span>
    </div>
    <div class="rules-directory-grid">
      <nav class="rules-scope-list" aria-label="赛事一级分类">
        <strong>1级 · 参赛体系</strong>
        <button class="active" type="button" data-rules-scope="">全部赛事 <span>${competitions.length}</span></button>
        ${(["national", "club", "other"] as CompetitionScope[]).map((scope) => `<button type="button" data-rules-scope="${scope}">${scopeLabel(scope)} <span>${competitions.filter((item) => competitionScope(item) === scope).length}</span></button>`).join("")}
      </nav>
      <nav class="rules-region-list" aria-label="赛事二级分类">
        <strong>2级 · 地区 / 足联</strong>
        <button class="active" type="button" data-rules-region="" data-region-scopes="national,club,other">全部地区 <span>${competitions.length}</span></button>
        ${regions.map((region) => `<button type="button" data-rules-region="${escapeHtml(region)}" data-region-scopes="${escapeHtml(Array.from(regionScopeMap.get(region) ?? []).join(","))}">${escapeHtml(region)} <span>${competitions.filter((item) => regionLabel(item) === region).length}</span></button>`).join("")}
      </nav>
      <div class="rules-competition-pane">
        <div class="rules-pane-heading"><div><strong>3级 · 具体赛事</strong><span>选择赛事后查看规则路径或维护目录</span></div></div>
        <div class="rules-table-scroll">
          <table class="rules-competition-table">
            <thead><tr><th>赛事</th><th>体系</th><th>地区</th><th>赛制</th><th>自动规则</th><th>操作</th></tr></thead>
            <tbody>${rows}</tbody>
          </table>
          <div id="rules-empty-filter" class="empty-state compact" hidden><strong>没有匹配赛事</strong><span>调整一级、二级分类或搜索条件。</span></div>
        </div>
      </div>
    </div>
  </div>`;
}

function bindingCatalogue(state: BootstrapResponse): string {
  const p4Bindings = state.data.competition_bindings.filter((binding) => binding.is_active && (binding.model_id === "p4" || binding.model_id.startsWith("p4_")));
  if (p4Bindings.length === 0) return `<div class="empty-state"><strong>暂无 P4 绑定</strong><span>系统会尝试使用 P4 赛事类型默认规则。</span></div>`;
  const grouped = new Map<string, typeof p4Bindings>();
  for (const binding of [...p4Bindings].sort((left, right) => right.priority - left.priority)) {
    const competition = binding.competition_id ? state.data.competitions.find((item) => item.id === binding.competition_id) : null;
    const group = competition ? `${regionLabel(competition)} / ${competition.name}` : `通用默认 / ${binding.competition_kind ? competitionKindLabel(binding.competition_kind) : "未限定类型"}`;
    grouped.set(group, [...(grouped.get(group) ?? []), binding]);
  }
  return `<div class="binding-catalogue compact-binding-catalogue">${Array.from(grouped.entries()).map(([group, bindings]) => `<details class="binding-group"><summary><div><strong>${escapeHtml(group)}</strong><span>${bindings.length} 条规则</span></div><b>查看</b></summary>${bindings.map((item, index) => `<div class="list-row"><div><strong>${escapeHtml(item.binding_name)}</strong><small>${escapeHtml(item.rule_package_name)} · ${item.competition_id ? "赛事专属" : "类型默认"}</small></div><span class="priority">${index === 0 ? "优先采用" : `备用 ${index + 1}`}</span></div>`).join("")}</details>`).join("")}</div>`;
}

export function rulesPage(
  state: BootstrapResponse,
  pendingRulePackage: RulePackageDraft | null,
  activeSection: string,
): string {
  const disabled = state.data.database_configured ? "" : "disabled";
  const competitionOptions = state.data.competitions.map((item) => `<option value="${escapeHtml(item.id)}">${escapeHtml(item.name)}</option>`).join("");
  const seasonOptions = state.data.seasons.map((item) => `<option value="${escapeHtml(item.id)}" data-competition-id="${escapeHtml(item.competition_id)}">${escapeHtml(item.competition_name)} · ${escapeHtml(item.name)}</option>`).join("");
  const stageOptions = state.data.stages.map((item) => `<option value="${escapeHtml(item.id)}" data-season-id="${escapeHtml(item.season_id)}" data-competition-id="${escapeHtml(item.competition_id)}">${escapeHtml(item.competition_name)} · ${escapeHtml(item.season_name)} · ${escapeHtml(item.name)}</option>`).join("");
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
      description: "一级和二级导航只负责到达本页；赛事目录、层级结构、模型路由与规则包都在当前页面完成。",
      status: { label: connected ? `${state.data.competitions.length} 项赛事 · ${p4ProductionPackages.length} 个 P4 生产规则` : "等待数据库", tone: connected ? "success" : "warning" },
      actions: '<button class="secondary" data-page="prediction">进入赛事推演</button>',
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
          <div class="module-section-heading"><div><span>赛事目录</span><h2>按参赛体系、地区与具体赛事查找</h2><p>目录面板内部滚动；选择赛事后可查看实际规则路径或删除自定义赛事。</p></div></div>
          ${connected ? "" : inlineDatabaseSetup("连接数据服务以管理赛事和规则", "连接成功后可创建赛事、赛季、阶段、轮次和模型绑定。", state.connection_error)}
          <section class="panel rules-directory-panel"><div class="panel-heading"><div><span>赛事目录</span><h2>三级筛选与紧凑列表</h2></div><span class="field-note">第 4 级及更深操作在当前列表和详情中完成</span></div>${competitionCatalogue(state)}</section>
          <section class="rules-route-strip"><strong>自动路由顺序</strong><span>本场覆盖 → 阶段 → 赛季 → 具体赛事 → 赛事类型默认</span><button class="secondary tiny" data-action="select-workspace-section" data-section-id="routing">维护路由</button></section>
        </div>
      </section>

      <section class="workspace-module-view ${section === "structure" ? "active" : ""}" data-workspace-section="structure">
        <div class="module-section-stack">
          <div class="module-section-heading"><div><span>赛事结构</span><h2>从赛事身份继续建立赛季、阶段与轮次</h2><p>每一层都依赖上一层的明确身份；所有创建操作保持在本页。</p></div></div>
          <div class="core-local-navigation">${workspaceTaskAnchorNavigation([
            { id: "rules-custom-competition", index: "A", label: "赛事身份", description: "创建自定义赛事" },
            { id: "rules-season-structure", index: "B", label: "赛季与阶段", description: "建立赛季、阶段、轮次" },
          ])}</div>
          <section id="rules-custom-competition" class="panel workspace-anchor-target">
            <div class="panel-heading"><div><span>赛事身份</span><h2>新增自定义赛事</h2></div><span class="status-pill">内置赛事不会被覆盖</span></div>
            <div class="two-column rules-admin-grid">
              <article class="subpanel"><label class="field"><span>赛事名称</span><input id="competition-name" placeholder="例如 地区邀请赛" /></label><div class="field-row"><label class="field"><span>赛事类型</span><select id="new-competition-kind">${competitionKindOptions(null)}</select></label><label class="field"><span>国家或地区</span><input id="competition-country" placeholder="例如 韩国（也可填写 KR）" /></label></div><label class="field"><span>比赛所在地时区</span><select id="competition-timezone"><option value="Asia/Seoul">韩国时间</option><option value="Asia/Tokyo">日本时间</option><option value="Asia/Shanghai">中国时间</option><option value="Europe/London">英国时间</option><option value="Europe/Paris">欧洲中部时间</option><option value="America/New_York">美国东部时间</option><option value="UTC">世界协调时间</option></select></label><button class="primary" data-action="create-competition" ${disabled}>创建赛事</button></article>
              <article class="subpanel"><div class="panel-heading"><div><span>当前状态</span><h2>赛事与规则概览</h2></div></div><div class="summary-number-grid"><div><strong>${state.data.competitions.length}</strong><span>赛事</span></div><div><strong>${state.data.seasons.length}</strong><span>赛季</span></div><div><strong>${state.data.stages.length}</strong><span>阶段</span></div><div><strong>${p4Bindings.length}</strong><span>P4 规则</span></div></div><p class="field-note">内置赛事与用户赛事共用同一目录；升级不会删除用户自建赛事、赛季、球队或比赛。</p></article>
            </div>
          </section>
          <section id="rules-season-structure" class="panel workspace-anchor-target">
            <div class="panel-heading"><div><span>赛事层级</span><h2>创建赛季、阶段和轮次</h2></div><span class="field-note">按左至右顺序完成</span></div>
            <div class="three-column hierarchy-grid">
              <article class="subpanel"><div class="panel-heading"><div><span>赛季</span><h2>创建赛季</h2></div></div><label class="field"><span>所属赛事</span><select id="season-competition-id"><option value="">选择赛事</option>${competitionOptions}</select></label><label class="field"><span>赛季名称</span><input id="season-name" placeholder="2026" /></label><div class="field-row"><label class="field"><span>开始日期</span><input id="season-starts-on" type="date" /></label><label class="field"><span>结束日期</span><input id="season-ends-on" type="date" /></label></div><label class="field"><span>状态</span><select id="season-status"><option value="planned">计划中</option><option value="active">进行中</option><option value="completed">已完成</option><option value="archived">已归档</option></select></label><button class="primary" data-action="create-season" ${disabled}>创建赛季</button></article>
              <article class="subpanel"><div class="panel-heading"><div><span>阶段</span><h2>创建阶段</h2></div></div><label class="field"><span>所属赛季</span><select id="stage-season-id"><option value="">选择赛季</option>${seasonOptions}</select></label><label class="field"><span>阶段名称</span><input id="stage-name" placeholder="常规赛" /></label><div class="field-row"><label class="field"><span>阶段类型</span><select id="stage-kind">${stageKindOptions("league")}</select></label><label class="field"><span>顺序</span><input id="stage-sequence" type="number" value="1" /></label></div><button class="primary" data-action="create-stage" ${disabled}>创建阶段</button></article>
              <article class="subpanel"><div class="panel-heading"><div><span>轮次</span><h2>创建轮次</h2></div></div><label class="field"><span>所属阶段</span><select id="round-stage-id"><option value="">选择阶段</option>${stageOptions}</select></label><label class="field"><span>轮次名称</span><input id="round-name" placeholder="第 1 轮" /></label><label class="field"><span>顺序</span><input id="round-sequence" type="number" value="1" /></label><button class="primary" data-action="create-round" ${disabled}>创建轮次</button></article>
            </div>
          </section>
        </div>
      </section>

      <section class="workspace-module-view ${section === "routing" ? "active" : ""}" data-workspace-section="routing">
        <div class="module-section-stack">
          <div class="module-section-heading"><div><span>模型路由</span><h2>指定赛事、赛季或阶段使用的生产规则包</h2><p>更具体的范围优先；同一范围内按匹配顺序从高到低判定。</p></div></div>
          <section class="rules-route-strip"><strong>判定优先级</strong><span>本场覆盖 → 阶段 → 赛季 → 具体赛事 → 赛事类型默认</span></section>
          <section class="panel"><div class="two-column">
            <article class="subpanel"><div class="panel-heading"><div><span>作用范围</span><h2>建立自动匹配规则</h2></div></div><label class="field"><span>赛事</span><select id="binding-competition-id"><option value="">不限定赛事</option>${competitionOptions}</select></label><label class="field"><span>赛季</span><select id="binding-season-id"><option value="">不限定赛季</option>${seasonOptions}</select></label><label class="field"><span>阶段</span><select id="binding-stage-id"><option value="">不限定阶段</option>${stageOptions}</select></label><label class="field"><span>规则包</span><select id="binding-rule-package-id"><option value="">选择规则包</option>${packageOptions}</select></label><div class="field-row"><label class="field"><span>规则名称</span><input id="binding-name" placeholder="例如 2026 淘汰赛规则" /></label><label class="field"><span>匹配顺序</span><input id="binding-priority" type="number" value="100" /><small class="field-note">数字越大越优先。</small></label></div><button class="primary" data-action="create-binding" ${disabled}>保存自动匹配规则</button></article>
            <article class="subpanel table-panel"><div class="panel-heading padded"><div><span>已启用 P4 规则</span><h2>${p4Bindings.length} 条</h2></div></div>${bindingCatalogue(state)}</article>
          </div></section>
        </div>
      </section>

      <section class="workspace-module-view ${section === "packages" ? "active" : ""}" data-workspace-section="packages">
        <div class="module-section-stack">
          <div class="module-section-heading"><div><span>规则包</span><h2>导入、预检并注册生产规则</h2><p>客户端只接受标准赛事文档生成的 JSON；注册前显示模型、版本与适用范围。</p></div></div>
          <section class="panel"><div class="panel-heading"><div><span>导入规则包</span><h2>选择标准规则包文件</h2></div><span class="status-pill ${pendingRulePackage ? "online" : ""}">${pendingRulePackage ? "等待注册" : "等待文件"}</span></div><div class="file-import-row"><label class="secondary file-button">选择规则包文件<input id="rule-package-file" type="file" accept=".json,application/json" /></label><span class="field-note">先做格式、版本和适用范围校验，不直接覆盖现有生产规则。</span></div>${pendingRulePackage ? `<article class="rule-package-preview"><div><span>待注册规则包</span><strong>${escapeHtml(pendingRulePackage.display_name)}</strong><small>版本 ${escapeHtml(pendingRulePackage.version)}</small></div><div class="route-chain"><div><span>赛事类型</span><b>${escapeHtml(competitionKindLabel(pendingRulePackage.competition_profile.competition_kind))}</b></div><div><span>模型</span><b>${escapeHtml(modelLabel(pendingRulePackage.routing.model_id))}</b></div><div><span>模型版本</span><b>${escapeHtml(pendingRulePackage.routing.model_version)}</b></div><div><span>参数版本</span><b>${escapeHtml(pendingRulePackage.routing.parameter_version)}</b></div></div><div class="button-row"><button class="primary" data-action="register-rule-package" ${disabled}>校验并注册</button><button class="ghost" data-action="clear-rule-package">取消</button><button class="secondary" data-action="show-pending-rule-package">只读详情</button></div></article>` : `<div class="empty-state compact"><strong>尚未选择文件</strong><span>选择文件后显示名称、适用赛事、模型和参数版本。</span></div>`}</section>
        </div>
      </section>
    </div>
  </section>`;
}
