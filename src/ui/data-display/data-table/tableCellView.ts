import { escapeHtml } from "../../../components/format";
import { buttonView } from "../../primitives/button";
import { statusBadgeView } from "../status-badge";
import type { TableCell } from "./dataTableTypes";

export function tableCellView(value: TableCell | undefined, disabled: boolean): string {
  if (value === null || value === undefined || typeof value !== "object") return escapeHtml(value ?? "—");
  if (value.kind === "status") return statusBadgeView(value.status);
  if (value.kind === "action") return buttonView({...value.button,disabled:disabled || value.button.disabled});
  return `<span class="ui-table-cell-copy" data-cell-type="${value.kind}"><span>${escapeHtml(value.text)}</span>${value.supporting === undefined ? "" : `<small>${escapeHtml(value.supporting)}</small>`}</span>`;
}
