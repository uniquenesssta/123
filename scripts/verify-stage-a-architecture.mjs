import fs from "node:fs";

const root = new URL("../", import.meta.url);
const read = (path) => fs.readFileSync(new URL(path, root), "utf8");
const contract = JSON.parse(read("contracts/match-review-workflow-contract.json"));
const failures = [];
const check = (ok, message) => {
  if (!ok) failures.push(message);
};
const pascal = (value) => value
  .split("_")
  .map((part) => part.slice(0, 1).toUpperCase() + part.slice(1))
  .join("");

const domainRecord = read("crates/domain/src/match_review_package.rs");
const domainWorkflow = read("crates/domain/src/match_review_workflow.rs");
const application = read("crates/application/src/match_review_package.rs");
const postmatchApplication = read("crates/application/src/postmatch.rs");
const persistence = read("crates/persistence-postgres/src/match_review_package.rs");
const commands = read("src-tauri/src/commands/postmatch.rs");
const apiClient = read("src/api/client.ts");
const types = read("src/types.ts");
const page = read("src/pages/review.ts");
const main = read("src/main.ts");
const frontendWorkflowHelpers = read("src/app/matchReviewWorkflow.ts");
const integration = read("crates/persistence-postgres/tests/postgres_integration.rs");

check(contract.authority.frontend_must_not_rank_status_strings === true, "契约未禁止前端自行排列状态字符串");
check(contract.authority.frontend_must_not_branch_on_status_strings === true, "契约未禁止前端用状态字符串决定业务权限");
check(domainWorkflow.includes("pub enum MatchReviewPackageWorkflowStatus"), "领域层缺少强类型工作流状态");
check(domainWorkflow.includes("pub enum MatchReviewPackageWorkflowAction"), "领域层缺少强类型工作流动作");
check(domainWorkflow.includes("pub enum MatchReviewPackageWorkflowStep"), "领域层缺少强类型工作流步骤");
for (const status of contract.statuses) {
  check(domainWorkflow.includes(`Self::${pascal(status)} => "${status}"`), `Rust 状态缺少 ${status}`);
  check(types.includes(`| "${status}"`) || types.includes(`=\n  | "${status}"`), `TypeScript 状态缺少 ${status}`);
}
for (const step of contract.steps) {
  check(domainWorkflow.includes(pascal(step)), `Rust 步骤缺少 ${step}`);
  check(types.includes(`"${step}"`), `TypeScript 步骤缺少 ${step}`);
}
for (const action of contract.actions) {
  check(domainWorkflow.includes(pascal(action)), `Rust 动作缺少 ${action}`);
  check(types.includes(`"${action}"`), `TypeScript 动作缺少 ${action}`);
}
for (const field of contract.workflow_record_fields) {
  check(domainRecord.includes(`pub ${field}:`), `Rust 工作流记录缺少 ${field}`);
  check(types.includes(`${field}:`), `TypeScript 工作流记录缺少 ${field}`);
}
check(domainWorkflow.includes("fn allowed_actions(") && domainWorkflow.includes("fn blocking_reason("), "领域层没有集中计算动作权限与阻塞原因");
check(domainWorkflow.includes("pub fn require_action("), "领域层没有统一前置动作门禁");
check(persistence.includes("MatchReviewPackageWorkflowStatus::parse"), "持久化层仍直接传播字符串状态");
check(persistence.includes(".with_capabilities()"), "持久化读取后没有生成统一工作流能力");
check(persistence.includes("read_match_review_package_workflow_by_review"), "结算链路无法按复盘定位资料包工作流");
for (const action of ["PreviewImport", "ConfirmImport", "CommitFacts", "GenerateReview"]) {
  check(application.includes(`MatchReviewPackageWorkflowAction::${action}`), `应用层缺少 ${action} 门禁`);
}
check(postmatchApplication.includes("MatchReviewPackageWorkflowAction::SettleReview"), "正式结算没有复用统一工作流门禁");
check(!page.includes("function workflowStateRank"), "前端仍保留状态排名表");
const frontendWorkflow = `${page}\n${main}`;
check(!/(?:matchReviewPackageWorkflow|activeWorkflow|workflow)\?*\.status\s*(?:===|!==|==|!=)/.test(frontendWorkflow), "前端仍根据工作流状态字符串决定业务权限");
check(main.includes("matchReviewWorkflowAllows"), "前端事件处理没有复用工作流能力助手");
check(frontendWorkflowHelpers.includes("allowed_actions.includes(action)") && frontendWorkflowHelpers.includes("completed_steps.includes(step)") && frontendWorkflowHelpers.includes("blocking_reasons.find"), "前端工作流能力助手没有只消费后端能力 DTO");
for (const field of ["completed_steps", "allowed_actions", "blocking_reasons"]) {
  check(frontendWorkflowHelpers.includes(field), `前端能力助手没有消费后端字段 ${field}`);
}
check(page.includes("next_action"), "复盘页面没有消费后端字段 next_action");
check(page.includes("业务权限由后端工作流统一返回"), "界面未说明后端状态权威边界");

for (const query of contract.operations.queries) {
  check(commands.includes(`pub async fn ${query}(`), `Tauri 查询入口缺少 ${query}`);
  check(apiClient.includes(`invoke("${query}"`), `前端查询入口缺少 ${query}`);
}
for (const command of contract.operations.commands) {
  check(commands.includes(`pub async fn ${command}(`), `Tauri 命令入口缺少 ${command}`);
  check(apiClient.includes(`invoke("${command}"`), `前端命令入口缺少 ${command}`);
}
check(!commands.includes("sqlx::") && !commands.includes("SELECT ") && !commands.includes("UPDATE "), "Tauri 赛后命令包含 SQL，未保持薄适配层");
check(commands.includes(".service") && commands.includes(".await"), "Tauri 赛后命令未委托应用服务");
check(integration.includes(contract.verification.postgres_integration_test), "缺少工作流 PostgreSQL 集成测试");
check(integration.includes("PreviewBlocked") && integration.includes("PreviewValid") && integration.includes("FactsCommitted") && integration.includes("ReviewCreated") && integration.includes("Settled"), "数据库集成测试未覆盖完整资料包状态迁移");
check(integration.includes("read_match_review_package_workflow_by_review") && integration.includes("重复结算保持幂等"), "数据库集成测试未覆盖按复盘定位与幂等结算");

if (failures.length) {
  console.error("阶段 A 架构收敛验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("阶段 A 架构收敛验证通过：赛后工作流由 Rust 领域状态统一解释，前端只消费能力 DTO，Tauri 保持薄适配，并具备 PostgreSQL 状态迁移测试。 ");
