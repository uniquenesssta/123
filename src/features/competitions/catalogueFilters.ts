import { fitCatalogueRegions } from "./catalogueRegionLayout";
import { refreshSearchableSelects } from "../../components/searchableSelect";

/** Reset catalogue UI filters without changing records, bindings or the current workspace. */
export function clearRulesCatalogueFilters(refresh: () => void, root: ParentNode = document): void {
  const search = root.querySelector<HTMLInputElement>("#rules-competition-search");
  const kind = root.querySelector<HTMLSelectElement>("#rules-competition-kind");
  if (search) search.value = "";
  if (kind) {
    kind.value = "";
    refreshSearchableSelects(kind);
  }
  root.querySelectorAll<HTMLElement>("[data-rules-scope]").forEach(button => button.classList.toggle("active", button.dataset.rulesScope === ""));
  root.querySelectorAll<HTMLElement>("[data-rules-region]").forEach(button => button.classList.toggle("active", button.dataset.rulesRegion === ""));
  refresh();
}

export function filterRulesCompetitionCatalogue(): void {
  const directory = document.querySelector<HTMLElement>("[data-rules-directory]");
  if (!directory) return;
  const query = (document.querySelector<HTMLInputElement>("#rules-competition-search")?.value ?? "").trim().toLowerCase();
  const kind = document.querySelector<HTMLSelectElement>("#rules-competition-kind")?.value ?? "";
  const rows = Array.from(directory.querySelectorAll<HTMLElement>("[data-rules-competition-row]"));
  const baseMatches = (row: HTMLElement): boolean =>
    (!kind || row.dataset.kind === kind) &&
    (!query || (row.dataset.search ?? "").includes(query));

  let scope = directory.querySelector<HTMLElement>("[data-rules-scope].active")?.dataset.rulesScope ?? "";
  const scopeButtons = Array.from(directory.querySelectorAll<HTMLButtonElement>("[data-rules-scope]"));
  for (const button of scopeButtons) {
    const value = button.dataset.rulesScope ?? "";
    const count = rows.filter((row) => baseMatches(row) && (!value || row.dataset.scope === value)).length;
    const badge = button.querySelector<HTMLElement>("span");
    if (badge) badge.textContent = String(count);
    button.disabled = value !== "" && count === 0;
  }
  const activeScopeButton = scopeButtons.find((button) => button.dataset.rulesScope === scope);
  if (!activeScopeButton || activeScopeButton.disabled) {
    scope = "";
    scopeButtons.forEach((button) => button.classList.toggle("active", (button.dataset.rulesScope ?? "") === ""));
  }

  let region = directory.querySelector<HTMLElement>("[data-rules-region].active")?.dataset.rulesRegion ?? "";
  const regionButtons = Array.from(directory.querySelectorAll<HTMLButtonElement>("[data-rules-region]"));
  for (const button of regionButtons) {
    const value = button.dataset.rulesRegion ?? "";
    const count = rows.filter((row) =>
      baseMatches(row) &&
      (!scope || row.dataset.scope === scope) &&
      (!value || row.dataset.region === value),
    ).length;
    const badge = button.querySelector<HTMLElement>("span");
    if (badge) badge.textContent = String(count);
    button.disabled = value !== "" && count === 0;
    button.hidden = button.disabled;
  }
  const activeRegionButton = regionButtons.find((button) => button.dataset.rulesRegion === region);
  if (!activeRegionButton || activeRegionButton.disabled || activeRegionButton.hidden) {
    region = "";
    regionButtons.forEach((button) => button.classList.toggle("active", (button.dataset.rulesRegion ?? "") === ""));
  }

  let visible = 0;
  for (const row of rows) {
    const matches =
      baseMatches(row) &&
      (!scope || row.dataset.scope === scope) &&
      (!region || row.dataset.region === region);
    row.hidden = !matches;
    if (matches) visible += 1;
  }
  const count = directory.querySelector<HTMLElement>("#rules-visible-count");
  if (count) count.textContent = String(visible);
  const empty = directory.querySelector<HTMLElement>("#rules-empty-filter");
  if (empty) empty.hidden = visible !== 0;
  fitCatalogueRegions(directory);
}

export function selectRulesDirectoryLevel(target: HTMLElement): boolean {
  const scopeButton = target.closest<HTMLElement>("[data-rules-scope]");
  const regionButton = target.closest<HTMLElement>("[data-rules-region]");
  const directory = target.closest<HTMLElement>("[data-rules-directory]");
  if (!directory || (!scopeButton && !regionButton)) return false;
  if (scopeButton) {
    directory.querySelectorAll<HTMLElement>("[data-rules-scope]").forEach((item) => item.classList.toggle("active", item === scopeButton));
    const scope = scopeButton.dataset.rulesScope ?? "";
    let firstVisibleRegion: HTMLElement | null = null;
    directory.querySelectorAll<HTMLElement>("[data-rules-region]").forEach((item) => {
      const scopes = (item.dataset.regionScopes ?? "").split(",").filter(Boolean);
      const available = !scope || !item.dataset.rulesRegion || scopes.includes(scope);
      item.hidden = !available;
      if (available && !firstVisibleRegion) firstVisibleRegion = item;
      item.classList.remove("active");
    });
    const allRegion = directory.querySelector<HTMLElement>('[data-rules-region=""]');
    (allRegion && !allRegion.hidden ? allRegion : firstVisibleRegion)?.classList.add("active");
  } else if (regionButton) {
    directory.querySelectorAll<HTMLElement>("[data-rules-region]").forEach((item) => item.classList.toggle("active", item === regionButton));
  }
  filterRulesCompetitionCatalogue();
  return true;
}
