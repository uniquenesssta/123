import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const contract = JSON.parse(read("contracts/match-event-facts-contract.json"));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const domain = read("crates/domain/src/match_event.rs");
const packageDomain = read("crates/domain/src/match_review_package.rs");
const reviewDomain = read("crates/domain/src/review.rs");
const persistence = read("crates/persistence-postgres/src/review.rs");
const workbook = read("crates/spreadsheet-io/src/match_review_workbook.rs");
const application = read("crates/application/src/match_review_package.rs");
const migration = read("crates/persistence-postgres/migrations/0042_structured_match_event_facts.sql");
const importFix = read("crates/persistence-postgres/migrations/0043_import_row_team_period_identity.sql");
const types = read("src/types.ts");
const page = read("src/pages/review.ts");
const styles = read("src/styles/app.css");
const integration = read("crates/persistence-postgres/tests/postgres_integration.rs");
const forceDelete = read("crates/persistence-postgres/src/team_force_delete.rs");
const packageJson = JSON.parse(read("package.json"));
const frontendVerifier = read("scripts/verify-frontend.mjs");

check(contract.contract_id === "football.match-event-facts-contract.v1", "D2 比赛事件契约 ID 错误");
check(contract.contract_version === "1.0.0", "D2 比赛事件契约版本错误");
check(contract.package_format === "football.match-review-package.v1", "D2 必须保持赛后资料包格式兼容");
check(contract.workflow.verified_requires_timestamp === true, "D2 契约必须要求已核验事件记录核验时间");
check(contract.workflow.revision_reference_same_match === true, "D2 契约必须限制事件修订关系在同一比赛内");
check(contract.workflow.score_snapshot_errors_block_preview === true, "D2 契约必须在预检阶段阻断事件比分错误");
check(contract.workflow.source_documents_protected_on_force_delete === true, "D2 契约必须保护结构化事件来源文档");
check(contract.workflow.extra_time_score_consistency === true, "D2 契约必须校验加时阶段事件比分与正式赛果");

for (const type of contract.event_types) {
  check(domain.includes(`"${type}"`), `Rust 领域事件类型缺少 ${type}`);
  check(migration.includes(`'${type}'`), `数据库事件类型约束缺少 ${type}`);
  check(types.includes(`| "${type}"`) || types.includes(`= "${type}"`), `TypeScript 事件类型缺少 ${type}`);
}
for (const status of contract.verification_statuses) {
  check(migration.includes(`'${status}'`), `数据库核验状态缺少 ${status}`);
}
for (const status of contract.revision_statuses) {
  check(migration.includes(`'${status}'`), `数据库修订状态缺少 ${status}`);
}

for (const field of [
  "event_key", "sequence_no", "home_score", "away_score", "verification_status",
  "revision_status", "verified_at", "source_document_id", "source_package_id",
  "revision_of_event_id", "updated_at"
]) {
  check(migration.includes(`ADD COLUMN ${field}`), `0042 迁移缺少字段 ${field}`);
  check(packageDomain.includes(`pub ${field}:`) || reviewDomain.includes(`pub ${field}:`), `Rust 事件 DTO 缺少字段 ${field}`);
  check(types.includes(`${field}:`), `TypeScript 事件 DTO 缺少字段 ${field}`);
}
check(migration.includes("match_events_match_event_key_uidx") && migration.includes("(match_id, event_key)"), "D2 缺少比赛内稳定 event_key 唯一约束");
check(migration.includes("match_events_match_sequence_idx"), "D2 缺少事件顺序查询索引");
check(migration.includes("match_events_team_type_idx"), "D2 缺少球队与事件类型查询索引");
check(migration.includes("match_events_source_package_idx"), "D2 缺少资料包来源查询索引");
check(migration.includes("match_events_verified_at_check") && migration.includes("verification_status <> 'verified' OR verified_at IS NOT NULL"), "D2 数据库未强制已核验事件具备 verified_at");
check(migration.includes("verification_status text NOT NULL DEFAULT 'unverified'") && migration.includes("verification_status = 'verified'"), "D2 新事件默认核验状态或旧事件回填策略错误");
check(migration.includes("match_events_revision_of_idx"), "D2 缺少事件修订关系查询索引");

check(persistence.includes("ON CONFLICT (match_id, event_key) DO UPDATE"), "相同 event_key 没有按正式事实执行幂等更新");
check(persistence.includes("SET revision_status = 'superseded'") && persistence.includes("NOT (event_key = ANY($2::text[]))"), "新资料包缺失的旧事件没有进入 superseded 审计状态");
check(persistence.includes("event.revision_status <> 'superseded'"), "当前事件查询没有排除 superseded 历史版本");
check(persistence.includes("fn summarize_match_events") && reviewDomain.includes("pub event_summary:"), "正式复盘详情缺少事件统计摘要");
check(persistence.includes("MatchEventType::GoalkeeperChange"), "后端门禁未覆盖门将更换事件的双球员关系");
check(application.includes("MatchEventType::OwnGoal") && application.includes("乌龙球球员应属于事件受益球队的对手"), "应用层事件身份校验未正确处理乌龙球球队关系");
check(persistence.includes("event.event_type == MatchEventType::OwnGoal") && persistence.includes("乌龙球球员必须属于事件受益球队的对手"), "持久化事务门禁未正确处理乌龙球球队关系");
check(persistence.includes("revision_event_ids") && persistence.includes("比赛事件只能修订同一场比赛的历史事件"), "持久化事务门禁未校验 revision_of_event_id 的比赛归属");
check(forceDelete.includes("UNION SELECT source_document_id FROM review.match_events") && forceDelete.includes("NOT EXISTS (SELECT 1 FROM review.match_events WHERE source_document_id=document.id)"), "球队强制删除未正确保护结构化事件来源文档");

for (const header of [
  "event_key", "sequence_no", "home_score", "away_score", "verification_status",
  "revision_status", "verified_at", "source_document_id", "revision_of_event_id"
]) {
  check(workbook.includes(`"${header}"`), `赛后资料包事件工作表缺少 ${header}`);
}
for (const guard of [
  "event_key 重复", "sequence_no 重复", "已核验事件必须填写 verified_at",
  "不能手工填写 superseded", "主客队事件后比分必须同时填写或同时留空"
]) {
  check(workbook.includes(guard), `赛后资料包预检缺少门禁：${guard}`);
}
check(workbook.includes("validate_event_score_consistency"), "赛后资料包没有校验事件比分与正式赛果");
check(workbook.includes("home_goals_extra_time") && workbook.includes("最后一条加时有效事件后比分") && workbook.includes("存在加时阶段的进球或比分事件"), "资料包预检没有校验加时事件比分与正式加时赛果");
check(workbook.includes("errors: &mut Vec<String>") && workbook.includes("最后一条 90 分钟有效事件后比分") && workbook.includes("cancelled/corrected"), "资料包预检没有在提交前阻断会被后端拒绝的事件比分错误");
check(persistence.includes("has_extra_time_score") && persistence.includes("正式合计赛果") && persistence.includes("主客队加时进球必须同时填写或同时留空"), "持久化门禁没有校验加时比分与正式赛果");
check(domain.includes("matches!(self, Self::Goal | Self::OwnGoal | Self::PenaltyGoal)"), "比赛事件计分语义必须使用正确的 matches! 表达式");
check(domain.includes("!matches!(self, Self::Var | Self::Other)"), "比赛事件球队必填语义必须使用正确的 matches! 表达式");
check(domain.includes("matches!(self, Self::Active | Self::Corrected)"), "比赛事件有效修订状态必须使用正确的 matches! 表达式");
check(!domain.includes("matches!(Self::Goal | Self::OwnGoal | Self::PenaltyGoal, self)"), "检测到反向 matches! 表达式，Rust 将无法正确编译");
check(!integration.includes("SELECT subrecord_key\n        SELECT subrecord_key"), "球队资料包效力身份集成测试包含重复 SELECT");

check(types.includes("export interface MatchEventSummary") && types.includes("event_summary: MatchEventSummary"), "前端类型缺少事件摘要");
check(page.includes("结构化比赛事实") && page.includes("事件时间线") && page.includes("event_summary"), "赛后复盘页面没有显示结构化事件时间线");
for (const className of [".match-event-panel", ".match-event-summary", ".match-event-timeline", ".match-event-row"]) {
  check(styles.includes(className), `事件时间线缺少样式 ${className}`);
}

check(packageDomain.includes("legacy_event_payload_receives_safe_structured_defaults"), "缺少旧版赛后事件载荷兼容性测试");
check(integration.includes("structured_match_events_are_queryable_and_revision_aware"), "缺少结构化比赛事件 PostgreSQL 集成测试");
check(integration.includes("team_package_player_team_period_subrecords_are_distinct"), "缺少球队资料包效力履历唯一身份回归测试");
check(importFix.includes("WHEN 'player_team_period'") && importFix.includes("payload ->> 'team_id'") && importFix.includes("payload ->> 'team_key'") && importFix.includes("payload ->> 'team_name'"), "0043 未按球队身份区分同一物理行的两条效力履历");

check(
  packageJson.scripts.build?.includes("verify-frontend.mjs")
    && packageJson.scripts["verify:frontend"]?.includes("verify-frontend.mjs")
    && frontendVerifier.includes('"verify-match-event-facts.mjs"'),
  "build 或 verify:frontend 未通过统一验证编排接入 D2 事件事实门禁",
);
check(packageJson.scripts["verify:match-event-facts"] === "node scripts/verify-match-event-facts.mjs", "缺少 verify:match-event-facts 脚本");

if (failures.length) {
  console.error("阶段 D2 结构化比赛事件验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log(`阶段 D2 验证通过：${contract.event_types.length} 类比赛事件具备稳定身份、来源核验、修订状态、结构化查询、统计摘要与赛后时间线。`);
