import { availabilityLabel, positionLabel } from "../../components/footballText";
import { escapeHtml, formatPercent } from "../../components/format";
import type {
  BootstrapResponse,
  PlayerCatalogReferenceData,
  PlayerDetail
} from "../../types";
import { workspaceAnchorNavigation } from "../../ui/navigation";
import { availabilityLabels, currentLocalizedPlayerName, dateTimeLocalValue, footLabels, options, playerNameHistory, recordBoolean, recordNumber, recordString } from "./playerPresentation";
export function abilityCards(
  detail: PlayerDetail,
  dimensions: PlayerCatalogReferenceData["ability_dimensions"],
): string {
  const abilities = detail.ability_profile?.abilities ?? {};
  const entries = Object.entries(abilities).slice(0, 8);
  if (entries.length === 0) {
    return `<div class="empty-state compact"><strong>尚无能力数据</strong><span>在“能力观察”中添加第一条记录。</span></div>`;
  }
  return `<div class="ability-card-grid">${entries
    .map(([key, raw]) => {
      const record =
        raw && typeof raw === "object" && !Array.isArray(raw)
          ? (raw as Record<string, unknown>)
          : {};
      const value =
        typeof record.value === "number"
          ? record.value
          : typeof raw === "number"
            ? raw
            : null;
      const confidence =
        typeof record.confidence === "number" ? record.confidence : null;
      const dimensionName =
        dimensions.find((item) => item.code === key)?.name ??
        key.replaceAll("_", " ");
      return `<div class="ability-card"><span>${escapeHtml(dimensionName)}</span><strong>${value === null ? "—" : value.toFixed(1)}</strong><small>${confidence === null ? "" : `可信度 ${formatPercent(confidence)}`}</small></div>`;
    })
    .join("")}</div>`;
}
export function detailPanel(
  detail: PlayerDetail | null,
  references: PlayerCatalogReferenceData | null,
  state: BootstrapResponse,
): string {
  if (!detail) {
    return `<section id="player-profile-overview" class="panel player-detail-panel workspace-anchor-target"><div class="empty-state"><strong>选择一名球员</strong><span>右侧将显示球队、位置、可用状态和能力。</span></div></section>`;
  }
  const player = detail.player;
  const teams = references?.teams ?? [];
  const positions = references?.positions ?? [];
  const dimensions = references?.ability_dimensions ?? [];
  const dynamicTagDefinitions = references?.dynamic_tag_definitions ?? [];
  const providers = references?.providers ?? [];
  const now = Date.now();
  const today = new Date().toISOString().slice(0, 10);
  const latestAvailability = detail.availability.find((item) => {
    const record = item as Record<string, unknown>;
    const from =
      typeof record.valid_from === "string"
        ? Date.parse(record.valid_from)
        : Number.NaN;
    const to =
      typeof record.valid_to === "string"
        ? Date.parse(record.valid_to)
        : Number.POSITIVE_INFINITY;
    return from <= now && to >= now;
  }) as Record<string, unknown> | undefined;
  const currentTeam = detail.team_periods.find((item) => {
    const to = item.valid_to;
    return (
      item.valid_from <= today &&
      (to === null || to >= today) &&
      ["registered", "loan", "trial"].includes(item.registration_status)
    );
  });
  const localizedName = currentLocalizedPlayerName(detail);
  const currentPosition = (detail.positions.find((item) => item.is_primary === true) ?? detail.positions[0]) as Record<string, unknown> | undefined;
  const latestAbilityObservation = detail.ability_observations[0] as Record<string, unknown> | undefined;
  const currentExternalId = detail.external_ids[0] as Record<string, unknown> | undefined;
  const teamHistory = detail.team_periods.length === 0
    ? '<div class="empty-state compact"><strong>暂无球队履历</strong></div>'
    : detail.team_periods
        .map((period) => `<div class="history-row"><strong>${escapeHtml(period.team_name)}</strong><span>${escapeHtml(period.registration_status)}${period.squad_number === null ? "" : ` · #${period.squad_number}`}</span><span>${escapeHtml(period.valid_from)} → ${escapeHtml(period.valid_to ?? "至今")}</span><b>${escapeHtml(period.season_name ?? "")}</b></div>`)
        .join("");
  const availability = String(latestAvailability?.status ?? "unknown");
  const availabilityReason = String(latestAvailability?.reason ?? "").trim();
  const availabilityConfidence =
    typeof latestAvailability?.confidence === "number"
      ? latestAvailability.confidence
      : null;
  const availabilityValidTo =
    typeof latestAvailability?.valid_to === "string"
      ? latestAvailability.valid_to
      : null;
  const availabilityCompetitionId =
    typeof latestAvailability?.competition_id === "string"
      ? latestAvailability.competition_id
      : null;
  const availabilityScope = availabilityCompetitionId
    ? (state.data.competitions.find(
        (item) => item.id === availabilityCompetitionId,
      )?.name ?? "指定赛事")
    : "全部赛事";
  const uncertaintyPanel =
    availability === "doubtful"
      ? `<article class="uncertainty-panel">
    <div><span>当前存疑点</span><strong>${escapeHtml(availabilityReason || "尚未记录具体原因，需要补充伤病部位、训练状态或出场限制")}</strong></div>
    <dl><div><dt>可信度</dt><dd>${availabilityConfidence === null ? "未记录" : formatPercent(availabilityConfidence)}</dd></div><div><dt>影响范围</dt><dd>${escapeHtml(availabilityScope)}</dd></div><div><dt>有效至</dt><dd>${availabilityValidTo ? escapeHtml(new Date(availabilityValidTo).toLocaleString()) : "未设置结束时间"}</dd></div></dl>
  </article>`
      : "";
  const positionHistory = detail.positions.length
    ? `<div class="history-list">${detail.positions.map((item) => {
        const record = item as Record<string, unknown>;
        const defaultRole = recordString(record, "default_role_code");
        return `<div class="history-row"><strong>${escapeHtml(positionLabel(recordString(record, "position_code")))}</strong><span>${recordBoolean(record, "is_primary") ? "主位置" : "兼任位置"}</span><span>默认角色 ${escapeHtml(defaultRole ?? "未设置")} · 熟练度 ${formatPercent(recordNumber(record, "proficiency") ?? 0)}</span><b>已保存</b></div>`;
      }).join("")}</div>`
    : '<div class="empty-state compact"><strong>暂无位置记录</strong></div>';
  const availabilityHistory = detail.availability.length
    ? `<div class="history-list">${detail.availability.slice(0, 20).map((item) => {
        const record = item as Record<string, unknown>;
        return `<div class="history-row"><strong>${escapeHtml(availabilityLabel(recordString(record, "status") as never))}</strong><span>${escapeHtml(recordString(record, "reason") ?? "无补充说明")}</span><span>${escapeHtml(recordString(record, "valid_from") ?? "未设置")} → ${escapeHtml(recordString(record, "valid_to") ?? "至今")}</span><b>${formatPercent(recordNumber(record, "confidence") ?? 0)}</b></div>`;
      }).join("")}</div>`
    : '<div class="empty-state compact"><strong>暂无可用状态记录</strong></div>';
  const abilityHistory = detail.ability_observations.length
    ? `<div class="history-list">${detail.ability_observations.slice(0, 30).map((item) => {
        const record = item as Record<string, unknown>;
        return `<div class="history-row"><strong>${escapeHtml(recordString(record, "dimension_name") ?? recordString(record, "dimension_code") ?? "能力维度")}</strong><span>数值 ${recordNumber(record, "value")?.toFixed(1) ?? "—"}</span><span>${escapeHtml(recordString(record, "observed_at") ?? "时间未记录")}</span><b>${formatPercent(recordNumber(record, "confidence") ?? 0)}</b></div>`;
      }).join("")}</div>`
    : '<div class="empty-state compact"><strong>暂无能力观察历史</strong></div>';
  const externalHistory = detail.external_ids.length
    ? `<div class="history-list">${detail.external_ids.map((item) => {
        const record = item as Record<string, unknown>;
        return `<div class="history-row"><strong>${escapeHtml(recordString(record, "provider_name") ?? "数据源")}</strong><span>${escapeHtml(recordString(record, "external_id") ?? "未设置")}</span><span>外部关联</span><b>已保存</b></div>`;
      }).join("")}</div>`
    : '<div class="empty-state compact"><strong>暂无外部数据源关联</strong></div>';
  return `<section id="player-profile-overview" class="panel player-detail-panel workspace-anchor-target" data-player-draft-id="${escapeHtml(detail.player.id)}" data-workspace-persist="false">
    <div class="panel-heading"><div><span>球员档案</span><h2>${escapeHtml(player.canonical_name)}</h2></div><div class="button-row compact"><button class="secondary" data-action="open-player-api-workspace" data-player-id="${escapeHtml(player.id)}">AI 问答</button><button class="secondary quiet" data-action="show-player-json">查看完整档案</button><button class="ghost danger-quiet" data-action="request-delete-player" data-player-id="${escapeHtml(player.id)}" data-player-name="${escapeHtml(player.canonical_name)}">删除球员</button></div></div>
    <div class="player-profile-hero">
      <div class="player-avatar">${escapeHtml(player.canonical_name.slice(0, 1).toUpperCase())}</div>
      <div><strong>${escapeHtml(String(currentTeam?.team_name ?? "未登记球队"))}</strong><span>${escapeHtml(player.nationality_code ?? "国籍未记录")} · ${escapeHtml(footLabels[player.preferred_foot] ?? player.preferred_foot)}</span></div>
      <span class="availability availability-${escapeHtml(availability)}">${escapeHtml(availabilityLabels[availability] ?? availability)}</span>
    </div>
    ${uncertaintyPanel}
    <div class="player-profile-summary compact-summary">
      <div><span>出生日期</span><strong>${escapeHtml(player.date_of_birth ?? "未记录")}</strong></div>
      <div><span>能力均值</span><strong>${detail.ability_profile?.average_value?.toFixed(1) ?? "—"}</strong></div>
      <div><span>能力维度</span><strong>${detail.ability_profile?.dimension_count ?? 0}</strong></div>
      <div><span>能力可信度</span><strong>${detail.ability_profile?.average_confidence == null ? "—" : formatPercent(detail.ability_profile.average_confidence)}</strong></div>
    </div>
    <div class="section-label"><span>当前能力</span><small>最多显示 8 个维度</small></div>
    ${abilityCards(detail, dimensions)}
    <div class="section-label"><span>当前动态标签</span><small>标签到期后自动失效，不代表长期能力</small></div>
    <div class="tag-chip-grid">${detail.dynamic_tags.length === 0 ? `<span class="muted">当前没有有效动态标签</span>` : detail.dynamic_tags.map((tag) => `<article class="dynamic-tag-chip"><strong>${escapeHtml(tag.label ?? tag.tag_name)}</strong><span>${tag.value.toFixed(2)} · 可信度 ${formatPercent(tag.confidence)}</span><small>有效至 ${escapeHtml(new Date(tag.valid_to).toLocaleString())}</small></article>`).join("")}</div>

    ${workspaceAnchorNavigation("球员档案", [
      { id: "player-profile-overview", label: "概览" },
      { id: "player-profile-actions", label: "动态标签" },
      { id: "player-profile-base", label: "基础资料" },
      { id: "player-profile-names", label: "名称别名" },
      { id: "player-profile-positions", label: "位置与角色" },
      { id: "player-profile-teams", label: "球队履历" },
      { id: "player-profile-availability", label: "伤停状态" },
      { id: "player-profile-ability", label: "能力观察" },
      { id: "player-profile-external", label: "外部数据" },
    ])}
    <div class="profile-section-stack">
    <details id="player-profile-actions" class="editor-details workspace-anchor-target"><summary>添加动态标签与计算本场贡献</summary>
      <div class="compact-form three-column">
        <label class="field"><span>标签</span><select id="player-dynamic-tag-code">${dynamicTagDefinitions.map((tag) => `<option value="${escapeHtml(tag.code)}" data-default="${tag.default_value}" data-ttl="${tag.default_ttl_hours}">${escapeHtml(tag.name)}</option>`).join("")}</select></label>
        <label class="field"><span>数值</span><input id="player-dynamic-tag-value" type="number" step="0.01" value="1"></label>
        <label class="field"><span>显示标签</span><input id="player-dynamic-tag-label" placeholder="例如 高负荷 / 状态上升"></label>
        <label class="field"><span>可信度</span><input id="player-dynamic-tag-confidence" type="number" min="0" max="1" step="0.01" value="0.8"></label>
        <label class="field"><span>生效时间</span><input id="player-dynamic-tag-from" type="datetime-local"></label>
        <label class="field"><span>失效时间</span><input id="player-dynamic-tag-to" type="datetime-local"></label>
        <label class="field"><span>赛事范围</span><select id="player-dynamic-tag-competition"><option value="">所有赛事</option>${state.data.competitions.map((competition) => `<option value="${escapeHtml(competition.id)}">${escapeHtml(competition.name)}</option>`).join("")}</select></label>
        <label class="field"><span>位置范围</span><select id="player-dynamic-tag-position"><option value="">所有位置</option>${positions.map((position) => `<option value="${escapeHtml(position.code)}">${escapeHtml(position.name)}</option>`).join("")}</select></label>

      </div>
      <div class="button-row"><button class="primary" data-action="add-player-dynamic-tag" data-player-id="${escapeHtml(player.id)}">保存动态标签</button><button class="secondary" data-action="calculate-player-contribution" data-player-id="${escapeHtml(player.id)}">计算当前有效贡献</button></div>
    </details>

    <details id="player-profile-base" class="editor-details workspace-anchor-target" open><summary>编辑基础资料</summary>
      <p class="field-note">保存后会重新读取数据库并继续显示已确认值；中文姓名留空不会覆盖或创建“未指定”记录。</p>
      <div class="compact-form three-column">
        <label class="field"><span>正式姓名</span><input id="edit-player-name" value="${escapeHtml(player.canonical_name)}"></label>
        <label class="field"><span>中文姓名</span><input id="edit-player-localized-name" value="${escapeHtml(localizedName)}" placeholder="可留空；留空保持现有中文名"></label>
        <label class="field"><span>出生日期</span><input id="edit-player-birth" type="date" value="${escapeHtml(player.date_of_birth ?? "")}"></label>
        <label class="field"><span>国籍</span><input id="edit-player-nationality" value="${escapeHtml(player.nationality_code ?? "")}"></label>
        <label class="field"><span>惯用脚</span><select id="edit-player-foot"><option value="unknown" ${player.preferred_foot === "unknown" ? "selected" : ""}>未知</option><option value="right" ${player.preferred_foot === "right" ? "selected" : ""}>右脚</option><option value="left" ${player.preferred_foot === "left" ? "selected" : ""}>左脚</option><option value="both" ${player.preferred_foot === "both" ? "selected" : ""}>双脚</option></select></label>
        <label class="field"><span>身高 cm</span><input id="edit-player-height" type="number" min="120" max="230" value="${player.height_cm ?? ""}"></label>
        <label class="field"><span>球员状态</span><select id="edit-player-status"><option value="active" ${player.status === "active" ? "selected" : ""}>现役</option><option value="inactive" ${player.status === "inactive" ? "selected" : ""}>非活跃</option><option value="retired" ${player.status === "retired" ? "selected" : ""}>退役</option><option value="unknown" ${player.status === "unknown" ? "selected" : ""}>未知</option></select></label>
      </div>
      <button class="primary" data-action="update-player" data-player-id="${escapeHtml(player.id)}">保存基础资料</button>
    </details>

    <details id="player-profile-names" class="editor-details workspace-anchor-target"><summary>名称与别名（${detail.names.length}）</summary>
      ${playerNameHistory(detail)}
      <div class="compact-form three-column top-gap">
        <label class="field"><span>新增其他名称</span><input id="player-alias-name" placeholder="英文、韩文、日文或历史名称"></label>
        <label class="field"><span>语言</span><select id="player-alias-language"><option value="">请选择语言</option><option value="zh-CN">中文</option><option value="ko-KR">韩文</option><option value="ja-JP">日文</option><option value="en">英文</option></select></label>
        <label class="check-field"><input id="player-alias-primary" type="checkbox"><span>设为主名称</span></label>
      </div>
      <button class="primary" data-action="add-player-name" data-player-id="${escapeHtml(player.id)}">添加其他名称</button>
    </details>

    <details id="player-profile-positions" class="editor-details workspace-anchor-target"><summary>位置与角色（${detail.positions.length}）</summary>
      ${positionHistory}
      <div class="compact-form three-column top-gap">
        <label class="field"><span>位置</span><select id="player-position-code">${positions.map((position) => `<option value="${escapeHtml(position.code)}" ${recordString(currentPosition, "position_code") === position.code ? "selected" : ""}>${escapeHtml(position.name)}</option>`).join("")}</select></label>
        <label class="field"><span>熟练度</span><input id="player-position-proficiency" type="number" min="0" max="1" step="0.01" value="${recordNumber(currentPosition, "proficiency") ?? 0.8}"></label>
        <label class="field"><span>默认战术角色</span><input id="player-position-default-role" value="${escapeHtml(recordString(currentPosition, "default_role_code") ?? "")}" placeholder="例如：组织核心、单后腰、抢点中锋"></label>
        <label class="check-field"><input id="player-position-primary" type="checkbox" ${recordBoolean(currentPosition, "is_primary") ? "checked" : ""}><span>主位置</span></label>
      </div>
      <button class="primary" data-action="assign-player-position" data-player-id="${escapeHtml(player.id)}">保存位置</button>
    </details>

    <details id="player-profile-teams" class="editor-details workspace-anchor-target"><summary>球队履历</summary>
      <p class="field-note">完整保留俱乐部、国家队、租借与历史注册关系；新增记录不会覆盖旧履历。</p>
      <div class="history-list">${teamHistory}</div>
      <div class="compact-form three-column top-gap">
        <label class="field"><span>球队</span><select id="player-team-id">${options(teams, currentTeam?.team_id ?? null, "选择球队")}</select></label>
        <label class="field"><span>开始日期</span><input id="player-team-valid-from" type="date" value="${escapeHtml(currentTeam?.valid_from ?? "")}"></label>
        <label class="field"><span>结束日期</span><input id="player-team-valid-to" type="date" value="${escapeHtml(currentTeam?.valid_to ?? "")}"></label>
        <label class="field"><span>球衣号</span><input id="player-squad-number" type="number" min="0" max="99" value="${currentTeam?.squad_number ?? ""}"></label>
        <label class="field"><span>注册状态</span><select id="player-registration-status"><option value="registered" ${currentTeam?.registration_status === "registered" ? "selected" : ""}>已注册</option><option value="loan" ${currentTeam?.registration_status === "loan" ? "selected" : ""}>租借</option><option value="trial" ${currentTeam?.registration_status === "trial" ? "selected" : ""}>试训</option><option value="released" ${currentTeam?.registration_status === "released" ? "selected" : ""}>离队</option><option value="unknown" ${currentTeam?.registration_status === "unknown" ? "selected" : ""}>未知</option></select></label>
      </div>
      <button class="primary" data-action="add-player-team-period" data-player-id="${escapeHtml(player.id)}">保存球队履历</button>
    </details>

    <details id="player-profile-availability" class="editor-details workspace-anchor-target"><summary>伤停与可用状态（${detail.availability.length}）</summary>
      ${availabilityHistory}
      <div class="compact-form three-column top-gap">
        <label class="field"><span>状态</span><select id="player-availability-status">${["available","doubtful","unavailable","injured","suspended","rested","returning","unknown"].map((status) => `<option value="${status}" ${availability === status ? "selected" : ""}>${escapeHtml(availabilityLabels[status] ?? status)}</option>`).join("")}</select></label>
        <label class="field"><span>可信度</span><input id="player-availability-confidence" type="number" min="0" max="1" step="0.01" value="${availabilityConfidence ?? 1}"></label>
        <label class="field"><span>原因</span><input id="player-availability-reason" value="${escapeHtml(availabilityReason)}" placeholder="伤病部位或停赛原因"></label>
        <label class="field"><span>生效时间</span><input id="player-availability-from" type="datetime-local" value="${escapeHtml(dateTimeLocalValue(recordString(latestAvailability, "valid_from")))}"></label>
        <label class="field"><span>结束时间</span><input id="player-availability-to" type="datetime-local" value="${escapeHtml(dateTimeLocalValue(availabilityValidTo))}"></label>
      </div>
      <button class="primary" data-action="add-player-availability" data-player-id="${escapeHtml(player.id)}">保存状态</button>
    </details>

    <details id="player-profile-ability" class="editor-details workspace-anchor-target"><summary>能力观察（${detail.ability_observations.length}）</summary>
      ${abilityHistory}
      <div class="compact-form three-column top-gap">
        <label class="field"><span>能力维度</span><select id="player-ability-dimension">${dimensions.map((dimension) => `<option value="${escapeHtml(dimension.code)}" ${recordString(latestAbilityObservation, "dimension_code") === dimension.code ? "selected" : ""}>${escapeHtml(dimension.name)}</option>`).join("")}</select></label>
        <label class="field"><span>能力值</span><input id="player-ability-value" type="number" min="0" max="100" step="0.1" value="${recordNumber(latestAbilityObservation, "value") ?? 50}"></label>
        <label class="field"><span>可信度</span><input id="player-ability-confidence" type="number" min="0" max="1" step="0.01" value="${recordNumber(latestAbilityObservation, "confidence") ?? 0.7}"></label>
        <label class="field"><span>样本量</span><input id="player-ability-sample-size" type="number" min="0" step="1" value="${recordNumber(latestAbilityObservation, "sample_size") ?? 1}"></label>
        <label class="field"><span>观察时间</span><input id="player-ability-observed-at" type="datetime-local" value="${escapeHtml(dateTimeLocalValue(recordString(latestAbilityObservation, "observed_at")))}"></label>
      </div>
      <button class="primary" data-action="add-player-ability" data-player-id="${escapeHtml(player.id)}">保存能力观察</button>
    </details>

    <details id="player-profile-external" class="editor-details workspace-anchor-target"><summary>外部数据源（${detail.external_ids.length}）</summary>
      ${externalHistory}
      <div class="compact-form three-column top-gap">
        <label class="field"><span>数据源</span><select id="player-provider-id">${providers.map((provider) => `<option value="${escapeHtml(provider.id)}" ${recordString(currentExternalId, "provider_id") === provider.id ? "selected" : ""}>${escapeHtml(provider.name)}</option>`).join("")}</select></label>
        <label class="field"><span>外部数据编号</span><input id="player-external-id" value="${escapeHtml(recordString(currentExternalId, "external_id") ?? "")}" placeholder="由数据供应商提供，可留空"><small class="field-note">已绑定内容会持续显示；日常手工维护无需填写。</small></label>
      </div>
      <button class="primary" data-action="add-player-external-id" data-player-id="${escapeHtml(player.id)}">绑定数据源</button>
    </details>
    </div>
  </section>`;
}
