import { blockingMessageView } from "../../../ui/feedback/blocking-message/blockingMessageView";

export function contentRegionView(content: string, pending: boolean, failed = false): string {
  return `${pending ? '<div class="page-load-progress" role="status" aria-label="正在载入页面"></div>' : ""}
    <div class="page-container ${failed ? "shell-load-failed" : ""}" data-workspace-scroll-key="page-content" ${pending || failed ? 'data-workspace-capture="suspended"' : ""} aria-busy="${pending}" ${pending ? "inert" : ""}>${failed ? blockingMessageView("retry-shell-page", "shell-open-logs") : content}</div>
    <div id="modal-root" class="workspace-panel-root" aria-live="polite" aria-busy="${pending}" ${pending ? "inert" : ""}></div>`;
}
