/** Switch composed panels in place so transient inputs never enter persisted workspace state. */
export function activateSectionPanel(root: ParentNode, id: string): boolean {
  const target = Array.from(root.querySelectorAll<HTMLElement>("[data-navigation-panel]")).find(item => item.id === id);
  if (!target) return false;
  const workspace = target.closest<HTMLElement>("[data-section-workspace]");
  if (!workspace) return false;
  workspace.querySelectorAll<HTMLElement>("[data-navigation-panel]").forEach(panel => { panel.hidden = panel !== target; });
  workspace.querySelectorAll<HTMLElement>("[data-anchor-id]").forEach(button => {
    const active = button.dataset.anchorId === id;
    button.classList.toggle("active", active);
    button.setAttribute("aria-current", active ? "page" : "false");
  });
  const title = workspace.querySelector<HTMLElement>("h1");
  if (title && target.dataset.panelTitle) title.textContent = target.dataset.panelTitle;
  if (document.activeElement?.closest("[hidden]")) {
    const heading = target.querySelector<HTMLElement>("h2");
    if (heading) { heading.tabIndex = -1; heading.focus({ preventScroll: true }); }
  }
  return true;
}

/** Switch already rendered workspace sections without replacing the navigation or draft controls. */
export function activateWorkspaceSection(trigger: HTMLElement, id: string): boolean {
  const workspace = trigger.closest("[data-composed-workspace-sections]");
  if (!workspace) return false;
  const panels = Array.from(workspace.querySelectorAll<HTMLElement>(".workspace-module-view[data-workspace-section]"));
  const target = panels.find(panel => panel.dataset.workspaceSection === id);
  if (!target) return false;
  for (const panel of panels) panel.classList.toggle("active", panel === target);
  for (const button of workspace.querySelectorAll<HTMLButtonElement>(".workspace-section-nav button[data-section-id]")) {
    const active = button.dataset.sectionId === id;
    button.classList.toggle("active", active);
    const current = active ? "page" : "false";
    if (button.getAttribute("aria-current") !== current) button.setAttribute("aria-current", current);
  }
  return true;
}
