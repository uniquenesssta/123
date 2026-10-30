import { WorkspacePanelController } from "./workspacePanelController";
import type { ConfirmationDialogRequest } from "../../ui/overlays/confirmation-dialog";
import { DestructiveActionController } from "../../ui/patterns/destructive-action";
export type ModalConfirmationOptions=Pick<ConfirmationDialogRequest,"tone"|"typed">;
/** Coordinates existing detail history and a separate transient native confirmation host. */
export class ModalController {
  private readonly workspace=new WorkspacePanelController();
  private readonly confirmation=new DestructiveActionController();
  constructor(private readonly formatError?: (error: unknown)=>string) {}
  showHtml(...args: Parameters<WorkspacePanelController["showHtml"]>): void { this.workspace.showHtml(...args); }
  show(...args: Parameters<WorkspacePanelController["show"]>): void { this.workspace.show(...args); }
  capture(): void { this.workspace.capture(); }
  restore(): void { this.workspace.restore();this.confirmation?.restore(); }
  close(): void { if(this.confirmation?.busy)return;if(this.confirmation?.active)this.confirmation.dismiss();else this.workspace.close(); }
  back(): void { if(!this.confirmation?.active)this.workspace.back(); }
  forward(): void { if(!this.confirmation?.active)this.workspace.forward(); }
  reset(): void { this.confirmation.destroy();this.workspace.reset(); }
  confirm(title: string,description: string,facts: Array<[string,string]>,confirmLabel: string,action: ()=>Promise<void>,options: ModalConfirmationOptions={}): void {
    const identity=this.workspace.identity;
    this.confirmation.show({title,description,facts,confirmLabel,...options,formatError:this.formatError,onConfirm:async()=>{await action();if(identity&&identity===this.workspace.identity)this.workspace.close();}});
  }
  async runPendingAction(): Promise<void> { await this.confirmation?.submit(); }
}
