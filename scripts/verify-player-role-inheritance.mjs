import fs from "node:fs";
import path from "node:path";

const root = path.resolve(import.meta.dirname, "..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const requireTokens = (file, tokens, label = file) => {
  const content = read(file);
  for (const token of tokens) check(content.includes(token), `${label} 缺少：${token}`);
  return content;
};

const contract = JSON.parse(read("contracts/player-role-inheritance-contract.json"));
check(contract.format_version === "football.player-role-inheritance.v1", "战术角色契约版本错误");
check(contract.resolution_priority.join(",") === "lineup_override,selected_position_default,primary_position_default,missing", "角色继承优先级被改写");
check(contract.p4_semantics.missing_role_score === 50, "缺失角色必须保持 P4 中性分 50");

requireTokens("crates/persistence-postgres/migrations/0046_player_position_default_roles.sql", [
  "ADD COLUMN IF NOT EXISTS default_role_code text",
  "player_positions_default_role_code_check",
  "player_positions_role_lookup_idx",
], "数据库迁移");
requireTokens("crates/domain/src/player/position.rs", [
  "pub default_role_code: Option<String>",
], "球员位置领域模型");
requireTokens("crates/domain/src/player/listing.rs", [
  "pub primary_role_code: Option<String>",
  "pub position_role_map: Value",
], "球员列表领域模型");
requireTokens("crates/domain/src/lineup/player.rs", [
  "pub role_origin: String",
], "阵容领域模型");
requireTokens("crates/domain/src/exchange/contribution.rs", [
  "pub role_code: Option<String>",
  "pub role_origin: Option<String>",
  "pub role_source_position_code: Option<String>",
  "pub tactical_role_code: Option<String>",
  "pub tactical_role_origin: String",
  "pub tactical_role_source_position_code: Option<String>",
  "pub tactical_role_confidence: f64",
], "交换模型：比赛贡献");
requireTokens("crates/domain/src/exchange/ai_match.rs", [
  "pub tactical_role_code: Option<String>",
  "pub tactical_role_origin: String",
  "pub tactical_role_source_position_code: Option<String>",
  "pub lineup_status: String",
], "交换模型：AI 比赛包");
requireTokens("crates/persistence-postgres/src/role_resolution.rs", [
  "resolve_default_tactical_role_in_tx",
  "resolve_tactical_role",
  "ROLE_ORIGIN_LINEUP_OVERRIDE",
  "ROLE_ORIGIN_PLAYER_POSITION_DEFAULT",
  "role_resolution_version",
], "统一角色解析器");

const playerCatalog = read("crates/persistence-postgres/src/player_catalog.rs");
check(
  /FROM football\.lineup_players player[\s\S]{0,5000}JOIN football\.lineups lineup ON lineup\.id = player\.lineup_id[\s\S]{0,5000}position\.valid_from <= lineup\.captured_at::date/.test(playerCatalog),
  "历史比赛阵容角色继承没有锁定 lineup.captured_at 时点",
);
check(
  /primary_position\.default_role_code AS primary_role_code[\s\S]{0,7000}position\.valid_from <= current_date/.test(playerCatalog),
  "当前球员档案角色查询不应被历史阵容时点污染",
);
const presetSource = read("crates/persistence-postgres/src/team_lineup_presets.rs");
check(
  presetSource.includes("let role_as_of = Utc::now().date_naive();") &&
    presetSource.includes("resolve_default_tactical_role_in_tx(") &&
    presetSource.includes("role_as_of,"),
  "同一阵容预设保存过程没有共用一致的角色审计日期",
);
requireTokens("crates/persistence-postgres/src/player_catalog.rs", [
  "position.default_role_code AS primary_role_code",
  "jsonb_object_agg(position.position_code, position.default_role_code)",
  "lineup.captured_at::date",
  "role_source_position_code",
  "metadata_with_role_resolution",
], "球员与比赛阵容持久化");
requireTokens("crates/persistence-postgres/src/team_lineup_presets.rs", [
  "resolve_default_tactical_role_in_tx",
  "metadata_with_role_resolution",
  "player_position_default",
], "阵容预设");
requireTokens("crates/persistence-postgres/src/match_exchange.rs", [
  "role_code: lineup_player.role_code.clone()",
  "role_origin: Some(lineup_player.role_origin.clone())",
  "role_source_position_code: lineup_player",
  "tactical_role_code: lineup_player.role_code.clone()",
  "lineup_status:",
], "比赛 Excel 与 AI 包");
requireTokens("crates/persistence-postgres/src/dynamic_tags.rs", [
  "match-contribution-v2-role-context",
  "metadata->>'role_code'",
  "requested_role_source_position_code",
  "resolve_tactical_role(None, inherited_role.as_ref())",
  "tactical_role_confidence",
  "position.proficiency",
  "ROLE_ORIGIN_PLAYER_POSITION_DEFAULT",
], "球员贡献计算");
requireTokens("crates/persistence-postgres/src/match_prediction.rs", [
  "role_code: player.role_code.clone()",
  "item.tactical_role_confidence * 100.0",
  '"tactical_role_origin"',
  '"tactical_role_source_position_code"',
  '"role_certainty_score"',
  '"missing_role_count"',
], "P4 输入构建");
for (const [file, tokens, label] of [
  ["crates/spreadsheet-io/src/team_package.rs", ["default_role_code", "默认战术角色", "tag_tactical_fit", "apply_default_role_alias"], "球队完整资料包"],
  ["crates/spreadsheet-io/src/lib.rs", ["default_role_code", "自动继承到比赛阵容", "apply_default_role_alias"], "球员 Excel"],
  ["crates/spreadsheet-io/src/monthly_workbook.rs", ["default_role_code", "组织核心", "apply_default_role_alias"], "月度工作簿"],
  ["crates/spreadsheet-io/src/match_workbook.rs", ["primary_role_code", "仅覆盖本场"], "比赛工作簿"],
  ["crates/persistence-postgres/src/spreadsheet_exchange.rs", ["default_role_code", "spreadsheet_clear_fields"], "Excel 提交"],
  ["src/pages/players.ts", ["默认战术角色", "player-position-default-role"], "球员档案 UI"],
  ["src/pages/teams.ts", ["默认角色", "role_code"], "球队名单 UI"],
  ["src/main.ts", ["defaultRoleForPlayer", "position_role_map?.[code]", "player_position_default", "lineup_override", "role_source_position_code"], "阵容编辑 UI"],
  ["src/types.ts", ["role_source_position_code?: string | null", "tactical_role_source_position_code"], "前端角色类型"],
  ["src/pages/review.ts", ["role_source_position_code", "资料继承"], "复盘角色来源 UI"],
]) requireTokens(file, tokens, label);

const predictionReadiness = read("crates/application/src/use_cases/prediction/shared/readiness_checks.rs");
for (const token of ["missing_role_count", "inherited_role_count", "overridden_role_count", "球员位置默认角色"]) {
  check(predictionReadiness.includes(token), `推演完整度缺少：${token}`);
}
const testing = read("docs/TESTING.md");
check(testing.includes("默认战术角色") && testing.includes("自动继承"), "测试文档没有说明角色继承验收");
const readme = read("README.md");
check(readme.includes("默认战术角色全链路"), "根 README 没有本次完整变更记录");

if (failures.length) {
  console.error("默认战术角色全链路验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("默认战术角色全链路验证通过：Excel、存储、球队与球员档案、阵容预设、比赛阵容、AI 包、P4 输入、来源审计和历史时点均已锁定。");
