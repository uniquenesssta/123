import type { TeamDetail } from "../../types";
export const teamTypeLabels: Record<string, string> = {
  club: "俱乐部一线队",
  national: "国家/地区代表队",
  reserve: "预备队",
  youth: "青年队",
  women: "女子队",
  other: "其他",
};

export const tacticalStyleLabels: Record<string, string> = {
  balanced: "均衡",
  possession: "控球组织",
  direct: "直接推进",
  counter: "快速反击",
  pressing: "高位压迫",
  defensive: "防守优先",
  custom: "自定义",
};

export const availabilityLabels: Record<string, string> = {
  available: "可用",
  unavailable: "不可出场",
  doubtful: "存疑",
  injured: "伤病",
  suspended: "停赛",
  rested: "轮休",
  returning: "恢复中",
  unknown: "未知",
};

export function currentLocalizedTeamName(detail: TeamDetail, at = new Date().toISOString().slice(0, 10)): string {
  const records = detail.names.filter((item) => {
    if ((item.valid_from && item.valid_from > at) || (item.valid_to && item.valid_to < at)) return false;
    const language = item.language_code?.toLowerCase() ?? "";
    return ["zh-cn", "zh-hans", "zh"].includes(language) || /[一-龥]/u.test(item.name);
  });
  records.sort((left, right) => {
    const languageRank = (language: string | null) => ["zh-cn", "zh-hans", "zh"].includes(language?.toLowerCase() ?? "") ? 1 : 0;
    const byLanguage = languageRank(right.language_code) - languageRank(left.language_code);
    if (byLanguage !== 0) return byLanguage;
    return (right.valid_from ?? "").localeCompare(left.valid_from ?? "") || right.id.localeCompare(left.id);
  });
  return records[0]?.name ?? "";
}
