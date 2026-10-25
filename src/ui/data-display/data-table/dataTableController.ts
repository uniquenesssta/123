import { EventScope } from "../../../app/lifecycle";
import type { DataTableActions } from "./dataTableTypes";

/** Reports gestures only; host owns ordering, selection state and async lifetimes. */
export function mountDataTable(root: HTMLElement, actions: DataTableActions) {
  const events=new EventScope();
  const all=root.querySelector<HTMLInputElement>("[data-table-select-all]");
  if(all) all.indeterminate=root.dataset.tableMixed==="true";
  events.listen(root,"click",event=>{
    if(!(event.target instanceof Element)||root.getAttribute("aria-busy")==="true")return;
    const button=event.target.closest<HTMLButtonElement>("[data-table-sort]");
    if(!button||!root.contains(button)||button.matches(":disabled")||button.closest("[inert]"))return;
    actions.sort?.(button.dataset.tableSort!,button.dataset.sort==="ascending"?"descending":"ascending");
  });
  events.listen(root,"change",event=>{
    if(!(event.target instanceof HTMLInputElement)||root.getAttribute("aria-busy")==="true"||event.target.matches(":disabled")||event.target.closest("[inert]"))return;
    const input=event.target;
    if(input.hasAttribute("data-table-select-all"))actions.selectAll?.(input.checked);
    else if(input.dataset.tableSelect!==undefined)actions.select?.(input.dataset.tableSelect,input.checked);
  });
  return { destroy(){events.destroy();} };
}
