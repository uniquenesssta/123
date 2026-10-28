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
