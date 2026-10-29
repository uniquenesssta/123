import { mainFrameView } from "../../../ui/layout";
import type { Page } from "../../../types";
import { contentRegionView } from "../content-region/contentRegionView";
import type { ShellOptions } from "../application-shell/applicationShellState";

export const workspacePages = new Set<Page>(["lineups", "teams", "players", "lineup_presets", "prediction", "workbooks", "rules", "release", "analytics", "api_workspace", "openai", "database", "logs", "architecture"]);

export function pageFrameView(options: ShellOptions, topbar: string): string {
  return mainFrameView(topbar, contentRegionView(options.content, options.navigationPending, options.navigationFailed));
}
