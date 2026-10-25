import type { Theme } from "../../types";
import { localPreferences, preferenceKey } from "../../platform/storage";

export const SESSION_THEME_KEY = preferenceKey("theme");
type PreferenceStorage = Pick<Storage, "getItem" | "setItem">;

/** Only the existing raw theme preference; business drafts and credentials stay elsewhere. */
export class SessionPersistence {
  constructor(private readonly storage: () => PreferenceStorage = () => localPreferences) {}

  readTheme(): Theme {
    try {
      const value = this.storage().getItem(SESSION_THEME_KEY);
      return value === "dark" ? "dark" : "light";
    } catch { return "light"; }
  }

  saveTheme(theme: Theme): boolean {
    try { this.storage().setItem(SESSION_THEME_KEY, theme); return true; }
    catch { return false; }
  }
}
