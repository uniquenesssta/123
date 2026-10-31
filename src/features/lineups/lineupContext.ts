import type { createMatchExchange } from "./matchExchange";
import type * as T from "../../types";
import type { LineupApi, LineupFiles } from "./lineupPorts";
import type { pageTitle } from "../../app/shell";
import { type ModalConfirmationOptions } from "../../app/modal";
import { Router } from "../../app/router";
import { AppSession } from "../../app/session";
import { WorkspaceCoordinator } from "../../app/workspace";
import type { CoachListItem, FormationRecord, IssueSeverity, MatchLineupChain, MatchLineupTeamChain, Page } from "../../types";
import type { createLegacyLineup } from "./legacyLineup";
import type { createLineupState } from "./lineupState";
import type { createMatchEditor } from "./matchEditor";
import type { createPairedActions } from "./pairedActions";
import type { createPairedBuilder } from "./pairedBuilder";
import type { createPresetActions } from "./presetActions";
import type { createPresetApplication } from "./presetApplication";
import type { createPresetEditor } from "./presetEditor";
import type { createPresetPositions } from "./presetPositions";
export type LineupSide = "home" | "away";
interface RenderOptions {
    readonly preserveForm?: boolean;
}
export interface LineupPorts {
    recordUiOperation: (kind: "ui_action", operation: string, metadata: Record<string, unknown>) => string;
    recordUiOperationFailure: (operation: string, error: unknown, traceId: string, metadata: Record<string, unknown>) => void;
    pageTitle: typeof pageTitle;
    readonly references: T.PlayerCatalogReferenceData | null;
    setReferences(value: T.PlayerCatalogReferenceData): void;
    coachList: CoachListItem[];
    localDateTimeInputValue: (raw: string) => string;
    api: LineupApi;
    files: LineupFiles;
    render: (options?: RenderOptions) => void;
    formationCatalog: FormationRecord[];
    toast: (message: string, kind?: "normal" | "success" | "error") => void;
    readonly teamDetail: T.TeamDetail | null;
    readonly teamPresets: T.TeamLineupPresetRecord[];
    setTeamPresets(teamId: string, presets: T.TeamLineupPresetRecord[]): void;
    refreshTeam(): Promise<void>;
    showHtmlModal: (title: string, subtitle: string, body: string, footer?: string, modalClass?: string) => void;
    value: (id: string) => string;
    nullableValue: (id: string) => string | null;
    nullableNumber: (id: string) => number | null;
    checked: (id: string) => boolean;
    runBusy: <T>(operation: () => Promise<T>) => Promise<T>;
    showConfirmation: (title: string, description: string, facts: Array<[
        string,
        string
    ]>, confirmLabel: string, action: () => Promise<void>, options?: ModalConfirmationOptions) => void;
    closeModal: () => void;
    localDateTimeToIso: (raw: string | null, fallbackNow?: boolean) => string | null;
    lastPredictionResult: unknown;
    workspaceState: WorkspaceCoordinator<Page>;
    session: AppSession;
    filterSelectOptions: (selectId: string, dataKey: string, expected: string | null) => void;
    app: HTMLDivElement;
    refreshBootstrapAndCatalog: () => Promise<void>;
    navigation: Router;
    recordClientIssue: (error: unknown, operation: string, severity?: IssueSeverity) => void;
    userFacingError: (error: unknown) => string;
    selectedReviewMatchId: string | null;
    loadReviewLineups: (matchId: string) => Promise<void>;
    missingLineupTeam: (chain: MatchLineupChain) => MatchLineupTeamChain;
}
export type LineupActions = ReturnType<typeof createMatchExchange> & ReturnType<typeof createPairedBuilder> & ReturnType<typeof createPresetPositions> & ReturnType<typeof createPresetEditor> & ReturnType<typeof createPresetActions> & ReturnType<typeof createPresetApplication> & ReturnType<typeof createPairedActions> & ReturnType<typeof createMatchEditor> & ReturnType<typeof createLegacyLineup>;
export interface LineupContext {
    active: boolean;
    state: ReturnType<typeof createLineupState>;
    ports: LineupPorts;
    actions: LineupActions;
}
