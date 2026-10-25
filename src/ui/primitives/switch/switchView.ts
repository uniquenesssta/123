import offDefault from "../../../assets/controls/switch-off-default.svg?raw";
import offHover from "../../../assets/controls/switch-off-hover.svg?raw";
import offPressed from "../../../assets/controls/switch-off-pressed.svg?raw";
import offFocus from "../../../assets/controls/switch-off-focus.svg?raw";
import offDisabled from "../../../assets/controls/switch-off-disabled.svg?raw";
import onDefault from "../../../assets/controls/switch-on-default.svg?raw";
import onHover from "../../../assets/controls/switch-on-hover.svg?raw";
import onPressed from "../../../assets/controls/switch-on-pressed.svg?raw";
import onFocus from "../../../assets/controls/switch-on-focus.svg?raw";
import onDisabled from "../../../assets/controls/switch-on-disabled.svg?raw";
import { escapeHtml } from "../../../components/format";
import { primitiveAttributes, primitiveName, type PrimitiveAttributes } from "../controlAttributes";

export interface SwitchOptions { label: string; checked?: boolean; disabled?: boolean; description?: string; descriptionId?: string; showLabel?: boolean; attributes?: PrimitiveAttributes }

export function switchView(options: SwitchOptions): string {
  if (options.description && !options.descriptionId) throw new Error("Switch description requires a stable id");
  const sources = { "off-default": offDefault, "off-hover": offHover, "off-pressed": offPressed, "off-focus": offFocus, "off-disabled": offDisabled, "on-default": onDefault, "on-hover": onHover, "on-pressed": onPressed, "on-focus": onFocus, "on-disabled": onDisabled };
  return `<label class="ui-switch" data-ui-primitive="switch"><input type="checkbox" role="switch" aria-label="${primitiveName(options.label)}" ${options.checked ? "checked" : ""} ${options.disabled ? "disabled" : ""} ${options.description ? `aria-describedby="${primitiveName(options.descriptionId!)}"` : ""} ${primitiveAttributes(options.attributes)}><span class="ui-switch-control" aria-hidden="true">${Object.entries(sources).map(([state, source]) => `<span data-switch-state="${state}">${source}</span>`).join("")}</span><span class="ui-switch-content">${options.showLabel === false ? "" : `<strong>${escapeHtml(options.label)}</strong>`}${options.description ? `<small id="${primitiveName(options.descriptionId!)}">${escapeHtml(options.description)}</small>` : ""}</span></label>`;
}
