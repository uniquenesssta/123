import { blockingMessageView } from "../../../ui/feedback/blocking-message/blockingMessageView";
import { workspaceRegionView } from "../../../ui/layout";
export function contentRegionView(content:string,pending:boolean,failed=false):string {
 return workspaceRegionView(failed?blockingMessageView("retry-shell-page","shell-open-logs"):content,pending,failed);
}
