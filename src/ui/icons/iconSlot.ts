import plusSource from "../../assets/icons/plus.svg?raw";
import calendarSource from "../../assets/icons/calendar.svg?raw";
import { escapeHtml } from "../../components/format";
import { registeredIcon, type UiIcon } from "./iconRegistry";
export type { UiIcon } from "./iconRegistry";
/** Approved vectors retain exact roots; only their wrapping slot is scaled. */
export function iconSlot(name: UiIcon,size:16|20|24,className=""): string {
  const entry=registeredIcon(name);
  if(![16,20,24].includes(size))throw new Error("Unsupported icon slot size");
  const classes=`ui-icon-slot ui-icon-${entry.name} ui-icon-size-${size}${className?" "+escapeHtml(className):""}`;
  if(name==="plus"||name==="calendar")return `<span class="${classes}" aria-hidden="true"><span class="ui-icon-${name}-master">${name==="plus"?plusSource:calendarSource}</span></span>`;
  return `<span class="${classes}" aria-hidden="true"></span>`;
}
