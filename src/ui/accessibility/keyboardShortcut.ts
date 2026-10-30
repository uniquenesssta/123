import { EventScope } from "../../app/lifecycle";
export interface KeyboardShortcut {
  readonly key: string;
  readonly run: ()=>void;
  readonly enabled?: ()=>boolean;
  readonly allowInEditable?: boolean;
  readonly allowInModal?: boolean;
}
/** Explicit primary-modifier commands; lifecycle and input composition are always respected. */
export class KeyboardShortcutController {
  private events: EventScope|null=null;
  private closed=false;
  constructor(private readonly root: Document|HTMLElement,private readonly shortcuts: readonly KeyboardShortcut[]) {}
  start(): void {
    if(this.closed||this.events)return;
    this.events=new EventScope();
    this.events.listen(this.root,"keydown",event=>{
      if(!(event instanceof KeyboardEvent)||event.defaultPrevented||event.isComposing||event.keyCode===229||event.repeat||event.altKey||event.getModifierState("AltGraph")||!(event.ctrlKey||event.metaKey))return;
      const target=event.target instanceof Element?event.target:null;
      const document=this.root instanceof Document?this.root:this.root.ownerDocument;
      if(target?.closest('[hidden],[inert],[aria-disabled="true"]'))return;
      const shortcut=this.shortcuts.find(item=>item.key.toLowerCase()===event.key.toLowerCase());
      if(!shortcut || shortcut.enabled?.()===false)return;
      if(!shortcut.allowInEditable&&target?.closest('input,textarea,select,[contenteditable]:not([contenteditable="false"])'))return;
      if(!shortcut.allowInModal&&document.querySelector('dialog[open]'))return;
      event.preventDefault();shortcut.run();
    });
  }
  destroy(): void { if(this.closed)return;this.closed=true;this.events?.destroy();this.events=null; }
}
