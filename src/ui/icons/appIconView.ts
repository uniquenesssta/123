import { iconSlot } from "./iconSlot";
import type { UiIcon } from "./iconRegistry";
export type AppIcon =
  | "home"
  | "calendar"
  | "spark"
  | "shield"
  | "users"
  | "sheet"
  | "chat"
  | "review"
  | "chart"
  | "settings"
  | "history"
  | "alert"
  | "database"
  | "plug"
  | "info"
  | "search"
  | "panel-left"
  | "panel-right"
  | "refresh"
  | "reset"
  | "compare"
  | "cards"
  | "detail"
  | "close"
  | "more";

/** Legacy semantic names remain source-compatible; geometry comes only from approved assets. */
const appIcons: Record<AppIcon,UiIcon> = {
  "home": "dashboard",
  "calendar": "calendar",
  "spark": "prediction",
  "shield": "shield",
  "users": "users",
  "sheet": "sheet",
  "chat": "chat",
  "review": "release-acceptance",
  "chart": "chart",
  "settings": "settings",
  "history": "history",
  "alert": "alert-circle",
  "database": "database",
  "plug": "plug",
  "info": "info",
  "search": "search",
  "panel-left": "panel-left",
  "panel-right": "panel-right",
  "refresh": "refresh",
  "reset": "reset",
  "compare": "compare",
  "cards": "cards",
  "detail": "detail",
  "close": "close",
  "more": "more"
};
export function icon(name: AppIcon,className="app-icon"): string {
  if(!Object.hasOwn(appIcons,name))throw new Error("Unknown application icon: "+name);
  return iconSlot(appIcons[name],16,className);
}
