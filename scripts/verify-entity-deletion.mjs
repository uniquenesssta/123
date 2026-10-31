import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outer = resolve(root, "..");
const text = (path) => readFileSync(join(root, path), "utf8");
const failures = [];
const requireTrue = (condition, message) => { if (!condition) failures.push(message); };

const teamPersistence = [
  text("crates/persistence-postgres/src/adapters/catalog/deletion/safe_delete.rs"),
  text("crates/persistence-postgres/src/adapters/catalog/deletion/delete_write.rs"),
].join("\n");
const entityPersistence = [
  text("crates/persistence-postgres/src/adapters/catalog/deletion/preflight/check.rs"),
  text("crates/persistence-postgres/src/adapters/catalog/deletion/preflight/references.rs"),
  text("crates/persistence-postgres/src/adapters/catalog/deletion/archive/bulk.rs"),
  text("crates/persistence-postgres/src/adapters/catalog/deletion/archive/write.rs"),
].join("\n");
const main = text("src/main.ts");
const teamDeletion = text("src/features/teams/teamDeletion.ts");
const teamFeature = text("src/features/teams/teamsFeature.ts");
const client = text("src/api/client.ts") + "\n" + text("src/platform/tauri/invokeResponse.ts");
const teams = ["src/features/teams/teamPresentation.ts","src/features/teams/profileView.ts","src/features/teams/packageView.ts","src/features/teams/teamsView.ts"].map(text).join("\n");
const readme = readFileSync(join(root, "README.md"), "utf8");

for (const relation of [
  "matches",
  "lineups",
  "player_team_periods",
  "team_coach_periods",
  "team_season_memberships",
  "player_availability",
  "formation_usage",
  "team_tactical_observations",
  "team_ability_observations",
  "substitutions",
  "dynamic_tag_opponents",
  "team_match_reviews",
  "player_match_reviews",
  "player_match_observations",
]) {
  requireTrue(entityPersistence.includes(`(\"${relation}\"`), `球队删除预检缺少引用统计：${relation}`);
}
requireTrue(entityPersistence.includes("can_permanently_delete: total == 0"), "永久删除必须由完整引用统计决定");
requireTrue(entityPersistence.includes("只允许归档"), "引用球队缺少明确归档提示");
requireTrue(teamPersistence.includes('check_entity_deletion("team"'), "球队永久删除未接入统一引用预检");
for (const protectedTable of [
  "football.player_team_periods",
  "football.team_coach_periods",
  "football.player_availability",
  "feature.formation_usage_observations",
  "feature.team_tactical_observations",
  "feature.team_ability_observations",
]) {
  requireTrue(!teamPersistence.includes(`DELETE FROM ${protectedTable}`), `球队永久删除不得级联清理历史或P4输入：${protectedTable}`);
}

requireTrue(main.includes("function removeWorkspaceObjects") && main.includes("function isMissingWorkspaceObjectError") && main.includes("openAvailableWorkspaceTab"), "前端缺少陈旧标签清理与自动恢复");
requireTrue(main.includes("teamsFeature.requestDelete()") && teamDeletion.includes('api.checkEntityDeletion("team", id)') && teamDeletion.includes("check.exists && check.can_permanently_delete"), "球队永久删除前未逐项执行预检");
requireTrue(teamDeletion.includes("api.bulkDeleteTeams(ids)") && teamDeletion.includes("this.consumed.has(ticket)"), "后端删除调用仍包含受保护球队");
requireTrue(teamFeature.includes("teamDeletionCheckSummary") && teamFeature.includes("请使用归档"), "球队受保护原因未在前端解释");
requireTrue(main.includes('removeWorkspaceObjects("teams", [...missing, ...result.deleted_ids])') && main.includes('persistWorkspaceSelection("teams")'), "球队删除结果未同步清理实际删除标签和持久选择");
requireTrue(main.includes('removeWorkspaceObjects("teams", [...result.archived_ids, ...result.already_archived_ids])') && teamFeature.includes("this.ports.afterArchive(result)"), "球队归档后未清理当前活跃工作区标签");
requireTrue(teamFeature.includes("!check.exists") && main.includes('removeWorkspaceObjects("teams", ids)'), "已不存在球队未从陈旧工作区清理");
requireTrue(teams.includes('data-action="bulk-delete-teams"') && teams.includes("永久删除（无引用）") && teams.includes("仅无任何业务或历史引用"), "球队删除按钮仍未明确无引用限制");

requireTrue(client.includes("Array.isArray(childValue)") && client.includes("summary[key] = summarizeLogValue(childValue, key, 1)"), "运行日志仍不能记录删除结果数量、ID和拦截摘要");
requireTrue(readme.includes("永久删除预检") && readme.includes("陈旧标签页"), "README未记录球队删除和工作区一致性修复");

if (failures.length) {
  console.error("实体删除与工作区一致性验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("实体删除与工作区一致性验证通过：P4输入和历史关系不级联删除、永久删除先预检、归档路径明确、陈旧标签自动清理且日志可诊断。");
