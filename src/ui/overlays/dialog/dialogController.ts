/** Native modality, focus containment and restoration shared by application dialogs. */
export class DialogController {
  private dialog: HTMLDialogElement | null = null;
  private returnFocus: HTMLElement | null = null;

  constructor(private readonly root: HTMLElement, private readonly fallback: () => HTMLElement | null, private readonly onCancel: () => void) {}
  get element(): HTMLDialogElement | null { return this.dialog; }

  restore(host: HTMLElement, markup: string): void {
    if (this.dialog?.isConnected && this.dialog.open) return;
    if (!this.dialog) {
      const active = this.root.ownerDocument.activeElement;
      this.returnFocus = active instanceof HTMLElement ? active : null;
    }
    this.dialog?.close();
    host.innerHTML = markup;
    this.dialog = host.querySelector<HTMLDialogElement>("dialog")!;
    this.dialog.showModal();
    this.dialog.querySelector<HTMLElement>("[autofocus]")?.focus();
  }

  handleCancel(event: Event): boolean {
    if (event.target !== this.dialog || !this.dialog?.open) return false;
    event.preventDefault();
    this.onCancel();
    return true;
  }

  handleKeydown(event: Event): boolean {
    if (!this.dialog?.open || !(event instanceof KeyboardEvent)) return false;
    if (event.key === "Escape") return true;
    if (event.key !== "Tab") return false;
    const controls = Array.from(this.dialog.querySelectorAll<HTMLElement>('button:not(:disabled), a[href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex="0"]'))
      .filter(node => !node.closest("[hidden], [inert]") && node.getClientRects().length > 0);
    const first = controls[0], last = controls.at(-1), active = this.root.ownerDocument.activeElement;
    if (event.shiftKey && active === first) { event.preventDefault(); last?.focus(); }
    if (!event.shiftKey && active === last) { event.preventDefault(); first?.focus(); }
    return true;
  }

  close(restoreFocus = true): void {
    this.dialog?.close();
    this.dialog?.remove();
    this.dialog = null;
    const target = this.returnFocus?.isConnected ? this.returnFocus : this.fallback();
    this.returnFocus = null;
    if (restoreFocus) target?.focus({ preventScroll: true });
  }
}
