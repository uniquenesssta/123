import { canReceiveFocus, focusElement } from "./focusManager";
const focusSelector='button,input:not([type="hidden"]),select,textarea,a[href],area[href],summary,[tabindex],[contenteditable]:not([contenteditable="false"]),audio[controls],video[controls]';
/** Uses live geometry, native disabled state and radio-group semantics on every Tab press. */
export function focusableElements(container: HTMLElement): HTMLElement[] {
  const candidates=Array.from(container.querySelectorAll<HTMLElement>(focusSelector)).filter(element=>element.tabIndex>=0&&canReceiveFocus(element));
  const radioFiltered=candidates.filter(element=>{
    if(!(element instanceof HTMLInputElement)||element.type!=="radio"||!element.name)return true;
    const group=candidates.filter((candidate): candidate is HTMLInputElement=>candidate instanceof HTMLInputElement && candidate.type==="radio" && candidate.name===element.name && candidate.form===element.form);
    return element===(group.find(radio=>radio.checked)??group[0]);
  });
  return radioFiltered.map((element,index)=>({element,index})).sort((a,b)=>{
    const left=a.element.tabIndex>0?a.element.tabIndex:Number.MAX_SAFE_INTEGER,right=b.element.tabIndex>0?b.element.tabIndex:Number.MAX_SAFE_INTEGER;
    return left-right||a.index-b.index;
  }).map(item=>item.element);
}

export function trapFocus(event: KeyboardEvent,container: HTMLElement): boolean {
  if(event.key!=="Tab"||event.defaultPrevented||event.isComposing||event.keyCode===229||event.altKey||event.ctrlKey||event.metaKey)return false;
  const target=event.target instanceof Element?event.target:null;
  const nestedDialog=target?.closest('dialog[open]');
  if(target?.closest('[data-ui-popover]') || (nestedDialog&&nestedDialog!==container))return false;
  const items=focusableElements(container),first=items[0],last=items.at(-1),active=container.ownerDocument.activeElement;
  if(!first){event.preventDefault();focusElement(container);return true;}
  if(!container.contains(active)||active===container){event.preventDefault();focusElement(event.shiftKey?last:first);}
  else if(event.shiftKey&&active===first){event.preventDefault();focusElement(last);}
  else if(!event.shiftKey&&active===last){event.preventDefault();focusElement(first);}
  return true;
}
