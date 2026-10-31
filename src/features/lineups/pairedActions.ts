import type { LineupRecord } from "../../types";
import { escapeHtml } from "../../components/format";
import type { LineupBuilderPlayer, LineupDraft, LineupPairDraft, LineupPlayerDraft } from "../../types";
import type { LineupContext, LineupSide } from "./lineupContext";
export function createPairedActions(c: LineupContext) {
    let settingsIdentity: {
        side: LineupSide;
        playerId: string;
        draft: string;
        page: string;
    } | null = null;
    function openPairedLineupPlayerSettings(side: LineupSide, playerId: string): void {
        c.actions.capturePairedLineupFromDom();
        const player = c.actions.pairedSide(side).players.find((item) => item.player_id === playerId);
        if (!player)
            throw new Error("当前阵容中不存在该球员");
        settingsIdentity = { side, playerId, draft: JSON.stringify(c.state.pairedLineupBuilder), page: c.ports.navigation.page };
        const expectedMinutes = player.expected_minutes ?? (player.is_starter ? 90 : 20);
        const startingProbability = player.starting_probability ?? (player.is_starter ? 1 : 0);
        const candidate = c.actions.pairedSide(side).candidates.find((item) => item.id === playerId);
        const inheritedRole = candidate
            ? c.actions.defaultRoleForPlayer(candidate, player.position_code)
            : null;
        const roleSourceNote = inheritedRole && player.role_code === inheritedRole
            ? `当前继承球员位置档案：${inheritedRole}`
            : inheritedRole
                ? `位置档案默认：${inheritedRole}；填写不同内容将作为本场覆盖`
                : "当前位置档案没有默认角色，可在球员中心或此处补充";
        const body = `<div class="form-grid compact lineup-player-settings-form">
    <label class="field"><span>角色与职责</span><input id="lineup-player-role-code" value="${escapeHtml(player.role_code ?? inheritedRole ?? "")}" placeholder="例如：组织核心、单后腰"><small>${escapeHtml(roleSourceNote)}</small></label>
    <label class="field"><span>预计分钟</span><input id="lineup-player-expected-minutes" type="number" min="0" max="130" value="${expectedMinutes}"></label>
    <label class="field"><span>首发概率（0–1）</span><input id="lineup-player-starting-probability" type="number" min="0" max="1" step="0.01" value="${startingProbability}"></label>
    <label class="field"><span>替补顺序</span><input id="lineup-player-bench-order" type="number" min="1" value="${player.bench_order ?? ""}" ${player.is_starter ? "disabled" : ""}></label>
    <label class="field"><span>球衣号码</span><input id="lineup-player-shirt-number" type="number" min="1" max="99" value="${player.shirt_number ?? ""}"></label>
    <label class="check-row"><input id="lineup-player-membership-override" type="checkbox" ${player.membership_override ? "checked" : ""}><span><strong>允许履历例外</strong><small>仅在球员关系资料尚未补齐、但来源已核验时使用。</small></span></label>
  </div>`;
        const footer = `<button type="button" class="secondary" data-action="close-workspace-detail">取消</button><button type="button" class="primary" data-action="save-lineup-player-settings" data-lineup-side="${side}" data-player-id="${escapeHtml(playerId)}">保存球员设置</button>`;
        c.ports.showHtmlModal(player.player_name, side === "home" ? "主队球员高级设置" : "客队球员高级设置", body, footer, "lineup-player-settings-modal");
    }
    function savePairedLineupPlayerSettings(side: LineupSide, playerId: string): void {
        const ticket = settingsIdentity;
        if (!c.active || !ticket || ticket.side !== side || ticket.playerId !== playerId || ticket.draft !== JSON.stringify(c.state.pairedLineupBuilder) || ticket.page !== c.ports.navigation.page)
            throw new Error("球员设置对象已变化，请重新打开");
        const roleCode = c.ports.nullableValue("lineup-player-role-code");
        const expectedMinutes = c.ports.nullableNumber("lineup-player-expected-minutes");
        const startingProbability = c.ports.nullableNumber("lineup-player-starting-probability");
        const benchOrder = c.ports.nullableNumber("lineup-player-bench-order");
        const shirtNumber = c.ports.nullableNumber("lineup-player-shirt-number");
        if (expectedMinutes !== null && (expectedMinutes < 0 || expectedMinutes > 130)) {
            throw new Error("预计分钟必须在 0 到 130 之间");
        }
        if (startingProbability !== null && (startingProbability < 0 || startingProbability > 1)) {
            throw new Error("首发概率必须在 0 到 1 之间");
        }
        if (benchOrder !== null && benchOrder < 1)
            throw new Error("替补顺序必须大于 0");
        if (shirtNumber !== null && (shirtNumber < 1 || shirtNumber > 99)) {
            throw new Error("球衣号码必须在 1 到 99 之间");
        }
        const membershipOverride = document.querySelector<HTMLInputElement>("#lineup-player-membership-override")?.checked ?? false;
        const current = c.actions.pairedSide(side);
        const exists = current.players.some((item) => item.player_id === playerId);
        if (!exists)
            throw new Error("当前阵容中不存在该球员");
        c.state.pairedLineupBuilder = {
            ...c.state.pairedLineupBuilder,
            [side]: {
                ...current,
                players: current.players.map((item) => item.player_id === playerId
                    ? {
                        ...item,
                        role_code: roleCode,
                        expected_minutes: expectedMinutes,
                        starting_probability: startingProbability,
                        bench_order: item.is_starter ? null : benchOrder,
                        shirt_number: shirtNumber,
                        membership_override: membershipOverride,
                    }
                    : item),
            },
        };
        settingsIdentity = null;
        c.ports.closeModal();
        c.ports.render({ preserveForm: true });
        c.ports.toast("球员高级设置已保存", "success");
    }
    function removePairedLineupPlayer(side: LineupSide, playerId: string): void {
        c.actions.capturePairedLineupFromDom();
        const current = c.actions.pairedSide(side);
        c.state.pairedLineupBuilder = {
            ...c.state.pairedLineupBuilder,
            [side]: {
                ...current,
                players: current.players.filter((item) => item.player_id !== playerId),
            },
        };
        c.ports.render();
    }
    function clearPairedLineupSide(side: LineupSide): void {
        c.actions.capturePairedLineupFromDom();
        c.state.pairedLineupBuilder = {
            ...c.state.pairedLineupBuilder,
            [side]: { ...c.actions.pairedSide(side), players: [] },
        };
        c.ports.render();
    }
    function lineupPlayersDraft(players: LineupBuilderPlayer[]): LineupPlayerDraft[] {
        return players.map((item, index) => ({
            player_id: item.player_id,
            position_code: item.position_code,
            role_code: item.role_code,
            is_starter: item.is_starter,
            shirt_number: item.shirt_number,
            expected_minutes: item.expected_minutes,
            actual_minutes: null,
            sequence_no: index + 1,
            bench_order: item.is_starter ? null : item.bench_order,
            availability_status: item.availability_status,
            starting_probability: item.starting_probability,
            membership_override: item.membership_override,
            source_urls: [],
            metadata: {},
        }));
    }
    function buildPairedLineupDraft(side: LineupSide, capturedAt: string): LineupDraft {
        const current = c.actions.pairedSide(side);
        const starterCount = current.players.filter((item) => item.is_starter).length;
        if (starterCount !== 11) {
            throw new Error(`${current.team_name}必须恰好 11 名首发，当前为 ${starterCount} 名`);
        }
        if (current.players.length < 11) {
            throw new Error(`${current.team_name}阵容人数不足 11 人`);
        }
        if (new Set(current.players.map(player => player.player_id)).size !== current.players.length)
            throw new Error("阵容包含重复球员");
        if (current.players.some(player => !player.position_code))
            throw new Error("请补齐球员位置");
        return {
            match_id: c.state.pairedLineupBuilder.match_id,
            team_id: current.team_id,
            lineup_type: c.state.pairedLineupBuilder.lineup_type,
            snapshot_type: c.state.pairedLineupBuilder.snapshot_type,
            formation: current.formation.trim() || null,
            formation_id: current.formation_id || null,
            coach_id: current.coach_id || null,
            captured_at: capturedAt,
            source_document_id: null,
            source_urls: c.state.pairedLineupBuilder.source_urls
                .split(/[;\n,]/)
                .map((item) => item.trim())
                .filter(Boolean),
            quality_score: current.quality_score,
            metadata: { entry_mode: "paired_visual_builder", team_side: side },
            players: lineupPlayersDraft(current.players),
        };
    }
    let savingPair = false;
    let uncertainPair: {
        pair: LineupPairDraft;
        previous: Set<string>;
    } | null = null;
    async function createPairedLineups(): Promise<void> {
        if (!c.active)
            throw new Error("阵容工作区已关闭");
        if (savingPair)
            return;
        if (uncertainPair)
            throw new Error("上次双方提交结果尚未核实，请先检查原比赛模型链路，禁止直接重复提交");
        c.actions.capturePairedLineupFromDom();
        const match = c.actions.selectedManagedMatch();
        if (!match)
            throw new Error("请选择比赛");
        const snapshot = c.state.pairedLineupBuilder.snapshot_type;
        const page = c.ports.navigation.page;
        const capturedAt = c.ports.localDateTimeToIso(c.state.pairedLineupBuilder.captured_at || null, snapshot === "T-N");
        if (!capturedAt)
            throw new Error("请填写双方阵容的记录时间");
        // Freeze both payloads BEFORE the first await; validation and write share one identity.
        const pair: LineupPairDraft = { home: buildPairedLineupDraft("home", capturedAt), away: buildPairedLineupDraft("away", capturedAt) };
        const identity = JSON.stringify(c.state.pairedLineupBuilder);
        const current = () => { c.actions.capturePairedLineupFromDom(); return c.active && page === c.ports.navigation.page && identity === JSON.stringify(c.state.pairedLineupBuilder); };
        savingPair = true;
        try {
            const chain = await c.ports.api.readMatchLineupChain(match.id, snapshot);
            if (!current())
                throw new Error("比赛、窗口或阵容已变化，请重新检查后提交");
            if (chain.match_record.id !== match.id || chain.snapshot_type !== snapshot)
                throw new Error("链路检查对象不匹配，请重新加载");
            const captured = Date.parse(capturedAt), cutoff = Date.parse(chain.data_cutoff_time), start = chain.data_window_start_time ? Date.parse(chain.data_window_start_time) : null;
            if (!Number.isFinite(captured) || !Number.isFinite(cutoff) || (start !== null && !Number.isFinite(start)))
                throw new Error("阵容时间或窗口无效");
            if (captured > cutoff || (start !== null && captured < start))
                throw new Error("记录时间不在所选数据窗口内");
            if (pair.home.lineup_type !== "actual" && captured >= Date.parse(match.kickoff_time))
                throw new Error("预计或确认阵容的记录时间必须早于开球时间");
            const previous = new Set([...(chain.home?.versions ?? []), ...(chain.away?.versions ?? [])].map(item => item.id));
            uncertainPair = { pair, previous };
            const created = await c.ports.runBusy(() => c.ports.api.createLineupPair(pair));
            // The original write succeeded even if its view disappeared. Never update another match.
            if (!current()) {
                c.ports.toast("原比赛双方阵容已提交；当前对象已变化，请查看原比赛历史", "success");
                return;
            }
            const records = await c.ports.api.listLineups(null, 100);
            if (!current())
                return;
            const readback = await c.ports.api.readMatchLineupChain(match.id, snapshot);
            if (!current())
                return;
            if (readback.match_record.id !== match.id || readback.snapshot_type !== snapshot)
                throw new Error("阵容已提交，但读回链路对象不匹配，请重新检查原比赛");
            c.state.lineupRecords = records;
            c.state.selectedMatchLineupChain = readback;
            c.ports.lastPredictionResult = created;
            uncertainPair = null;
            c.ports.workspaceState.patchModule("lineups", { active_section: "chain" });
            c.ports.render({ preserveForm: true });
            c.ports.toast(match.home_team_name + "与" + match.away_team_name + "阵容已同时提交", "success");
        }
        finally {
            savingPair = false;
        }
    }
    async function inspectPairedLineupChain(): Promise<void> {
        c.actions.capturePairedLineupFromDom();
        const { match_id, snapshot_type } = c.state.pairedLineupBuilder;
        if (!match_id)
            throw new Error("请选择比赛");
        const page = c.ports.navigation.page;
        const request = ++chainRequest;
        const chain = await c.ports.runBusy(() => c.ports.api.readMatchLineupChain(match_id, snapshot_type));
        if (!c.active || request !== chainRequest || page !== c.ports.navigation.page || match_id !== c.state.pairedLineupBuilder.match_id || snapshot_type !== c.state.pairedLineupBuilder.snapshot_type)
            return;
        if (uncertainPair && uncertainPair.pair.home.match_id === match_id) {
            const pending = uncertainPair;
            const records = await c.ports.api.listLineups(match_id, 100);
            if (!c.active || request !== chainRequest || page !== c.ports.navigation.page || match_id !== c.state.pairedLineupBuilder.match_id || snapshot_type !== c.state.pairedLineupBuilder.snapshot_type)
                return;
            const samePlayers = (records: LineupRecord["players"], drafts: LineupDraft["players"]) => {
                const key = (player: LineupDraft["players"][number] | LineupRecord["players"][number]) => JSON.stringify([player.player_id, player.is_starter, player.position_code ?? null, player.role_code ?? null, player.shirt_number ?? null, player.expected_minutes ?? null, player.bench_order ?? null, player.starting_probability ?? null, player.membership_override ?? false]);
                return JSON.stringify(records.map(key).sort()) === JSON.stringify(drafts.map(key).sort());
            };
            const same = (record: LineupRecord, draft: LineupDraft) => !pending.previous.has(record.id) && record.match_id === draft.match_id && record.team_id === draft.team_id && record.snapshot_type === draft.snapshot_type && record.lineup_type === draft.lineup_type && Date.parse(record.captured_at) === Date.parse(draft.captured_at) && record.formation_id === draft.formation_id && record.coach_id === draft.coach_id && record.player_count === draft.players.length && samePlayers(record.players, draft.players);
            const home = records.find(record => same(record, pending.pair.home)), away = records.find(record => same(record, pending.pair.away));
            if (home && away) {
                uncertainPair = null;
                c.ports.toast("已读回原比赛双方阵容记录，请核对历史版本", "success");
            }
            else
                c.ports.toast("尚未读回原提交的双方记录，结果仍未核实，请勿重复提交", "error");
        }
        c.state.selectedMatchLineupChain = chain;
        c.ports.workspaceState.patchModule("lineups", { active_section: "chain" });
        c.ports.render({ preserveForm: true });
    }
    let chainRequest = 0;
    return { openPairedLineupPlayerSettings, savePairedLineupPlayerSettings, removePairedLineupPlayer, clearPairedLineupSide, lineupPlayersDraft, buildPairedLineupDraft, createPairedLineups, inspectPairedLineupChain };
}
