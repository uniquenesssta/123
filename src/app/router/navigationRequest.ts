import type { AsyncRequest } from "../lifecycle";
import type { Route } from "./route";

/** A navigation lease, never a page's business state or a server task ID. */
export interface NavigationRequest extends AsyncRequest {
  readonly sequence: number;
  readonly page: Route;
}
