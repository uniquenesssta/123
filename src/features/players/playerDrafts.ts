import type { BootstrapResponse } from "../../types";
type Values = Record<string, { value: string; checked?: boolean }>;

/** Per-player independent edit drafts are transient, scoped to the rendered Session snapshot. Workspace owns layout only here. */
export class PlayerDrafts {
  private readonly drafts = new WeakMap<BootstrapResponse, Map<string, Values>>();
  private renderedSnapshot: BootstrapResponse | null = null;
  private acknowledged = new Map<string, Values>();
  private values(root: ParentNode): Values {
    return Object.fromEntries([...root.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>("input[id],select[id],textarea[id]")].map(input => [input.id, { value: input.value, ...(input instanceof HTMLInputElement && input.type === "checkbox" ? { checked: input.checked } : {}) }]));
  }
  submitted(root: ParentNode): Values { const scope = root.querySelector<HTMLElement>("[data-player-draft-id]"); return scope ? this.values(scope) : {}; }
  acknowledge(id: string, submitted: Values): void { this.acknowledged.set(id, submitted); }
  capture(root: ParentNode): void {
    const scope = root.querySelector<HTMLElement>("[data-player-draft-id]"), state = this.renderedSnapshot;
    const id = scope?.dataset.playerDraftId; if (!scope || !id || !state) return;
    let values = this.values(scope); const saved = this.acknowledged.get(id);
    if (saved) { values = Object.fromEntries(Object.entries(values).filter(([key, value]) => JSON.stringify(value) !== JSON.stringify(saved[key]))); this.acknowledged.delete(id); }
    let perTeam = this.drafts.get(state); if (!perTeam) { perTeam = new Map(); this.drafts.set(state, perTeam); }
    perTeam.set(id, values);
  }
  restore(root: ParentNode, snapshot: BootstrapResponse | null): void {
    this.renderedSnapshot = snapshot;
    const scope = root.querySelector<HTMLElement>("[data-player-draft-id]"), id = scope?.dataset.playerDraftId;
    if (!snapshot || !scope || !id) return;
    for (const [key, stored] of Object.entries(this.drafts.get(snapshot)?.get(id) ?? {})) {
      const input = scope.querySelector<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>(`#${CSS.escape(key)}`);
      if (!input) continue;
      if (input instanceof HTMLSelectElement && !Array.from(input.options).some(option => option.value === stored.value)) continue;
      input.value = stored.value;
      if (input instanceof HTMLInputElement && stored.checked !== undefined) input.checked = stored.checked;
    }
  }
  clear(snapshot: BootstrapResponse | null): void { if (snapshot) this.drafts.delete(snapshot); this.acknowledged.clear(); }
}
