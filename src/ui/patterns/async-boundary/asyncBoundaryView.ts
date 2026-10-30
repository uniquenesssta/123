import { blockingMessageView } from "../../feedback/blocking-message/blockingMessageView";
import { workspaceRegionView } from "../../layout";
export interface AsyncBoundaryOptions {
  readonly content: string;
  readonly pending: boolean;
  readonly failed?: boolean;
  readonly retryAction: string;
  readonly logsAction: string;
}
/** Read recovery only. Request lifetime, snapshots and retry callbacks stay with the host. */
export function asyncBoundaryView(options: AsyncBoundaryOptions): string {
  const failed=options.failed ?? false;
  return workspaceRegionView(failed ? blockingMessageView(options.retryAction,options.logsAction) : options.content,options.pending,failed);
}
