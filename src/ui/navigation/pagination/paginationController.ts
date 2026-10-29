import { paginationState, type PaginationState } from "./paginationState";
/** Update in place so real directory pagination preserves its focused button and scroll. */
export function updatePaginationView(root: HTMLElement, state: PaginationState): void {
    const flags = paginationState(state);
    const previous = root.querySelector<HTMLButtonElement>('[data-pagination-direction="previous"]');
    const next = root.querySelector<HTMLButtonElement>('[data-pagination-direction="next"]');
    const summary = root.querySelector<HTMLElement>("[data-pagination-summary]");
    const current = root.querySelector<HTMLElement>("[data-pagination-current]");
    const status = root.querySelector<HTMLElement>("[data-pagination-status]");
    if (!previous || !next || !summary || !current || !status)
        throw new Error("Pagination markup is incomplete");
    // Park focus before disabling a trigger; completion never steals focus from another control.
    if (state.busy && (document.activeElement === previous || document.activeElement === next)) current.focus({ preventScroll: true });
    root.setAttribute("aria-busy", String(!!state.busy));
    previous.disabled = flags.previousDisabled;
    next.disabled = flags.nextDisabled;
    summary.textContent = state.summary ?? "第 " + state.page + " 页";
    current.textContent = String(state.page);
    current.setAttribute("aria-label", "当前第 " + state.page + " 页");
    status.hidden = !state.busy;
}
