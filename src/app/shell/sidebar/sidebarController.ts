import { isRoute } from "../../router";
import type { ShellActions } from "../application-shell/applicationShellState";

/** Only interprets native navigation controls; it never retains the route or theme. */
export class SidebarController {
  constructor(private readonly actions: ShellActions) {}

  activate(button: HTMLButtonElement): boolean {
    if (!button.closest(".primary-rail, .secondary-sidebar")) return false;
    const page = button.dataset.page;
    if (page && isRoute(page)) this.actions.navigate(page);
    else if (["toggle-theme", "toggle-global-sidebar"].includes(button.dataset.action ?? "")) {
      this.actions.action(button.dataset.action!, button);
    }
    return true;
  }
}
