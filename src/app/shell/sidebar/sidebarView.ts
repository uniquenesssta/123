import { escapeHtml } from "../../../components/format";
import { PRIMARY_NAVIGATION, type PrimaryNavigationModule } from "../../router";
import { navigationSidebarView } from "../../../ui/navigation";
import { navigationIcon } from "./sidebarIcons";
import { serviceStatusView } from "../topbar/serviceStatusView";
import type { ShellOptions } from "../application-shell/applicationShellState";
export function sidebarView(options:ShellOptions,activeModule:PrimaryNavigationModule):string {
 const version=options.state.data.app_version;
 return navigationSidebarView({title:activeModule.label,description:activeModule.description,themeLabel:options.theme==="dark"?"切换浅色主题":"切换深色主题",collapsed:options.sidebarCollapsed,
 primary:PRIMARY_NAVIGATION.map(m=>({label:m.label,icon:navigationIcon(m.default_page),title:m.label+" · "+m.description,selected:m.key===activeModule.key,attributes:{"data-page":m.default_page,"data-primary-module":m.key}})),
 secondary:activeModule.items.map(i=>({label:i.label,icon:navigationIcon(i.page),title:i.description,selected:i.page===options.page,attributes:{"data-page":i.page}})),
 footer:serviceStatusView(options.state.data.database_configured,false)+"<small>"+(version?escapeHtml("v"+version):"版本未提供")+"</small>"});
}
