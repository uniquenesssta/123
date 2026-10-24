import { ConfirmationDialogState, type ConfirmationDialogRequest } from "./confirmationDialogState";
import { confirmationDialogView } from "./confirmationDialogView";
import { DialogController } from "../dialog";

/** Native modality inerts the document; explicit Tab boundaries retain focus in the controls. */
export class ConfirmationDialogController {
  private readonly state = new ConfirmationDialogState();
  private readonly dialog: DialogController;

  constructor(private readonly root: HTMLElement) {
    this.dialog = new DialogController(root, () => root.querySelector('[data-action="reset-current-workspace"]'), () => this.dismiss(false));
  }
  get active(): boolean { return this.state.current !== null; }

  show(request: ConfirmationDialogRequest): void {
    if (this.active) return;
    this.state.begin(request);
    this.restore();
  }

  restore(): void {
    const request = this.state.current;
    if (!request) return;
    const host = this.root.querySelector<HTMLElement>("[data-confirmation-host]");
    if (!host) return;
    this.dialog.restore(host, confirmationDialogView(request));
  }

  handleClick(event: Event): boolean {
    if (!(event.target instanceof Element) || !this.active) return false;
    const button = event.target.closest<HTMLButtonElement>("[data-confirmation-action]");
    if (!button || !this.dialog.element?.contains(button) || button.disabled) return false;
    this.dismiss(button.dataset.confirmationAction === "confirm");
    return true;
  }

  handleCancel(event: Event): boolean {
    return this.active && this.dialog.handleCancel(event);
  }

  handleKeydown(event: Event): boolean {
    return this.active && this.dialog.handleKeydown(event);
  }

  private dismiss(confirmed: boolean): void {
    const request = this.state.take();
    this.dialog.close();
    if (confirmed) request?.onConfirm();
  }

  destroy(): void { if (this.active || this.dialog.element) this.dismiss(false); }
}
