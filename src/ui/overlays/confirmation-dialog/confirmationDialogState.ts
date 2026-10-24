export interface ConfirmationDialogRequest {
  readonly title: string;
  readonly description: string;
  readonly facts: readonly (readonly [string, string])[];
  readonly confirmLabel: string;
  readonly onConfirm: () => void;
}

/** Transient UI request only; never placed in detail history or persisted workspace data. */
export class ConfirmationDialogState {
  private request: ConfirmationDialogRequest | null = null;
  get current(): ConfirmationDialogRequest | null { return this.request; }
  begin(request: ConfirmationDialogRequest): void { if (!this.request) this.request = request; }
  take(): ConfirmationDialogRequest | null { const request = this.request; this.request = null; return request; }
}
