import { escapeHtml } from "../../../components/format";
import { iconSlot, type UiIcon } from "../../icons/iconSlot";
import { primitiveAttributes, type PrimitiveAttributes } from "../../primitives/controlAttributes";
export interface NavigationItem {
    label: string;
    icon: UiIcon;
    title: string;
    selected: boolean;
    attributes: PrimitiveAttributes;
    disabled?: boolean;
}
/** Render navigation facts supplied by a route owner; retain no current-route state. */
export function navigationItemView(item: NavigationItem, level: "primary" | "secondary"): string {
    return `<button type="button" class="${level}-nav-item ${item.selected ? "active" : ""}" ${primitiveAttributes(item.attributes)} title="${escapeHtml(item.title)}" aria-label="${escapeHtml(item.label)}" ${item.selected ? 'aria-current="' + (level === "primary" ? "true" : "page") + '"' : ""} ${item.disabled ? "disabled" : ""}>${iconSlot(item.icon, 20)}<b>${escapeHtml(item.label)}</b></button>`;
}
