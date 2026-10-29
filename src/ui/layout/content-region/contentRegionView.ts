export function workspaceRegionView(content: string, pending: boolean, failed: boolean): string {
    return `${pending ? '<div class="page-load-progress" role="status" aria-label="正在载入页面"></div>' : ""}<div class="page-container ${failed ? "shell-load-failed" : ""}" data-workspace-scroll-key="page-content" ${pending || failed ? 'data-workspace-capture="suspended"' : ""} aria-busy="${pending}" ${pending ? "inert" : ""}>${content}</div><div id="modal-root" class="workspace-panel-root" aria-live="polite" aria-busy="${pending}" ${pending ? "inert" : ""}></div>`;
}
