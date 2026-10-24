import { EventScope, LifecycleError } from "../../lifecycle";
import { WarningStore } from "../warning-store";
import type { WarningInput, WarningRecord, WarningTarget, FeedbackTone, FeedbackAction } from "../warning-dto";
import { safeWarningText } from "../warning-dto";
import { DialogController } from "../../../ui/overlays/dialog";
import { ToastController } from "../../../ui/feedback/toast/toastController";
import { taskActivityView } from "../../../ui/feedback/task-activity/taskActivityView";
import { WarningCenterState } from "./warningCenterState";
import { warningEntryView, warningDialogView, warningListView } from "./warningCenterView";

export interface WarningCenterAdapters { navigate(target: WarningTarget): void; }

/** Coordinates the single warning owner and feedback surfaces; business gates stay with their features. */
export class WarningCenterController {
  private readonly store: WarningStore;
  private readonly state = new WarningCenterState();
  private readonly events = new EventScope();
  private readonly dialog: DialogController;
  private readonly host: HTMLDivElement;
  private readonly dialogHost: HTMLDivElement;
  private readonly toast: ToastController;
  private readonly activity: HTMLElement;
  private started = false;

  constructor(private readonly root: HTMLElement, private readonly adapters: WarningCenterAdapters, clock: () => number = Date.now) {
    this.store = new WarningStore(clock);
    this.host = root.ownerDocument.createElement("div");
    this.host.className = "warning-center-root";
    this.host.innerHTML = '<div class="warning-feedback-stack"><div data-notice-slot hidden></div>' + taskActivityView() + '</div><div class="warning-announcer" role="status" aria-live="polite" aria-atomic="true"></div><div data-warning-dialog-host></div>';
    this.dialogHost = this.host.querySelector<HTMLDivElement>("[data-warning-dialog-host]")!;
    this.activity = this.host.querySelector<HTMLElement>(".ui-task-activity")!;
    this.dialog = new DialogController(root, () => this.entry(), () => this.close());
    this.toast = new ToastController(this.host.querySelector<HTMLElement>("[data-notice-slot]")!, this.host.querySelector<HTMLElement>(".warning-announcer")!, id => { if (this.store.dismissNotice(id)) this.paintNotice(); }, () => {
      const action = this.store.notice?.action;
      if (!action) return;
      this.dismissNotice(); this.adapters.navigate(action.target);
    }, () => this.entry());
  }

  get unreadCount(): number { return this.store.unreadCount; }
  get records(): readonly WarningRecord[] { return this.store.records; }
  get taskActive(): boolean { return this.store.taskActive; }
  get active(): boolean { return this.state.open; }

  start(): void {
    if (this.started || this.events.signal.aborted) return;
    this.started = true;
    this.root.ownerDocument.body.appendChild(this.host);
    const click = (event: Event) => {
      const button = event.target instanceof Element ? event.target.closest<HTMLButtonElement>("[data-action]") : null;
      if (!button || button.disabled || button.closest("[inert]")) return;
      switch (button.dataset.action) {
        case "open-warning-center": this.open(); break;
        case "close-warning-center": this.close(); break;
        case "read-all-warnings": this.store.markAllRead(); this.paint(); break;
        case "warning-open-logs": this.close(); this.adapters.navigate("logs"); break;
        default: return;
      }
      event.preventDefault(); event.stopPropagation();
    };
    this.events.listen(this.root, "click", click, { capture: true });
    this.events.listen(this.host, "click", click, { capture: true });
    this.events.listen(this.host, "cancel", event => { if (this.dialog.handleCancel(event)) event.stopPropagation(); }, { capture: true });
    this.events.listen(this.host, "keydown", event => { if (this.dialog.handleKeydown(event)) event.stopPropagation(); }, { capture: true });
    this.restore();
  }

  record(input: WarningInput): void { if (!this.events.signal.aborted) { this.store.record(input); this.paint(); } }
  observeConnection(message: string | null): void {
    if (this.events.signal.aborted) return;
    if (message) this.store.observe({ key: "database.connection", source: "数据服务", severity: "error", title: "数据服务连接失败", message, target: "database" });
    else this.store.resolve("database.connection");
    this.paint();
  }

  notify(message: string, tone: FeedbackTone, source: string, action?: FeedbackAction): void {
    if (this.events.signal.aborted) return;
    if (tone === "warning" || tone === "danger") this.store.record({ key: `${source}:${safeWarningText(message)}`, source, severity: tone === "warning" ? "warning" : "error", title: tone === "warning" ? "操作提醒" : "操作失败", message, target: "logs" });
    this.store.notify(message, tone, action); this.paint();
  }

  async runTask<T>(operation: () => Promise<T>): Promise<T> {
    if (this.events.signal.aborted) throw new LifecycleError();
    if (!this.store.beginTask()) throw new Error("已有任务正在执行");
    this.paintActivity();
    try { return await operation(); }
    finally { this.store.endTask(); if (!this.events.signal.aborted) this.paintActivity(); }
  }

  open(): void {
    if (this.events.signal.aborted || this.state.open || this.root.ownerDocument.querySelector("dialog[open]")) return;
    this.state.show();
    this.dialog.restore(this.dialogHost, warningDialogView(this.store.records, this.store.unreadCount));
  }
  close(): void { this.state.hide(); this.dialog.close(); }
  dismissNotice(): void { const id = this.store.notice?.id; if (id !== undefined && this.store.dismissNotice(id)) this.paintNotice(); }
  restore(): void { if (!this.events.signal.aborted) this.paint(); }
  private entry(): HTMLElement | null { return this.root.querySelector('[data-action="open-warning-center"]'); }
  private paintNotice(): void {
    const notice = this.store.notice;
    this.toast.show(notice ? { ...notice, actionLabel: notice.action?.label } : null);
  }
  private paintActivity(): void { this.activity.hidden = !this.store.taskActive; this.activity.setAttribute("aria-hidden", String(!this.store.taskActive)); }
  private paint(): void {
    const slot = this.root.querySelector<HTMLElement>("[data-warning-entry]");
    if (slot) {
      const focused = slot.contains(slot.ownerDocument.activeElement);
      slot.innerHTML = warningEntryView(this.store.unreadCount);
      if (focused && !this.state.open) this.entry()?.focus({ preventScroll: true });
    }
    this.paintNotice(); this.paintActivity();
    const dialog = this.dialog.element;
    if (this.state.open && dialog) {
      const content = dialog.querySelector<HTMLElement>(".warning-center-content")!;
      const scroll = content.scrollTop;
      content.innerHTML = warningListView(this.store.records);
      content.scrollTop = scroll;
      dialog.querySelector("#warning-center-description")!.textContent = `本次会话 · ${this.store.unreadCount} 条未读`;
      const readButton = dialog.querySelector<HTMLButtonElement>('[data-action="read-all-warnings"]')!;
      if (!this.store.unreadCount && dialog.ownerDocument.activeElement === readButton) dialog.querySelector<HTMLElement>("[autofocus]")?.focus();
      readButton.disabled = !this.store.unreadCount; readButton.hidden = !this.store.records.length;
    }
  }
  destroy(): void {
    if (this.events.signal.aborted) return;
    this.events.destroy(); this.state.hide(); this.dialog.close(false); this.toast.destroy(); this.host.remove();
  }
}
