import { EventScope } from "../../../app/lifecycle";
import { DialogController } from "../dialog";
import { commandMenuView, commandResultsView, matchingCommands, type UiCommand, type CommandMenuState } from "./commandMenuView";
export class CommandMenuController {
  private readonly host: HTMLDivElement;private readonly events=new EventScope();private readonly dialog: DialogController;
  private commands: readonly UiCommand[]=[];private state: CommandMenuState={query:"",selected:null,busy:false,error:null};private epoch=0;private composing=false;
  constructor(private readonly document: Document,private readonly formatError: (error: unknown)=>string) {
    this.host=document.createElement("div");this.dialog=new DialogController(this.host,()=>document.querySelector<HTMLElement>('[data-action="open-command-menu"]'),()=>this.close(),{canDismiss:()=>!this.state.busy,dismissOnBackdrop:true});
    this.events.listen(this.host,"cancel",e=>{this.dialog.handleCancel(e);});
    this.events.listen(this.host,"click",e=>{if(this.dialog.handleBackdrop(e))return;const t=e.target instanceof Element?e.target:null;if(t?.closest('[data-command-close]'))this.close();const id=t?.closest<HTMLElement>('[data-command-id]')?.dataset.commandId;if(id)void this.execute(id);e.stopPropagation();});
    this.events.listen(this.host,"compositionstart",()=>{this.composing=true;});this.events.listen(this.host,"compositionend",()=>{this.composing=false;});
    this.events.listen(this.host,"input",e=>{if(e.target instanceof HTMLInputElement&&!this.state.busy){this.state.query=e.target.value;this.state.error=null;this.chooseFirst();this.paint();}});
    this.events.listen(this.host,"keydown",e=>this.keydown(e));
  }
  get active(): boolean { return !!this.dialog.element?.open; }
  open(commands: readonly UiCommand[]): void {
    if(this.events.signal.aborted||this.active||this.document.querySelector('dialog[open]'))return;
    this.document.body.append(this.host);this.commands=commands;this.state={query:"",selected:null,busy:false,error:null};this.epoch++;this.chooseFirst();
    this.dialog.restore(this.host,commandMenuView(this.filtered(),this.state));this.paint();
  }
  private filtered(): readonly UiCommand[] { return matchingCommands(this.commands,this.state.query); }
  private chooseFirst(): void { this.state.selected=this.filtered().find(c=>!c.disabled)?.id??null; }
  private paint(): void {
    const d=this.dialog.element;if(!d)return;d.setAttribute("aria-busy",String(this.state.busy));
    const input=d.querySelector<HTMLInputElement>('#command-search')!;input.disabled=this.state.busy;
    const close=d.querySelector<HTMLButtonElement>('[data-command-close]')!;close.disabled=this.state.busy;
    d.querySelector<HTMLElement>('[data-command-results]')!.innerHTML=commandResultsView(this.filtered(),this.state);
    if(this.state.selected&&!this.state.busy)input.setAttribute("aria-activedescendant","command-"+this.state.selected);else input.removeAttribute("aria-activedescendant");
    d.querySelector<HTMLElement>('[aria-selected="true"]')?.scrollIntoView({block:"nearest"});
    if(this.state.busy)d.focus();
  }
  private keydown(event: Event): void {
    if(!(event instanceof KeyboardEvent))return;
    if(this.composing||event.isComposing||event.keyCode===229)return;
    if(event.key==="Escape"){event.preventDefault();event.stopPropagation();this.close();return;}
    if(this.state.busy){if(event.key==="Tab")this.dialog.handleKeydown(event);return;}
    const commands=this.filtered().filter(c=>!c.disabled),index=commands.findIndex(c=>c.id===this.state.selected);
    let next: number|undefined;
    if(event.key==="ArrowDown")next=(index+1)%commands.length;if(event.key==="ArrowUp")next=(index-1+commands.length)%commands.length;
    if((event.ctrlKey||event.metaKey)&&event.key==="Home")next=0;if((event.ctrlKey||event.metaKey)&&event.key==="End")next=commands.length-1;
    if(next!==undefined){event.preventDefault();this.state.selected=commands[next]?.id??null;this.paint();return;}
    if(event.key==="Enter"&&event.target instanceof HTMLInputElement){event.preventDefault();if(this.state.selected)void this.execute(this.state.selected);return;}
    this.dialog.handleKeydown(event);
  }
  private async execute(id: string): Promise<void> {
    const command=this.filtered().find(c=>c.id===id);if(!command||command.disabled||this.state.busy)return;
    this.state.busy=true;this.state.error=null;const epoch=this.epoch;this.paint();
    try{await command.run();if(epoch!==this.epoch||this.events.signal.aborted)return;this.state.busy=false;this.close();}
    catch(error){if(epoch!==this.epoch||this.events.signal.aborted)return;this.state.busy=false;this.state.error=this.formatError(error);this.paint();this.dialog.element?.querySelector<HTMLInputElement>('#command-search')?.focus();}
  }
  close(): void { if(this.state.busy)return;this.epoch++;this.dialog.close();this.host.remove(); }
  destroy(): void { this.epoch++;this.events.destroy();this.dialog.close(false);this.host.remove(); }
}
