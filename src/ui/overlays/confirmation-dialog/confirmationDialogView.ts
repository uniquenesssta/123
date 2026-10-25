import { escapeHtml } from "../../../components/format";
import { iconSlot } from "../../icons/iconSlot";
import { buttonView } from "../../primitives/button";
import { iconButtonView } from "../../primitives/icon-button";
import type { ConfirmationDialogRequest } from "./confirmationDialogState";

/** Approved compact Danger/Default variant, bound to a synchronous local UI reset. */
export function confirmationDialogView(request: ConfirmationDialogRequest): string {
  return `<dialog class="ui-confirmation-dialog" aria-labelledby="confirmation-title" aria-describedby="confirmation-description">
    <header class="confirmation-dialog-header">${iconSlot("alert-circle", 16)}<div><h2 id="confirmation-title">${escapeHtml(request.title)}</h2><small>请确认操作影响</small></div>
      ${iconButtonView({label:"关闭对话框",icon:"close",attributes:{"data-confirmation-action":"cancel"}})}</header>
    <div class="confirmation-dialog-body"><p id="confirmation-description">${escapeHtml(request.description)}</p>
      <dl class="confirmation-dialog-facts">${request.facts.map(([label, value]) => `<div><dt>${escapeHtml(label)}</dt><dd>${escapeHtml(value)}</dd></div>`).join("")}</dl></div>
    <footer class="confirmation-dialog-actions">${buttonView({label:"取消",attributes:{"data-confirmation-action":"cancel",autofocus:true}})}${buttonView({label:request.confirmLabel,variant:"danger",attributes:{"data-confirmation-action":"confirm"}})}</footer>
  </dialog>`;
}
