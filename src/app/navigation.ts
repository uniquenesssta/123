import type { AppIcon } from "../components/icons";
import type { Page } from "../types";

export interface NavigationRequest<PageKey> {
  readonly sequence: number;
  readonly page: PageKey;
}

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

/**
 * One global business level and one contextual function level.
 * Any deeper hierarchy belongs inside the active page.
 */
export const PRIMARY_NAVIGATION: readonly PrimaryNavigationModule[] = [
  {
    key: "home",
    label: "首页",
    description: "平台状态与快捷入口",
    icon: "home",
    default_page: "dashboard",
    items: [
      { page: "dashboard", label: "数据总览", description: "查看运行状态、待办与快捷入口", icon: "home" },
    ],
  },
  {
    key: "matches",
    label: "比赛",
    description: "比赛、阵容、推演与复盘",
    icon: "calendar",
    default_page: "lineups",
    items: [
      { page: "lineups", label: "比赛中心", description: "管理比赛、阵容与快照", icon: "calendar" },
      { page: "prediction", label: "赛事推演", description: "检查并运行 P4 / P7", icon: "spark" },
      { page: "review", label: "赛后复盘", description: "录入赛果并完成归因复盘", icon: "review" },
      { page: "runs", label: "推演记录", description: "查看正式与影子运行记录", icon: "history" },
    ],
  },
  {
    key: "resources",
    label: "资源",
    description: "球队、球员与工作簿",
    icon: "shield",
    default_page: "teams",
    items: [
      { page: "teams", label: "球队中心", description: "球队档案、阵容与预设", icon: "shield" },
      { page: "players", label: "球员中心", description: "球员档案、能力与动态标签", icon: "users" },
      { page: "lineup_presets", label: "阵容预设", description: "维护球队常用首发、替补与阵型", icon: "review" },
      { page: "workbooks", label: "Excel 工作包", description: "模板导出、预检与导入", icon: "sheet" },
    ],
  },
  {
    key: "model",
    label: "模型",
    description: "规则、路由与发布门禁",
    icon: "settings",
    default_page: "rules",
    items: [
      { page: "rules", label: "赛事设置", description: "赛事目录、规则包与模型路由", icon: "settings" },
      { page: "release", label: "发布验收", description: "执行完整发布门禁并留存证据", icon: "review" },
    ],
  },
  {
    key: "analysis",
    label: "分析",
    description: "历史数据与归因分析",
    icon: "chart",
    default_page: "analytics",
    items: [
      { page: "analytics", label: "分析与历史", description: "查看赛前、赛后与收敛数据", icon: "chart" },
    ],
  },
  {
    key: "ai",
    label: "AI",
    description: "问答工作台与兼容接口",
    icon: "chat",
    default_page: "api_workspace",
    items: [
      { page: "api_workspace", label: "AI 问答", description: "围绕比赛、球队和球员进行问答", icon: "chat" },
      { page: "openai", label: "兼容 API", description: "管理兼容协议与连接配置", icon: "plug" },
    ],
  },
  {
    key: "management",
    label: "管理",
    description: "数据库、日志与系统信息",
    icon: "database",
    default_page: "database",
    items: [
      { page: "database", label: "数据库", description: "连接状态、迁移与维护", icon: "database" },
      { page: "logs", label: "问题日志", description: "查看运行问题和诊断线索", icon: "alert" },
      { page: "architecture", label: "系统信息", description: "查看架构与运行边界", icon: "info" },
    ],
  },
] as const;

const PAGE_TO_MODULE = new Map<Page, PrimaryNavigationModule>();
const PAGE_TO_ITEM = new Map<Page, SecondaryNavigationItem>();
for (const module of PRIMARY_NAVIGATION) {
  for (const item of module.items) {
    if (PAGE_TO_MODULE.has(item.page)) {
      throw new Error(`页面 ${item.page} 被重复分配到一级菜单`);
    }
    PAGE_TO_MODULE.set(item.page, module);
    PAGE_TO_ITEM.set(item.page, item);
  }
}

export function navigationModuleForPage(page: Page): PrimaryNavigationModule {
  const module = PAGE_TO_MODULE.get(page);
  if (!module) throw new Error(`页面 ${page} 未配置一级菜单`);
  return module;
}

export function navigationItemForPage(page: Page): SecondaryNavigationItem {
  const item = PAGE_TO_ITEM.get(page);
  if (!item) throw new Error(`页面 ${page} 未配置二级菜单`);
  return item;
}

/**
 * Tracks the latest page navigation so slow responses from an older page
 * cannot overwrite the state of the page the user is currently viewing.
 */
export class NavigationCoordinator<PageKey> {
  private sequence = 0;
  private active: NavigationRequest<PageKey> | null = null;

  begin(page: PageKey): NavigationRequest<PageKey> {
    const request = { sequence: ++this.sequence, page } as const;
    this.active = request;
    return request;
  }

  isCurrent(request: NavigationRequest<PageKey>): boolean {
    return this.active?.sequence === request.sequence;
  }

  complete(request: NavigationRequest<PageKey>): boolean {
    if (!this.isCurrent(request)) return false;
    this.active = null;
    return true;
  }

  cancel(): void {
    this.sequence += 1;
    this.active = null;
  }
}
