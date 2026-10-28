import { PopoverController, type PopoverItem } from "../../ui/overlays/popover";
/** Domain labels/identities stay here; the shared Popover owns focus, placement and teardown. */
export class ContextMenuAdapter {
  private readonly popover=new PopoverController();
  open(event: MouseEvent,target: HTMLElement): void {
    const kind=target.dataset.contextKind;if(kind!=="run"&&kind!=="lineup"&&kind!=="match")return;
    event.preventDefault();const item: PopoverItem=kind==="run"?{id:"hide-run",label:"从历史列表删除",description:"底层运行、快照和收敛数据保留",icon:"history",attributes:{"data-action":"request-hide-run-history","data-run-id":target.dataset.runId??"","data-run-label":target.dataset.runLabel??"未命名推演"}}:kind==="lineup"?{id:"remove-lineup",label:"删除或归档阵容版本",description:"已引用版本保留模型血缘，未引用版本永久删除",danger:true,icon:"alert-circle",attributes:{"data-action":"request-remove-lineup-history","data-lineup-id":target.dataset.lineupId??"","data-lineup-label":target.dataset.lineupLabel??"未命名阵容"}}:{id:"delete-match",label:"删除比赛",description:"将同时删除该比赛的阵容和复盘数据",danger:true,icon:"alert-circle",attributes:{"data-action":"request-delete-match","data-match-id":target.dataset.matchId??"","data-match-label":target.dataset.matchLabel??"未命名比赛"}};
    const anchor=target.matches('button,[tabindex],a[href]')?target:target.querySelector<HTMLElement>('button:not(:disabled),[tabindex="0"],a[href]')??target;
    if(anchor.tabIndex<0)anchor.tabIndex=-1;
    this.popover.open({anchor,items:[item],label:"记录操作",id:"app-context-menu",point:{x:event.clientX,y:event.clientY}});
  }
  close(): void { this.popover.close(); }
  destroy(): void { this.popover.destroy(); }
}
