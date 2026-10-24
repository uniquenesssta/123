import { ConfirmationDialogState, type ConfirmationDialogRequest } from "./confirmationDialogState";
import { confirmationDialogView } from "./confirmationDialogView";

/** Native modality inerts the document; explicit Tab boundaries retain focus in the controls. */
export class ConfirmationDialogController {
  private readonly state = new ConfirmationDialogState();
  private dialog: HTMLDialogElement | null = null;
  private returnFocus: HTMLElement | null = null;

  constructor(private readonly root: HTMLElement) {}
  get active(): boolean { return this.state.current !== null; }

  show(request: ConfirmationDialogRequest): void {
    if (this.active) return;
    const active = this.root.ownerDocument.activeElement;
    this.returnFocus = active instanceof HTMLElement ? active : null;
    this.state.begin(request);
    this.restore();
  }

  restore(): void {
    const request = this.state.current;
    if (!request || (this.dialog?.isConnected && this.dialog.open)) return;
    const host = this.root.querySelector<HTMLElement>("[data-confirmation-host]");
    if (!host) return;
    if (this.dialog?.open) this.dialog.close();
    host.innerHTML = confirmationDialogView(request);
    this.dialog = host.querySelector<HTMLDialogElement>("dialog")!;
    this.dialog.showModal();
    this.dialog.querySelector<HTMLButtonElement>("[autofocus]")?.focus();
  }

  handleClick(event: Event): boolean {
    if (!(event.target instanceof Element) || !this.active) return false;
    const button = event.target.closest<HTMLButtonElement>("[data-confirmation-action]");
    if (!button || !this.dialog?.contains(button) || button.disabled) return false;
    this.dismiss(button.dataset.confirmationAction === "confirm");
    return true;
  }

  handleCancel(event: Event): boolean {
    if (event.target !== this.dialog || !this.active) return false;
    event.preventDefault();
    this.dismiss(false);
    return true;
  }

  handleKeydown(event: Event): boolean {
    if (!this.active || !(event instanceof KeyboardEvent)) return false;
    if (event.key === "Escape") return true;
    if (event.key !== "Tab" || !this.dialog) return false;
    const buttons = Array.from(this.dialog.querySelectorAll<HTMLButtonElement>("button:not(:disabled)"));
    const first = buttons[0], last = buttons.at(-1), active = this.root.ownerDocument.activeElement;
    if (event.shiftKey && active === first) { event.preventDefault(); last?.focus(); }
    if (!event.shiftKey && active === last) { event.preventDefault(); first?.focus(); }
    return true;
  }

  private dismiss(confirmed: boolean): void {
    const request = this.state.take();
    this.dialog?.close();
    this.dialog?.remove();
    this.dialog = null;
    const target = this.returnFocus?.isConnected ? this.returnFocus : this.root.querySelector<HTMLElement>('[data-action="reset-current-workspace"]');
    this.returnFocus = null;
    target?.focus({ preventScroll: true });
    if (confirmed) request?.onConfirm();
  }

  destroy(): void { if (this.active || this.dialog) this.dismiss(false); }
}
