import { controlledAsset } from "../assets";
export const iconRegistry = {
  "dashboard": {
    "name": "dashboard",
    "assetId": "icons/dashboard.svg"
  },
  "match-center": {
    "name": "match-center",
    "assetId": "icons/match-center.svg"
  },
  "prediction": {
    "name": "prediction",
    "assetId": "icons/prediction.svg"
  },
  "release-acceptance": {
    "name": "release-acceptance",
    "assetId": "icons/release-acceptance.svg"
  },
  "history": {
    "name": "history",
    "assetId": "icons/history.svg"
  },
  "shield": {
    "name": "shield",
    "assetId": "icons/shield.svg"
  },
  "users": {
    "name": "users",
    "assetId": "icons/users.svg"
  },
  "lineup-preset": {
    "name": "lineup-preset",
    "assetId": "icons/lineup-preset.svg"
  },
  "sheet": {
    "name": "sheet",
    "assetId": "icons/sheet.svg"
  },
  "settings": {
    "name": "settings",
    "assetId": "icons/settings.svg"
  },
  "chart": {
    "name": "chart",
    "assetId": "icons/chart.svg"
  },
  "chat": {
    "name": "chat",
    "assetId": "icons/chat.svg"
  },
  "plug": {
    "name": "plug",
    "assetId": "icons/plug.svg"
  },
  "database": {
    "name": "database",
    "assetId": "icons/database.svg"
  },
  "issue-log": {
    "name": "issue-log",
    "assetId": "icons/issue-log.svg"
  },
  "info": {
    "name": "info",
    "assetId": "icons/info.svg"
  },
  "contrast": {
    "name": "contrast",
    "assetId": "icons/contrast.svg"
  },
  "panel-left": {
    "name": "panel-left",
    "assetId": "icons/panel-left.svg"
  },
  "panel-right": {
    "name": "panel-right",
    "assetId": "icons/panel-right.svg"
  },
  "arrow-left": {
    "name": "arrow-left",
    "assetId": "icons/arrow-left.svg"
  },
  "arrow-right": {
    "name": "arrow-right",
    "assetId": "icons/arrow-right.svg"
  },
  "reset": {
    "name": "reset",
    "assetId": "icons/reset.svg"
  },
  "alert-circle": {
    "name": "alert-circle",
    "assetId": "icons/alert-circle.svg"
  },
  "close": {
    "name": "close",
    "assetId": "icons/close.svg"
  },
  "check": {
    "name": "check",
    "assetId": "icons/check.svg"
  },
  "minus": {
    "name": "minus",
    "assetId": "icons/minus.svg"
  },
  "plus": {
    "name": "plus",
    "assetId": "icons/plus.svg"
  },
  "chevron-down": {
    "name": "chevron-down",
    "assetId": "icons/chevron-down.svg"
  },
  "table-sort-up": {
    "name": "table-sort-up",
    "assetId": "icons/chevron-up.svg"
  },
  "table-sort-down": {
    "name": "table-sort-down",
    "assetId": "icons/chevron-sort-down.svg"
  },
  "search": {
    "name": "search",
    "assetId": "icons/search.svg"
  },
  "refresh": {
    "name": "refresh",
    "assetId": "icons/refresh.svg"
  },
  "compare": {
    "name": "compare",
    "assetId": "icons/compare.svg"
  },
  "cards": {
    "name": "cards",
    "assetId": "icons/cards.svg"
  },
  "detail": {
    "name": "detail",
    "assetId": "icons/detail.svg"
  },
  "more": {
    "name": "more",
    "assetId": "icons/more.svg"
  },
  "calendar": {
    "name": "calendar",
    "assetId": "icons/calendar.svg"
  }
} as const;
export type UiIcon = keyof typeof iconRegistry;
export function registeredIcon(name: UiIcon) {
  if(!Object.hasOwn(iconRegistry,name))throw new Error("Unknown UI icon: "+name);
  const entry=iconRegistry[name];controlledAsset(entry.assetId);return entry;
}
