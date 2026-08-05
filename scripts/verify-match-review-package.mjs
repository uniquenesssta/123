import fs from "node:fs";
const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const check = (ok, message) => { if (!ok) failures.push(message); };
const domain = read("crates/domain/src/match_review_package.rs");
const reviewDomain = read("crates/domain/src/review.rs");
const workbook = read("crates/spreadsheet-io/src/match_review_workbook.rs");
const application = read("crates/application/src/match_review_package.rs");
const persistence = read("crates/persistence-postgres/src/match_review_package.rs");
const reviewPersistence = read("crates/persistence-postgres/src/review.rs");
const entityPersistence = read("crates/persistence-postgres/src/entity_catalog.rs");
const forceDeletePersistence = read("crates/persistence-postgres/src/team_force_delete.rs");
const migration = read("crates/persistence-postgres/migrations/0040_match_review_package_workflow.sql");
const commands = read("src-tauri/src/commands/postmatch.rs");
const commandRegistry = read("src-tauri/src/bootstrap/command_registry.rs");
const main = read("src/main.ts");
const page = read("src/pages/review.ts");
const api = read("src/api/client.ts");
const loader = read("src/controllers/pageLoaders.ts");

check(workbook.includes('let format_version = required_text(&metadata, "format_version")?.to_string();'), "赛后复盘资料包 format_version 必须转换为 String，避免 Rust E0308");
for (const token of ["football.match-review-package.v1", "MatchReviewPackagePreview", "MatchReviewPackageWorkflowRecord", "MatchReviewPackageConfirmationRequest", "MatchReviewPackageComparison", "MatchReviewPackageIdentityCheck"]) check(domain.includes(token), `领域契约缺少 ${token}`);
check(reviewDomain.includes("pub events: Vec<crate::MatchReviewEventDraft>"), "MatchReviewDraft 未携带结构化比赛事件");
check(reviewDomain.includes("pub struct MatchReviewEventRecord") && reviewDomain.includes("pub events: Vec<MatchReviewEventRecord>"), "正式复盘详情缺少可查询事件记录");
for (const sheet of ["说明与校验", "元数据", "比赛与赛果", "实际阵容", "换人与事件", "球员表现", "球员参考", "赛前快照", "字段字典"]) check(workbook.includes(`"${sheet}"`), `赛后复盘资料包缺少工作表：${sheet}`);
for (const guard of ["必须恰好 11 名首发", "缺少 0–10 评分", "替补出场球员", "进球事件计数"]) check(workbook.includes(guard), `资料包预检缺少门禁：${guard}`);
check(workbook.includes("events: events.clone()"), "工作簿事件未进入 MatchReviewDraft 正式字段");
check(workbook.includes("performance_score: None") && workbook.includes("provider_rating: rating"), "0–10 外部评分未通过 provider_rating 归一化链路");
check(workbook.includes("valid_periods") && workbook.includes("补时分钟必须位于 0–30"), "事件阶段或补时分钟未在预检阶段阻断");

for (const table of ["review.match_events", "review.match_review_package_workflows"]) check(migration.includes(table), `数据库迁移缺少 ${table}`);
for (const status of ["exported", "preview_blocked", "preview_valid", "confirmed", "facts_committed", "review_created", "settled", "superseded"]) check(migration.includes(`'${status}'`), `工作流迁移缺少状态 ${status}`);
check(migration.includes("match_review_package_one_active_per_match_uidx"), "缺少同场比赛单一活动资料包约束");
check(migration.includes("pre_match_snapshot jsonb") && migration.includes("export_database_snapshot jsonb"), "导出时没有冻结赛前值和数据库值摘要");
check(reviewPersistence.includes("INSERT INTO review.match_events") && reviewPersistence.includes("list_match_events"), "普通比赛事件未结构化写入并查询");
check(entityPersistence.includes('("match_events", "SELECT count(*)::bigint FROM review.match_events') && forceDeletePersistence.includes("FROM review.match_events WHERE team_id=$1"), "结构化比赛事件未接入实体删除保护与强制清除影响集合");
for (const method of ["register_match_review_package_export", "record_match_review_package_preview", "confirm_match_review_package_workflow", "mark_match_review_package_facts_committed", "mark_match_review_package_review_created", "mark_match_review_package_settled"]) check(persistence.includes(`fn ${method}`), `持久化层缺少 ${method}`);

for (const method of ["export_match_review_package", "preview_match_review_package", "read_match_review_package_workflow", "confirm_match_review_package", "commit_match_review_package_facts", "generate_match_review_from_package"]) {
  check(application.includes(`fn ${method}`), `应用层缺少 ${method}`);
  check(commands.includes(`fn ${method}`), `Tauri 缺少 ${method}`);
  check(commandRegistry.includes(`commands::${method}`), `Tauri 命令注册缺少 ${method}`);
}
check(application.includes("register_match_review_package_export"), "导出后没有登记本轮 package_id 和 SHA256");
check(application.includes("read_active_match_review_package_workflow") && application.includes("最近一次导出的资料包"), "预检没有严格绑定本轮导出");
check(application.includes("validate_match_review_package(import_path") && application.includes("已确认资料包发生变化"), "确认/写入阶段没有重新读取并复检文件");
check(application.includes("commit_match_review_facts") && application.includes("generate_match_review_from_package"), "真实事实写入与正式复盘未拆分为独立阶段");
check(application.includes("snapshot_from_lineups") && application.includes("snapshot_from_pair") && application.includes("validate_event_identities"), "三方值对照或事件身份预检未接入后端");

for (const method of ["exportMatchReviewPackage", "previewMatchReviewPackage", "readMatchReviewPackageWorkflow", "confirmMatchReviewPackage", "commitMatchReviewPackageFacts", "generateMatchReviewFromPackage"]) check(api.includes(`${method}:`), `前端 API 缺少 ${method}`);
for (const action of ["export-match-review-package", "preview-match-review-package", "confirm-match-review-package", "commit-match-review-package-facts", "generate-match-review-from-package", "inspect-postmatch-readiness", "settle-postmatch-review"]) check(main.includes(`case "${action}"`) && page.includes(`data-action="${action}"`), `赛后复盘链路缺少动作 ${action}`);
for (const label of ["选择比赛", "导出赛后复盘资料包", "在外部补充真实比赛事实和球员量化数据", "导入并预检", "人工确认", "写入真实赛后事实", "生成正式复盘", "正式结算", "进入分析与历史"]) check(page.includes(label), `复盘页缺少固定链路阶段：${label}`);
for (const label of ["本步用途", "完成条件", "阻塞原因", "下一步动作"]) check(page.includes(label), `复盘步骤缺少可见说明：${label}`);
for (const label of ["赛前值", "当前数据库值", "准备导入值", "身份匹配情况"]) check(page.includes(label), `复盘预检缺少可见对照：${label}`);
check(page.includes("reviewPackageWorkspace(selected, packageWorkflow, packagePreview, detail, settlement, activeWorkflowStep)"), "完整链路未在无比赛/未初始化状态下固定渲染");
check(page.includes("review-stage-rail") && page.includes("review-stage-workspace") && page.includes("select-review-workflow-step"), "九步链路未采用固定步骤轨道与当前步骤工作区");
check(!page.includes("selected ? reviewPackageWorkspace"), "赛后复盘链路仍会在未选择比赛时被整体隐藏");
check(page.includes("赛前预计/确认阵容与模型快照不会被覆盖") || page.includes("不覆盖赛前快照"), "复盘页未明确赛前快照不可覆盖");
check(loader.includes("readMatchReviewPackageWorkflow") && loader.includes("listPostmatchSettlements"), "复盘页加载器未恢复持久化工作流和结算状态");
check(domain.includes("pub preview: Option<MatchReviewPackagePreview>") && persistence.includes("workflow.preview_payload") && loader.includes("preview: workflow?.preview ?? null") && main.includes("matchReviewPackagePreview = result.preview"), "页面重启后未恢复本轮持久化预检结果");
check(loader.includes(": matches[0]?.match_record.id ?? null;"), "进入赛后复盘页时必须自动选择第一场可复盘比赛");

if (failures.length) {
  console.error("赛后复盘资料包链路验证失败：");
  failures.forEach((x) => console.error(`- ${x}`));
  process.exit(1);
}
console.log("赛后复盘资料包链路验证通过：九步状态机固定可见，本轮导出严格绑定，事件结构化入库，确认、事实写入、复盘和结算顺序受前后端门禁约束。");
