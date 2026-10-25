import plusSource from "../../assets/icons/plus.svg?raw";

export type UiIcon = "dashboard" | "match-center" | "prediction" | "release-acceptance" | "history" | "shield" | "users" | "lineup-preset" | "sheet" | "settings" | "chart" | "chat" | "plug" | "database" | "issue-log" | "info" | "contrast" | "panel-left" | "panel-right" | "arrow-left" | "arrow-right" | "reset" | "alert-circle" | "close" | "check" | "minus" | "plus" | "chevron-down";

/** Exact downloaded Figma vectors, colored by the shared semantic tone through a mask. */
export function iconSlot(name: UiIcon, size: 16 | 20 | 24): string {
  if (name === "plus") return `<span class="ui-icon-slot ui-icon-plus ui-icon-size-${size}" aria-hidden="true"><span class="ui-icon-plus-master">${plusSource}</span></span>`;
  return `<span class="ui-icon-slot ui-icon-${name} ui-icon-size-${size}" aria-hidden="true"></span>`;
}
