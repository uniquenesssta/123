import { escapeHtml } from "../../../components/format";
import { iconButtonView } from "../../primitives/icon-button";
import type { PrimitiveAttributes } from "../../primitives/controlAttributes";
export interface DialogViewOptions { id?: string; title: string; description?: string; titleId?: string; descriptionId?: string; body: string; actions?: string; width?: 480|640; className?: string; bodyClassName?: string; busy?: boolean; closeAttributes?: PrimitiveAttributes; closeAction?: string }
export function dialogView(o: DialogViewOptions): string {
  const titleId=o.titleId??"dialog-title",descriptionId=o.descriptionId??"dialog-description";
  return `<dialog class="ui-dialog ${escapeHtml(o.className??"")}" ${o.id?`id="${escapeHtml(o.id)}"`:""} data-dialog-width="${o.width??480}" tabindex="-1" aria-modal="true" aria-labelledby="${escapeHtml(titleId)}" ${o.description?`aria-describedby="${escapeHtml(descriptionId)}"`:""} aria-busy="${!!o.busy}"><header class="ui-dialog-header"><div><h2 id="${escapeHtml(titleId)}">${escapeHtml(o.title)}</h2>${o.description?`<p id="${escapeHtml(descriptionId)}">${escapeHtml(o.description)}</p>`:""}</div>${iconButtonView({label:"关闭对话框",icon:"close",action:o.closeAction,disabled:o.busy,attributes:o.closeAttributes})}</header><div class="ui-dialog-body ${escapeHtml(o.bodyClassName??"")}">${o.body}</div>${o.actions?`<footer class="ui-dialog-actions">${o.actions}</footer>`:""}</dialog>`;
}
