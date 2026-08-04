import { escapeHtml, formatPercent } from "../components/format";
import { inlineDatabaseSetup } from "../components/databaseSetup";
import { icon } from "../components/icons";
import { taskContextRibbon, taskPageHeader } from "../components/taskWorkspace";
import { workspaceAnchorNavigation, workspaceSectionNavigation } from "../components/workspace";
import { ageFromBirthDate, availabilityLabel, detailPlayerName, displayPlayerName, initials, playerStatusLabel, positionLabel, preferredFootLabel } from "../components/footballText";
import type { WorkspaceLayoutMode, WorkspaceTabState } from "../app/viewState";
import type {
  BootstrapResponse,
  PlayerCatalogReferenceData,
  PlayerDetail,
  PlayerListPage,
  PlayerListQuery,
  PlayerNavigationContext,
  SpreadsheetImportPreview,
} from "../types";

const availabilityLabels: Record<string, string> = {
  available: "可用",
  unavailable: "不可出场",
  doubtful: "存疑",
  injured: "伤病",
  suspended: "停赛",
  rested: "轮休",
  returning: "恢复中",
  unknown: "未知",
};

const footLabels: Record<string, string> = {
  left: "左脚",
  right: "右脚",
  both: "双脚",
  unknown: "未知",
};

function spreadsheetStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    ready_add: "待新增",
    ready_update: "待更新",
    ready_end_previous: "新增并结束旧履历",
    conflict: "冲突",
    error: "错误",
    skip: "跳过",
    imported: "已导入",
  };
  return labels[status] ?? status;
}

function spreadsheetEntityLabel(type: string): string {
  const labels: Record<string, string> = {
    team: "球队",
    player: "球员",
    player_name: "球员名称",
    player_position: "球员位置",
    player_team_period: "球员所属球队",
    player_ability: "球员能力",
    player_availability: "球员状态",
    player_dynamic_tag: "球员动态标签",
    external_entity_id: "外部数据关联",
    match: "比赛",
    lineup: "阵容",
    lineup_player: "阵容球员",
  };
  return labels[type] ?? "其他数据";
}

function spreadsheetPanel(preview: SpreadsheetImportPreview | null): string {
  const counts = preview?.counts;
  const blocking = (counts?.conflict ?? 0) + (counts?.error ?? 0);
  const rows = preview?.rows.slice(0, 80) ?? [];
  return `<section class="panel spreadsheet-panel">
    <div class="panel-heading"><div><span>球员月度工作包</span><h2>身份、履历、位置、状态与能力观察</h2></div></div>
    <div class="spreadsheet-actions">
      <article><strong>空白模板</strong><span>包含说明、参考字典、数据缺口和月度维护字段。</span><button class="secondary" data-action="export-player-template">导出模板</button></article>
      <article><strong>月度数据包</strong><span>导出球员、名称、履历、位置、可用性、能力观察和动态标签。</span><button class="secondary" data-action="export-player-data">导出现有数据</button></article>
      <article><strong>导入并预检</strong><span>仅接受球员月度或球员目录工作包；球队月度文件会被识别并阻止误导入。</span><label class="field"><span>导入模式</span><select id="spreadsheet-import-mode"><option value="add_and_update">新增并更新</option><option value="add_only">仅新增</option></select></label><button class="primary" data-action="preview-player-import">选择球员文件并预检</button><button class="ghost" data-page="workbooks">查看全部工作包入口</button></article>
    </div>
    ${
      preview
        ? `<div class="spreadsheet-preview">
      <div class="preview-summary">
        <div><span>文件</span><strong>${escapeHtml(preview.source_file_name)}</strong></div>
        <div class="preview-count add"><span>待新增</span><strong>${counts?.ready_add ?? 0}</strong></div>
        <div class="preview-count update"><span>待更新</span><strong>${counts?.ready_update ?? 0}</strong></div>
        <div class="preview-count update"><span>结束旧履历</span><strong>${counts?.ready_end_previous ?? 0}</strong></div>
        <div class="preview-count conflict"><span>冲突</span><strong>${counts?.conflict ?? 0}</strong></div>
        <div class="preview-count error"><span>错误</span><strong>${counts?.error ?? 0}</strong></div>
        <div class="preview-count skip"><span>跳过</span><strong>${counts?.skipped ?? 0}</strong></div>
      </div>
      <div class="spreadsheet-table-wrap"><table class="spreadsheet-table"><thead><tr><th>工作表</th><th>行</th><th>类型</th><th>状态</th><th>说明</th></tr></thead><tbody>${rows.map((row) => `<tr class="status-${escapeHtml(row.status)}"><td>${escapeHtml(row.sheet_name)}</td><td>${row.row_number}</td><td>${escapeHtml(spreadsheetEntityLabel(row.entity_type))}</td><td>${escapeHtml(spreadsheetStatusLabel(row.status))}</td><td>${escapeHtml(row.message ?? "")}${row.conflict_candidates.length ? `<div class="candidate-list">${row.conflict_candidates.map((candidate) => `<button class="candidate-choice" data-action="resolve-import-conflict" data-row-id="${escapeHtml(row.id)}" data-entity-id="${escapeHtml(candidate.entity_id)}"><strong>${escapeHtml(candidate.display_name)}</strong>${candidate.detail ? `<small>${escapeHtml(candidate.detail)}</small>` : ""}</button>`).join("")}<button class="candidate-skip" data-action="skip-import-conflict" data-row-id="${escapeHtml(row.id)}">跳过这一行</button></div>` : ""}</td></tr>`).join("")}</tbody></table></div>
      ${preview.rows.length > rows.length ? `<p class="field-note">仅显示前 ${rows.length} 行，完整结果保存在导入批次中。</p>` : ""}
      <div class="button-row"><button class="primary" data-action="commit-player-import" ${blocking > 0 || (counts?.ready_add ?? 0) + (counts?.ready_update ?? 0) + (counts?.ready_end_previous ?? 0) === 0 ? "disabled" : ""}>确认写入数据库</button><button class="secondary" data-action="show-import-preview-json">查看完整预检结果</button>${blocking > 0 ? `<span class="blocking-note">冲突可在上方直接选择正确记录；格式错误需修正表格后重新预检。</span>` : ""}</div>
    </div>`
        : `<div class="empty-inline">尚未选择球员月度工作簿。导入前会显示新增、更新、冲突和错误明细。</div>`
    }
  </section>`;
}

function options(
  items: Array<{ id: string; canonical_name: string }>,
  selected: string | null,
  empty = "全部球队",
): string {
  return `<option value="">${empty}</option>${items
    .map(
      (item) =>
        `<option value="${escapeHtml(item.id)}" ${item.id === selected ? "selected" : ""}>${escapeHtml(item.canonical_name)}</option>`,
    )
    .join("")}`;
}

function recordString(record: Record<string, unknown> | undefined, key: string): string | null {
  const value = record?.[key];
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

function recordNumber(record: Record<string, unknown> | undefined, key: string): number | null {
  const value = record?.[key];
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function recordBoolean(record: Record<string, unknown> | undefined, key: string): boolean {
  return record?.[key] === true;
}

function isChineseNameRecord(record: Record<string, unknown>): boolean {
  const language = recordString(record, "language_code")?.toLowerCase() ?? "";
  const name = recordString(record, "name") ?? "";
  return ["zh-cn", "zh-hans", "zh"].includes(language) || /[一-龥]/u.test(name);
}

function currentLocalizedPlayerName(detail: PlayerDetail): string {
  const records = (detail.names as Array<Record<string, unknown>>).filter(isChineseNameRecord);
  records.sort((left, right) => {
    const leftLanguage = recordString(left, "language_code")?.toLowerCase() ?? "";
    const rightLanguage = recordString(right, "language_code")?.toLowerCase() ?? "";
    const languageRank = (language: string) => ["zh-cn", "zh-hans", "zh"].includes(language) ? 1 : 0;
    const byLanguage = languageRank(rightLanguage) - languageRank(leftLanguage);
    if (byLanguage !== 0) return byLanguage;
    return (recordString(right, "valid_from") ?? "").localeCompare(recordString(left, "valid_from") ?? "") ||
      (recordString(right, "id") ?? "").localeCompare(recordString(left, "id") ?? "");
  });
  return recordString(records[0], "name") ?? "";
}

function dateTimeLocalValue(value: string | null): string {
  if (!value) return "";
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return "";
  const local = new Date(parsed.getTime() - parsed.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 16);
}

function playerNameHistory(detail: PlayerDetail): string {
  const records = detail.names as Array<Record<string, unknown>>;
  if (records.length === 0) return '<div class="empty-state compact"><strong>暂无名称记录</strong><span>正式姓名仍保存在基础身份中。</span></div>';
  return `<div class="history-list">${records.map((record) => {
    const name = recordString(record, "name") ?? "未命名";
    const language = recordString(record, "language_code") ?? "未指定语言";
    const primary = recordBoolean(record, "is_primary") ? "主名称" : "别名";
    const from = recordString(record, "valid_from");
    const to = recordString(record, "valid_to");
    return `<div class="history-row"><strong>${escapeHtml(name)}</strong><span>${escapeHtml(language)} · ${primary}</span><span>${escapeHtml(from ?? "长期有效")} → ${escapeHtml(to ?? "至今")}</span><b>已保存</b></div>`;
  }).join("")}</div>`;
}

function abilityCards(
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

function detailPanel(
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
  return `<section id="player-profile-overview" class="panel player-detail-panel workspace-anchor-target">
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


export function playerTableRows(
  page: PlayerListPage | null,
  selectedId: string | null,
  selectedIds: ReadonlySet<string>,
): string {
  if (!page) return `<tr><td colspan="8"><div class="entity-table-empty"><strong>正在载入球员</strong><span>名单准备完成后会显示在这里。</span></div></td></tr>`;
  if (page.items.length === 0) return `<tr><td colspan="8"><div class="entity-table-empty"><strong>没有匹配球员</strong><span>调整搜索词、球队或位置筛选。</span></div></td></tr>`;
  return page.items.map((player) => {
    const name = displayPlayerName(player);
    const status = availabilityLabel(player.availability_status);
    const statusTone = player.availability_status && ["unavailable", "injured", "suspended", "doubtful"].includes(player.availability_status) ? "warning" : "positive";
    return `<tr class="${selectedId === player.id ? "active" : ""}" data-player-id="${escapeHtml(player.id)}">
      <td><label class="entity-table-check"><input type="checkbox" class="player-select-checkbox" data-player-id="${escapeHtml(player.id)}" ${selectedIds.has(player.id) ? "checked" : ""}><span></span></label></td>
      <td><button class="entity-table-person" data-action="open-player" data-player-id="${escapeHtml(player.id)}"><span class="entity-avatar player-avatar">${escapeHtml(initials(name.primary))}</span><span><strong>${escapeHtml(name.primary)}</strong>${name.secondary ? `<small>${escapeHtml(name.secondary)}</small>` : `<small>${escapeHtml(player.nationality_code ?? "国籍未设置")}</small>`}</span></button></td>
      <td><span class="position-chip">${escapeHtml(positionLabel(player.primary_position_code))}</span></td>
      <td><span class="table-main-text">${escapeHtml(player.current_team_name ?? "未登记球队")}</span></td>
      <td>${escapeHtml(ageFromBirthDate(player.date_of_birth))}</td>
      <td><strong class="rating-value">${player.ability_average === null ? "—" : player.ability_average.toFixed(1)}</strong>${player.ability_confidence === null ? "" : `<small class="table-subtext">${formatPercent(player.ability_confidence)}</small>`}</td>
      <td><span class="status-chip ${statusTone}">${escapeHtml(status)}</span></td>
      <td><div class="table-row-actions"><button class="table-row-action primary-link" data-action="open-player-profile" data-player-id="${escapeHtml(player.id)}">完整档案</button></div></td>
    </tr>`;
  }).join("");
}

function playerQuickInspector(detail: PlayerDetail): string {
  const name = detailPlayerName(detail);
  const primaryPosition = detail.positions.find((item) => item.is_primary === true) ?? detail.positions[0];
  const positionCode = typeof primaryPosition?.position_code === "string" ? primaryPosition.position_code : null;
  const availability = detail.availability[0];
  const availabilityStatus = typeof availability?.status === "string" ? availability.status : null;
  const availabilityReason = typeof availability?.reason === "string" ? availability.reason : null;
  const currentTeam = detail.team_periods.find((item) => item.valid_to === null) ?? detail.team_periods[0];
  const ability = detail.ability_profile;
  return `<div class="entity-inspector-content">
    <div class="inspector-identity"><span class="entity-avatar player-avatar large">${escapeHtml(initials(name.primary))}</span><div><span>球员速览</span><h2>${escapeHtml(name.primary)}</h2>${name.secondary ? `<p>${escapeHtml(name.secondary)}</p>` : `<p>${escapeHtml(detail.player.nationality_code ?? "国籍未设置")}</p>`}</div></div>
    <div class="inspector-facts three"><div><span>位置</span><strong>${escapeHtml(positionLabel(positionCode))}</strong></div><div><span>年龄</span><strong>${escapeHtml(ageFromBirthDate(detail.player.date_of_birth))}</strong></div><div><span>惯用脚</span><strong>${escapeHtml(preferredFootLabel(detail.player.preferred_foot))}</strong></div></div>
    <div class="inspector-score-row"><div><span>综合能力</span><strong>${ability?.average_value?.toFixed(1) ?? "—"}</strong><small>${ability ? `${ability.dimension_count} 个维度` : "暂无能力观察"}</small></div><div><span>当前状态</span><strong>${escapeHtml(availabilityLabel(availabilityStatus as never))}</strong><small>${escapeHtml(availabilityReason ?? "无补充说明")}</small></div></div>
    <dl class="inspector-description"><div><dt>当前球队</dt><dd>${escapeHtml(currentTeam?.team_name ?? "未登记")}</dd></div><div><dt>身高</dt><dd>${detail.player.height_cm ? `${detail.player.height_cm} cm` : "未设置"}</dd></div><div><dt>档案状态</dt><dd>${escapeHtml(playerStatusLabel(detail.player.status))}</dd></div><div><dt>动态标签</dt><dd>${detail.dynamic_tags.length} 项有效</dd></div></dl>
    <div class="inspector-section"><div class="inspector-section-title"><strong>能力与标签</strong><span>${detail.dynamic_tags.length + (ability?.dimension_count ?? 0)} 项</span></div><div class="inspector-tag-cloud">${detail.dynamic_tags.slice(0, 6).map((tag) => `<span>${escapeHtml(tag.label ?? tag.tag_name)}</span>`).join("") || `<span class="muted-chip">暂无动态标签</span>`}</div></div>
    <div class="inspector-actions"><button class="primary" data-action="select-workspace-section" data-section-id="profile">编辑完整档案</button><button class="secondary" data-action="open-player-api-workspace" data-player-id="${escapeHtml(detail.player.id)}">AI 问答</button></div>
  </div>`;
}

function playerTaskWorkspace(
  section: string,
  state: BootstrapResponse,
  references: PlayerCatalogReferenceData | null,
  selectedPlayer: PlayerDetail | null,
  preview: SpreadsheetImportPreview | null,
  navigationContext: PlayerNavigationContext | null,
): string {
  if (section === "directory") return "";
  let title = "";
  let content = "";
  if (section === "profile") {
    title = selectedPlayer ? `${detailPlayerName(selectedPlayer).primary} · 完整档案` : "球员完整档案";
    content = detailPanel(selectedPlayer, references, state);
  } else if (section === "workbook") {
    title = "球员资料工作包";
    content = spreadsheetPanel(preview);
  } else {
    title = "新增球员";
    content = `<section class="panel task-create-panel"><div class="task-form-heading"><span>基础身份</span><h3>创建后继续补充球队履历、位置和能力观察</h3></div><div class="form-grid two-column-form clean-form"><label class="field"><span>球员正式姓名</span><input id="new-player-name"></label><label class="field"><span>出生日期</span><input id="new-player-birth" type="date"></label><label class="field"><span>国籍</span><input id="new-player-nationality"></label><label class="field"><span>身高（cm）</span><input id="new-player-height" type="number" min="100" max="240"></label><label class="field"><span>惯用脚</span><select id="new-player-foot"><option value="unknown">未知</option><option value="right">右脚</option><option value="left">左脚</option><option value="both">双脚</option></select></label></div><div class="workflow-actions"><button class="primary" data-action="create-player">创建球员</button></div></section>`;
  }
  const returnAction = section === "profile" && navigationContext?.origin_page === "teams"
    ? `<button class="secondary" data-action="return-to-source-team-profile" data-team-id="${escapeHtml(navigationContext.team_id)}">返回${escapeHtml(navigationContext.team_name)}完整档案</button>`
    : section === "profile" && navigationContext?.origin_page === "lineups"
      ? `<button class="secondary" data-action="return-to-lineup-workspace" data-return-section="${escapeHtml(navigationContext.return_section ?? "chain")}">返回比赛阵容</button>`
      : `<button class="secondary" data-action="select-workspace-section" data-section-id="directory">返回球员目录</button>`;
  return `<section class="entity-task-workspace"><header><div><span>球队与人员</span><h2>${escapeHtml(title)}</h2></div>${returnAction}</header><div class="entity-task-body">${content}</div></section>`;
}

export function playersPage(
  state: BootstrapResponse,
  references: PlayerCatalogReferenceData | null,
  playerPage: PlayerListPage | null,
  selectedPlayer: PlayerDetail | null,
  query: PlayerListQuery,
  spreadsheetPreview: SpreadsheetImportPreview | null,
  selectedPlayerIds: ReadonlySet<string>,
  _tabs: readonly WorkspaceTabState[],
  _activeTabId: string | null,
  _layoutMode: WorkspaceLayoutMode,
  _moduleSidebarCollapsed: boolean,
  inspectorCollapsed: boolean,
  activeSection: string,
  pageNumber = 1,
  navigationContext: PlayerNavigationContext | null = null,
): string {
  if (!state.data.database_configured) {
    return `<section class="task-empty-workspace">${taskPageHeader({ eyebrow: "球队与人员", title: "球员浏览与管理", description: "连接数据库后维护球员身份、球队履历、位置、可用性、能力和动态标签。", status: { label: "等待数据库", tone: "warning" } })}${taskContextRibbon([{ label: "当前状态", value: "数据库未连接", note: "连接成功后自动加载球员目录", tone: "warning" }])}${inlineDatabaseSetup("连接数据服务以维护球员", "连接成功后本页会自动加载球员目录。", state.connection_error)}</section>`;
  }
  const positions = references?.positions ?? [];
  const teams = references?.teams ?? [];
  const section = ["directory", "profile", "workbook", "create"].includes(activeSection) ? activeSection : "directory";
  const selectedCount = selectedPlayerIds.size;
  const selectedTeamOption = teams.find((team) => team.id === query.team_id) ?? null;
  const activeFilterCount = [query.team_id, query.position_code, query.availability_status, query.player_status].filter(Boolean).length;
  const sourceContext = navigationContext?.team_id === query.team_id ? navigationContext : null;
  const pageActions = `<button class="secondary" data-action="refresh-player-catalog">${icon("refresh")}<span>刷新名单</span></button>${selectedPlayer ? `<button class="primary" data-action="select-workspace-section" data-section-id="profile">打开完整档案</button>` : `<button class="primary" data-action="select-workspace-section" data-section-id="create">新增球员</button>`}`;
  const sectionNav = workspaceSectionNavigation([
    { id: "directory", index: "01", label: "球员目录", description: "筛选、名单和快速检查", badge: `${playerPage?.items.length ?? 0}` },
    { id: "profile", index: "02", label: "完整档案", description: selectedPlayer ? detailPlayerName(selectedPlayer).primary : "选择球员后开放", disabled: !selectedPlayer },
    { id: "workbook", index: "03", label: "球员工作包", description: "导出、预检和批量导入" },
    { id: "create", index: "04", label: "新增球员", description: "创建基础身份" },
  ], section);
  const contextRibbon = taskContextRibbon([
    { label: "当前球员", value: selectedPlayer ? detailPlayerName(selectedPlayer).primary : "尚未选择", note: selectedPlayer ? `${selectedPlayer.team_periods[0]?.team_name ?? "暂无球队"} · 完整档案按需打开` : "单击名单显示速览，主动打开完整档案", tone: selectedPlayer ? "success" : "neutral" },
    { label: "名单结果", value: `${playerPage?.items.length ?? 0} 名球员`, note: `第 ${pageNumber} 页 · ${activeFilterCount} 项筛选` },
  ]);
  const directoryWorkspace = `<section class="entity-browser player-browser master-detail-workspace ${inspectorCollapsed ? "inspector-collapsed" : "inspector-open"}" data-entity-browser="players">
      <aside class="entity-filter-panel panel master-pane" data-workspace-panel="players-filter" data-workspace-persist="false">
        <div class="entity-directory-header"><div><span>筛选器</span><strong>${activeFilterCount ? `${activeFilterCount} 项已应用` : "全部球员"}</strong></div><button class="icon-button" data-action="refresh-player-catalog" title="刷新球员">${icon("refresh")}</button></div>
        <section class="entity-filter-groups">
          <div class="entity-filter-group"><header><span>归属范围</span><small>先确定当前球队或查看全部球员</small></header>${sourceContext ? `<div class="player-source-prefill"><span>从球队页带入</span><strong>${escapeHtml(sourceContext.team_name)}</strong><small>已自动选中，可直接修改或清除</small></div>` : ""}<label class="entity-filter-field"><span>当前所属球队</span><select id="player-filter-team"><option value="">不限球队</option>${teams.map((team) => `<option value="${escapeHtml(team.id)}" ${query.team_id === team.id ? "selected" : ""}>${escapeHtml(team.canonical_name)}</option>`).join("")}</select></label></div>
          <div class="entity-filter-group"><header><span>场上角色</span><small>按登记的主要位置精确筛选</small></header><label class="entity-filter-field"><span>主要位置</span><select id="player-filter-position"><option value="">不限位置</option>${positions.map((position) => `<option value="${escapeHtml(position.code)}" ${query.position_code === position.code ? "selected" : ""}>${escapeHtml(positionLabel(position.code))}</option>`).join("")}</select></label></div>
          <div class="entity-filter-group"><header><span>比赛可用性</span><small>伤病、停赛和恢复状态来自当前有效记录</small></header><label class="entity-filter-field"><span>当前可用状态</span><select id="player-filter-availability"><option value="">不限可用状态</option>${["available","doubtful","injured","suspended","rested","returning","unknown"].map((value) => `<option value="${value}" ${query.availability_status === value ? "selected" : ""}>${escapeHtml(availabilityLabel(value as never))}</option>`).join("")}</select></label></div>
          <div class="entity-filter-group"><header><span>档案生命周期</span><small>控制现役、停用、退役和未知档案</small></header><label class="entity-filter-field"><span>球员档案状态</span><select id="player-filter-status"><option value="">不限档案状态</option>${["active","inactive","retired","unknown"].map((value) => `<option value="${value}" ${query.player_status === value ? "selected" : ""}>${escapeHtml(playerStatusLabel(value))}</option>`).join("")}</select></label></div>
        </section>
        <div class="entity-active-filters"><b>${activeFilterCount ? `${activeFilterCount} 项已应用` : "当前未限制"}</b>${query.team_id ? `<span>${escapeHtml(selectedTeamOption?.canonical_name ?? sourceContext?.team_name ?? "指定球队")}</span>` : ""}${query.position_code ? `<span>${escapeHtml(positionLabel(query.position_code))}</span>` : ""}${query.availability_status ? `<span>${escapeHtml(availabilityLabel(query.availability_status as never))}</span>` : ""}${query.player_status ? `<span>${escapeHtml(playerStatusLabel(query.player_status))}</span>` : ""}</div>
        <div class="entity-filter-actions"><button class="primary" data-action="search-players">应用筛选</button><button class="secondary" data-action="clear-player-filters">清除全部</button></div>
      </aside>
      <main class="entity-main panel detail-pane" data-workspace-scroll-key="players-main">
        <div class="player-list-toolbar"><label class="entity-search wide" data-workspace-persist="false">${icon("search")}<input id="player-search" value="${escapeHtml(query.search ?? "")}" placeholder="支持中文名、原名或别名的部分匹配"><button data-action="search-players">搜索</button></label><div class="entity-list-actions"><span>${query.search ? `搜索“${escapeHtml(query.search)}” · ` : ""}${playerPage?.items.length ?? 0} 条当前结果</span><button class="secondary" data-action="toggle-workspace-pane" data-pane="inspector" ${selectedPlayer ? "" : "disabled"}>球员速览</button></div></div>
        <div class="entity-table-wrap player-table-wrap"><table class="entity-data-table player-directory-table"><thead><tr><th><label class="entity-table-check"><input id="player-select-all" type="checkbox"><span></span></label></th><th>球员</th><th>位置</th><th>当前球队</th><th>年龄</th><th>能力</th><th>状态</th><th>操作</th></tr></thead><tbody>${playerTableRows(playerPage, selectedPlayer?.player.id ?? null, selectedPlayerIds)}</tbody></table></div>
        <footer class="entity-main-footer"><span>${selectedCount ? `已选择 ${selectedCount} 名球员` : "点击球员查看右侧速览"}</span><div><button class="secondary tiny" data-action="previous-player-page" ${pageNumber <= 1 ? "disabled" : ""}>上一页</button><b>第 ${pageNumber} 页</b><button class="secondary tiny" data-action="next-player-page" ${playerPage?.has_more ? "" : "disabled"}>下一页</button></div></footer>
      </main>
      <aside class="entity-inspector panel inspector-pane" data-workspace-panel="players-inspector"><button class="entity-inspector-close icon-button" data-action="toggle-workspace-pane" data-pane="inspector" aria-label="关闭速览">×</button>${selectedPlayer ? playerQuickInspector(selectedPlayer) : `<div class="entity-inspector-empty"><span class="empty-orbit">${icon("shield")}</span><strong>选择一名球员</strong><span>速览不会离开当前名单。</span></div>`}</aside>
    </section>${selectedCount ? `<div class="entity-selection-bar"><strong>已选 ${selectedCount} 名球员</strong><span>可批量打开、归档或删除空对象</span><button class="secondary" data-action="open-selected-players">打开</button><button class="secondary" data-action="bulk-archive-players">归档</button><button class="danger" data-action="bulk-delete-players">批量删除空对象</button></div>` : ""}`;
  const activeWorkspace = section === "directory"
    ? directoryWorkspace
    : playerTaskWorkspace(section, state, references, selectedPlayer, spreadsheetPreview, navigationContext);
  return `<section class="entity-page entity-page-players task-page core-workspace-page core-player-workspace">
    ${taskPageHeader({ eyebrow: "球员中心", title: "球员浏览与管理", description: "目录、完整档案、工作包和新增球员保持在同一页面；档案内部按基础资料、位置角色、履历、状态和能力继续分层。", status: { label: section === "profile" ? "正在编辑完整档案" : sourceContext ? "球队筛选已带入" : selectedPlayer ? "球员速览已就绪" : "等待选择球员", tone: section === "profile" || sourceContext || selectedPlayer ? "success" : "neutral" }, actions: pageActions })}
    ${contextRibbon}
    <div class="core-local-navigation">${sectionNav}</div>
    <div class="core-workspace-stage">${activeWorkspace}</div>
  </section>`;
}
