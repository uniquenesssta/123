import { displayPlayerName } from "../../components/footballText";
import type { LineupBuilderFormState, LineupDraft, LineupPlayerDraft } from "../../types";
import type { LineupContext } from "./lineupContext";
export function createLegacyLineup(c: LineupContext) {
    interface CaptureLineupFormOptions {
        readonly allowBlankIdentity?: boolean;
    }
    function captureLineupFormFromDom(options: CaptureLineupFormOptions = {}): void {
        const match = document.querySelector<HTMLSelectElement>("#new-lineup-match");
        const team = document.querySelector<HTMLSelectElement>("#new-lineup-team");
        const type = document.querySelector<HTMLSelectElement>("#new-lineup-type");
        const snapshot = document.querySelector<HTMLSelectElement>("#new-lineup-snapshot");
        const coach = document.querySelector<HTMLSelectElement | HTMLInputElement>("#new-lineup-coach");
        const sourceUrls = document.querySelector<HTMLInputElement>("#new-lineup-source-urls");
        const formation = document.querySelector<HTMLInputElement>("#new-lineup-formation");
        const formationSelect = document.querySelector<HTMLSelectElement>("#new-lineup-formation-id");
        const captured = document.querySelector<HTMLInputElement>("#new-lineup-captured-at");
        const quality = document.querySelector<HTMLInputElement>("#new-lineup-quality");
        if (!match || !team || !type || !snapshot || !formation || !captured || !quality)
            return;
        const selectedFormation = formationSelect?.selectedOptions[0];
        const formationCode = selectedFormation?.dataset.code ?? formation.value;
        const preserveIdentity = options.allowBlankIdentity !== true;
        const matchId = match.value || (preserveIdentity ? c.state.lineupBuilderForm.match_id : "");
        const teamId = team.value || (preserveIdentity ? c.state.lineupBuilderForm.team_id : "");
        c.state.lineupBuilderForm = {
            match_id: matchId,
            team_id: teamId,
            lineup_type: type.value as LineupBuilderFormState["lineup_type"],
            snapshot_type: snapshot.value as LineupBuilderFormState["snapshot_type"],
            formation_id: formationSelect?.value ?? "",
            formation: formationCode,
            coach_id: coach?.value ?? "",
            source_urls: sourceUrls?.value ?? "",
            captured_at: captured.value,
            quality_score: Number(quality.value || 0.8),
        };
    }
    function captureLineupBuilderFromDom(): void {
        const rows = Array.from(document.querySelectorAll<HTMLElement>("[data-lineup-builder-row]:not([data-lineup-side])"));
        if (rows.length === 0)
            return;
        c.state.lineupBuilderPlayers = rows.map((row) => {
            const playerId = row.dataset.playerId ?? "";
            const existing = c.state.lineupBuilderPlayers.find((item) => item.player_id === playerId);
            if (!existing)
                throw new Error("阵容球员状态丢失，请重新加载球队名单");
            const read = (field: string): string => row.querySelector<HTMLInputElement | HTMLSelectElement>(`[data-lineup-field="${field}"]`)?.value ?? "";
            const minutes = read("expected_minutes").trim();
            const shirt = read("shirt_number").trim();
            const benchOrder = read("bench_order").trim();
            const startingProbability = read("starting_probability").trim();
            const membershipOverride = row.querySelector<HTMLInputElement>('[data-lineup-field="membership_override"]')?.checked ?? false;
            const positionCode = read("position_code") || null;
            const currentRole = read("role_code") || null;
            const candidate = c.state.lineupPlayerCandidates.find((item) => item.id === playerId);
            const previousDefaultRole = candidate
                ? c.actions.defaultRoleForPlayer(candidate, existing.position_code)
                : null;
            const nextDefaultRole = candidate
                ? c.actions.defaultRoleForPlayer(candidate, positionCode)
                : null;
            const roleCode = !currentRole || currentRole === previousDefaultRole
                ? nextDefaultRole
                : currentRole;
            return {
                ...existing,
                is_starter: read("is_starter") === "true",
                position_code: positionCode,
                role_code: roleCode,
                expected_minutes: minutes ? Number(minutes) : null,
                shirt_number: shirt ? Number(shirt) : null,
                bench_order: benchOrder ? Number(benchOrder) : null,
                starting_probability: startingProbability ? Number(startingProbability) : null,
                membership_override: membershipOverride,
            };
        });
    }
    async function loadLineupPlayersForTeam(teamId: string): Promise<number> {
        const matchId = c.state.lineupBuilderForm.match_id, page = c.ports.navigation.page;
        const requestSequence = c.state.lineupPlayerLoadSequence.next();
        const result = await c.ports.api.listPlayers({
            search: null,
            team_id: teamId,
            position_code: null,
            availability_status: null,
            player_status: "active",
            limit: 200,
            cursor_name: null,
            cursor_id: null,
        });
        if (!c.active || matchId !== c.state.lineupBuilderForm.match_id || page !== c.ports.navigation.page || !c.state.lineupPlayerLoadSequence.isCurrent(requestSequence) ||
            c.state.lineupBuilderForm.team_id !== teamId) {
            return 0;
        }
        c.state.lineupPlayerCandidates = result.items;
        c.state.lineupBuilderPlayers = c.state.lineupBuilderPlayers.filter((item) => result.items.some((candidate) => candidate.id === item.player_id));
        return result.items.length;
    }
    async function autoLoadLineupPlayers(teamId: string): Promise<void> {
        const matchId = c.state.lineupBuilderForm.match_id, page = c.ports.navigation.page;
        const teamName = c.ports.references?.teams.find((team) => team.id === teamId)?.canonical_name ??
            "所选球队";
        const traceId = c.ports.recordUiOperation("ui_action", "action:auto-load-lineup-players", { page: c.ports.navigation.page, teamId, teamName });
        try {
            const count = await c.ports.runBusy(() => loadLineupPlayersForTeam(teamId));
            if (!c.active || matchId !== c.state.lineupBuilderForm.match_id || page !== c.ports.navigation.page || c.state.lineupBuilderForm.team_id !== teamId)
                return;
            c.ports.render();
            c.ports.toast(count > 0
                ? `${teamName}已自动加载 ${count} 名球员`
                : `${teamName}已完成查询，但没有找到有效球员`, count > 0 ? "success" : "normal");
        }
        catch (error) {
            c.ports.recordUiOperationFailure("action:auto-load-lineup-players", error, traceId, { page: c.ports.navigation.page, teamId, teamName });
            c.ports.recordClientIssue(error, `${c.ports.pageTitle(c.ports.navigation.page)} / 自动加载球队名单`);
            c.ports.toast(c.ports.userFacingError(error), "error");
        }
    }
    async function loadLineupPlayers(): Promise<void> {
        captureLineupFormFromDom();
        const teamId = c.state.lineupBuilderForm.team_id || null;
        if (!teamId)
            throw new Error("请先选择球队");
        captureLineupBuilderFromDom();
        const matchId = c.state.lineupBuilderForm.match_id, page = c.ports.navigation.page;
        const count = await c.ports.runBusy(() => loadLineupPlayersForTeam(teamId));
        if (!c.active || matchId !== c.state.lineupBuilderForm.match_id || page !== c.ports.navigation.page || teamId !== c.state.lineupBuilderForm.team_id)
            return;
        c.ports.render();
        c.ports.toast(`已加载 ${count} 名球员`, "success");
    }
    function addLineupPlayer(playerId: string, starter: boolean): void {
        captureLineupFormFromDom();
        captureLineupBuilderFromDom();
        if (c.state.lineupBuilderPlayers.some((item) => item.player_id === playerId))
            return;
        const player = c.state.lineupPlayerCandidates.find((item) => item.id === playerId);
        if (!player)
            throw new Error("球员不在当前球队名单中");
        if (starter &&
            c.state.lineupBuilderPlayers.filter((item) => item.is_starter).length >= 11) {
            throw new Error("首发人数不能超过 11 人");
        }
        const playerName = displayPlayerName(player);
        c.state.lineupBuilderPlayers.push({
            player_id: player.id,
            player_name: playerName.primary,
            player_secondary_name: playerName.secondary,
            position_code: player.primary_position_code,
            role_code: c.actions.defaultRoleForPlayer(player, player.primary_position_code),
            is_starter: starter,
            expected_minutes: starter ? 90 : 20,
            shirt_number: null,
            bench_order: starter ? null : c.state.lineupBuilderPlayers.filter((item) => !item.is_starter).length + 1,
            starting_probability: starter ? 1 : 0,
            membership_override: false,
            availability_status: player.availability_status,
        });
        c.ports.render();
    }
    function removeLineupPlayer(playerId: string): void {
        captureLineupFormFromDom();
        captureLineupBuilderFromDom();
        c.state.lineupBuilderPlayers = c.state.lineupBuilderPlayers.filter((item) => item.player_id !== playerId);
        c.ports.render();
    }
    let chainRequest = 0;
    async function inspectLineupChain(): Promise<void> {
        captureLineupFormFromDom();
        const { match_id, snapshot_type } = c.state.lineupBuilderForm, page = c.ports.navigation.page, request = ++chainRequest;
        if (!match_id)
            throw new Error("请选择比赛");
        const chain = await c.ports.runBusy(() => c.ports.api.readMatchLineupChain(match_id, snapshot_type));
        if (!c.active || request !== chainRequest || page !== c.ports.navigation.page || match_id !== c.state.lineupBuilderForm.match_id || snapshot_type !== c.state.lineupBuilderForm.snapshot_type)
            return;
        if (chain.match_record.id !== match_id || chain.snapshot_type !== snapshot_type)
            throw new Error("链路检查对象不匹配，请重新加载");
        c.state.selectedMatchLineupChain = chain;
        c.ports.render();
    }
    let savingSingle = false;
    async function createLineup(): Promise<void> {
        if (!c.active)
            throw new Error("阵容工作区已关闭");
        if (savingSingle)
            return;
        captureLineupFormFromDom();
        captureLineupBuilderFromDom();
        const matchId = c.state.lineupBuilderForm.match_id || null;
        const teamId = c.state.lineupBuilderForm.team_id || null;
        if (!matchId || !teamId)
            throw new Error("请选择比赛和参赛队");
        if (c.state.lineupBuilderPlayers.length === 0)
            throw new Error("请先从球队名单加入球员");
        const starterCount = c.state.lineupBuilderPlayers.filter((item) => item.is_starter).length;
        if (starterCount !== 11)
            throw new Error(`正式阵容必须恰好 11 名首发，当前为 ${starterCount} 名`);
        const capturedAt = c.ports.localDateTimeToIso(c.state.lineupBuilderForm.captured_at || null, c.state.lineupBuilderForm.snapshot_type === "T-N");
        if (!capturedAt) {
            throw new Error("固定时点阵容必须填写不晚于模型截止时间的历史记录时间");
        }
        const activeChain = c.state.selectedMatchLineupChain?.match_record.id === matchId &&
            c.state.selectedMatchLineupChain.snapshot_type === c.state.lineupBuilderForm.snapshot_type
            ? c.state.selectedMatchLineupChain : null;
        if (activeChain &&
            new Date(capturedAt).getTime() >
                new Date(activeChain.data_cutoff_time).getTime()) {
            throw new Error(`${c.state.lineupBuilderForm.snapshot_type} 阵容记录时间不能晚于模型截止时间 ${new Date(activeChain.data_cutoff_time).toLocaleString()}`);
        }
        const selectedMatch = c.ports.references?.managed_matches.find((match) => match.id === matchId) ??
            c.ports.references?.upcoming_matches.find((match) => match.id === matchId);
        if (selectedMatch &&
            c.state.lineupBuilderForm.lineup_type !== "actual" &&
            new Date(capturedAt).getTime() >=
                new Date(selectedMatch.kickoff_time).getTime()) {
            throw new Error("预计或确认阵容的记录时间必须早于开球时间");
        }
        const players: LineupPlayerDraft[] = c.state.lineupBuilderPlayers.map((item, index) => ({
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
        const draft: LineupDraft = {
            match_id: matchId,
            team_id: teamId,
            lineup_type: c.state.lineupBuilderForm.lineup_type,
            snapshot_type: c.state.lineupBuilderForm.snapshot_type,
            formation: c.state.lineupBuilderForm.formation.trim() || null,
            formation_id: c.state.lineupBuilderForm.formation_id || null,
            coach_id: c.state.lineupBuilderForm.coach_id || null,
            captured_at: capturedAt,
            source_document_id: null,
            source_urls: c.state.lineupBuilderForm.source_urls.split(/[;\n,]/).map((item) => item.trim()).filter(Boolean),
            quality_score: c.state.lineupBuilderForm.quality_score,
            metadata: { entry_mode: "visual_builder" },
            players,
        };
        const snapshotType = c.state.lineupBuilderForm.snapshot_type;
        const page = c.ports.navigation.page;
        const identity = () => JSON.stringify([c.state.lineupBuilderForm, c.state.lineupBuilderPlayers]);
        const submittedIdentity = identity();
        const current = () => { captureLineupFormFromDom(); captureLineupBuilderFromDom(); return c.active && page === c.ports.navigation.page && submittedIdentity === identity(); };
        savingSingle = true;
        try {
            const created = await c.ports.runBusy(() => c.ports.api.createLineup(draft));
            if (!current()) {
                c.ports.toast("原比赛阵容已保存，请在原比赛查看历史", "success");
                return;
            }
            const records = await c.ports.api.listLineups(null, 100);
            if (!current())
                return;
            const readback = await c.ports.api.readMatchLineupChain(matchId, snapshotType);
            if (!current())
                return;
            if (readback.match_record.id !== matchId || readback.snapshot_type !== snapshotType)
                throw new Error("阵容已保存，但读回链路对象不匹配");
            c.state.lineupRecords = records;
            c.state.selectedMatchLineupChain = readback;
            c.ports.lastPredictionResult = created;
            c.state.lineupBuilderPlayers = [];
            c.state.lineupPlayerCandidates = [];
            if (c.ports.navigation.page === "review" && c.ports.selectedReviewMatchId) {
                const reviewMatch = c.ports.selectedReviewMatchId;
                await c.ports.loadReviewLineups(reviewMatch);
                if (!c.active || page !== c.ports.navigation.page || reviewMatch !== c.ports.selectedReviewMatchId || matchId !== c.state.lineupBuilderForm.match_id)
                    return;
                c.state.lineupBuilderForm = {
                    ...c.state.lineupBuilderForm,
                    match_id: c.ports.selectedReviewMatchId,
                    team_id: "",
                    lineup_type: "actual",
                };
                c.ports.render();
                c.ports.toast("阵容已保存，可继续补录另一队或生成复盘", "success");
                return;
            }
            const chain = c.state.selectedMatchLineupChain;
            if (!chain)
                throw new Error("阵容已保存，但未能读取模型链路");
            if (chain.ready_for_model) {
                c.ports.workspaceState.patchModule("lineups", { active_section: "chain" });
                c.ports.render();
                c.ports.toast("双方阵容已保存并通过门禁，可以返回正式推演", "success");
                return;
            }
            const target = c.ports.missingLineupTeam(chain);
            c.state.lineupBuilderForm = {
                ...c.state.lineupBuilderForm,
                match_id: matchId,
                team_id: target.team_id,
                lineup_type: "expected",
                snapshot_type: snapshotType,
                captured_at: c.ports.localDateTimeInputValue(chain.data_cutoff_time),
            };
            const nextIdentity = identity();
            const count = await c.ports.runBusy(() => loadLineupPlayersForTeam(target.team_id));
            if (!c.active || page !== c.ports.navigation.page || nextIdentity !== identity())
                return;
            c.ports.workspaceState.patchModule("lineups", {
                active_section: "builder",
                controls: {},
            });
            c.ports.render();
            c.ports.toast(`当前阵容已保存；继续补齐${target.team_side === "home" ? "主队" : "客队"}${target.team_name}（已加载 ${count} 名球员）`, "success");
        }
        finally {
            savingSingle = false;
        }
    }
    return { captureLineupFormFromDom, captureLineupBuilderFromDom, loadLineupPlayersForTeam, autoLoadLineupPlayers, loadLineupPlayers, addLineupPlayer, removeLineupPlayer, inspectLineupChain, createLineup };
}
