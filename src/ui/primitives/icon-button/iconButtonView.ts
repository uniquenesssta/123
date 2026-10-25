import { escapeHtml } from "../../../components/format";
import { iconSlot, type UiIcon } from "../../icons/iconSlot";
import { primitiveAttributes, primitiveName, type PrimitiveAttributes } from "../controlAttributes";

export interface IconButtonOptions {
  action?: string;
  label: string;
  icon: UiIcon;
  size?: 32 | 40 | 48;
  className?: string;
  disabled?: boolean;
  pressed?: boolean;
  attributes?: PrimitiveAttributes;
}

export function iconButtonView(options: IconButtonOptions): string {
  const name = primitiveName(options.label), size = options.size ?? 32;
  return `<button type="button" class="ui-icon-button ${escapeHtml(options.className ?? "")}" data-ui-primitive="icon-button" data-fip-control-scale="${size}" ${options.action ? `data-action="${escapeHtml(options.action)}"` : ""} title="${name}" aria-label="${name}" ${options.pressed === undefined ? "" : `aria-pressed="${options.pressed}"`} ${primitiveAttributes(options.attributes)} ${options.disabled ? "disabled" : ""}>${iconSlot(options.icon, size === 48 ? 20 : 16)}</button>`;
}
