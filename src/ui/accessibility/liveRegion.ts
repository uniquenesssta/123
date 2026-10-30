import { EventScope } from "../../app/lifecycle";
/** Plain-text notices, including repeated messages, with disposal of delayed announcements. */
export class LiveRegion {
  private readonly events=new EventScope();
  private timer:number|null=null;
  constructor(private readonly region: HTMLElement) {
    region.setAttribute("role","status");region.setAttribute("aria-live","polite");region.setAttribute("aria-atomic","true");
  }
  announce(message: string): void {
    if(this.events.signal.aborted)return;
    if(this.timer!==null)this.events.clearTimeout(this.timer);this.timer=null;
    if(message&&this.region.textContent===message){
      this.region.textContent="";
      this.timer=this.events.setTimeout(()=>{this.timer=null;this.region.textContent=message;},0);
    }else this.region.textContent=message;
  }
  destroy(): void { this.events.destroy();this.timer=null;this.region.textContent=""; }
}
