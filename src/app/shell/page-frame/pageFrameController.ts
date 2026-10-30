import { focusElement } from "../../../ui/accessibility";
/** Preserve native shell focus across replacement of the view; page drafts stay in Workspace. */
export class PageFrameController {
  constructor(private readonly root: HTMLElement) {}

  captureFocus(): string | null {
    const active = this.root.ownerDocument.activeElement;
    if (!(active instanceof HTMLButtonElement) || !active.closest("[data-shell-controls]")) return null;
    return active.dataset.action ? `action:${active.dataset.action}` : active.dataset.primaryModule ? `module:${active.dataset.primaryModule}` : active.dataset.page ? `page:${active.dataset.page}` : null;
  }

  restoreFocus(key: string | null): void {
    if (!key) return;
    const [kind, value] = key.split(":");
    const attribute = kind === "action" ? "data-action" : kind === "module" ? "data-primary-module" : "data-page";
    const button = Array.from(this.root.querySelectorAll<HTMLButtonElement>("[data-shell-controls] button")).find(item => item.getAttribute(attribute) === value && !item.closest("[hidden], [inert]"));
    focusElement(button);
  }
}
