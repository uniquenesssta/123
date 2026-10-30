import { FocusManager, focusElement, trapFocus } from "../../accessibility";
export interface DialogPolicy { canDismiss?: () => boolean; dismissOnBackdrop?: boolean }
/** Owns modality and focus only. Request state and the close decision remain with callers. */
export class DialogController {
  private dialog: HTMLDialogElement|null=null;
  private readonly focus: FocusManager;
  constructor(root: HTMLElement, private readonly fallback: ()=>HTMLElement|null, private readonly onCancel: ()=>void, private readonly policy: DialogPolicy={}) { this.focus=new FocusManager(root.ownerDocument); }
  get element(): HTMLDialogElement|null { return this.dialog; }
  restore(host: HTMLElement,markup: string): void {
    if(this.dialog?.isConnected&&this.dialog.open)return;
    if(!this.dialog)this.focus.capture();
    this.dialog?.close();host.innerHTML=markup;this.dialog=host.querySelector<HTMLDialogElement>("dialog");
    if(!this.dialog)throw new Error("Dialog markup requires a native dialog");
    this.dialog.showModal();focusElement(this.dialog.querySelector<HTMLElement>("[autofocus]")??this.dialog);
  }
  handleCancel(event: Event): boolean {
    if(event.target!==this.dialog||!this.dialog?.open)return false;
    event.preventDefault();if(this.policy.canDismiss?.()!==false)this.onCancel();return true;
  }
  handleBackdrop(event: Event): boolean {
    if(event.target!==this.dialog||!this.dialog?.open)return false;
    if(event instanceof MouseEvent){const r=this.dialog.getBoundingClientRect();if(event.clientX>=r.left&&event.clientX<=r.right&&event.clientY>=r.top&&event.clientY<=r.bottom)return false;}
    if(this.policy.dismissOnBackdrop&&this.policy.canDismiss?.()!==false)this.onCancel();return true;
  }
  handleKeydown(event: Event): boolean {
    if(!this.dialog?.open||!(event instanceof KeyboardEvent)||event.defaultPrevented)return false;
    if(event.key==="Escape") {
      const target=event.target instanceof Element?event.target:null;
      if(target?.closest('[data-ui-popover]')||target?.closest('[role="combobox"][aria-expanded="true"]'))return false;
      // Native cancel fires after keydown, including when dismissal is locked by a request.
      return true;
    }
    return trapFocus(event,this.dialog);
  }

  close(restoreFocus=true): void {
    this.dialog?.close();this.dialog?.remove();this.dialog=null;
    if(restoreFocus)this.focus.restore(this.fallback);else this.focus.clear();
  }
}
