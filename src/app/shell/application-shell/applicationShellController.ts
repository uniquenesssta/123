import { NavigationKeyboardController } from "../../../ui/navigation";
import { EventScope } from "../../lifecycle";
import { SidebarController } from "../sidebar/sidebarController";
import { TopbarController } from "../topbar/topbarController";
import { PageFrameController } from "../page-frame/pageFrameController";
import { ContentRegionController } from "../content-region/contentRegionController";
import type { ShellActions, ShellOperations } from "./applicationShellState";
import { ConfirmationDialogController } from "../../../ui/overlays/confirmation-dialog/confirmationDialogController";

/** One delegated listener survives renders and is removed with the browser module. */
export class ApplicationShellController {
  private events: EventScope | null = null;
  private closed = false;
  private readonly navigationKeyboard: NavigationKeyboardController;
  private readonly sidebar: SidebarController;
  private readonly topbar: TopbarController;
  private readonly frame: PageFrameController;
  private readonly content: ContentRegionController;
  private readonly confirmation: ConfirmationDialogController;

  constructor(private readonly root: HTMLElement, private readonly actions: ShellActions, private readonly operations: ShellOperations) {
    this.navigationKeyboard = new NavigationKeyboardController(root);
    this.sidebar = new SidebarController(actions);
    this.topbar = new TopbarController(actions);
    this.frame = new PageFrameController(root);
    this.content = new ContentRegionController(root);
    this.confirmation = new ConfirmationDialogController(root);
  }

  start(): void {
    if (this.closed || this.events) return;
    this.events = new EventScope();
    this.navigationKeyboard.start();
    this.events.listen(this.root, "click", event => {
      if (this.confirmation.handleClick(event)) { event.stopPropagation(); return; }
      if (this.confirmation.active) { event.stopPropagation(); return; }
      const target = event.target;
      if (!(target instanceof Element)) return;
      const button = target.closest<HTMLButtonElement>("[data-shell-controls] button, .ui-blocking-message button");
      if (!button || button.disabled || !this.content.canActivate(button)) return;
      if (this.sidebar.activate(button) || this.topbar.activate(button) || this.content.activate(button, this.actions)) event.stopPropagation();
    });
    this.events.listen(this.root, "cancel", event => { if (this.confirmation.handleCancel(event)) event.stopPropagation(); }, { capture: true });
    this.events.listen(this.root, "keydown", event => {
      if (this.confirmation.handleKeydown(event)) event.stopPropagation();
    }, { capture: true });
  }

  captureFocus(): string | null { return this.frame.captureFocus(); }
  get hasConfirmation(): boolean { return this.confirmation.active; }
  confirmWorkspaceReset(pageTitle: string, onConfirm: () => void): void {
    if (this.closed) return;
    this.confirmation.show({ title: "重置当前工作区", description: "将重置当前页面已保存的工作区状态。未保存的输入可能丢失。", facts: [["当前页面", pageTitle], ["保留", "数据库内容、主题与全局导航"]], confirmLabel: "重置工作区", onConfirm });
  }
  handleAction(action: string): boolean | Promise<boolean> {
    if (this.closed) return false;
    switch (action) {
      case "toggle-global-sidebar":
        this.operations.setSidebarCollapsed(!this.operations.sidebarCollapsed());
        this.operations.renderPreservingWorkspace();
        return true;
      case "toggle-theme":
        this.operations.toggleTheme();
        this.operations.renderPreservingWorkspace();
        return true;
      case "workspace-history-back": this.operations.historyBack(); return true;
      case "workspace-history-forward": this.operations.historyForward(); return true;
      case "reset-current-workspace": this.operations.resetWorkspace(); return true;
      case "retry-shell-page": return Promise.resolve(this.operations.retryPageRead()).then(() => true);
      case "shell-open-logs": return Promise.resolve(this.operations.openLogs()).then(() => true);
      default: return false;
    }
  }
  get navigationFailed(): boolean { return this.content.state.navigationFailed; }
  clearNavigationFailure(): void { if (!this.closed) this.content.state.clear(); }
  showNavigationFailure(): void { if (!this.closed) this.content.state.fail(); }
  restoreFocus(key: string | null): void {
    if (this.closed) return;
    this.confirmation.restore();
    if (!this.confirmation.active) this.frame.restoreFocus(key);
  }

  destroy(): void {
    if (this.closed) return;
    this.closed = true;
    this.confirmation.destroy();
    this.navigationKeyboard.destroy();
    this.events?.destroy();
    this.events = null;
  }
}
