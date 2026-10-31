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
