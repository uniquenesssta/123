import spinnerSource from "../../assets/icons/spinner.svg?raw";

export type UiIcon = "dashboard" | "match-center" | "prediction" | "release-acceptance" | "history" | "shield" | "users" | "lineup-preset" | "sheet" | "settings" | "chart" | "chat" | "plug" | "database" | "issue-log" | "info" | "contrast" | "panel-left" | "panel-right" | "arrow-left" | "arrow-right" | "reset" | "spinner" | "alert-circle" | "close" | "check";

/** Exact downloaded Figma vectors, colored by the shared semantic tone through a mask. */
export function iconSlot(name: UiIcon, size: 16 | 20 | 24): string {
  if (name === "spinner") return `<span class="ui-icon-slot ui-icon-spinner ui-icon-size-${size}" aria-hidden="true">${spinnerSource}</span>`;
  return `<span class="ui-icon-slot ui-icon-${name} ui-icon-size-${size}" aria-hidden="true"></span>`;
}
