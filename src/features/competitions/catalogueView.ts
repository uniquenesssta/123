import { escapeHtml } from "../../components/format";
import { competitionKindLabel, competitionKindOptions } from "../../components/competition";
import type { BootstrapResponse, CompetitionRecord } from "../../types";
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

export function competitionRegionLabel(item: CompetitionRecord): string {
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

function bindingForCompetition(state: BootstrapResponse, item: CompetitionRecord) {
  const direct = state.data.competition_bindings
    .filter((binding) => binding.is_active && !binding.season_id && !binding.stage_id && (binding.model_id === "p4" || binding.model_id.startsWith("p4_")) && binding.competition_id === item.id)
    .sort((left, right) => right.priority - left.priority)[0];
  const fallback = state.data.competition_bindings
    .filter((binding) => binding.is_active && !binding.season_id && !binding.stage_id && (binding.model_id === "p4" || binding.model_id.startsWith("p4_")) && !binding.competition_id && binding.competition_kind === item.competition_kind)
    .sort((left, right) => right.priority - left.priority)[0];
  return { binding: direct ?? fallback, direct: Boolean(direct) };
}

export function competitionCatalogue(state: BootstrapResponse): string {
  if (state.data.competitions.length === 0) {
    return `<div class="empty-state"><strong>暂无赛事</strong><span>可在下方创建赛事，或通过数据导入恢复赛事目录。</span></div>`;
  }

  const competitions = [...state.data.competitions].sort((left, right) => {
    const scopeDifference = ["national", "club", "other"].indexOf(competitionScope(left)) - ["national", "club", "other"].indexOf(competitionScope(right));
    if (scopeDifference !== 0) return scopeDifference;
    const regionDifference = regionSortOrder(competitionRegionLabel(left)) - regionSortOrder(competitionRegionLabel(right));
    if (regionDifference !== 0) return regionDifference;
    return Number(left.metadata?.sort_order ?? 9999) - Number(right.metadata?.sort_order ?? 9999) || left.name.localeCompare(right.name, "zh-CN");
  });

  const regions = Array.from(new Set(competitions.map(competitionRegionLabel))).sort((left, right) => regionSortOrder(left) - regionSortOrder(right) || left.localeCompare(right, "zh-CN"));
  const regionScopeMap = new Map<string, Set<CompetitionScope>>();
  for (const item of competitions) {
    const region = competitionRegionLabel(item);
    const scopes = regionScopeMap.get(region) ?? new Set<CompetitionScope>();
    scopes.add(competitionScope(item));
    regionScopeMap.set(region, scopes);
  }

  const rows = competitions.map((item) => {
    const scope = competitionScope(item);
    const region = competitionRegionLabel(item);
    const { binding, direct } = bindingForCompetition(state, item);
    const route = binding ? `${direct ? "赛事专属" : "类型默认"}：${binding.rule_package_name}` : "尚未绑定规则";
    const searchText = `${item.name} ${item.code} ${item.country_code ?? ""} ${region} ${competitionKindLabel(item.competition_kind)} ${metadataText(item, "official_name")}`.toLowerCase();
    const seasonPattern = metadataText(item, "season_pattern");
    return `<article class="rules-competition-card" data-rules-competition-row data-scope="${scope}" data-region="${escapeHtml(region)}" data-kind="${escapeHtml(item.competition_kind)}" data-search="${escapeHtml(searchText)}">
      <header><strong>${escapeHtml(item.name)}</strong><small>${escapeHtml(item.code)} · ${escapeHtml(scopeLabel(scope))} · ${escapeHtml(region)} · ${escapeHtml(competitionKindLabel(item.competition_kind))}</small></header>
      <p class="rules-route-text">${escapeHtml(route)}${seasonPattern ? ` · ${escapeHtml(seasonPattern === "cross_year" ? "跨年赛季" : seasonPattern === "calendar" ? "自然年赛季" : "锦标赛赛季")}` : ""}</p>
      <div class="rules-row-actions"><button class="secondary" data-action="show-competition-path" data-competition-id="${escapeHtml(item.id)}">规则路径</button><button class="ghost danger-quiet" data-action="request-delete-competition" data-competition-id="${escapeHtml(item.id)}" data-competition-name="${escapeHtml(item.name)}">删除</button></div>
    </article>`;
  }).join("");

  return `<div class="rules-directory" data-rules-directory>
    <div class="rules-directory-toolbar">
      <label class="rules-search"><span>搜索赛事</span><input id="rules-competition-search" type="search" placeholder="赛事、国家、地区或代码"></label>
      <label class="rules-kind-filter"><span>赛制</span><select id="rules-competition-kind"><option value="">全部赛制</option>${competitionKindOptions(null)}</select></label>
      <button class="secondary rules-clear" data-action="clear-rules-catalogue-filters">清除</button>
      <span class="rules-visible-count"><b id="rules-visible-count">${competitions.length}</b> / ${competitions.length} 项</span>
    </div>
    <div class="rules-directory-grid">
      <nav class="rules-scope-list" aria-label="赛事一级分类">
        <strong>参赛体系</strong>
        <button class="active" type="button" data-rules-scope="">全部赛事 <span>${competitions.length}</span></button>
        ${(["national", "club", "other"] as CompetitionScope[]).map((scope) => `<button type="button" data-rules-scope="${scope}">${scopeLabel(scope)} <span>${competitions.filter((item) => competitionScope(item) === scope).length}</span></button>`).join("")}
      </nav>
      <nav class="rules-region-list" aria-label="赛事二级分类">
        <strong>地区 / 足联</strong>
        <button class="active" type="button" data-rules-region="" data-region-scopes="national,club,other">全部地区 <span>${competitions.length}</span></button>
        ${regions.map((region) => `<button type="button" data-rules-region="${escapeHtml(region)}" data-region-scopes="${escapeHtml(Array.from(regionScopeMap.get(region) ?? []).join(","))}">${escapeHtml(region)} <span>${competitions.filter((item) => competitionRegionLabel(item) === region).length}</span></button>`).join("")}
      </nav>
      <div class="rules-competition-pane">
        <div class="rules-pane-heading"><div><strong>具体赛事</strong><span>选择赛事后查看规则路径或维护目录</span></div></div>
        <div class="rules-table-scroll">
          <div class="rules-competition-cards">${rows}</div>
          <div id="rules-empty-filter" class="empty-state compact" hidden><strong>没有匹配赛事</strong><span>调整一级、二级分类或搜索条件。</span></div>
        </div>
      </div>
    </div>
  </div>`;
}
