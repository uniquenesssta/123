/** Keep classification fixed. A native compact selector exposes every region when buttons cannot fit. */
export function fitCatalogueRegions(root: ParentNode = document): void {
  const rail = root.querySelector<HTMLElement>(".rules-region-list");
  const selector = rail?.querySelector<HTMLSelectElement>("[data-rules-region-select]");
  if (!rail || !selector) return;
  rail.removeAttribute("data-compact"); selector.hidden = true;
  const buttons = Array.from(rail.querySelectorAll<HTMLButtonElement>("[data-rules-region]"));
  const compact = rail.scrollHeight > rail.clientHeight + 1;
  rail.toggleAttribute("data-compact", compact); selector.hidden = !compact;
  for (const option of Array.from(selector.options)) {
    const button = buttons.find(item => item.dataset.rulesRegion === option.value);
    option.hidden = !button || button.hidden; option.disabled = !button || button.disabled;
    option.textContent = button?.textContent ?? option.textContent;
    if (button?.classList.contains("active")) selector.value = option.value;
  }
}
export function selectCatalogueRegion(selector: HTMLSelectElement): void {
  const directory = selector.closest("[data-rules-directory]");
  directory?.querySelectorAll<HTMLElement>("[data-rules-region]").forEach(button => button.classList.toggle("active", button.dataset.rulesRegion === selector.value));
}
