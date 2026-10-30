/** Shared DOM focus eligibility; business draft and route identities stay with their owners. */
export function canReceiveFocus(element: HTMLElement | null | undefined): element is HTMLElement {
  if(!element?.isConnected || element.matches(":disabled") || element.closest('[hidden],[inert],[aria-hidden="true"]'))return false;
  const view=element.ownerDocument.defaultView;
  if(!view || !element.getClientRects().length)return false;
  const style=view.getComputedStyle(element);
  if(style.visibility==="hidden" || style.visibility==="collapse")return false;
  const modals=Array.from(element.ownerDocument.querySelectorAll<HTMLDialogElement>('dialog[open]')).filter(dialog=>dialog.matches(":modal"));
  const modal=modals.find(dialog=>dialog.contains(element.ownerDocument.activeElement)) ?? modals.at(-1);
  return !modal || modal.contains(element);
}

export function focusElement(element: HTMLElement | null | undefined, options: FocusOptions={preventScroll:true}): boolean {
  if(!canReceiveFocus(element))return false;
  element.focus(options);
  return element.ownerDocument.activeElement===element;
}

/** One transient return target per overlay, never a global or persistent focus owner. */
export class FocusManager {
  private previous: HTMLElement | null=null;
  constructor(private readonly document: Document) {}
  capture(): void {
    const active=this.document.activeElement;
    this.previous=active instanceof HTMLElement && active!==this.document.body && active!==this.document.documentElement ? active : null;
  }
  restore(fallback: ()=>HTMLElement|null): boolean {
    const previous=this.previous;this.previous=null;
    return focusElement(previous) || focusElement(fallback());
  }
  clear(): void { this.previous=null; }
}
