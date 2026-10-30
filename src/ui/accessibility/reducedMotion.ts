/** Read the current OS/browser preference at the point of use; no stale cached preference. */
export function prefersReducedMotion(view: Pick<Window,"matchMedia">=window): boolean {
  return view.matchMedia("(prefers-reduced-motion: reduce)").matches;
}
export function motionScrollBehavior(preferred: ScrollBehavior="smooth",view: Pick<Window,"matchMedia">=window): ScrollBehavior {
  return prefersReducedMotion(view)?"auto":preferred;
}
