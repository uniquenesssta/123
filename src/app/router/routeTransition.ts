import { AsyncScope } from "../lifecycle";
import type { NavigationRequest } from "./navigationRequest";
import type { Route } from "./route";

/** Owns only the latest in-flight navigation, using the lifecycle cancellation channel. */
export class RouteTransition {
  private readonly requests = new AsyncScope();
  private active: NavigationRequest | null = null;

  get pendingPage(): Route | null { return this.active?.page ?? null; }

  begin(page: Route): NavigationRequest {
    const request = this.requests.begin();
    const navigation = Object.freeze({ ...request, sequence: request.id, page });
    // Aborting the previous request may synchronously start another navigation.
    if (request.isCurrent()) this.active = navigation;
    return navigation;
  }

  isCurrent(request: NavigationRequest): boolean {
    return this.active === request && request.isCurrent();
  }

  complete(request: NavigationRequest): boolean {
    if (!this.isCurrent(request)) return false;
    this.active = null;
    request.finish();
    return true;
  }

  cancel(): void {
    this.active = null;
    this.requests.invalidate();
  }

  destroy(): void {
    this.active = null;
    this.requests.destroy();
  }
}
