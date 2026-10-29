export interface PaginationState {
    page: number;
    hasMore: boolean;
    busy?: boolean;
    summary?: string;
}
/** Derived presentation only: the host remains the cursor and request owner. */
export function paginationState(state: PaginationState) {
    if (!Number.isSafeInteger(state.page) || state.page < 1)
        throw new Error("Pagination page must be a positive integer");
    return { previousDisabled: !!state.busy || state.page === 1, nextDisabled: !!state.busy || !state.hasMore };
}
