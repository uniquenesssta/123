import { escapeHtml } from "../../../components/format";
import { dialogView } from "../dialog";
import { textInputView } from "../../primitives/text-input";
import { iconSlot, type UiIcon } from "../../icons/iconSlot";
import { operationStateView } from "../../feedback/operation-state";
export interface UiCommand { id: string; label: string; description: string; keywords?: string; icon?: UiIcon; disabled?: boolean; run: ()=>void|Promise<void> }
export interface CommandMenuState { query: string; selected: string|null; busy: boolean; error: string|null }
export function matchingCommands(commands: readonly UiCommand[],query: string): readonly UiCommand[] { const tokens=query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);return commands.filter(c=>tokens.every(t=>(c.label+" "+c.description+" "+(c.keywords??"")).toLocaleLowerCase().includes(t))); }
export function commandResultsView(commands: readonly UiCommand[],s: CommandMenuState): string {
  if(s.busy)return operationStateView({state:"loading",message:"正在处理"});
  return (s.error?operationStateView({state:"error",title:"操作未完成",message:s.error}):"")+`<div id="command-results" class="ui-command-results" role="listbox" aria-label="页面与操作">${commands.length?commands.map(c=>`<button type="button" id="command-${escapeHtml(c.id)}" role="option" aria-selected="${s.selected===c.id}" data-command-id="${escapeHtml(c.id)}" tabindex="-1" ${c.disabled?"disabled":""}>${iconSlot(c.icon??"info",16)}<span class="ui-command-copy"><strong>${escapeHtml(c.label)}</strong><small>${escapeHtml(c.description)}</small></span></button>`).join(""):"<p class=ui-command-empty>没有匹配的页面或操作</p>"}</div>`;
}
export function commandMenuView(commands: readonly UiCommand[],s: CommandMenuState): string {
  const search=textInputView({label:"搜索页面与操作",placeholder:"搜索页面与操作",value:s.query,disabled:s.busy,autoComplete:"off",attributes:{id:"command-search",role:"combobox","aria-expanded":"true","aria-controls":"command-results","aria-autocomplete":"list",autofocus:true}});
  return dialogView({id:"command-menu",title:"快速导航",titleId:"command-title",description:"搜索页面与操作",descriptionId:"command-description",body:search+`<div data-command-results>${commandResultsView(commands,s)}</div>`,busy:s.busy,className:"ui-command-menu",closeAttributes:{"data-command-close":true}});
}
