import type { Route } from "./route";

export interface NavigationReturnTarget {
  readonly page: Route;
  readonly section: string | null;
}

/** A return location only. Match, player, draft and workflow data belong to features. */
export class NavigationContext {
  private location: NavigationReturnTarget | null = null;

  get target(): NavigationReturnTarget | null { return this.location; }
  set(target: NavigationReturnTarget): void { this.location = Object.freeze({ page: target.page, section: target.section }); }
  clear(): void { this.location = null; }
  take(): NavigationReturnTarget | null {
    const target = this.location;
    this.clear();
    return target;
  }
}
