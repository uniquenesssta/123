import { escapeHtml } from "../../../components/format";
import { checkboxView } from "../../primitives/checkbox";
import { spinnerView } from "../../primitives/spinner";
import { iconSlot } from "../../icons/iconSlot";
import { tableCellView } from "./tableCellView";
import type { DataTableOptions } from "./dataTableTypes";

/** Native table, host-supplied row order/selection. Pagination remains separate. */
export function dataTableView(o: DataTableOptions): string {
  if (!o.label.trim() || !o.columns.length) throw new Error("Data Table requires a name and columns");
  const columns=o.columns.map(c=>c.key),ids=o.rows.map(r=>r.id);
  if (columns.some(k=>!k)||new Set(columns).size!==columns.length||ids.some(k=>!k)||new Set(ids).size!==ids.length) throw new Error("Table keys must be non-empty and unique");
  for (const c of o.columns) if(c.width!==undefined&&(!Number.isFinite(c.width)||c.width<=0)) throw new Error("Column widths must be finite and positive");
  if(o.columns.filter(c=>c.sort&&c.sort!=="none").length>1)throw new Error("Only one sort column can be active");
  const available=o.rows.filter(r=>!r.disabled), selected=available.filter(r=>r.selected).length;
  const mixed=selected>0&&selected<available.length;
  const header=o.columns.map(c=>`<th scope="col" data-align="${c.type ?? "Text"}" ${c.sortable ? `aria-sort="${c.sort ?? "none"}"` : ""}>${c.sortable ? `<button type="button" class="ui-table-sort" data-table-sort="${escapeHtml(c.key)}" data-sort="${c.sort ?? "none"}" aria-label="按${escapeHtml(c.label)}排序" ${o.busy ? "disabled" : ""}>${escapeHtml(c.label)}${c.sort&&c.sort!=="none" ? `<span class="ui-table-sort-icon">${iconSlot(c.sort==="ascending" ? "table-sort-up" : "table-sort-down",16)}</span>` : ""}</button>` : escapeHtml(c.label)}</th>`).join("");
  const rows=o.rows.map(r=>`<tr data-table-row="${escapeHtml(r.id)}" data-selected="${!!r.selected}" data-disabled="${!!(r.disabled||o.busy)}">${o.selection ? `<td class="ui-table-selection">${checkboxView({label:"选择行 "+r.id,showLabel:false,checked:r.selected,disabled:r.disabled||o.busy,attributes:{"data-table-select":r.id}})}</td>` : ""}${o.columns.map(c=>`<td data-align="${c.type ?? "Text"}">${tableCellView(r.cells[c.key],!!(o.busy||r.disabled))}</td>`).join("")}</tr>`).join("");
  const colspan=o.columns.length+Number(!!o.selection);
  return `<div class="ui-data-table" data-ui-display="data-table" role="region" aria-label="${escapeHtml(o.label)}" tabindex="0" aria-busy="${!!o.busy}" data-table-mixed="${mixed}"><table><caption>${escapeHtml(o.label)}</caption><colgroup>${o.selection?'<col class="ui-table-selection-col">':""}${o.columns.map(c=>`<col ${c.width===undefined ? "" : `style="width:${c.width}px"`}>`).join("")}</colgroup><thead><tr>${o.selection ? `<th class="ui-table-selection" scope="col">${checkboxView({label:"选择全部行",showLabel:false,checked:available.length>0&&selected===available.length,disabled:o.busy||!available.length,attributes:{"data-table-select-all":"true"}})}</th>` : ""}${header}</tr></thead><tbody>${rows || `<tr><td colspan="${colspan}" class="ui-table-empty">${o.busy ? '<span>正在加载…</span>' : `<strong>${escapeHtml(o.emptyTitle??"暂无记录")}</strong>${o.emptyDescription ? `<p>${escapeHtml(o.emptyDescription)}</p>` : ""}`}</td></tr>`}</tbody></table>${o.busy ? `<div class="ui-table-loading" role="status">${spinnerView(16)}<span>${escapeHtml(o.loadingLabel??"正在加载…")}</span></div>` : ""}</div>`;
}
