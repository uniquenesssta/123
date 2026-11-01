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
    const imports = new MatchImports({ api: ports.api, files: ports.files, busy: ports.runBusy,
      recovery: ports.importRecovery, changed: () => ports.render({preserveForm:true}),
      capture: () => { const scope=ports.captureImportData?.(),snapshot=ports.session.bootstrap;return ()=>c.active&&(scope?scope():snapshot===ports.session.bootstrap); },
      captureView: () => {
        const view=ports.captureImportView?.(), page=ports.navigation.page, section=ports.workspaceState.module(page).active_section, match=c.state.pairedLineupBuilder.match_id;
        return () => (view?.()??true) && c.active && page===ports.navigation.page && section===ports.workspaceState.module(page).active_section && match===c.state.pairedLineupBuilder.match_id;
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
