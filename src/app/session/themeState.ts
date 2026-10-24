import type { Theme } from "../../types";

/** Owns the session theme value, without owning storage, DOM or feature state. */
export class ThemeState {
  constructor(private value: Theme) {}
  get current(): Theme { return this.value; }
  set(theme: Theme): void { this.value = theme; }
}
