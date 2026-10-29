import { escapeHtml } from "../../../components/format";
import { iconSlot, type UiIcon } from "../../icons/iconSlot";
import { primitiveAttributes, type PrimitiveAttributes } from "../../primitives/controlAttributes";
export interface SegmentItem {
    value: string;
    label: string;
    icon?: UiIcon;
    disabled?: boolean;
    attributes?: PrimitiveAttributes;
}
export interface SegmentedOptions {
    label: string;
    items: readonly SegmentItem[];
    value: string;
    size?: 32 | 40;
}
export function segmentedControlView(options: SegmentedOptions): string {
    if (!options.label.trim() || new Set(options.items.map(i => i.value)).size !== options.items.length)
        throw new Error("Segmented control requires a name and unique choices");
    return `<div class="ui-segmented" role="group" aria-label="${escapeHtml(options.label)}" data-ui-navigation="horizontal" data-fip-control-scale="${options.size ?? 32}">${options.items.map(i => `<button type="button" aria-pressed="${i.value === options.value}" ${i.disabled ? "disabled" : ""} ${primitiveAttributes(i.attributes)}>${i.icon ? iconSlot(i.icon, 16) : ""}<span>${escapeHtml(i.label)}</span></button>`).join("")}</div>`;
}
