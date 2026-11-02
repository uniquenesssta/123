import type {CompetitionRecord} from '../../types';
import type {PostmatchFeature} from './postmatchFeature';
import {postmatchHistoryView,postmatchProvidersView} from './monitoringView';
import {postmatchEvidenceView} from './evidenceView';
/** Compose only the H-owned region; the surrounding analytics stages remain separately owned. */
export function postmatchPanel(feature:PostmatchFeature,competitions:CompetitionRecord[]):string {
  return `<div class="postmatch-workbench" data-workspace-persist="false">${postmatchHistoryView(feature,competitions)}${postmatchEvidenceView(feature)}${postmatchProvidersView(feature)}</div>`;
}
