import { api } from "../api/client";
import type {
  AbilityUpdateCandidateRecord,
  AiAnalysisSuggestionRecord,
  CoachListItem,
  AnalyticsOverview,
  BackgroundJob,
  LineupRecord,
  MatchReviewSummary,
  MatchReviewPackagePreview,
  MatchReviewPackageWorkflowRecord,
  ParameterTuningCandidateRecord,
  PostmatchOverview,
  PostmatchSettlementRecord,
  PlayerCatalogReferenceData,
  PlayerDetail,
  PlayerListPage,
  PlayerListQuery,
  ReviewableMatch,
  TeamDetail,
  TeamListPage,
  TeamListQuery,
} from "../types";

export interface PlayerCatalogLoadResult {
  readonly references: PlayerCatalogReferenceData;
  readonly list: PlayerListPage;
  readonly query: PlayerListQuery;
  readonly selected: PlayerDetail | null;
}


export interface TeamCatalogLoadResult {
  readonly list: TeamListPage;
  readonly query: TeamListQuery;
  readonly selected: TeamDetail | null;
}

export async function fetchTeamCatalog(
  query: TeamListQuery,
  selectedTeam: TeamDetail | null,
  resetCursor: boolean,
): Promise<TeamCatalogLoadResult> {
  const effectiveQuery: TeamListQuery = resetCursor
    ? { ...query, cursor_name: null, cursor_id: null }
    : { ...query };
  const list = await api.listTeams(effectiveQuery);
  const selectedTeamId = selectedTeam?.team.id ?? null;
  let selected = selectedTeamId && list.items.some((item) => item.id === selectedTeamId)
    ? selectedTeam
    : null;
  if (!selected && list.items[0]) {
    selected = await api.readTeam(list.items[0].id);
  }
  return { list, query: effectiveQuery, selected };
}
export interface LineupsLoadResult {
  readonly references: PlayerCatalogReferenceData;
  readonly records: LineupRecord[];
  readonly coaches: CoachListItem[];
}

export interface ReviewCenterLoadResult {
  readonly matches: ReviewableMatch[];
  readonly reviews: MatchReviewSummary[];
  readonly selectedMatchId: string | null;
  readonly lineups: LineupRecord[];
  readonly workflow: MatchReviewPackageWorkflowRecord | null;
  readonly preview: MatchReviewPackagePreview | null;
  readonly settlement: PostmatchSettlementRecord | null;
}

export interface AnalysisCenterLoadResult {
  readonly overview: AnalyticsOverview;
  readonly jobs: BackgroundJob[];
  readonly suggestions: AiAnalysisSuggestionRecord[];
  readonly abilityCandidates: AbilityUpdateCandidateRecord[];
  readonly tuningCandidates: ParameterTuningCandidateRecord[];
  readonly postmatch: PostmatchOverview;
}

export async function fetchPlayerCatalog(
  query: PlayerListQuery,
  selectedPlayer: PlayerDetail | null,
  resetCursor: boolean,
  cachedReferences: PlayerCatalogReferenceData | null = null,
): Promise<PlayerCatalogLoadResult> {
  const effectiveQuery: PlayerListQuery = resetCursor
    ? { ...query, cursor_name: null, cursor_id: null }
    : { ...query };
  const [references, list] = await Promise.all([
    cachedReferences
      ? Promise.resolve(cachedReferences)
      : api.playerCatalogReferenceData(),
    api.listPlayers(effectiveQuery),
  ]);
  const selectedPlayerId = selectedPlayer?.player.id ?? null;
  const selected = selectedPlayerId && list.items.some((item) => item.id === selectedPlayerId)
    ? selectedPlayer
    : null;
  return { references, list, query: effectiveQuery, selected };
}

export async function fetchLineups(): Promise<LineupsLoadResult> {
  const [references, records, coaches] = await Promise.all([
    api.playerCatalogReferenceData(),
    api.listLineups(null, 100),
    api.listCoaches({ search: null, active_only: false, limit: 500 }),
  ]);
  return { references, records, coaches };
}

export async function fetchReviewLineups(matchId: string): Promise<LineupRecord[]> {
  const summaries = await api.listLineups(matchId, 30);
  const active = summaries.filter((lineup) => lineup.status === "active");
  const rank: Record<LineupRecord["lineup_type"], number> = { actual: 0, confirmed: 1, expected: 2 };
  const selected = new Map<string, LineupRecord>();
  for (const lineup of active.sort((left, right) => rank[left.lineup_type] - rank[right.lineup_type])) {
    if (!selected.has(lineup.team_id)) selected.set(lineup.team_id, lineup);
  }
  return Promise.all(Array.from(selected.values()).map((lineup) => api.readLineup(lineup.id)));
}

export async function fetchReviewCenter(selectedReviewMatchId: string | null): Promise<ReviewCenterLoadResult> {
  const [matches, reviews] = await Promise.all([
    api.listReviewableMatches(100),
    api.listMatchReviews(100),
  ]);
  const selectedMatchId = selectedReviewMatchId
    && matches.some((item) => item.match_record.id === selectedReviewMatchId)
    ? selectedReviewMatchId
    : matches[0]?.match_record.id ?? null;
  const [lineups, workflow, settlements] = selectedMatchId
    ? await Promise.all([
        fetchReviewLineups(selectedMatchId),
        api.readMatchReviewPackageWorkflow(selectedMatchId),
        api.listPostmatchSettlements(100),
      ])
    : [[], null, [] as PostmatchSettlementRecord[]];
  const settlement = selectedMatchId
    ? settlements.find((item) => item.match_id === selectedMatchId) ?? null
    : null;
  return {
    matches,
    reviews,
    selectedMatchId,
    lineups,
    workflow,
    preview: workflow?.preview ?? null,
    settlement,
  };
}

export async function fetchAnalysisCenter(): Promise<AnalysisCenterLoadResult> {
  const [overview, jobs, suggestions, abilityCandidates, tuningCandidates, postmatch] = await Promise.all([
    api.analyticsOverview(),
    api.listBackgroundJobs(100),
    api.listAiAnalysisSuggestions(null, 500),
    api.listAbilityCandidates("pending", 500, null),
    api.listParameterTuningCandidates(100),
    api.postmatchOverview(100),
  ]);
  return { overview, jobs, suggestions, abilityCandidates, tuningCandidates, postmatch };
}
