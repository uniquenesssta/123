import { escapeHtml } from "../../../components/format";

export function badgeView(label: string, tone: "Neutral" | "Info" | "Success" | "Warning" | "Danger" = "Neutral", style: "Filled" | "Outline" = "Filled"): string {
  return `<span class="ui-badge" data-ui-primitive="badge" data-tone="${tone}" data-style="${style}">${escapeHtml(label)}</span>`;
}
