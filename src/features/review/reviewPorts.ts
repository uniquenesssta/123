import type { BootstrapResponse, LineupRecord, ReviewableMatch, MatchReviewSummary, MatchReviewDetail, MatchReviewDraft, MatchReviewPackageWorkflowRecord, MatchReviewPackagePreview, MatchReviewPackageSummary, MatchReviewPackageConfirmationRequest, MatchReviewPackageFactsCommitResult, MatchReviewPackageReviewResult, AbilityCandidateDecisionDraft, AbilityUpdateCandidateRecord } from '../../types';
export interface ReviewApi {
    listReviewableMatches(limit: number): Promise<ReviewableMatch[]>;
    listMatchReviews(limit: number): Promise<MatchReviewSummary[]>;
    listLineups(matchId: string | null, limit: number): Promise<LineupRecord[]>;
    readLineup(id: string): Promise<LineupRecord>;
    readMatchReview(id: string): Promise<MatchReviewDetail>;
    readMatchReviewPackageWorkflow(matchId: string): Promise<MatchReviewPackageWorkflowRecord | null>;
    chooseExcelImportFile(): Promise<string | null>;
    chooseExcelExportFile(name: string): Promise<string | null>;
    exportMatchReviewPackage(path: string, matchId: string): Promise<MatchReviewPackageSummary>;
    previewMatchReviewPackage(path: string, matchId: string | null): Promise<MatchReviewPackagePreview>;
    confirmMatchReviewPackage(request: MatchReviewPackageConfirmationRequest): Promise<MatchReviewPackageWorkflowRecord>;
    commitMatchReviewPackageFacts(id: string): Promise<MatchReviewPackageFactsCommitResult>;
    generateMatchReviewFromPackage(id: string): Promise<MatchReviewPackageReviewResult>;
    generateMatchReview(draft: MatchReviewDraft): Promise<MatchReviewDetail>;
    decideAbilityCandidate(request: AbilityCandidateDecisionDraft): Promise<AbilityUpdateCandidateRecord>;
}
export type ReviewOperation = 'export' | 'preview' | 'confirm' | 'facts' | 'generate' | 'manual' | 'candidate';
export interface ReviewReceipt {
    matchId: string;
    kind: ReviewOperation;
    packageId: string | null;
    sha: string | null;
    path: string | null;
    reviewId: string | null;
    reviewVersion: string | null;
    candidateId: string | null;
    decision: 'accept' | 'reject' | null;
    startedAt: string;
    baseline: string;
}
export interface ReviewRecovery {
    read(matchId: string): ReviewReceipt | null;
    save(receipt: ReviewReceipt): void;
    clear(matchId: string): void;
}
export interface ReviewPorts {
    postmatch: import("../postmatch/index").SettlementWorkflow;
    api: ReviewApi;
    scope(): () => boolean;
    view(): () => boolean;
    changed(): void;
    recovery: ReviewRecovery;
    snapshot(): BootstrapResponse | null;
    formatError(error: unknown): string;
}
export interface ReviewState {
    matches: ReviewableMatch[];
    reviews: MatchReviewSummary[];
    choiceId: string | null;
    selectedMatchId: string | null;
    lineups: LineupRecord[];
    detail: MatchReviewDetail | null;
    workflow: MatchReviewPackageWorkflowRecord | null;
    loading: boolean;
    listLoading: boolean;
    loaded: boolean;
    error: string | null;
    notice: string | null;
}
export interface ReviewIntent {
    kind: 'confirm' | 'facts' | 'generate' | 'manual' | 'candidate';
    matchId: string;
    version: string;
    confirmedBy: string | null;
    note: string | null;
    draft: MatchReviewDraft | null;
    candidateId: string | null;
    decision: 'accept' | 'reject' | null;
}
