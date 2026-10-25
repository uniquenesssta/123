import { EventScope } from "../../../app/lifecycle";
import { monotonicTime } from "../../../platform/timing";
import { toastView, type ToastPresentation } from "./toastView";

export interface TimedToast extends ToastPresentation { readonly id: number; readonly autoDismiss: boolean; }

/** Owns presentation timing only. Notice identity/content remain with the application owner. */
export class ToastController {
  private readonly events = new EventScope();
  private identity: number | null = null;
  private timer: number | null = null;
  private remaining = 0;
  private scheduledAt = 0;
  private hovered = false;
  private focused = false;
  private autoDismiss = false;

  constructor(private readonly host: HTMLElement, private readonly announce: HTMLElement, private readonly dismiss: (id: number) => void, private readonly action: () => void, private readonly fallbackFocus: () => HTMLElement | null) {
    this.events.listen(host, "pointerover", () => { this.hovered = true; this.pause(); });
    this.events.listen(host, "pointerout", event => {
      if (event instanceof PointerEvent && event.relatedTarget instanceof Node && host.contains(event.relatedTarget)) return;
      this.hovered = false; this.resume();
    });
    this.events.listen(host, "focusin", () => { this.focused = true; this.pause(); });
    this.events.listen(host, "focusout", () => {
      queueMicrotask(() => {
        if (this.events.signal.aborted) return;
        this.focused = host.contains(host.ownerDocument.activeElement);
        this.resume();
      });
    });
    this.events.listen(host, "click", event => {
      const button = event.target instanceof Element ? event.target.closest<HTMLButtonElement>("button") : null;
      if (!button || button.disabled) return;
      if (button.dataset.action === "dismiss-notice" && this.identity !== null) this.dismiss(this.identity);
      if (button.dataset.action === "notice-action") this.action();
      event.stopPropagation();
    });
  }

  show(notice: TimedToast | null): void {
    if (this.events.signal.aborted || this.identity === (notice?.id ?? null)) return;
    this.clearTimer();
    const focused = this.host.contains(this.host.ownerDocument.activeElement);
    this.identity = notice?.id ?? null;
    this.remaining = notice?.autoDismiss ? 3200 : 0;
    this.autoDismiss = notice?.autoDismiss ?? false;
    this.host.innerHTML = notice ? toastView(notice) : "";
    this.host.hidden = !notice;
    this.announce.textContent = notice?.message ?? "";
    this.hovered = !!notice && this.host.matches(":hover");
    this.focused = false;
    if (focused) {
      if (notice) this.host.querySelector<HTMLButtonElement>("button")?.focus();
      else this.fallbackFocus()?.focus({ preventScroll: true });
    }
    this.resume();
  }

  private clearTimer(): void { if (this.timer !== null) this.events.clearTimeout(this.timer); this.timer = null; }
  private pause(): void {
    if (this.timer === null) return;
    this.remaining = Math.max(0, this.remaining - (monotonicTime() - this.scheduledAt));
    this.clearTimer();
  }
  private resume(): void {
    if (this.events.signal.aborted || this.timer !== null || this.hovered || this.focused || this.identity === null || !this.autoDismiss) return;
    const identity = this.identity;
    this.scheduledAt = monotonicTime();
    this.timer = this.events.setTimeout(() => { this.timer = null; this.remaining = 0; this.dismiss(identity); }, this.remaining);
  }
  destroy(): void { this.events.destroy(); this.timer = null; this.identity = null; this.host.replaceChildren(); this.announce.textContent = ""; }
}
