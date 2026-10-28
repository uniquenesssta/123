export interface DialogPolicy { canDismiss?: () => boolean; dismissOnBackdrop?: boolean }
/** Owns modality and focus only. Request state and the close decision remain with callers. */
export class DialogController {
  private dialog: HTMLDialogElement|null=null;
  private returnFocus: HTMLElement|null=null;
  constructor(private readonly root: HTMLElement, private readonly fallback: ()=>HTMLElement|null, private readonly onCancel: ()=>void, private readonly policy: DialogPolicy={}) {}
  get element(): HTMLDialogElement|null { return this.dialog; }
  restore(host: HTMLElement,markup: string): void {
    if(this.dialog?.isConnected&&this.dialog.open)return;
    if(!this.dialog){const active=this.root.ownerDocument.activeElement;this.returnFocus=active instanceof HTMLElement?active:null;}
    this.dialog?.close();host.innerHTML=markup;this.dialog=host.querySelector<HTMLDialogElement>("dialog");
    if(!this.dialog)throw new Error("Dialog markup requires a native dialog");
    this.dialog.showModal();(this.dialog.querySelector<HTMLElement>("[autofocus]")??this.dialog).focus({preventScroll:true});
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
    if(event.key!=="Tab")return false;
    const controls=Array.from(this.dialog.querySelectorAll<HTMLElement>('button, a[href], input, select, textarea, [tabindex]')).filter(n=>n.tabIndex>=0&&!n.matches(":disabled")&&!n.closest("[hidden], [inert]")&&n.getClientRects().length>0);
    const first=controls[0],last=controls.at(-1),active=this.root.ownerDocument.activeElement;
    if(!first){event.preventDefault();this.dialog.focus();}
    else if(!this.dialog.contains(active)||(event.shiftKey&&(active===first||active===this.dialog))){event.preventDefault();last?.focus();}
    else if(!event.shiftKey&&(active===last||active===this.dialog)){event.preventDefault();first.focus();}
    return true;
  }
  close(restoreFocus=true): void {
    this.dialog?.close();this.dialog?.remove();this.dialog=null;
    const target=this.returnFocus?.isConnected&&!this.returnFocus.closest('[inert],[hidden]')?this.returnFocus:this.fallback();this.returnFocus=null;
    if(restoreFocus)target?.focus({preventScroll:true});
  }
}
