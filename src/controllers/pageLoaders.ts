import { api } from "../api/client";
import type {
  AbilityUpdateCandidateRecord,
  AiAnalysisSuggestionRecord,
  AnalyticsOverview,
  BackgroundJob,
  CoachListItem,
  LineupRecord,
  ParameterTuningCandidateRecord,
  PlayerCatalogReferenceData,
  PostmatchOverview,
} from "../types";


export interface LineupsLoadResult {
  readonly references: PlayerCatalogReferenceData;
  readonly records: LineupRecord[];
  readonly coaches: CoachListItem[];
}

export interface AnalysisCenterLoadResult {
  readonly overview: AnalyticsOverview;
  readonly jobs: BackgroundJob[];
  readonly suggestions: AiAnalysisSuggestionRecord[];
  readonly abilityCandidates: AbilityUpdateCandidateRecord[];
  readonly tuningCandidates: ParameterTuningCandidateRecord[];
  readonly postmatch: PostmatchOverview;
}

export async function fetchLineups(): Promise<LineupsLoadResult> {
  const [references, records, coaches] = await Promise.all([
    api.playerCatalogReferenceData(),
    api.listLineups(null, 100),
    api.listCoaches({ search: null, active_only: false, limit: 500 }),
  ]);
  return { references, records, coaches };
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
