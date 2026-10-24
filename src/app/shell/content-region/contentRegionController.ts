import { ContentRegionState } from "./contentRegionState";
import type { ShellActions } from "../application-shell/applicationShellState";

/** Gate workspace interactions and interpret read-only recovery actions. */
export class ContentRegionController {
  readonly state = new ContentRegionState();
  constructor(private readonly root: HTMLElement) {}

  activate(button: HTMLButtonElement, callbacks: ShellActions): boolean {
    if (!button.closest(".ui-blocking-message")) return false;
    if (button.dataset.action === "retry-shell-page") callbacks.action("retry-shell-page", button);
    if (button.dataset.action === "shell-open-logs") callbacks.action("shell-open-logs", button);
    return true;
  }

  canActivate(button: HTMLElement): boolean {
    return this.root.contains(button) && !button.closest("[inert], [hidden], [aria-disabled='true']");
  }
}
