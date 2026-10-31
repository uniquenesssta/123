import { escapeHtml } from "../../components/format";
import { enhanceSearchableSelects } from "../../components/searchableSelect";
import type { TeamLineupPresetDraft, TeamLineupPresetRecord } from "../../types";
import type { LineupContext, LineupSide } from "./lineupContext";
export function createPresetActions(c: LineupContext) {
    let savingPreset = false;
    let duplicateIdentity: {
        teamId: string;
        presetId: string;
        page: string;
    } | null = null;
    let quickIdentity: {
        side: LineupSide;
        identity: string;
        page: string;
    } | null = null;
    function lineupPresetManagerBody(teamId: string, teamName: string, presets: TeamLineupPresetRecord[]): string {
        const active = presets.filter((preset) => preset.status === "active");
        const archived = presets.filter((preset) => preset.status === "archived");
        const canEdit = c.ports.teamDetail?.team.id === teamId;
        const rows = presets.length
            ? presets.map((preset) => `<article class="lineup-preset-manager-row ${preset.status}">
        <div><span>${preset.status === "active" ? (preset.is_default ? "默认 · 使用中" : "使用中") : "已归档"}</span><strong>${escapeHtml(preset.name)}</strong><small>${escapeHtml(preset.formation_code ?? "阵型未设置")} · 首发 ${preset.starter_count} · 共 ${preset.member_count} 人 · v${preset.version}</small></div>
        <div class="lineup-preset-manager-actions">${canEdit && preset.status === "active" ? `<button class="secondary tiny" data-action="open-team-lineup-preset-editor" data-preset-id="${escapeHtml(preset.id)}">编辑</button>` : ""}<button class="ghost tiny danger" data-action="request-delete-team-lineup-preset" data-preset-id="${escapeHtml(preset.id)}" data-preset-name="${escapeHtml(preset.name)}" data-team-id="${escapeHtml(teamId)}" data-team-name="${escapeHtml(teamName)}" data-preset-status="${escapeHtml(preset.status)}" data-member-count="${preset.member_count}">永久删除</button></div>
      </article>`).join("")
            : `<div class="empty-state compact"><strong>暂无阵容预设</strong><p>当前球队没有活动或已归档预设。</p></div>`;
        return `<div class="lineup-preset-manager">
    <div class="lineup-preset-manager-summary"><div><span>活动预设</span><strong>${active.length}</strong></div><div><span>已归档</span><strong>${archived.length}</strong></div><div><span>合计</span><strong>${presets.length}</strong></div></div>
    <div class="blocking-note warning">永久删除会同时删除该预设的首发与替补成员，无法恢复；不会删除球队、球员或比赛阵容。</div>
    <div class="lineup-preset-manager-list">${rows}</div>
  </div>`;
    }
    async function openTeamLineupPresetManager(teamId: string, teamName: string): Promise<void> {
        if (!teamId)
            throw new Error("缺少球队 ID，无法管理阵容预设");
        const page = c.ports.navigation.page, matchId = c.state.pairedLineupBuilder.match_id, selected = c.ports.teamDetail?.team.id;
        const presets = await c.ports.runBusy(() => c.ports.api.listTeamLineupPresets(teamId, true));
        if (!c.active || page !== c.ports.navigation.page || matchId !== c.state.pairedLineupBuilder.match_id || selected !== c.ports.teamDetail?.team.id)
            return;
        c.ports.setTeamPresets(teamId, presets);
        c.ports.showHtmlModal("预设阵容管理", teamName || "球队阵容预设", lineupPresetManagerBody(teamId, teamName, presets), `<button class="secondary" data-action="close-workspace-detail">关闭</button>${c.ports.teamDetail?.team.id === teamId ? '<button class="primary" data-action="open-team-lineup-preset-editor">新建阵容预设</button>' : ""}`, "lineup-preset-manager-modal");
    }
    async function openLineupPresetManagerForSide(side: LineupSide): Promise<void> {
        const current = c.actions.pairedSide(side);
        if (!current.team_id)
            throw new Error(`${side === "home" ? "主队" : "客队"}尚未确定`);
        await openTeamLineupPresetManager(current.team_id, current.team_name);
    }
    async function refreshTeamLineupPresetCaches(teamId: string): Promise<void> {
        const matchId = c.state.pairedLineupBuilder.match_id, page = c.ports.navigation.page;
        const [allPresets, activePresets] = await Promise.all([
            c.ports.api.listTeamLineupPresets(teamId, true),
            c.ports.api.listTeamLineupPresets(teamId, false),
        ]);
        if (!c.active || matchId !== c.state.pairedLineupBuilder.match_id || page !== c.ports.navigation.page)
            return;
        c.ports.setTeamPresets(teamId, allPresets);
        if (c.actions.pairedSide("home").team_id === teamId) {
            c.state.pairedLineupPresets = { ...c.state.pairedLineupPresets, home: activePresets };
        }
        if (c.actions.pairedSide("away").team_id === teamId) {
            c.state.pairedLineupPresets = { ...c.state.pairedLineupPresets, away: activePresets };
        }
    }
    function requestDeleteTeamLineupPreset(presetId: string, presetName: string, teamId: string, teamName: string, status: string, memberCount: number): void {
        const page = c.ports.navigation.page, selected = c.ports.teamDetail?.team.id;
        c.ports.showConfirmation("永久删除阵容预设", "该操作无法恢复。只删除此预设及其成员，不影响球队、球员、比赛阵容和历史快照。", [
            ["球队", teamName],
            ["预设", presetName],
            ["状态", status === "archived" ? "已归档" : "活动"],
            ["预设成员", `${memberCount} 人`],
        ], "确认永久删除", async () => {
            if (!c.active || page !== c.ports.navigation.page || selected !== c.ports.teamDetail?.team.id)
                throw new Error("当前球队已变化，请重新确认删除预设");
            await c.ports.runBusy(() => c.ports.api.deleteTeamLineupPreset(presetId));
            if (!c.active || page !== c.ports.navigation.page || selected !== c.ports.teamDetail?.team.id)
                return;
            await refreshTeamLineupPresetCaches(teamId);
            if (!c.active || page !== c.ports.navigation.page || selected !== c.ports.teamDetail?.team.id)
                return;
            c.ports.render({ preserveForm: true });
            c.ports.toast("阵容预设已永久删除", "success");
            await openTeamLineupPresetManager(teamId, teamName);
        });
    }
    async function saveTeamLineupPreset(presetId: string | null): Promise<void> {
        if (savingPreset)
            return;
        c.actions.assertPresetEditorIdentity(presetId);
        const origin = c.ports.teamDetail?.team.id, page = c.ports.navigation.page;
        const draft = c.actions.collectTeamLineupPresetDraft(presetId);
        c.actions.validateTeamLineupPresetDraft(draft);
        savingPreset = true;
        try {
            await c.ports.runBusy(() => c.ports.api.saveTeamLineupPreset(draft));
        }
        finally {
            savingPreset = false;
        }
        if (!c.active || origin !== c.ports.teamDetail?.team.id || page !== c.ports.navigation.page) {
            c.ports.toast("原球队预设已保存，请在原球队查看", "success");
            return;
        }
        c.ports.closeModal();
        await c.ports.refreshTeam();
        if (!c.active || origin !== c.ports.teamDetail?.team.id || page !== c.ports.navigation.page)
            return;
        c.ports.render();
        c.ports.toast(presetId ? "阵容预设已更新" : "阵容预设已保存", "success");
    }
    function openDuplicateLineupPreset(presetId: string, sourceName: string): void {
        if (!c.ports.teamDetail || !c.ports.teamPresets.some(item => item.id === presetId))
            throw new Error("请重新选择预设");
        duplicateIdentity = { teamId: c.ports.teamDetail.team.id, presetId, page: c.ports.navigation.page };
        const body = `<label class="field"><span>新预设名称</span><input id="duplicate-lineup-preset-name" value="${escapeHtml(`${sourceName} - 副本`)}"></label>`;
        const footer = `<button class="secondary" data-action="close-workspace-detail">取消</button><button class="primary" data-action="confirm-duplicate-lineup-preset" data-preset-id="${escapeHtml(presetId)}">创建副本</button>`;
        c.ports.showHtmlModal("复制阵容预设", sourceName, body, footer, "duplicate-lineup-preset-modal");
    }
    async function duplicateLineupPreset(presetId: string): Promise<void> {
        const ticket = duplicateIdentity;
        if (!ticket || ticket.teamId !== c.ports.teamDetail?.team.id || ticket.presetId !== presetId || ticket.page !== c.ports.navigation.page)
            throw new Error("复制对象已变化，请重新打开");
        const name = c.ports.value("duplicate-lineup-preset-name").trim();
        if (!name)
            throw new Error("请输入新预设名称");
        duplicateIdentity = null;
        await c.ports.runBusy(() => c.ports.api.duplicateTeamLineupPreset(presetId, name));
        if (!c.active || ticket.teamId !== c.ports.teamDetail?.team.id || ticket.page !== c.ports.navigation.page)
            return;
        c.ports.closeModal();
        await c.ports.refreshTeam();
        if (!c.active || ticket.teamId !== c.ports.teamDetail?.team.id || ticket.page !== c.ports.navigation.page)
            return;
        c.ports.render();
        c.ports.toast("阵容预设副本已创建", "success");
    }
    function requestArchiveLineupPreset(presetId: string, name: string): void {
        const teamId = c.ports.teamDetail?.team.id, page = c.ports.navigation.page;
        c.ports.showConfirmation("归档阵容预设", "归档后不会出现在比赛快速套用列表中，历史数据仍保留。", [["预设", name]], "确认归档", async () => {
            if (!c.active || !teamId || teamId !== c.ports.teamDetail?.team.id || page !== c.ports.navigation.page)
                throw new Error("归档对象已变化，请重新确认");
            await c.ports.runBusy(() => c.ports.api.archiveTeamLineupPreset(presetId));
            if (!c.active || teamId !== c.ports.teamDetail?.team.id || page !== c.ports.navigation.page)
                return;
            await c.ports.refreshTeam();
            if (!c.active || teamId !== c.ports.teamDetail?.team.id || page !== c.ports.navigation.page)
                return;
            c.ports.render();
            c.ports.toast("阵容预设已归档", "success");
        });
    }
    function openSaveCurrentLineupPreset(side: LineupSide): void {
        c.actions.capturePairedLineupFromDom();
        const current = c.actions.pairedSide(side);
        const starterCount = current.players.filter((player) => player.is_starter).length;
        if (starterCount !== 11)
            throw new Error(`当前阵容必须恰好有 11 名首发，当前为 ${starterCount} 名`);
        quickIdentity = { side, identity: JSON.stringify(c.state.pairedLineupBuilder), page: c.ports.navigation.page };
        const body = `<div class="form-grid two-column clean-form">
    <label class="field"><span>预设名称</span><input id="quick-lineup-preset-name" value="${escapeHtml(`${current.team_name} 常用阵容`)}"></label>
    <label class="field"><span>适用场景</span><select id="quick-lineup-preset-context">${["general", "league", "cup", "attacking", "defensive", "rotation", "temporary"].map((context) => `<option value="${context}">${c.actions.presetContextLabel(context)}</option>`).join("")}</select></label>
    <label class="field"><span>使用概率（0–1）</span><input id="quick-lineup-preset-probability" type="number" min="0" max="1" step="0.01"></label>
    <label class="check-row"><input id="quick-lineup-preset-default" type="checkbox"><span><strong>设为默认方案</strong><small>替换该球队现有默认预设</small></span></label>
    <label class="field span-2"><span>备注</span><textarea id="quick-lineup-preset-notes" rows="2">从 ${c.actions.selectedManagedMatch()?.home_team_name ?? "比赛"} vs ${c.actions.selectedManagedMatch()?.away_team_name ?? ""} 的本次阵容保存</textarea></label>
  </div>`;
        const footer = `<button class="secondary" data-action="close-workspace-detail">取消</button><button class="primary" data-action="confirm-save-current-lineup-preset" data-lineup-side="${side}">保存预设</button>`;
        c.ports.showHtmlModal("保存当前阵容为预设", current.team_name, body, footer, "quick-lineup-preset-modal");
        queueMicrotask(() => enhanceSearchableSelects(document.querySelector("#modal-root") ?? document));
    }
    async function saveCurrentLineupAsPreset(side: LineupSide): Promise<void> {
        c.actions.capturePairedLineupFromDom();
        const ticket = quickIdentity;
        if (!ticket || ticket.side !== side || ticket.identity !== JSON.stringify(c.state.pairedLineupBuilder) || ticket.page !== c.ports.navigation.page)
            throw new Error("当前阵容已变化，请重新打开保存预设");
        quickIdentity = null;
        c.actions.capturePairedLineupFromDom();
        const current = c.actions.pairedSide(side);
        const draft: TeamLineupPresetDraft = {
            id: null,
            team_id: current.team_id,
            name: c.ports.value("quick-lineup-preset-name").trim(),
            formation_id: current.formation_id || null,
            coach_id: current.coach_id || null,
            usage_context: c.ports.value("quick-lineup-preset-context"),
            usage_probability: c.ports.nullableNumber("quick-lineup-preset-probability"),
            is_default: c.ports.checked("quick-lineup-preset-default"),
            source_lineup_id: null,
            notes: c.ports.nullableValue("quick-lineup-preset-notes"),
            members: current.players.map(c.actions.presetMemberDraftFromBuilder),
        };
        await c.ports.runBusy(() => c.ports.api.saveTeamLineupPreset(draft));
        const presets = await c.ports.api.listTeamLineupPresets(current.team_id, false);
        if (!c.active || ticket.identity !== JSON.stringify(c.state.pairedLineupBuilder) || ticket.page !== c.ports.navigation.page)
            return;
        c.state.pairedLineupPresets = { ...c.state.pairedLineupPresets, [side]: presets };
        c.ports.closeModal();
        c.ports.render({ preserveForm: true });
        c.ports.toast("当前阵容已保存为球队预设", "success");
    }
    return { lineupPresetManagerBody, openTeamLineupPresetManager, openLineupPresetManagerForSide, refreshTeamLineupPresetCaches, requestDeleteTeamLineupPreset, saveTeamLineupPreset, openDuplicateLineupPreset, duplicateLineupPreset, requestArchiveLineupPreset, openSaveCurrentLineupPreset, saveCurrentLineupAsPreset };
}
