import { EventScope } from "../lifecycle";
import { WorkspacePanelController } from "./workspacePanelController";
import { ConfirmationDialogController, type ConfirmationDialogRequest } from "../../ui/overlays/confirmation-dialog";
export type ModalConfirmationOptions=Pick<ConfirmationDialogRequest,"tone"|"typed">;
/** Coordinates existing detail history and a separate transient native confirmation host. */
export class ModalController {
  private readonly workspace=new WorkspacePanelController();private host: HTMLDivElement|null=null;private events: EventScope|null=null;private confirmation: ConfirmationDialogController|null=null;
  constructor(private readonly formatError?: (error: unknown)=>string) {}
  private ensureConfirmation(): ConfirmationDialogController {
    if(this.confirmation)return this.confirmation;
    const host=this.host=document.createElement("div");host.innerHTML='<div data-confirmation-host></div>';document.body.append(host);
    const c=this.confirmation=new ConfirmationDialogController(host),events=this.events=new EventScope();
    events.listen(host,"click",e=>{if(c.handleClick(e))e.stopPropagation();});
    events.listen(host,"cancel",e=>{if(c.handleCancel(e))e.stopPropagation();},{capture:true});
    events.listen(host,"keydown",e=>{if(c.handleKeydown(e))e.stopPropagation();},{capture:true});return c;
  }
  showHtml(...args: Parameters<WorkspacePanelController["showHtml"]>): void { this.workspace.showHtml(...args); }
  show(...args: Parameters<WorkspacePanelController["show"]>): void { this.workspace.show(...args); }
  capture(): void { this.workspace.capture(); }
  restore(): void { this.workspace.restore();this.confirmation?.restore(); }
  close(): void { if(this.confirmation?.busy)return;if(this.confirmation?.active)this.confirmation.dismiss();else this.workspace.close(); }
  back(): void { if(!this.confirmation?.active)this.workspace.back(); }
  forward(): void { if(!this.confirmation?.active)this.workspace.forward(); }
  reset(): void { this.confirmation?.destroy();this.confirmation=null;this.events?.destroy();this.events=null;this.host?.remove();this.host=null;this.workspace.reset(); }
  confirm(title: string,description: string,facts: Array<[string,string]>,confirmLabel: string,action: ()=>Promise<void>,options: ModalConfirmationOptions={}): void {
    const identity=this.workspace.identity;
    this.ensureConfirmation().show({title,description,facts,confirmLabel,...options,formatError:this.formatError,onConfirm:async()=>{await action();if(identity&&identity===this.workspace.identity)this.workspace.close();}});
  }
  async runPendingAction(): Promise<void> { await this.confirmation?.submit(); }
}
