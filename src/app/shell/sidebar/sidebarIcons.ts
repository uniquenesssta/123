import type { Page } from "../../../types";
import type { UiIcon } from "../../../ui/icons/iconSlot";

/** Accepted Figma icon mapping for the registered Shell routes. */
const routeIcons: Record<Page, UiIcon> = {
  dashboard: "dashboard", lineups: "match-center", prediction: "prediction", review: "release-acceptance", runs: "history",
  teams: "shield", players: "users", lineup_presets: "lineup-preset", workbooks: "sheet", rules: "settings", release: "release-acceptance",
  analytics: "chart", api_workspace: "chat", openai: "plug", database: "database", logs: "issue-log", architecture: "info",
};

export const navigationIcon = (page: Page): UiIcon => routeIcons[page];
