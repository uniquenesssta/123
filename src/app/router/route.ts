import type { AppIcon } from "../../ui/icons";
import type { Page } from "../../types";

export type Route = Page;

export type PrimaryNavigationKey =
  | "home"
  | "matches"
  | "resources"
  | "model"
  | "analysis"
  | "ai"
  | "management";

export interface SecondaryNavigationItem {
  readonly page: Page;
  readonly label: string;
  readonly description: string;
  readonly icon: AppIcon;
}

export interface PrimaryNavigationModule {
  readonly key: PrimaryNavigationKey;
  readonly label: string;
  readonly description: string;
  readonly icon: AppIcon;
  readonly default_page: Page;
  readonly items: readonly SecondaryNavigationItem[];
}
