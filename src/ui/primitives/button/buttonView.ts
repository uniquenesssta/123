import { escapeHtml } from "../../../components/format";
import { iconSlot, type UiIcon } from "../../icons/iconSlot";
import { primitiveAttributes, primitiveName, type PrimitiveAttributes } from "../controlAttributes";
import { spinnerView } from "../spinner";

export interface ButtonOptions {
  label: string;
  variant?: "primary" | "secondary" | "ghost" | "danger";
  size?: 32 | 40 | 48;
  leading?: UiIcon;
  trailing?: UiIcon;
  disabled?: boolean;
  loading?: boolean;
  type?: "button" | "submit" | "reset";
  attributes?: PrimitiveAttributes;
}

export function buttonView(options: ButtonOptions): string {
  primitiveName(options.label);
  const variant = options.variant ?? "secondary", size = options.size ?? 32;
  if (options.loading && variant === "ghost") throw new Error("Ghost has no approved Loading variant");
  const iconSize = size === 48 ? 20 : 16;
  const leading = options.loading ? spinnerView(iconSize, variant === "primary" || variant === "danger" ? "On Accent" : "Default") : options.leading ? iconSlot(options.leading, iconSize) : "";
  return `<button type="${options.type ?? "button"}" class="ui-button ui-button-${variant}" data-ui-primitive="button" data-fip-control-scale="${size}" ${primitiveAttributes(options.attributes)} ${options.disabled || options.loading ? "disabled" : ""} ${options.loading ? 'aria-busy="true"' : ""}>${leading}<span class="ui-button-label">${escapeHtml(options.label)}</span>${!options.loading && options.trailing ? iconSlot(options.trailing, iconSize) : ""}</button>`;
}
