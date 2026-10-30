import { icon } from "../../../ui/icons";
export function workspacePaneToggle(pane: "module-sidebar" | "inspector", collapsed: boolean): string {
    const isSidebar = pane === "module-sidebar";
    return `<button class="workspace-pane-toggle icon-button" data-action="toggle-workspace-pane" data-pane="${pane}" title="${collapsed ? "展开" : "收起"}${isSidebar ? "目录" : "检查器"}" aria-label="${collapsed ? "展开" : "收起"}${isSidebar ? "目录" : "检查器"}">${icon(isSidebar ? "panel-left" : "panel-right")}</button>`;
}
