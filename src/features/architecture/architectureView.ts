import { taskPageHeader } from "../../ui/patterns/workbench";
import { workspaceTaskAnchorNavigation } from "../../ui/navigation";
import { architectureModules, architectureOverview, architecturePrinciples } from "./architectureContent";
export function architecturePage(activeSection?: string | null): string {
  const panels = [
    {id:"architecture-flow",label:"系统边界",title:"系统架构与职责边界",content:architectureOverview()},
    {id:"architecture-principles",label:"运行原则",title:"系统运行原则",content:architecturePrinciples()},
    {id:"architecture-modules",label:"底层模块",title:"底层模块与维护职责",content:architectureModules()},
  ];
  const active = panels.find(item => item.id === activeSection) ?? panels[0];
  return `<section class="module-workspace-page management-module-workspace architecture-workspace" data-section-workspace>
    ${taskPageHeader({eyebrow:"系统信息",title:active.title,description:"用业务语言理解各层协作；本页是只读说明，不是实时健康、模型测试或发布验收。"})}
    <div class="core-local-navigation architecture-navigation">${workspaceTaskAnchorNavigation(panels.map((panel,index) => ({...panel,index:String(index + 1)})),active.id)}<span class="architecture-readonly">只读说明</span><button class="secondary" data-action="show-architecture-scope">说明范围</button></div>
    <div class="management-module-stage" data-workspace-scroll-key="architecture-stage">${panels.map(panel=>`<section id="${panel.id}" class="management-section architecture-panel" data-navigation-panel data-panel-title="${panel.title}" ${panel === active ? "" : "hidden"}>${panel.content}</section>`).join("")}</div>
  </section>`;
}
