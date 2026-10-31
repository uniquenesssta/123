import { positionLabel } from "../../components/footballText";
import { escapeHtml } from "../../components/format";
import { enhanceSearchableSelects } from "../../components/searchableSelect";
import type { TeamLineupPresetDraft, TeamLineupPresetMemberDraft, TeamLineupPresetRecord } from "../../types";
import type { LineupContext } from "./lineupContext";
export function createPresetEditor(c: LineupContext) {
    let editorIdentity: {
        teamId: string;
        presetId: string | null;
        version: number | null;
        page: string;
    } | null = null;
    function assertPresetEditorIdentity(presetId: string | null): void {
        const ticket = editorIdentity, preset = presetId ? c.ports.teamPresets.find(item => item.id === presetId) : null;
        if (!c.active || !ticket || ticket.teamId !== c.ports.teamDetail?.team.id || ticket.presetId !== presetId || ticket.page !== c.ports.navigation.page || (presetId && ticket.version !== preset?.version))
            throw new Error("球队或预设版本已变化，请重新打开编辑器");
    }
    function teamLineupPresetMemberRows(preset: TeamLineupPresetRecord | null): string {
        if (!c.ports.teamDetail)
            return "";
        const existing = new Map(preset?.members.map((member) => [member.player_id, member]) ?? []);
        return c.ports.teamDetail.squad.map((player, index) => {
            const member = existing.get(player.player_id);
            const enabled = Boolean(member) || (!preset && index < 18);
            const starter = member?.is_starter ?? (!preset && index < 11);
            const selectedPosition = member?.position_code ?? player.position_code ?? null;
            const inheritedRole = member?.role_origin === "player_position_default"
                ? member.role_code
                : player.role_code;
            const explicitRole = member?.role_origin === "lineup_override"
                ? member.role_code
                : null;
            return `<article class="preset-member-row ${enabled ? "" : "excluded"} ${starter ? "starter" : "substitute"}" data-preset-player-id="${escapeHtml(player.player_id)}" data-player-default-position="${escapeHtml(player.position_code ?? "")}" data-player-inherited-role="${escapeHtml(inheritedRole ?? "")}">
      <label class="preset-member-toggle"><input class="preset-member-enabled" type="checkbox" aria-label="选入 ${escapeHtml(player.player_name)}" ${enabled ? "checked" : ""}></label><div class="preset-member-name"><strong>${escapeHtml(player.player_name)}</strong><small>${escapeHtml(positionLabel(player.position_code))} · #${player.squad_number ?? "—"}</small></div>
      <select class="preset-member-role" data-native-select aria-label="出场身份" ${enabled ? "" : "disabled"}><option value="starter" ${starter ? "selected" : ""}>首发</option><option value="substitute" ${starter ? "" : "selected"}>替补</option></select>
      <select class="preset-member-position" data-native-select aria-label="战术位置" ${enabled ? "" : "disabled"}>${c.actions.positionOptionGroups(preset?.formation_id ?? null, selectedPosition)}</select>
      <select class="preset-member-tactical-role" data-native-select aria-label="战术角色" ${enabled ? "" : "disabled"}>${c.actions.tacticalRoleOptions(selectedPosition, inheritedRole, explicitRole)}</select>
      <label class="preset-member-captain"><input type="radio" name="preset-captain" value="${escapeHtml(player.player_id)}" ${member?.is_captain ? "checked" : ""} ${enabled ? "" : "disabled"} aria-label="队长 ${escapeHtml(player.player_name)}"></label>
    </article>`;
        }).join("");
    }
    function openTeamLineupPresetEditor(presetId: string | null = null): void {
        if (!c.ports.teamDetail)
            throw new Error("请先选择球队");
        const preset = presetId
            ? c.ports.teamPresets.find((item) => item.id === presetId) ?? null
            : null;
        if (presetId && !preset)
            throw new Error("阵容预设不存在或尚未加载");
        editorIdentity = { teamId: c.ports.teamDetail.team.id, presetId, version: preset?.version ?? null, page: c.ports.navigation.page };
        const teamCoaches = c.ports.coachList.filter((coach) => coach.current_team_id === c.ports.teamDetail!.team.id);
        const body = `<div class="lineup-preset-editor">
    <div id="lineup-preset-validation-note" class="preset-editor-validation"></div>
    <div class="preset-editor-scroll" data-workspace-scroll-key="lineup-preset-editor">
    <section class="panel preset-metadata-panel"><header class="preset-section-heading"><h3>方案资料</h3><p>归属球队：${escapeHtml(c.ports.teamDetail.team.canonical_name)}；教练可不绑定，使用概率可留空。</p></header>
    <div class="form-grid three-column clean-form">
      <label class="field"><span>预设名称</span><input id="lineup-preset-name" value="${escapeHtml(preset?.name ?? "")}" placeholder="例如：联赛主力阵容"></label>
      <label class="field"><span>阵型</span><select id="lineup-preset-formation"><option value="">未设置</option>${c.ports.formationCatalog.map((formation) => `<option value="${escapeHtml(formation.id)}" ${formation.id === preset?.formation_id ? "selected" : ""}>${escapeHtml(formation.code)} · ${escapeHtml(formation.name)}</option>`).join("")}</select></label>
      <label class="field"><span>教练</span><select id="lineup-preset-coach"><option value="">不绑定</option>${teamCoaches.map((coach) => `<option value="${escapeHtml(coach.id)}" ${coach.id === preset?.coach_id ? "selected" : ""}>${escapeHtml(coach.canonical_name)} · ${escapeHtml(coach.current_role ?? "教练")}</option>`).join("")}</select></label>
      <label class="field"><span>适用场景</span><select id="lineup-preset-context">${["general", "league", "cup", "attacking", "defensive", "rotation", "temporary"].map((context) => `<option value="${context}" ${context === (preset?.usage_context ?? "general") ? "selected" : ""}>${c.actions.presetContextLabel(context)}</option>`).join("")}</select></label>
      <label class="field"><span>使用概率（0–1）</span><input id="lineup-preset-probability" type="number" min="0" max="1" step="0.01" value="${preset?.usage_probability ?? ""}"></label>
      <label class="check-row"><input id="lineup-preset-default" type="checkbox" ${preset?.is_default ? "checked" : ""}><span><strong>设为默认方案</strong><small>同一球队只能有一个默认预设</small></span></label>
      <label class="field span-3"><span>备注</span><textarea id="lineup-preset-notes" rows="2">${escapeHtml(preset?.notes ?? "")}</textarea></label>
    </div>
    </section><section class="panel preset-members-panel">
    <div class="preset-member-editor-heading"><div><strong>出场人员</strong><span>首发必须覆盖当前阵型的 11 个战术位置；战术角色可直接选择或自动继承球员资料。</span></div><div class="preset-member-editor-actions"><span id="lineup-preset-member-summary">正在检查阵容</span><button class="secondary compact" data-action="auto-assign-preset-formation">按阵型自动分配</button></div></div>
    <div class="preset-member-table-head" aria-hidden="true"><span>入选</span><span>球员</span><span>身份</span><span>战术位置</span><span>战术角色 / 来源</span><span>队长</span></div>
    <div class="preset-member-list">${teamLineupPresetMemberRows(preset)}</div>
    </section></div>
  </div>`;
        const footer = `<button class="secondary" data-action="close-workspace-detail">取消</button><button class="primary" data-action="save-team-lineup-preset" data-preset-id="${escapeHtml(preset?.id ?? "")}">保存阵容预设</button>`;
        c.ports.showHtmlModal(preset ? `编辑：${preset.name}` : "新建阵容预设", c.ports.teamDetail.team.canonical_name, body, footer, "lineup-preset-modal");
        queueMicrotask(() => {
            const workspaceRoot = document.querySelector(".workspace-panel-root") ?? document;
            enhanceSearchableSelects(workspaceRoot);
            const memberList = document.querySelector<HTMLElement>(".preset-member-list");
            if (memberList)
                memberList.scrollTop = 0;
            c.actions.syncPresetEditorRows();
            c.actions.updatePresetEditorSummary();
        });
    }
    function validateTeamLineupPresetDraft(draft: TeamLineupPresetDraft): void {
        if (!draft.name)
            throw new Error("请输入预设名称");
        if (draft.members.length < 11)
            throw new Error("阵容预设至少需要 11 名球员");
        const starters = draft.members.filter((member) => member.is_starter);
        if (starters.length !== 11)
            throw new Error(`首发必须恰好为 11 人，当前为 ${starters.length} 人`);
        const missingPositions = starters.filter((member) => !member.position_code);
        if (missingPositions.length > 0)
            throw new Error(`仍有 ${missingPositions.length} 名首发未选择战术位置`);
        const positions = starters.map((member) => member.position_code as string);
        const duplicates = positions.filter((value, index) => positions.indexOf(value) !== index);
        if (duplicates.length > 0)
            throw new Error(`首发战术位置不能重复：${[...new Set(duplicates)].join("、")}`);
        const slots = c.actions.formationSlotCodes(draft.formation_id);
        if (slots.length > 0) {
            const missing = slots.filter((slot) => !positions.includes(slot));
            const unexpected = positions.filter((position) => !slots.includes(position));
            if (missing.length || unexpected.length) {
                throw new Error(`首发位置必须完整匹配当前阵型。缺少：${missing.join("、") || "无"}；不属于阵型：${unexpected.join("、") || "无"}`);
            }
        }
        if (draft.usage_probability !== null && (draft.usage_probability < 0 || draft.usage_probability > 1)) {
            throw new Error("使用概率必须位于 0 到 1 之间");
        }
    }
    function collectTeamLineupPresetDraft(presetId: string | null): TeamLineupPresetDraft {
        if (!c.ports.teamDetail)
            throw new Error("请先选择球队");
        const rows = Array.from(document.querySelectorAll<HTMLElement>("[data-preset-player-id]"));
        const members = rows.flatMap((row, index): TeamLineupPresetMemberDraft[] => {
            const enabled = row.querySelector<HTMLInputElement>(".preset-member-enabled")?.checked ?? false;
            if (!enabled)
                return [];
            const playerId = row.dataset.presetPlayerId ?? "";
            const role = row.querySelector<HTMLSelectElement>(".preset-member-role")?.value ?? "substitute";
            const position = row.querySelector<HTMLSelectElement>(".preset-member-position")?.value.trim() || null;
            const tacticalRole = row.querySelector<HTMLSelectElement>(".preset-member-tactical-role")?.value.trim() || null;
            const captain = row.querySelector<HTMLInputElement>(".preset-member-captain input")?.checked ?? false;
            return [{
                    player_id: playerId,
                    position_code: position,
                    role_code: tacticalRole,
                    is_starter: role === "starter",
                    shirt_number: c.ports.teamDetail!.squad.find((item) => item.player_id === playerId)?.squad_number ?? null,
                    expected_minutes: role === "starter" ? 90 : 20,
                    sequence_no: index + 1,
                    bench_order: role === "starter" ? null : index + 1,
                    is_captain: captain,
                    metadata: {},
                }];
        });
        return {
            id: presetId,
            team_id: c.ports.teamDetail.team.id,
            name: c.ports.value("lineup-preset-name").trim(),
            formation_id: c.ports.nullableValue("lineup-preset-formation"),
            coach_id: c.ports.nullableValue("lineup-preset-coach"),
            usage_context: c.ports.value("lineup-preset-context"),
            usage_probability: c.ports.nullableNumber("lineup-preset-probability"),
            is_default: c.ports.checked("lineup-preset-default"),
            source_lineup_id: null,
            notes: c.ports.nullableValue("lineup-preset-notes"),
            members,
        };
    }
    return { assertPresetEditorIdentity, teamLineupPresetMemberRows, openTeamLineupPresetEditor, validateTeamLineupPresetDraft, collectTeamLineupPresetDraft };
}
