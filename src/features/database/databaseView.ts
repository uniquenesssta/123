import type { BootstrapResponse } from "../../types";
import { taskPageHeader } from "../../ui/patterns/workbench";
import { workspaceTaskAnchorNavigation } from "../../ui/navigation";
import { databaseOverview } from "./databaseOverview";
import { databaseConnection, databaseDanger, databaseStatistics } from "./databasePanels";

export function databasePage(state: BootstrapResponse, activeSection?: string | null): string {
  const panels = [
    {id:"database-overview", label:"运行状态", title:"数据服务运行状态", content:databaseOverview(state)},
    {id:"database-connection", label:"连接设置", title:"数据库连接设置", content:databaseConnection(state)},
    {id:"database-statistics", label:"数据统计", title:"数据库核心表统计", content:databaseStatistics(state)},
    {id:"database-danger", label:"危险操作", title:"数据库危险操作", content:databaseDanger(state)},
  ];
  const active = panels.find(item => item.id === activeSection) ?? panels[state.data.database_configured ? 0 : 1];
  return `<section class="module-workspace-page management-module-workspace database-workspace" data-section-workspace>
    ${taskPageHeader({eyebrow:"数据库", title:active.title, description:"连接、自动迁移、统计与危险重置分别展示；状态以服务响应为准。"})}
    <div class="core-local-navigation database-navigation">${workspaceTaskAnchorNavigation(panels.map((panel,index) => ({...panel,index:String(index + 1)})), active.id)}<button class="secondary" data-action="refresh-service-state">刷新状态</button><button class="secondary" data-action="show-database-scope">使用边界</button></div>
    <div class="management-module-stage" data-workspace-scroll-key="database-stage">${panels.map(panel => `<section id="${panel.id}" class="management-section database-panel" data-navigation-panel data-panel-title="${panel.title}" ${panel === active ? "" : "hidden"}>${panel.content}</section>`).join("")}</div>
  </section>`;
}
