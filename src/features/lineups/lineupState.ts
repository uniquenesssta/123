import { RequestSequence } from "../../platform/timing";
import type { LineupBuilderFormState, LineupBuilderPlayer, LineupRecord, MatchLineupChain, PairedLineupBuilderState, PlayerListItem, TeamLineupPresetRecord } from "../../types";
import type { LineupSide } from "./lineupContext";
const DEFAULT_FORMATION_ID = "4737da75-7c7b-52f5-acf5-ea9bfa809c48";
const DEFAULT_FORMATION_CODE = "4-2-3-1";
export function emptyPairedLineupSide(): PairedLineupBuilderState[LineupSide] {
    return {
        team_id: "",
        team_name: "",
        formation_id: DEFAULT_FORMATION_ID,
        formation: DEFAULT_FORMATION_CODE,
        coach_id: "",
        quality_score: 0.8,
        players: [],
        candidates: [],
    };
}
export interface LineupState {
    pairedLineupPresets: Record<LineupSide, TeamLineupPresetRecord[]>;
    lineupRecords: LineupRecord[];
    selectedMatchLineupChain: MatchLineupChain | null;
    lineupPlayerCandidates: PlayerListItem[];
    lineupBuilderPlayers: LineupBuilderPlayer[];
    lineupBuilderForm: LineupBuilderFormState;
    pairedLineupBuilder: PairedLineupBuilderState;
    selectedManagedMatchId: string | null;
    lineupPlayerLoadSequence: RequestSequence;
    pairedLineupLoadSequence: Record<LineupSide, RequestSequence>;
}
export function createLineupState(): LineupState {
    let pairedLineupPresets: Record<"home" | "away", TeamLineupPresetRecord[]> = {
        home: [],
        away: [],
    };
    let lineupRecords: LineupRecord[] = [];
    let selectedMatchLineupChain: MatchLineupChain | null = null;
    let lineupPlayerCandidates: PlayerListItem[] = [];
    let lineupBuilderPlayers: LineupBuilderPlayer[] = [];
    let lineupPlayerLoadSequence = new RequestSequence();
    let lineupBuilderForm: LineupBuilderFormState = {
        match_id: "",
        team_id: "",
        lineup_type: "expected",
        snapshot_type: "T-N",
        formation_id: "4737da75-7c7b-52f5-acf5-ea9bfa809c48",
        formation: "4-2-3-1",
        coach_id: "",
        source_urls: "",
        captured_at: "",
        quality_score: 0.8,
    };
    let pairedLineupBuilder: PairedLineupBuilderState = {
        match_id: "",
        lineup_type: "expected",
        snapshot_type: "T-N",
        captured_at: "",
        source_urls: "",
        home: emptyPairedLineupSide(),
        away: emptyPairedLineupSide(),
    };
    let pairedLineupLoadSequence: Record<LineupSide, RequestSequence> = { home: new RequestSequence(), away: new RequestSequence() };
    let selectedManagedMatchId: string | null = null;
    return { pairedLineupPresets, lineupRecords, selectedMatchLineupChain, lineupPlayerCandidates, lineupBuilderPlayers, lineupBuilderForm, pairedLineupBuilder, selectedManagedMatchId, lineupPlayerLoadSequence, pairedLineupLoadSequence };
}
