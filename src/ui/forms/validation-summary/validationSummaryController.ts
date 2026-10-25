import { EventScope } from "../../../app/lifecycle";

/** Navigation stays inside the supplied region, including custom Select triggers. */
export function mountValidationSummary(summary: HTMLElement, region: HTMLElement) {
  const scope = new EventScope();
  scope.listen(summary, "click", event => { focusValidationTarget(event, summary, region); });
  return { focus() { if (!scope.signal.aborted && !summary.hidden) summary.focus(); }, destroy() { scope.destroy(); } };
}

export function focusValidationTarget(event: Event, summary: HTMLElement, region: HTMLElement): boolean {
    if (!(event.target instanceof Element)) return false;
    const link = event.target.closest<HTMLAnchorElement>("[data-validation-target]");
    if (!link || !summary.contains(link)) return false;
    event.preventDefault();
    const control = Array.from(region.querySelectorAll<HTMLElement>("[id]")).find(node => node.id === link.dataset.validationTarget);
    if (control && !control.matches(":disabled,[aria-disabled='true']") && !control.closest("[hidden],[inert]")) control.focus();
    return true;
}
