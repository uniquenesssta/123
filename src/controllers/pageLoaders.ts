import { api } from "../api/client";
import type {
  CoachListItem,
  LineupRecord,
  PlayerCatalogReferenceData,
} from "../types";


export interface LineupsLoadResult {
  readonly references: PlayerCatalogReferenceData;
  readonly records: LineupRecord[];
  readonly coaches: CoachListItem[];
}

export async function fetchLineups(): Promise<LineupsLoadResult> {
  const [references, records, coaches] = await Promise.all([
    api.playerCatalogReferenceData(),
    api.listLineups(null, 100),
    api.listCoaches({ search: null, active_only: false, limit: 500 }),
  ]);
  return { references, records, coaches };
}
