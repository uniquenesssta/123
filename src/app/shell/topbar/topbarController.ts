import type { ShellActions } from "../application-shell/applicationShellState";

const actions = new Set(["open-command-menu", "toggle-global-sidebar", "workspace-history-back", "workspace-history-forward", "reset-current-workspace"]);

export class TopbarController {
  constructor(private readonly callbacks: ShellActions) {}

  activate(button: HTMLButtonElement): boolean {
    if (!button.closest(".topbar")) return false;
    if (actions.has(button.dataset.action ?? "")) this.callbacks.action(button.dataset.action!, button);
    return true;
  }
}
