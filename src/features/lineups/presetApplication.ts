import { escapeHtml } from "../../components/format";
import type { LineupBuilderPlayer, TeamLineupPresetApplicationPreview } from "../../types";
import type { LineupContext, LineupSide } from "./lineupContext";
export function createPresetApplication(c: LineupContext) {
    let ticket: {
        side: LineupSide;
        presetId: string;
        identity: string;
        page: string;
        version: number;
    } | null = null;
    let previewRequest = 0;
    const identity = () => JSON.stringify(c.state.pairedLineupBuilder);
    async function previewApplyLineupPreset(side: LineupSide): Promise<void> {
        c.actions.capturePairedLineupFromDom();
        const presetId = c.ports.nullableValue(`paired-${side}-preset`);
        if (!presetId)
            throw new Error("请先选择阵容预设");
        ticket = null;
        const request = ++previewRequest, origin = identity(), page = c.ports.navigation.page;
        const preview = await c.ports.runBusy(() => c.ports.api.previewTeamLineupPresetApplication(presetId));
        if (!c.active || request !== previewRequest || page !== c.ports.navigation.page || origin !== identity())
            return;
        const current = c.actions.pairedSide(side);
        if (preview.preset.team_id !== current.team_id)
            throw new Error("预设与当前球队不一致");
        ticket = { side, presetId, identity: origin, page, version: preview.preset.version };
        const overwriteWarning = current.players.length
            ? `<div class="blocking-note warning">当前已编辑 ${current.players.length} 名球员，套用后将替换这一侧的当前草稿。</div>`
            : "";
        const body = `<div class="lineup-preset-preview">
    <div class="lineup-preset-preview-summary"><div><span>预设</span><strong>${escapeHtml(preview.preset.name)}</strong></div><div><span>阵型</span><strong>${escapeHtml(preview.preset.formation_code ?? "未设置")}</strong></div><div><span>人员</span><strong>首发 ${preview.preset.starter_count} · 共 ${preview.preset.member_count} 人</strong></div></div>
    ${overwriteWarning}
    <div class="lineup-preset-issues">
      ${preview.blockers.map((item) => `<div class="lineup-preset-issue blocker">${escapeHtml(item)}</div>`).join("")}
      ${preview.warnings.map((item) => `<div class="lineup-preset-issue warning">${escapeHtml(item)}</div>`).join("")}
    </div>
    ${preview.blockers.length === 0 && preview.warnings.length === 0 ? `<div class="success-note">当前预设成员均仍属于该球队，没有明确伤停阻断。</div>` : ""}
    <div class="lineup-preset-preview-members">${preview.preset.members.map((member, index) => `<article class="lineup-preset-preview-member"><span>${index + 1}</span><div><strong>${escapeHtml(member.player_name)}${member.alternate_name ? `（${escapeHtml(member.alternate_name)}）` : ""}</strong><small>${member.is_starter ? "首发" : "替补"} · ${escapeHtml(member.position_code ?? "位置待补")} · ${escapeHtml(member.role_code ?? "角色待补")} · ${member.role_origin === "player_position_default" ? `资料继承${member.role_source_position_code ? `（${escapeHtml(member.role_source_position_code)}）` : ""}` : member.role_origin === "lineup_override" ? "预设覆盖" : "角色缺失"}</small></div><b>${escapeHtml(member.availability_status ?? "unknown")}</b></article>`).join("")}</div>
  </div>`;
        const footer = `<button class="secondary" data-action="close-workspace-detail">取消</button><button class="primary" data-action="confirm-apply-lineup-preset" data-lineup-side="${side}" data-preset-id="${escapeHtml(presetId)}" ${preview.can_apply ? "" : "disabled"}>确认套用</button>`;
        c.ports.showHtmlModal("阵容预设套用预检", current.team_name, body, footer, "lineup-preset-preview-modal");
    }
    async function applyLineupPreset(side: LineupSide, presetId: string): Promise<void> {
        c.actions.capturePairedLineupFromDom();
        const approved = ticket;
        if (!c.active || !approved || approved.side !== side || approved.presetId !== presetId || approved.identity !== identity() || approved.page !== c.ports.navigation.page)
            throw new Error("套用对象或草稿已变化，请重新预览");
        ticket = null;
        const preview: TeamLineupPresetApplicationPreview = await c.ports.runBusy(() => c.ports.api.previewTeamLineupPresetApplication(presetId));
        c.actions.capturePairedLineupFromDom();
        if (!c.active || approved.identity !== identity() || approved.page !== c.ports.navigation.page || approved.version !== preview.preset.version)
            throw new Error("预设或套用对象已变化，请重新预览");
        if (!preview.can_apply)
            throw new Error(preview.blockers.join("；") || "阵容预设当前不可用");
        const current = c.actions.pairedSide(side);
        if (preview.preset.team_id !== current.team_id)
            throw new Error("阵容预设与当前球队不一致");
        const players: LineupBuilderPlayer[] = preview.preset.members.map((member) => ({
            player_id: member.player_id,
            player_name: member.player_name,
            player_secondary_name: member.alternate_name,
            position_code: member.position_code,
            role_code: member.role_code,
            is_starter: member.is_starter,
            expected_minutes: member.expected_minutes ?? (member.is_starter ? 90 : 20),
            shirt_number: member.shirt_number,
            bench_order: member.is_starter ? null : member.bench_order,
            starting_probability: member.is_starter ? 1 : 0,
            membership_override: false,
            availability_status: member.availability_status,
        }));
        c.state.pairedLineupBuilder = {
            ...c.state.pairedLineupBuilder,
            [side]: {
                ...current,
                formation_id: preview.preset.formation_id ?? current.formation_id,
                formation: preview.preset.formation_code ?? current.formation,
                coach_id: preview.preset.coach_id ?? current.coach_id,
                players,
            },
        };
        c.ports.closeModal();
        c.ports.render();
        c.ports.toast(`已套用“${preview.preset.name}”，本场阵容可继续独立调整`, "success");
    }
    return { previewApplyLineupPreset, applyLineupPreset };
}
