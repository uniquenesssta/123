export type PreferenceStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">;

/** Browser storage boundary; no feature state, credentials, key registry or shadow cache. */
export class PreferenceStore {
  constructor(private readonly storage: () => PreferenceStorage) {}
  getItem(key: string): string | null { return this.storage().getItem(key); }
  setItem(key: string, value: string): void { this.storage().setItem(key, value); }
  removeItem(key: string): void { this.storage().removeItem(key); }
  writeJson(key: string, value: unknown): void {
    const raw = JSON.stringify(value);
    if (raw === undefined) throw new TypeError("Preference value must be JSON serializable");
    this.setItem(key, raw);
  }
  /** Preserve the legacy plain-object format and its corrupt-JSON fallback. */
  readObject<T>(key: string): Partial<T> | null {
    try {
      const raw = this.getItem(key);
      if (!raw) return null;
      const value: unknown = JSON.parse(raw);
      return value && typeof value === "object" && !Array.isArray(value) ? value as Partial<T> : null;
    } catch {
      try { this.removeItem(key); } catch { /* Preference storage may be unavailable. */ }
      return null;
    }
  }
}

// Resolve lazily: blocked storage must not break module import or session fallback.
export const localPreferences = new PreferenceStore(() => window.localStorage);
export const sessionPreferences = new PreferenceStore(() => window.sessionStorage);
