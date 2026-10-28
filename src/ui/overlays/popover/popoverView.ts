import { escapeHtml } from "../../../components/format";
import { primitiveAttributes, type PrimitiveAttributes } from "../../primitives/controlAttributes";
import { iconSlot, type UiIcon } from "../../icons/iconSlot";
export interface PopoverItem { id: string; label: string; description?: string; icon?: UiIcon; danger?: boolean; disabled?: boolean; attributes?: PrimitiveAttributes; onSelect?: ()=>void }
export function popoverView(items: readonly PopoverItem[],label: string): string {
  return `<div data-ui-popover class="ui-popover" popover="manual" role="menu" tabindex="-1" aria-label="${escapeHtml(label)}">${items.length?items.map((item,index)=>`${item.danger&&index>0&&!items[index-1].danger?`<div class="ui-popover-separator" role="separator"></div>`:""}<button type="button" class="ui-popover-item" role="menuitem" data-popover-index="${index}" data-danger="${!!item.danger}" ${item.disabled?"disabled":""} ${primitiveAttributes(item.attributes)}>${item.icon?iconSlot(item.icon,16):`<span class="ui-popover-icon-empty" aria-hidden="true"></span>`}<span class="ui-popover-copy"><strong>${escapeHtml(item.label)}</strong>${item.description?`<small>${escapeHtml(item.description)}</small>`:""}</span></button>`).join(""):"<p class=ui-popover-empty>暂无可用操作</p>"}</div>`;
}
