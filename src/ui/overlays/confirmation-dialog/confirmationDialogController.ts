import { ConfirmationDialogState, type ConfirmationDialogRequest } from "./confirmationDialogState";
import { confirmationDialogView, confirmationDialogActions } from "./confirmationDialogView";
import { DialogController } from "../dialog";
import { mountConfirmationInput } from "../../forms/confirmation-input";
import { operationStateView } from "../../feedback/operation-state";

/** Host forwards events. A settled request owns its own close; no optimistic write replay. */
export class ConfirmationDialogController {
  private readonly state=new ConfirmationDialogState();
  private readonly dialog: DialogController;
  private gate: ReturnType<typeof mountConfirmationInput>|null=null;
  private input: HTMLInputElement|null=null;
  constructor(private readonly root: HTMLElement) {
    this.dialog=new DialogController(root,()=>root.querySelector<HTMLElement>('[data-action="reset-current-workspace"]')??root.querySelector<HTMLElement>('button:not(:disabled)'),()=>this.dismiss(),{canDismiss:()=>!this.state.busy});
  }
  get active(): boolean { return this.state.current!==null; }
  get busy(): boolean { return this.state.busy; }
  show(request: ConfirmationDialogRequest): void { if(this.active)return;this.state.begin(request);this.restore(); }
  restore(): void {
    const r=this.state.current;if(!r)return;
    const host=this.root.querySelector<HTMLElement>("[data-confirmation-host]");if(!host)return;
    if(this.dialog.element?.isConnected&&this.dialog.element.open)return;
    const draft=this.input?.value;
    this.gate?.destroy();this.dialog.restore(host,confirmationDialogView(r,this.state.busy,this.state.requiresReview));
    this.input=this.dialog.element?.querySelector<HTMLInputElement>("[data-confirmation-expected]")??null;
    if(this.input&&draft!==undefined)this.input.value=draft;
    this.paint();
    if(!this.state.busy)this.dialog.element?.querySelector<HTMLElement>("[autofocus]")?.focus();
  }
  handleClick(event: Event): boolean {
    if(!this.active)return false;
    if(this.dialog.handleBackdrop(event))return true;
    const button=event.target instanceof Element?event.target.closest<HTMLButtonElement>("[data-confirmation-action]"):null;
    if(!button||!this.dialog.element?.contains(button))return false;
    event.preventDefault();if(!button.disabled){if(button.dataset.confirmationAction==="confirm")void this.submit();else this.dismiss();}return true;
  }
  handleCancel(event: Event): boolean { return this.active&&this.dialog.handleCancel(event); }
  handleKeydown(event: Event): boolean { return this.active&&this.dialog.handleKeydown(event); }
  async submit(): Promise<void> {
    const r=this.state.current;
    if(!r||this.state.busy||this.state.requiresReview)return;
    if(r.typed&&(!this.input||this.input.matches(":disabled")||this.input.value.trim()!==r.typed.expected))return;
    const confirm=this.dialog.element?.querySelector<HTMLButtonElement>('[data-confirmation-action="confirm"]');
    if(confirm?.disabled)return;
    this.state.busy=true;this.state.error=null;const epoch=this.state.epoch;this.paint();
    try {
      const operation=r.onConfirm();
      if(operation&&typeof operation.then==="function")await operation;
      if(epoch!==this.state.epoch)return;
      this.finish(true);
    } catch(error) {
      if(epoch!==this.state.epoch)return;
      this.state.busy=false;
      this.state.requiresReview=r.tone!=="neutral";
      this.state.error=(r.formatError?.(error)??"操作未完成，请查看问题日志并核对当前结果。")+(this.state.requiresReview?" 请先关闭并核对实际结果，再重新发起操作；不要盲目重试。":"");
      this.paint();
      this.dialog.element?.querySelector<HTMLElement>('[data-confirmation-feedback]')?.focus();
    }
  }
  dismiss(): void { if(!this.state.busy)this.finish(true); }
  private paint(): void {
    const r=this.state.current,d=this.dialog.element;if(!r||!d)return;
    this.gate?.destroy();this.gate=null;d.setAttribute("aria-busy",String(this.state.busy));
    d.querySelector<HTMLElement>('.ui-dialog-actions')!.innerHTML=confirmationDialogActions(r,this.state.busy,this.state.requiresReview);
    const close=d.querySelector<HTMLButtonElement>('.ui-dialog-header button');if(close)close.disabled=this.state.busy;
    const meta=d.querySelector<HTMLElement>('#confirmation-meta');if(meta)meta.textContent=this.state.busy?"正在提交 · 请稍候":"请确认操作影响";
    const feedback=d.querySelector<HTMLElement>('[data-confirmation-feedback]')!;feedback.tabIndex=-1;
    feedback.innerHTML=this.state.busy?operationStateView({state:"loading",message:"正在处理，请勿重复提交"}):this.state.error?operationStateView({state:"error",title:"操作未完成",message:this.state.error}):"";
    if(this.input){this.input.disabled=this.state.busy;const button=d.querySelector<HTMLButtonElement>('[data-confirmation-action="confirm"]')!;this.gate=mountConfirmationInput(this.input,button,()=>!this.state.busy&&!this.state.requiresReview);}
    if(this.state.busy)d.focus({preventScroll:true});
  }
  private finish(restoreFocus: boolean): void { this.gate?.destroy();this.gate=null;this.input=null;this.state.take();this.dialog.close(restoreFocus); }
  destroy(): void { this.finish(false); }
}
