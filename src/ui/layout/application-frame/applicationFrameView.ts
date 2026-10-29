import { escapeHtml } from "../../../components/format";
export interface ApplicationFrameOptions {
    navigation: string;
    main: string;
    collapsed: boolean;
    workspace: boolean;
    page: string;
    module: string;
}
/** Layout composes trusted UI slots without owning route selection or persistence. */
export function applicationFrameView(options: ApplicationFrameOptions): string {
    return `<div class="app-shell dual-navigation application-shell ${options.collapsed ? "sidebar-collapsed secondary-collapsed" : ""} ${options.workspace ? "workspace-page" : ""}" data-current-page="${escapeHtml(options.page)}" data-current-module="${escapeHtml(options.module)}">${options.navigation}${options.main}</div><div data-confirmation-host></div>`;
}
