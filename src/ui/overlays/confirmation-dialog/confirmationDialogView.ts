import { escapeHtml } from "../../../components/format";
import { dialogView } from "../dialog";
import { formActionsView } from "../../forms/form-actions";
import { confirmationInputView } from "../../forms/confirmation-input";
import type { ConfirmationDialogRequest } from "./confirmationDialogState";
export function confirmationDialogActions(request: ConfirmationDialogRequest,busy=false,blocked=false): string {
  return formActionsView({busy,cancel:{label:"取消",attributes:{"data-confirmation-action":"cancel",autofocus:true}},confirm:{label:request.confirmLabel,variant:request.tone==="neutral"?"primary":"danger",disabled:blocked||!!request.typed,attributes:{"data-confirmation-action":"confirm",id:request.typed?.confirmButtonId??"confirmation-submit"}}});
}
export function confirmationDialogView(request: ConfirmationDialogRequest,busy=false,blocked=false): string {
  const body=`<p id="confirmation-description">${escapeHtml(request.description)}</p><dl class="confirmation-dialog-facts">${request.facts.map(([label,value])=>`<div><dt>${escapeHtml(label)}</dt><dd>${escapeHtml(value)}</dd></div>`).join("")}</dl>${request.typed?confirmationInputView({...request.typed,disabled:busy}):""}<div data-confirmation-feedback></div>`;
  return dialogView({title:request.title,titleId:"confirmation-title",description:busy?"正在提交 · 请稍候":"请确认操作影响",descriptionId:"confirmation-meta",body,actions:confirmationDialogActions(request,busy,blocked),busy,className:"ui-confirmation-dialog",closeAttributes:{"data-confirmation-action":"cancel"}}).replace('aria-describedby="confirmation-meta"','aria-describedby="confirmation-description"');
}
