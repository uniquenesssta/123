import { EventScope } from "../../../app/lifecycle";
import { popoverView, type PopoverItem } from "./popoverView";
export interface PopoverRequest { anchor: HTMLElement; items: readonly PopoverItem[]; label: string; point?: { x: number; y: number }; id?: string }
/** A transient menu, portalled inside its modal owner when present. No business I/O. */
export class PopoverController {
  private menu: HTMLElement|null=null;private scope: EventScope|null=null;private request: PopoverRequest|null=null;
  get active(): boolean { return !!this.menu; }
  open(request: PopoverRequest): void {
    this.close(false);this.request=request;const doc=request.anchor.ownerDocument;
    const holder=doc.createElement("div");holder.innerHTML=popoverView(request.items,request.label);this.menu=holder.firstElementChild as HTMLElement;
    if(request.id)this.menu.id=request.id;
    const menu=this.menu,scope=this.scope=new EventScope();
    (request.anchor.closest('dialog[open]')??doc.body).append(menu);
    if(typeof menu.showPopover==="function")menu.showPopover();else menu.removeAttribute("popover");
    const rect=request.anchor.getBoundingClientRect(),point=request.point??{x:rect.left,y:rect.bottom+4},bounds=menu.getBoundingClientRect();
    const win=doc.defaultView!;
    menu.style.left=Math.max(8,Math.min(point.x,doc.documentElement.clientWidth-bounds.width-8))+"px";
    menu.style.top=Math.max(8,Math.min(point.y,doc.documentElement.clientHeight-bounds.height-8))+"px";
    request.anchor.setAttribute("aria-expanded","true");request.anchor.setAttribute("aria-haspopup","menu");
    const enabled=()=>Array.from(menu.querySelectorAll<HTMLButtonElement>('button:not(:disabled)'));
    (enabled()[0]??menu).focus({preventScroll:true});
    scope.listen(menu,"keydown",event=>{
      if(!(event instanceof KeyboardEvent))return;const buttons=enabled(),index=buttons.indexOf(doc.activeElement as HTMLButtonElement);
      if(event.key==="Escape"){event.preventDefault();event.stopPropagation();this.close();return;}
      if(event.key==="Tab"){event.preventDefault();event.stopPropagation();this.close();return;}
      let next: number|undefined;if(event.key==="ArrowDown")next=(index+1)%buttons.length;if(event.key==="ArrowUp")next=(index-1+buttons.length)%buttons.length;if(event.key==="Home")next=0;if(event.key==="End")next=buttons.length-1;
      if(next!==undefined){event.preventDefault();event.stopPropagation();buttons[next]?.focus();}
    });
    scope.listen(menu,"click",event=>{
      const button=event.target instanceof Element?event.target.closest<HTMLButtonElement>('button[data-popover-index]'):null;
      if(!button||button.disabled)return;const item=request.items[Number(button.dataset.popoverIndex)];
      // Keep the target connected until the application delegated click has consumed data-action.
      request.anchor.focus({preventScroll:true});if(item.onSelect)item.onSelect();queueMicrotask(()=>{if(this.menu===menu)this.close(false);});
    });
    scope.listen(doc,"pointerdown",event=>{if(event.target instanceof Node&&!menu.contains(event.target)&&!request.anchor.contains(event.target))this.close(false);},{capture:true});
    scope.requestAnimationFrame(()=>scope.listen(doc,"scroll",event=>{if(event.target instanceof Node&&menu.contains(event.target))return;this.close();},{capture:true}));
    scope.listen(win,"resize",()=>this.close());
  }
  close(restoreFocus=true): void {
    const anchor=this.request?.anchor;this.scope?.destroy();this.scope=null;
    if(this.menu?.matches(":popover-open"))this.menu.hidePopover();this.menu?.remove();this.menu=null;this.request=null;
    anchor?.setAttribute("aria-expanded","false");if(restoreFocus&&anchor?.isConnected)anchor.focus({preventScroll:true});
  }
  destroy(): void { this.close(false); }
}
