import type { BootstrapResponse, ModelRunListItem } from "../../types";

function detachedSnapshot(value: BootstrapResponse): BootstrapResponse {
  const copy = structuredClone(value);
  const seen = new WeakSet<object>();
  const freeze = (item: unknown): void => {
    if (item === null || typeof item !== "object" || seen.has(item)) return;
    seen.add(item);
    for (const child of Object.values(item)) freeze(child);
    Object.freeze(item);
  };
  freeze(copy);
  return copy;
}

/** Sole summary owner. Existing view DTO signatures borrow a detached, deeply frozen snapshot. */
export class BootstrapState {
  private value: BootstrapResponse | null = null;
  get snapshot(): BootstrapResponse | null { return this.value; }
  replace(value: BootstrapResponse): void { this.value = detachedSnapshot(value); }

  setRecentRuns(recentRuns: ModelRunListItem[]): void {
    if (!this.value) return;
    this.replace({ ...this.value, data: { ...this.value.data, recent_runs: recentRuns } });
  }

  clear(): void { this.value = null; }
}
