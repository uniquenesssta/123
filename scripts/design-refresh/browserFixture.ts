import { playersPage } from "../../src/features/players";
import { NavigationKeyboardController } from "../../src/ui/navigation";
import { dashboardPage } from "../../src/features/dashboard";
import { teamsPage } from "../../src/features/teams";
import { lineupsPage } from "../../src/pages/lineups";
import { rulesPage } from "../../src/features/rules";
import { renderShell } from "../../src/app/shell/application-shell/applicationShellView";
import { enhanceSearchableSelects } from "../../src/components/searchableSelect";
import { state, catalogue, detail, references, builder, match, playerCatalogue } from "./fixtureData";
import type { Page, Theme } from "../../src/types";
export { clearRulesCatalogueFilters, filterRulesCompetitionCatalogue } from "../../src/features/competitions";

let navigationKeyboard: NavigationKeyboardController | null = null;

export function renderFixture(screen: string, theme: Theme, collapsed: boolean): void {
  const page: Page = screen === "profile" || screen.startsWith("team-") ? "teams" : screen as Page;
  const selected = screen === "profile" || screen.startsWith("team-") ? detail : null;
  const content = page === "dashboard" ? dashboardPage(state) : page === "teams"
    ? teamsPage(state, catalogue, selected, null, {active_only:true}, new Set(), [], [], null, [], [], [], null, "detail", false, screen !== "team-open", screen === "profile" ? "profile" : "directory")
    : page === "players" ? playersPage(state,references,playerCatalogue,null,{},null,new Set(),[],null,"detail",false,true,"directory")
    : page === "lineups" ? lineupsPage(state, references, [], null, builder, {home:[],away:[]}, [], null, match.id, false, false, "builder")
    : rulesPage(state, null, "catalog");
  document.documentElement.dataset.theme = theme;
  document.body.innerHTML = renderShell({state,page,theme,content,sidebarCollapsed:collapsed});
  enhanceSearchableSelects(document.body);
  navigationKeyboard?.destroy();
  navigationKeyboard = new NavigationKeyboardController(document.body);
  navigationKeyboard.start();
}

export { playerCatalogue as fixturePlayerPage } from "./fixtureData";
