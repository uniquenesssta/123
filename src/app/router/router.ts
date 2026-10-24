import type { NavigationRequest } from "./navigationRequest";
import type { Route } from "./route";
import { routeRegistry } from "./routeRegistry";
import { RouteTransition } from "./routeTransition";

export interface NavigationEffects {
  started(): void;
  settled(): void;
}

/** Owns the current route; effects and feature loaders stay with their callers. */
export class Router {
  private current: Route;
  private readonly transition = new RouteTransition();

  constructor(initialPage: Route = "dashboard") {
    routeRegistry.item(initialPage);
    this.current = initialPage;
  }

  get page(): Route { return this.current; }
  get pendingPage(): Route | null { return this.transition.pendingPage; }

  begin(page: Route): NavigationRequest {
    routeRegistry.item(page);
    const request = this.transition.begin(page);
    if (this.isCurrent(request)) this.current = page;
    return request;
  }

  isCurrent(request: NavigationRequest): boolean { return this.transition.isCurrent(request); }
  complete(request: NavigationRequest): boolean { return this.transition.complete(request); }
  cancel(): void { this.transition.cancel(); }
  destroy(): void { this.transition.destroy(); }

  async navigate(page: Route, load: (request: NavigationRequest) => Promise<void>, effects: NavigationEffects): Promise<boolean> {
    const request = this.begin(page);
    if (!this.isCurrent(request)) return false;
    try {
      effects.started();
      if (!this.isCurrent(request)) return false;
      await load(request);
      return this.isCurrent(request);
    } catch (error: unknown) {
      if (!this.isCurrent(request)) return false;
      throw error;
    } finally {
      if (this.complete(request)) effects.settled();
    }
  }
}
