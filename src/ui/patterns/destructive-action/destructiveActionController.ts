import { EventScope } from "../../../app/lifecycle";
import { ConfirmationDialogController, type ConfirmationDialogRequest } from "../../overlays/confirmation-dialog";
/** Transient confirmation host; no repository, undo claim or second pending-action state. */
export class DestructiveActionController {
  private host: HTMLDivElement | null=null;
  private events: EventScope | null=null;
  private confirmation: ConfirmationDialogController | null=null;
  get active(): boolean { return this.confirmation?.active ?? false; }
  get busy(): boolean { return this.confirmation?.busy ?? false; }
  show(request: ConfirmationDialogRequest): void {
    if(!this.confirmation){
      const host=this.host=document.createElement("div");
      host.innerHTML='<div data-confirmation-host data-ui-pattern="destructive-action"></div>';
      document.body.append(host);
      const dialog=this.confirmation=new ConfirmationDialogController(host);
      const events=this.events=new EventScope();
      events.listen(host,"click",event=>{if(dialog.handleClick(event))event.stopPropagation();});
      events.listen(host,"cancel",event=>{if(dialog.handleCancel(event))event.stopPropagation();},{capture:true});
      events.listen(host,"keydown",event=>{if(dialog.handleKeydown(event))event.stopPropagation();},{capture:true});
    }
    this.confirmation.show(request);
  }
  restore(): void { this.confirmation?.restore(); }
  dismiss(): void { this.confirmation?.dismiss(); }
  async submit(): Promise<void> { await this.confirmation?.submit(); }
  destroy(): void {
    this.confirmation?.destroy();this.confirmation=null;
    this.events?.destroy();this.events=null;
    this.host?.remove();this.host=null;
  }
}
