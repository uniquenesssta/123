import { escapeHtml } from "./format";
import { availabilityLabel, displayPlayerName, positionLabel } from "./footballText";
import type {
  LineupBuilderFormState,
  LineupBuilderPlayer,
  PlayerCatalogReferenceData,
  PlayerListItem,
} from "../types";


function formationSelect(references: PlayerCatalogReferenceData | null, selectedId: string, selectedCode: string): string {
  const formations = (references?.formations ?? []).filter((item) => item.code !== "CUSTOM");
  return `<select id="new-lineup-formation-id"><option value="">保留自由文本</option>${formations.map((item) => `<option value="${escapeHtml(item.id)}" data-code="${escapeHtml(item.code)}" ${(item.id === selectedId || (!selectedId && item.code === selectedCode)) ? "selected" : ""}>${escapeHtml(item.code)} · ${escapeHtml(item.name)}</option>`).join("")}</select>`;
}

function positionOptions(references: PlayerCatalogReferenceData | null, selected: string | null): string {
  return `<option value="">自动</option>${(references?.positions ?? []).map((item) => `<option value="${escapeHtml(item.code)}" ${item.code === selected ? "selected" : ""}>${escapeHtml(item.name)}</option>`).join("")}`;
}

export function lineupCandidateRows(candidates: PlayerListItem[], selected: LineupBuilderPlayer[], side?: "home" | "away"): string {
  const selectedIds = new Set(selected.map((item) => item.player_id));
  const available = candidates.filter((item) => !selectedIds.has(item.id));
  if (available.length === 0) {
    return `<div class="empty-state compact"><strong>没有可添加的球员</strong><span>该队名单为空或已全部加入阵容。</span>${side ? `<button class="secondary tiny" data-action="complete-workflow" data-target-page="players" data-target-section="directory" data-return-reason="补充${side === "home" ? "主队" : "客队"}球员后返回阵容编排">前往球员目录补充</button>` : ""}</div>`;
  }
  return `<div class="player-picker-list">${available.slice(0, 100).map((item) => { const name = displayPlayerName(item); return `<article class="player-picker-row"><div><strong>${escapeHtml(name.primary)}</strong><small>${name.secondary ? `${escapeHtml(name.secondary)} · ` : ""}${escapeHtml(positionLabel(item.primary_position_code))} · ${escapeHtml(availabilityLabel(item.availability_status))}</small></div><div class="button-row compact"><button class="secondary tiny" data-action="add-lineup-player" ${side ? `data-lineup-side="${side}"` : ""} data-player-id="${escapeHtml(item.id)}" data-role="starter">加入首发</button><button class="ghost tiny" data-action="add-lineup-player" ${side ? `data-lineup-side="${side}"` : ""} data-player-id="${escapeHtml(item.id)}" data-role="substitute">加入替补</button></div></article>`; }).join("")}</div>`;
}

export function lineupSelectedRows(selected: LineupBuilderPlayer[], references: PlayerCatalogReferenceData | null, side?: "home" | "away"): string {
  if (selected.length === 0) {
    return `<div class="empty-state compact"><strong>阵容尚为空</strong><span>从左侧名单加入首发或替补。</span></div>`;
  }
  return `<div class="lineup-builder-list">${selected.map((item, index) => `<article class="lineup-builder-row" data-lineup-builder-row ${side ? `data-lineup-side="${side}"` : ""} data-player-id="${escapeHtml(item.player_id)}"><div class="lineup-player-title"><span class="lineup-order">${index + 1}</span><div><strong>${escapeHtml(item.player_name)}</strong><small>${item.player_secondary_name ? `${escapeHtml(item.player_secondary_name)} · ` : ""}${item.is_starter ? "首发" : "替补"}</small></div></div><label class="field compact"><span>身份</span><select data-lineup-field="is_starter"><option value="true" ${item.is_starter ? "selected" : ""}>首发</option><option value="false" ${!item.is_starter ? "selected" : ""}>替补</option></select></label><label class="field compact"><span>位置</span><select data-lineup-field="position_code">${positionOptions(references, item.position_code)}</select></label><label class="field compact"><span>角色</span><input data-lineup-field="role_code" value="${escapeHtml(item.role_code ?? "")}" placeholder="可选"></label><label class="field compact"><span>预计分钟</span><input data-lineup-field="expected_minutes" type="number" min="0" max="150" value="${item.expected_minutes ?? (item.is_starter ? 90 : 20)}"></label><label class="field compact"><span>首发概率</span><input data-lineup-field="starting_probability" type="number" min="0" max="1" step="0.01" value="${item.starting_probability ?? (item.is_starter ? 1 : 0)}"></label><label class="field compact"><span>替补顺序</span><input data-lineup-field="bench_order" type="number" min="1" max="99" value="${item.bench_order ?? ""}"></label><label class="field compact"><span>号码</span><input data-lineup-field="shirt_number" type="number" min="0" max="99" value="${item.shirt_number ?? ""}"></label><label class="check-field compact"><input data-lineup-field="membership_override" type="checkbox" ${item.membership_override ? "checked" : ""}><span>履历例外</span></label><button class="ghost tiny danger" data-action="remove-lineup-player" ${side ? `data-lineup-side="${side}"` : ""} data-player-id="${escapeHtml(item.player_id)}">移除</button></article>`).join("")}</div>`;
}

export function lineupCompletionWorkspace(
  references: PlayerCatalogReferenceData | null,
  candidates: PlayerListItem[],
  selected: LineupBuilderPlayer[],
  form: LineupBuilderFormState,
  fixedMatchId: string,
  allowedTeamIds: string[],
): string {
  const matches = references?.managed_matches ?? references?.upcoming_matches ?? [];
  const match = matches.find((item) => item.id === fixedMatchId);
  const teams = (references?.teams ?? []).filter((item) => allowedTeamIds.includes(item.id));
  return `<section class="inline-lineup-completion">
    <div class="completion-note"><b>阵容缺失可在本页补录</b><span>先选择主队或客队，加载球员后保存；另一队按相同步骤补录。</span></div>
    <div class="form-grid four-column clean-form">
      <label class="field"><span>比赛</span><select id="new-lineup-match"><option value="${escapeHtml(fixedMatchId)}" data-home-team="${escapeHtml(match?.home_team_id ?? allowedTeamIds[0] ?? "")}" data-away-team="${escapeHtml(match?.away_team_id ?? allowedTeamIds[1] ?? "")}" selected>${escapeHtml(match ? `${match.home_team_name} vs ${match.away_team_name}` : "当前比赛")}</option></select></label>
      <label class="field"><span>球队</span><select id="new-lineup-team"><option value="">选择需要补录的球队</option>${teams.map((team) => `<option value="${escapeHtml(team.id)}" ${team.id === form.team_id ? "selected" : ""}>${escapeHtml(team.canonical_name)}</option>`).join("")}</select></label>
      <label class="field"><span>阵容类型</span><select id="new-lineup-type"><option value="actual" selected>实际阵容</option><option value="confirmed">确认阵容</option><option value="expected">预计阵容</option></select></label>
      <label class="field"><span>数据窗口</span><select id="new-lineup-snapshot"><option value="T-N" selected>T-N</option><option value="T-24h">T-24h</option><option value="T-6h">T-6h</option><option value="T-1h">T-1h</option></select></label>
      <label class="field"><span>阵型</span>${formationSelect(references, form.formation_id, form.formation || "4-2-3-1")}<input id="new-lineup-formation" type="hidden" value="${escapeHtml(form.formation || "4-2-3-1")}"></label>
      <input id="new-lineup-coach" type="hidden" value=""><input id="new-lineup-source-urls" type="hidden" value="">
      <label class="field"><span>记录时间</span><input id="new-lineup-captured-at" type="datetime-local" value="${escapeHtml(form.captured_at)}"></label>
      <label class="field"><span>数据可信度（0–1）</span><input id="new-lineup-quality" type="number" min="0" max="1" step="0.01" value="${form.quality_score}"></label>
      <div class="field action-field"><span>球队名单</span><button class="secondary" data-action="load-lineup-players">加载球员</button></div>
      <div class="field action-field"><span>当前选择</span><strong class="selection-count">${selected.length} 人 · 首发 ${selected.filter((item) => item.is_starter).length} 人</strong></div>
    </div>
    <div class="two-column lineup-picker-grid compact-workspace">
      <article class="subpanel"><div class="panel-heading compact"><div><span>可选球员</span><h3>${candidates.length} 人</h3></div></div>${lineupCandidateRows(candidates, selected)}</article>
      <article class="subpanel"><div class="panel-heading compact"><div><span>本次阵容</span><h3>${selected.length} 人</h3></div><button class="ghost tiny" data-action="clear-lineup-builder">清空</button></div>${lineupSelectedRows(selected, references)}</article>
    </div>
    <div class="workflow-actions"><span class="field-note">保存后仍停留在复盘页面，阵容会立即出现在球员评分区。</span><button class="primary" data-action="create-lineup">保存这支球队的阵容</button></div>
  </section>`;
}
