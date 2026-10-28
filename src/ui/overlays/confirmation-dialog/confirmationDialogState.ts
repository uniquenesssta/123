import type { ConfirmationInputOptions } from "../../forms/confirmation-input";
export interface ConfirmationDialogRequest {
  readonly title: string; readonly description: string; readonly facts: readonly (readonly [string,string])[];
  readonly confirmLabel: string; readonly tone?: "neutral"|"danger"; readonly typed?: ConfirmationInputOptions;
  readonly onConfirm: () => void|Promise<void>; readonly formatError?: (error: unknown)=>string;
}
/** Transient request; never persisted in workspace history. Epoch invalidates late completion. */
export class ConfirmationDialogState {
  private request: ConfirmationDialogRequest|null=null;
  busy=false; error: string|null=null; requiresReview=false; epoch=0;
  get current(): ConfirmationDialogRequest|null { return this.request; }
  begin(request: ConfirmationDialogRequest): void { if(!this.request){this.request=request;this.busy=false;this.error=null;this.requiresReview=false;this.epoch++;} }
  take(): ConfirmationDialogRequest|null { const r=this.request;this.request=null;this.busy=false;this.error=null;this.requiresReview=false;this.epoch++;return r; }
}
