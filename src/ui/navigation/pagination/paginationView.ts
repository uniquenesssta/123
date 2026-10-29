import { escapeHtml } from "../../../components/format";
import { primitiveAttributes, type PrimitiveAttributes } from "../../primitives/controlAttributes";
import { spinnerView } from "../../primitives/spinner";
import { paginationState, type PaginationState } from "./paginationState";
export interface PaginationOptions extends PaginationState {
    previousAction: string;
    nextAction: string;
    label?: string;
    attributes?: PrimitiveAttributes;
}
/** A cursor-backed result supplies hasMore; unknown totals are never invented. */
export function paginationView(options: PaginationOptions): string {
    const state = paginationState(options);
    return `<nav class="ui-pagination" aria-label="${escapeHtml(options.label ?? "结果分页")}" aria-busy="${!!options.busy}" ${primitiveAttributes(options.attributes)}><span class="ui-pagination-summary" data-pagination-summary aria-live="polite">${escapeHtml(options.summary ?? "第 " + options.page + " 页")}</span><div class="ui-pagination-controls"><button type="button" class="secondary tiny" data-pagination-direction="previous" data-action="${escapeHtml(options.previousAction)}" ${state.previousDisabled ? "disabled" : ""}>上一页</button><span class="ui-pagination-current" data-pagination-current tabindex="-1" aria-current="page" aria-label="当前第 ${options.page} 页">${options.page}</span><button type="button" class="secondary tiny" data-pagination-direction="next" data-action="${escapeHtml(options.nextAction)}" ${state.nextDisabled ? "disabled" : ""}>下一页</button></div><span data-pagination-status role="status" ${options.busy ? "" : "hidden"}>${spinnerView(16)}正在加载结果…</span></nav>`;
}
