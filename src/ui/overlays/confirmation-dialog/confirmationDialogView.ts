import { escapeHtml } from "../../../components/format";
import { iconSlot } from "../../icons/iconSlot";
import type { ConfirmationDialogRequest } from "./confirmationDialogState";

/** Approved compact Danger/Default variant, bound to a synchronous local UI reset. */
export function confirmationDialogView(request: ConfirmationDialogRequest): string {
  return `<dialog class="ui-confirmation-dialog" aria-labelledby="confirmation-title" aria-describedby="confirmation-description">
    <header class="confirmation-dialog-header">${iconSlot("alert-circle", 16)}<div><h2 id="confirmation-title">${escapeHtml(request.title)}</h2><small>请确认操作影响</small></div>
      <button type="button" class="ui-icon-button" data-confirmation-action="cancel" aria-label="关闭对话框" title="关闭对话框">${iconSlot("close", 16)}</button></header>
    <div class="confirmation-dialog-body"><p id="confirmation-description">${escapeHtml(request.description)}</p>
      <dl class="confirmation-dialog-facts">${request.facts.map(([label, value]) => `<div><dt>${escapeHtml(label)}</dt><dd>${escapeHtml(value)}</dd></div>`).join("")}</dl></div>
    <footer class="confirmation-dialog-actions"><button type="button" class="ui-button ui-button-secondary" data-confirmation-action="cancel" autofocus>取消</button><button type="button" class="ui-button ui-button-danger" data-confirmation-action="confirm">${escapeHtml(request.confirmLabel)}</button></footer>
  </dialog>`;
}
