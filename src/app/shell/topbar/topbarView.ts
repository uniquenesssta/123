import { navigationItemForPage, type PrimaryNavigationModule } from "../../router";
import { navigationTopbarView } from "../../../ui/navigation";
import { serviceStatusView } from "./serviceStatusView";
import type { ShellOptions } from "../application-shell/applicationShellState";
export function topbarView(options:ShellOptions,activeModule:PrimaryNavigationModule):string {
 return navigationTopbarView({location:activeModule.label+" / "+navigationItemForPage(options.page).label,sidebarCollapsed:options.sidebarCollapsed,serviceStatus:serviceStatusView(options.state.data.database_configured,options.navigationPending)});
}
