import { displayPlayerName } from "../../components/footballText";
import type { LineupBuilderPlayer, LineupSnapshotType, MatchRecord, PairedLineupBuilderState, PlayerListItem, TeamLineupPresetMemberDraft } from "../../types";
import type { LineupContext, LineupSide } from "./lineupContext";
import { emptyPairedLineupSide } from "./lineupState";
export function createPairedBuilder(c: LineupContext) {
    function pairedSide(side: LineupSide): PairedLineupBuilderState[LineupSide] {
        return c.state.pairedLineupBuilder[side];
    }
    function selectedManagedMatch(matchId = c.state.pairedLineupBuilder.match_id): MatchRecord | null {
        return (c.ports.references?.managed_matches.find((match) => match.id === matchId) ??
            c.ports.references?.upcoming_matches.find((match) => match.id === matchId) ??
            null);
    }
    function preferredCoachForTeam(teamId: string): string {
        const candidates = c.ports.coachList.filter((coach) => coach.current_team_id === teamId);
        return (candidates.find((coach) => coach.current_role === "head_coach") ??
            candidates.find((coach) => coach.current_role === "interim_head_coach") ??
            candidates.find((coach) => coach.current_role === "caretaker") ??
            candidates[0])?.id ?? "";
    }
    function resetPairedBuilderForMatch(matchId: string, preserveCommon = true): void {
        const match = selectedManagedMatch(matchId);
        if (!match)
            throw new Error("比赛不存在或尚未加载");
        const now = Date.now();
        const kickoff = new Date(match.kickoff_time).getTime();
        const defaultCaptured = c.ports.localDateTimeInputValue(new Date(Math.min(now, kickoff - 1000)).toISOString());
        c.state.pairedLineupLoadSequence.home.invalidate();
        c.state.pairedLineupLoadSequence.away.invalidate();
        c.state.pairedLineupBuilder = {
            match_id: match.id,
            lineup_type: preserveCommon ? c.state.pairedLineupBuilder.lineup_type : "expected",
            snapshot_type: preserveCommon ? c.state.pairedLineupBuilder.snapshot_type : "T-N",
            captured_at: preserveCommon && c.state.pairedLineupBuilder.captured_at
                ? c.state.pairedLineupBuilder.captured_at
                : defaultCaptured,
            source_urls: preserveCommon ? c.state.pairedLineupBuilder.source_urls : "",
            home: {
                ...emptyPairedLineupSide(),
                team_id: match.home_team_id,
                team_name: match.home_team_name,
                coach_id: preferredCoachForTeam(match.home_team_id),
            },
            away: {
                ...emptyPairedLineupSide(),
                team_id: match.away_team_id,
                team_name: match.away_team_name,
                coach_id: preferredCoachForTeam(match.away_team_id),
            },
        };
        c.state.pairedLineupPresets = { home: [], away: [] };
        c.state.selectedManagedMatchId = match.id;
        c.state.selectedMatchLineupChain = null;
    }
    function capturePairedSideFromDom(side: LineupSide): void {
        const current = pairedSide(side);
        const formationSelect = document.querySelector<HTMLSelectElement>(`#paired-${side}-formation-id`);
        const formationInput = document.querySelector<HTMLInputElement>(`#paired-${side}-formation`);
        const coach = document.querySelector<HTMLSelectElement>(`#paired-${side}-coach`);
        const quality = document.querySelector<HTMLInputElement>(`#paired-${side}-quality`);
        const rows = Array.from(document.querySelectorAll<HTMLElement>(`[data-lineup-builder-row][data-lineup-side="${side}"]`));
        const players = rows.length === 0
            ? current.players
            : rows.map((row) => {
                const playerId = row.dataset.playerId ?? "";
                const existing = current.players.find((item) => item.player_id === playerId);
                if (!existing)
                    throw new Error(`${current.team_name || "球队"}阵容状态丢失，请重新加载名单`);
                const read = (field: string): string => row.querySelector<HTMLInputElement | HTMLSelectElement>(`[data-lineup-field="${field}"]`)?.value ?? "";
                const minutes = read("expected_minutes").trim();
                const shirt = read("shirt_number").trim();
                const benchOrder = read("bench_order").trim();
                const startingProbability = read("starting_probability").trim();
                const membershipOverride = row.querySelector<HTMLInputElement>('[data-lineup-field="membership_override"]')?.checked ?? false;
                const positionCode = read("position_code") || null;
                const currentRole = read("role_code") || null;
                const candidate = current.candidates.find((item) => item.id === playerId);
                const previousDefaultRole = candidate
                    ? defaultRoleForPlayer(candidate, existing.position_code)
                    : null;
                const nextDefaultRole = candidate
                    ? defaultRoleForPlayer(candidate, positionCode)
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
        const selectedFormation = formationSelect?.selectedOptions[0];
        c.state.pairedLineupBuilder = {
            ...c.state.pairedLineupBuilder,
            [side]: {
                ...current,
                formation_id: formationSelect?.value ?? current.formation_id,
                formation: selectedFormation?.dataset.code ?? formationInput?.value ?? current.formation,
                coach_id: coach?.value ?? current.coach_id,
                quality_score: Number(quality?.value || current.quality_score),
                players,
            },
        };
    }
    function capturePairedLineupFromDom(): void {
        const match = document.querySelector<HTMLSelectElement>("#paired-lineup-match");
        if (!match)
            return;
        const lineupType = document.querySelector<HTMLSelectElement>("#paired-lineup-type");
        const snapshot = document.querySelector<HTMLSelectElement>("#paired-lineup-snapshot");
        const captured = document.querySelector<HTMLInputElement>("#paired-lineup-captured-at");
        const sourceUrls = document.querySelector<HTMLInputElement>("#paired-lineup-source-urls");
        c.state.pairedLineupBuilder = {
            ...c.state.pairedLineupBuilder,
            match_id: match.value || c.state.pairedLineupBuilder.match_id,
            lineup_type: (lineupType?.value ?? c.state.pairedLineupBuilder.lineup_type) as PairedLineupBuilderState["lineup_type"],
            snapshot_type: (snapshot?.value ?? c.state.pairedLineupBuilder.snapshot_type) as LineupSnapshotType,
            captured_at: captured?.value ?? c.state.pairedLineupBuilder.captured_at,
            source_urls: sourceUrls?.value ?? c.state.pairedLineupBuilder.source_urls,
        };
        capturePairedSideFromDom("home");
        capturePairedSideFromDom("away");
    }
    async function loadPairedLineupSide(side: LineupSide): Promise<number> {
        const current = pairedSide(side);
        if (!current.team_id)
            throw new Error(`${side === "home" ? "主队" : "客队"}尚未确定`);
        const requestSequence = c.state.pairedLineupLoadSequence[side].next();
        const teamId = current.team_id;
        const matchId = c.state.pairedLineupBuilder.match_id;
        const page = c.ports.navigation.page;
        const [result, presets] = await Promise.all([
            c.ports.api.listPlayers({
                search: null,
                team_id: teamId,
                position_code: null,
                availability_status: null,
                player_status: "active",
                limit: 200,
                cursor_name: null,
                cursor_id: null,
            }),
            c.ports.api.listTeamLineupPresets(teamId, false),
        ]);
        if (!c.active || !c.state.pairedLineupLoadSequence[side].isCurrent(requestSequence) ||
            pairedSide(side).team_id !== teamId || c.state.pairedLineupBuilder.match_id !== matchId || page !== c.ports.navigation.page) {
            return 0;
        }
        const selected = pairedSide(side).players.filter((item) => result.items.some((candidate) => candidate.id === item.player_id));
        c.state.pairedLineupPresets = { ...c.state.pairedLineupPresets, [side]: presets };
        c.state.pairedLineupBuilder = {
            ...c.state.pairedLineupBuilder,
            [side]: {
                ...pairedSide(side),
                coach_id: pairedSide(side).coach_id || preferredCoachForTeam(teamId),
                candidates: result.items,
                players: selected,
            },
        };
        return result.items.length;
    }
    async function loadBothPairedLineupSides(): Promise<[
        number,
        number
    ]> {
        if (!c.state.pairedLineupBuilder.match_id)
            throw new Error("请先选择比赛");
        return Promise.all([
            loadPairedLineupSide("home"),
            loadPairedLineupSide("away"),
        ]);
    }
    function defaultRoleForPlayer(player: PlayerListItem, positionCode: string | null): string | null {
        const code = positionCode || player.primary_position_code;
        if (code && player.position_role_map?.[code])
            return player.position_role_map[code];
        return player.primary_role_code;
    }
    function addPairedLineupPlayer(side: LineupSide, playerId: string, starter: boolean, positionCode: string | null = null): void {
        capturePairedLineupFromDom();
        const current = pairedSide(side);
        if (current.players.some((item) => item.player_id === playerId))
            return;
        const player = current.candidates.find((item) => item.id === playerId);
        if (!player)
            throw new Error(`${current.team_name}名单中不存在该球员`);
        if (starter && current.players.filter((item) => item.is_starter).length >= 11) {
            throw new Error(`${current.team_name}首发人数不能超过 11 人`);
        }
        const playerName = displayPlayerName(player);
        const next: LineupBuilderPlayer = {
            player_id: player.id,
            player_name: playerName.primary,
            player_secondary_name: playerName.secondary,
            position_code: positionCode || player.primary_position_code,
            role_code: defaultRoleForPlayer(player, positionCode),
            is_starter: starter,
            expected_minutes: starter ? 90 : 20,
            shirt_number: null,
            bench_order: starter ? null : current.players.filter((item) => !item.is_starter).length + 1,
            starting_probability: starter ? 1 : 0,
            membership_override: false,
            availability_status: player.availability_status,
        };
        c.state.pairedLineupBuilder = {
            ...c.state.pairedLineupBuilder,
            [side]: { ...current, players: [...current.players, next] },
        };
        c.ports.render();
    }
    function addSelectedPairedLineupPlayer(side: LineupSide): void {
        const playerSelect = document.querySelector<HTMLSelectElement>(`#paired-${side}-candidate`);
        const roleSelect = document.querySelector<HTMLSelectElement>(`#paired-${side}-candidate-role`);
        const positionSelect = document.querySelector<HTMLSelectElement>(`#paired-${side}-candidate-position`);
        const playerId = playerSelect?.value ?? "";
        if (!playerId)
            throw new Error("请先选择球员");
        addPairedLineupPlayer(side, playerId, (roleSelect?.value ?? "starter") === "starter", positionSelect?.value || null);
    }
    function presetMemberDraftFromBuilder(player: LineupBuilderPlayer, index: number): TeamLineupPresetMemberDraft {
        return {
            player_id: player.player_id,
            position_code: player.position_code,
            role_code: player.role_code,
            is_starter: player.is_starter,
            shirt_number: player.shirt_number,
            expected_minutes: player.expected_minutes,
            sequence_no: index + 1,
            bench_order: player.is_starter ? null : player.bench_order,
            is_captain: false,
            metadata: {},
        };
    }
    return { pairedSide, selectedManagedMatch, preferredCoachForTeam, resetPairedBuilderForMatch, capturePairedSideFromDom, capturePairedLineupFromDom, loadPairedLineupSide, loadBothPairedLineupSides, defaultRoleForPlayer, addPairedLineupPlayer, addSelectedPairedLineupPlayer, presetMemberDraftFromBuilder };
}
