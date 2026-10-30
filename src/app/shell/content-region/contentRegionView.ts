import { asyncBoundaryView } from "../../../ui/patterns/async-boundary";
export function contentRegionView(content:string,pending:boolean,failed=false):string {
 return asyncBoundaryView({content,pending,failed,retryAction:"retry-shell-page",logsAction:"shell-open-logs"});
}
