import { MatchImports } from "./matchImports";
import { createMatchExchange } from "./matchExchange";
import { createLegacyLineup } from "./legacyLineup";
import type { LineupActions, LineupContext, LineupPorts } from "./lineupContext";
import { createLineupState } from "./lineupState";
import { createMatchEditor } from "./matchEditor";
import { createPairedActions } from "./pairedActions";
import { createPairedBuilder } from "./pairedBuilder";
import { createPresetActions } from "./presetActions";
import { createPresetApplication } from "./presetApplication";
import { createPresetEditor } from "./presetEditor";
import { createPresetPositions } from "./presetPositions";
export function createLineupsFeature(ports: LineupPorts) {
    const c: LineupContext = { active: true, state: createLineupState(), ports, actions: {} as LineupActions };
    const imports = new MatchImports({ api: ports.api, files: ports.files, busy: ports.runBusy, capture: () => {
            const page = ports.navigation.page, section = ports.workspaceState.module(page).active_section, matchId = c.state.pairedLineupBuilder.match_id, mode = ports.value("match-import-mode");
            return () => c.active && page === ports.navigation.page && section === ports.workspaceState.module(page).active_section && matchId === c.state.pairedLineupBuilder.match_id && mode === ports.value("match-import-mode");
        } });
    const exchange = createMatchExchange(c, imports);
    c.actions = {
        ...exchange,
        ...createPairedBuilder(c),
        ...createPresetPositions(c),
        ...createPresetEditor(c),
        ...createPresetActions(c),
        ...createPresetApplication(c),
        ...createPairedActions(c),
        ...createMatchEditor(c),
        ...createLegacyLineup(c),
    };
    return {
        imports,
        state: c.state,
        actions: c.actions,
        stop() {
            c.active = false;
            imports.clear();
            c.state.lineupPlayerLoadSequence.invalidate();
            c.state.pairedLineupLoadSequence.home.invalidate();
            c.state.pairedLineupLoadSequence.away.invalidate();
        },
    };
}
export { lineupsPage } from "../../pages/lineups";
