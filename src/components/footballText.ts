import type { AvailabilityStatus, PlayerDetail, PlayerListItem, PreferredFoot, TeamSquadPlayer } from "../types";

const POSITION_LABELS: Record<string, string> = {
  GK: "门将",
  SW: "清道夫",
  CB: "中后卫",
  LCB: "左中后卫",
  RCB: "右中后卫",
  LB: "左后卫",
  RB: "右后卫",
  LWB: "左翼卫",
  RWB: "右翼卫",
  DM: "后腰",
  CDM: "防守型中场",
  CM: "中前卫",
  LCM: "左中前卫",
  RCM: "右中前卫",
  LDM: "左后腰",
  RDM: "右后腰",
  AM: "前腰",
  CAM: "攻击型中场",
  LAM: "左攻击型中场",
  RAM: "右攻击型中场",
  LM: "左前卫",
  RM: "右前卫",
  LW: "左边锋",
  RW: "右边锋",
  LF: "左前锋",
  RF: "右前锋",
  SS: "影锋",
  CF: "中锋",
  ST: "前锋",
  LST: "左前锋",
  RST: "右前锋",
};

const AVAILABILITY_LABELS: Record<AvailabilityStatus | "", string> = {
  available: "可用",
  unavailable: "不可出场",
  doubtful: "出场存疑",
  injured: "伤病",
  suspended: "停赛",
  rested: "轮休",
  returning: "恢复中",
  unknown: "状态未知",
  "": "状态未知",
};

const FOOT_LABELS: Record<PreferredFoot, string> = {
  left: "左脚",
  right: "右脚",
  both: "双脚",
  unknown: "未知",
};

export function positionLabel(code: string | null | undefined): string {
  const raw = code?.trim();
  if (!raw) return "位置待补";
  const tokens = raw.split(/[\s,/|;+]+/).filter(Boolean);
  return tokens
    .map((token) => {
      const upper = token.toUpperCase();
      return POSITION_LABELS[upper] ?? POSITION_LABELS[upper.replace(/[^A-Z]/g, "")] ?? token;
    })
    .join(" / ");
}

export function availabilityLabel(value: AvailabilityStatus | null | undefined): string {
  return AVAILABILITY_LABELS[value ?? ""] ?? "状态未知";
}

export function preferredFootLabel(value: PreferredFoot | null | undefined): string {
  return value ? FOOT_LABELS[value] : "未知";
}

export function playerStatusLabel(value: string | null | undefined): string {
  return ({ active: "现役", inactive: "非活跃", retired: "退役", unknown: "未知" } as Record<string, string>)[value ?? ""] ?? "未知";
}

export function teamTypeLabel(value: string | null | undefined): string {
  return ({ national: "国家队", club: "俱乐部", reserve: "预备队", youth: "青年队", women: "女子队", other: "其他" } as Record<string, string>)[value ?? ""] ?? "未分类";
}

export function hasChineseText(value: string): boolean {
  return /[\u3400-\u9fff]/u.test(value);
}

function recordString(record: Record<string, unknown>, key: string): string | null {
  const value = record[key];
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

export function detailLocalizedName(detail: PlayerDetail | null | undefined): string | null {
  if (!detail) return null;
  const aliases = detail.names as Array<Record<string, unknown>>;
  const preferred = aliases.find((item) => {
    const language = recordString(item, "language_code")?.toLowerCase() ?? "";
    const name = recordString(item, "name") ?? "";
    return language === "zh-cn" || language === "zh-hans" || language === "zh" || hasChineseText(name);
  });
  return preferred ? recordString(preferred, "name") : null;
}

export function displayPlayerName(
  player:
    | Pick<PlayerListItem, "canonical_name" | "localized_name" | "alternate_name">
    | Pick<TeamSquadPlayer, "player_name" | "localized_name">,
): { primary: string; secondary: string | null } {
  const canonical = "canonical_name" in player ? player.canonical_name : player.player_name;
  const localized = player.localized_name?.trim() || null;
  const alternate = "alternate_name" in player ? player.alternate_name?.trim() || null : null;
  if (hasChineseText(canonical)) {
    const secondary = alternate && alternate !== canonical ? alternate : localized && localized !== canonical ? localized : null;
    return { primary: canonical, secondary };
  }
  if (localized && localized !== canonical) return { primary: localized, secondary: canonical };
  if (alternate && alternate !== canonical) return { primary: canonical, secondary: alternate };
  return { primary: canonical, secondary: null };
}

export function detailPlayerName(detail: PlayerDetail): { primary: string; secondary: string | null } {
  const localized = detailLocalizedName(detail);
  if (localized && localized !== detail.player.canonical_name) {
    return { primary: localized, secondary: detail.player.canonical_name };
  }
  return { primary: detail.player.canonical_name, secondary: null };
}

export function ageFromBirthDate(value: string | null | undefined): string {
  if (!value) return "年龄未知";
  const birth = new Date(`${value}T00:00:00`);
  if (Number.isNaN(birth.getTime())) return "年龄未知";
  const now = new Date();
  let age = now.getFullYear() - birth.getFullYear();
  const monthDelta = now.getMonth() - birth.getMonth();
  if (monthDelta < 0 || (monthDelta === 0 && now.getDate() < birth.getDate())) age -= 1;
  return age >= 0 ? `${age}岁` : "年龄未知";
}

export function initials(value: string): string {
  const text = value.trim();
  if (!text) return "?";
  if (hasChineseText(text)) return Array.from(text).slice(0, 2).join("");
  return text.split(/\s+/).filter(Boolean).slice(0, 2).map((part) => part[0]?.toUpperCase() ?? "").join("") || "?";
}
