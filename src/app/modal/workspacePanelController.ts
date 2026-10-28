import { RouteHistory } from "../router";
import { mountConfirmationInput } from "../../ui/forms/confirmation-input";
import { escapeHtml } from "../../components/format";
import { renderVisualValue } from "./visualRecordView";
import type { WorkspacePanelEntry } from "./workspacePanelTypes";
export class WorkspacePanelController {
  private readonly history = new RouteHistory<WorkspacePanelEntry>();
  private releaseConfirmationInput: (() => void) | null = null;

  private currentEntry(): WorkspacePanelEntry | null {
    return this.history.current;
  }

  private controlKey(control: HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement, index: number): string {
    return control.id || control.getAttribute("name") || control.dataset.workspaceControlKey || `${control.tagName.toLowerCase()}:${index}`;
  }

  private captureCurrentState(): void {
    const entry = this.currentEntry();
    const root = document.querySelector<HTMLDivElement>("#modal-root");
    if (!entry || !root) return;
    const controls = Array.from(root.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>("input, select, textarea"));
    entry.formState = controls.map((control, index) => ({
      key: this.controlKey(control, index),
      value: control.value,
      checked: control instanceof HTMLInputElement && ["checkbox", "radio"].includes(control.type) ? control.checked : null,
      selectedValues: control instanceof HTMLSelectElement && control.multiple
        ? Array.from(control.selectedOptions).map((option) => option.value)
        : null,
    })).filter((state, index) => {
      const control = controls[index];
      return !(control instanceof HTMLInputElement && ["password", "file"].includes(control.type))
        && !/(api[-_]?key|password|secret|credential|database[-_]?url|attachment|file[-_]?content)/i.test(`${state.key} ${control.name}`)
        && !control.closest('[data-workspace-persist="false"]');
    });
    entry.bodyScrollTop = root.querySelector<HTMLElement>(".workspace-detail-body")?.scrollTop ?? 0;
    const active = document.activeElement;
    const activeIndex = active instanceof HTMLInputElement || active instanceof HTMLSelectElement || active instanceof HTMLTextAreaElement
      ? controls.indexOf(active)
      : -1;
    entry.activeControlKey = activeIndex >= 0 ? this.controlKey(controls[activeIndex], activeIndex) : null;
  }

  private restoreCurrentState(entry: WorkspacePanelEntry, root: HTMLDivElement): void {
    const controls = Array.from(root.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>("input, select, textarea"));
    const states = new Map((entry.formState ?? []).map((state) => [state.key, state]));
    controls.forEach((control, index) => {
      const state = states.get(this.controlKey(control, index));
      if (!state) return;
      if (control instanceof HTMLSelectElement && control.multiple && state.selectedValues) {
        const selected = new Set(state.selectedValues);
        Array.from(control.options).forEach((option) => { option.selected = selected.has(option.value); });
      } else {
        control.value = state.value;
      }
      if (control instanceof HTMLInputElement && state.checked !== null) control.checked = state.checked;
    });
    const body = root.querySelector<HTMLElement>(".workspace-detail-body");
    if (body) body.scrollTop = entry.bodyScrollTop ?? 0;
    if (entry.activeControlKey) {
      const control = controls.find((item, index) => this.controlKey(item, index) === entry.activeControlKey);
      control?.focus({ preventScroll: true });
    }
  }

  private syncHistoryButtons(): void {
    const canBack = this.history.canBack;
    const canForward = this.history.canForward;
    document.querySelectorAll<HTMLButtonElement>("[data-action='workspace-history-back']")
      .forEach((button) => { button.disabled = !canBack; });
    document.querySelectorAll<HTMLButtonElement>("[data-action='workspace-history-forward']")
      .forEach((button) => { button.disabled = !canForward; });
  }

  private renderCurrent(): void {
    this.releaseConfirmationInput?.();
    this.releaseConfirmationInput = null;
    const root = document.querySelector<HTMLDivElement>("#modal-root");
    const main = document.querySelector<HTMLElement>(".main-content");
    const page = document.querySelector<HTMLElement>(".page-container");
    const entry = this.currentEntry();
    if (!root || !main || !page) {
      this.syncHistoryButtons();
      return;
    }
    if (!entry) {
      root.replaceChildren();
      main.classList.remove("workspace-panel-open");
      page.removeAttribute("aria-hidden");
      this.syncHistoryButtons();
      return;
    }
    main.classList.add("workspace-panel-open");
    page.setAttribute("aria-hidden", "true");
    root.innerHTML = `<section class="workspace-detail-page ${entry.panelClass}" role="region" aria-label="${escapeHtml(entry.title)}">
      <div class="workspace-detail-toolbar">
        <div class="workspace-detail-history" aria-label="页面历史">
          <button type="button" class="secondary compact" data-action="workspace-history-back">← 返回</button>
          <button type="button" class="secondary compact" data-action="workspace-history-forward">前进 →</button>
        </div>
        <button type="button" class="secondary compact" data-action="close-workspace-detail">返回当前页面</button>
      </div>
      <header class="workspace-detail-header"><div><span>${escapeHtml(entry.subtitle)}</span><h2>${escapeHtml(entry.title)}</h2></div></header>
      <div class="workspace-detail-body">${entry.body}</div>
      ${entry.footer ? `<footer class="workspace-detail-footer">${entry.footer}</footer>` : ""}
    </section>`;
    this.syncHistoryButtons();
    this.restoreCurrentState(entry, root);
    const cleanups: Array<() => void> = [];
    for (const host of root.querySelectorAll<HTMLElement>("[data-confirmation-button]")) {
      const confirmation = host.querySelector<HTMLInputElement>("[data-confirmation-expected]");
      const confirmButton = Array.from(root.querySelectorAll<HTMLButtonElement>("button[id]")).find(button => button.id === host.dataset.confirmationButton);
      if (confirmation && confirmButton) cleanups.push(mountConfirmationInput(confirmation, confirmButton).destroy);
    }
    this.releaseConfirmationInput = () => { for (const cleanup of cleanups) cleanup(); };
    if (!entry.activeControlKey) root.querySelector<HTMLElement>("input, select, textarea, button:not([disabled])")?.focus();
  }

  private push(entry: WorkspacePanelEntry): void {
    this.captureCurrentState();
    this.history.push(entry);
    this.renderCurrent();
  }

  restore(): void {
    this.renderCurrent();
  }

  capture(): void {
    this.captureCurrentState();
  }

  close(): void {
    this.captureCurrentState();
    this.history.close();
    this.renderCurrent();
  }

  reset(): void {
    this.history.reset();
    this.renderCurrent();
  }

  back(): void {
    this.captureCurrentState();
    this.history.back();
    this.renderCurrent();
  }

  forward(): void {
    this.captureCurrentState();
    this.history.forward();
    this.renderCurrent();
  }

  showHtml(
    title: string,
    subtitle: string,
    body: string,
    footer = "",
    panelClass = "",
  ): void {
    const safeClass = panelClass.trim().replace(/[^a-zA-Z0-9_-]+/g, " ");
    this.push({ title, subtitle, body, footer, panelClass: safeClass });
  }

  show(title: string, payload: unknown, footer = ""): void {
    this.showHtml(title, "可视化详情", renderVisualValue(payload), footer);
  }

  get identity(): WorkspacePanelEntry|null { return this.currentEntry(); }
}
