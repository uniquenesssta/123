import type { Page, PlayerDetail, PlayerListPage, PlayerListQuery } from "../../types";
import { updatePaginationView } from "../../ui/navigation";
import { playerTableRows } from "./playerDirectoryView";
interface PlayerDirectoryState { page: Page; list: PlayerListPage | null; detail: PlayerDetail | null; query: PlayerListQuery; selectedIds: ReadonlySet<string>; pageNumber: number; loading: boolean }
export function setPlayerPageLoading(state: PlayerDirectoryState, active: boolean): void {

  const main = document.querySelector<HTMLElement>(
    '.app-shell[data-current-page="players"] .player-browser .entity-main',
  );
  if (!main) return;
  main.setAttribute("aria-busy", String(active));
  const pagination=main.querySelector<HTMLElement>(".ui-pagination");
  if(pagination)updatePaginationView(pagination,{page:state.pageNumber,hasMore:state.list?.has_more??false,busy:active,summary:state.list?`本页 ${state.list.items.length} 条结果`:"尚未载入"});
}
export function refreshPlayerPageDom(state: PlayerDirectoryState): boolean {
  if (state.page !== "players" || !state.list) return false;
  const tbody = document.querySelector<HTMLTableSectionElement>(
    ".entity-page-players .player-directory-table tbody",
  );
  const resultCount = document.querySelector<HTMLElement>(
    ".entity-page-players .player-list-toolbar .entity-list-actions > span",
  );
  const footer = document.querySelector<HTMLElement>(
    ".entity-page-players .entity-main-footer",
  );
  const pagination = footer?.querySelector<HTMLElement>(".ui-pagination") ?? null;
  const previousButton = footer?.querySelector<HTMLButtonElement>(
    '[data-action="previous-player-page"]',
  ) ?? null;
  const nextButton = footer?.querySelector<HTMLButtonElement>(
    '[data-action="next-player-page"]',
  ) ?? null;
  if (!tbody || !resultCount || !footer || !pagination || !previousButton || !nextButton) {
    return false;
  }

  tbody.innerHTML = playerTableRows(
    state.list,
    state.detail?.player.id ?? null,
    state.selectedIds,
  );
  resultCount.textContent = `${state.list.items.length} 条当前结果`;
  updatePaginationView(pagination,{page:state.pageNumber,hasMore:state.list.has_more,busy:state.loading,summary:`本页 ${state.list.items.length} 条结果`});
  const selectAll = document.querySelector<HTMLInputElement>("#player-select-all");
  if (selectAll) selectAll.checked = false;

  const ribbonItems = document.querySelectorAll<HTMLElement>(
    ".entity-page-players .task-context-ribbon .task-context-item",
  );
  const resultRibbon = ribbonItems.item(1);
  const ribbonValue = resultRibbon?.querySelector<HTMLElement>("strong") ?? null;
  const ribbonNote = resultRibbon?.querySelector<HTMLElement>("small") ?? null;
  if (ribbonValue) ribbonValue.textContent = `${state.list.items.length} 名球员`;
  if (ribbonNote) {
    const activeFilterCount = [
      state.query.team_id,
      state.query.position_code,
      state.query.availability_status,
      state.query.player_status,
    ].filter(Boolean).length;
    ribbonNote.textContent = `第 ${state.pageNumber} 页 · ${activeFilterCount} 项筛选`;
  }

  const sectionBadge = document.querySelector<HTMLElement>(
    '.entity-page-players [data-action="select-workspace-section"][data-section-id="directory"] .workspace-section-badge',
  );
  if (sectionBadge) sectionBadge.textContent = String(state.list.items.length);

  document.querySelector<HTMLElement>(
    ".entity-page-players .player-table-wrap",
  )?.scrollTo({ top: 0, behavior: "auto" });
  return true;
}
