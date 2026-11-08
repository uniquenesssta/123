import type { RulePackageSummary } from "../../types";

/** One newest creation per package key; callers keep their existing family/status filters. */
export function newestRulePackages(packages: readonly RulePackageSummary[]): RulePackageSummary[] {
  const newest = new Map<string, RulePackageSummary>();
  for (const item of [...packages].sort((left, right) => Date.parse(right.created_at) - Date.parse(left.created_at))) {
    if (!newest.has(item.package_key)) newest.set(item.package_key, item);
  }
  return [...newest.values()];
}
