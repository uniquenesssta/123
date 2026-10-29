import { escapeHtml } from "../../../components/format";
import { iconSlot } from "../../icons/iconSlot";
import { primitiveAttributes, type PrimitiveAttributes } from "../../primitives/controlAttributes";
export interface ObjectTab {
    id: string;
    label: string;
    panelId: string;
    disabled?: boolean;
    closable?: boolean;
    attributes?: PrimitiveAttributes;
    closeAttributes?: PrimitiveAttributes;
}
export interface TabsOptions {
    id: string;
    label: string;
    items: readonly ObjectTab[];
    activeId: string | null;
    size?: 32 | 40;
}
/** Host owns objects, selected identity and matching tabpanel nodes. */
export function tabsView(options: TabsOptions): string {
    const enabled = options.items.filter(i => !i.disabled), active = enabled.find(i => i.id === options.activeId), focusStop = active ?? enabled[0];
    if (new Set(options.items.map(i => i.id)).size !== options.items.length)
        throw new Error("Duplicate object tab identity");
    if (!options.label.trim() || options.items.some(i => !i.panelId.trim()))
        throw new Error("Tabs require a name and associated panels");
    return `<div id="${escapeHtml(options.id)}" class="ui-tabs" role="tablist" aria-label="${escapeHtml(options.label)}" data-ui-navigation="horizontal" data-fip-control-scale="${options.size ?? 32}">${options.items.map(i => `<div class="ui-tab ${i === active ? "active" : ""}" role="presentation"><button type="button" id="${escapeHtml(options.id + "-" + encodeURIComponent(i.id))}" role="tab" aria-controls="${escapeHtml(i.panelId)}" aria-selected="${i === active}" tabindex="${i === focusStop ? 0 : -1}" ${i.disabled ? "disabled" : ""} ${primitiveAttributes(i.attributes)} title="${escapeHtml(i.label)}"><span>${escapeHtml(i.label)}</span></button>${i.closable ? `<button type="button" class="ui-tab-close" data-tab-close aria-label="关闭 ${escapeHtml(i.label)}" ${i.disabled ? "disabled" : ""} ${primitiveAttributes(i.closeAttributes)}>${iconSlot("close", 16)}</button>` : ""}</div>`).join("")}</div>`;
}
