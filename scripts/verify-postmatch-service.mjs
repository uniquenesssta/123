import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const assert = (condition, message) => {
  if (!condition) throw new Error(message);
};

const facade = read("crates/application/src/services/postmatch/facade.rs");
const service = read("crates/application/src/services/postmatch/service.rs");
const ports = read("crates/application/src/ports/postmatch/mod.rs");
const reviewState = read("crates/application/src/ports/review/package.rs");
const adapter = read("crates/application/src/composition/adapters/postmatch.rs");
const reviewAdapter = read("crates/application/src/composition/adapters/review.rs");
const settle = read("crates/application/src/use_cases/postmatch/settle_postmatch_review/mod.rs");
const composition = read("crates/application/src/composition/application_composition.rs");
const applicationService = read("crates/application/src/service/application_service.rs");
const contract = JSON.parse(read("contracts/postmatch-settlement-contract.json"));
const useCaseFiles = [
  "postmatch_settlement_readiness",
  "settle_postmatch_review",
  "list_postmatch_settlements",
  "list_evidence_scoring_items",
  "decide_evidence_scoring_item",
  "refresh_postmatch_monitoring",
  "postmatch_overview",
].map((name) => read(`crates/application/src/use_cases/postmatch/${name}/mod.rs`));
const applicationLayer = [facade, service, ...useCaseFiles].join("\n");

assert(!exists("crates/application/src/postmatch.rs"), "旧 Postmatch Application owner 仍存在");
assert(
  contract.artifacts.includes("crates/application/src/services/postmatch/facade.rs"),
  "Postmatch 契约未指向新 authoritative facade",
);
for (const command of contract.commands) {
  assert(facade.includes(`fn ${command}`), `Postmatch facade 缺少公共入口：${command}`);
  assert(service.includes(`fn ${command}`), `Postmatch service 缺少委托：${command}`);
}
assert(
  ports.includes("trait PostmatchSettlementPort") && ports.includes("limit: u32"),
  "Postmatch Settlement Port 未对齐真实 limit 契约",
);
assert(
  ports.includes("status: Option<&str>") && ports.includes("async fn overview(&self, limit: u32)"),
  "Postmatch Port 查询能力未对齐真实 API",
);
assert(
  adapter.includes("postmatch_settlement_readiness") &&
    adapter.includes("settle_postmatch_review") &&
    adapter.includes("list_postmatch_settlements") &&
    adapter.includes("list_evidence_scoring_items") &&
    adapter.includes("decide_evidence_scoring_item") &&
    adapter.includes("refresh_postmatch_monitoring") &&
    adapter.includes("postmatch_overview"),
  "Postmatch PostgreSQL adapter 委托不完整",
);
assert(
  reviewState.includes("read_workflow_by_review") && reviewState.includes("mark_settled"),
  "Match Review Package State Port 缺少 Postmatch 状态推进能力",
);
assert(
  reviewAdapter.includes("read_match_review_package_workflow_by_review") &&
    reviewAdapter.includes("mark_match_review_package_settled"),
  "Review state adapter 未复用既有资料包状态持久化",
);
assert(
  settle.includes("MatchReviewPackageWorkflowAction::SettleReview") &&
    settle.includes("require_action") &&
    settle.includes("mark_settled"),
  "Postmatch settle use case 未保留资料包状态机门禁",
);
assert(
  composition.includes("PostmatchService") &&
    applicationService.includes("postmatch: PostmatchService"),
  "Application composition 未聚合唯一 PostmatchService",
);
for (const forbidden of [
  "football_persistence_postgres",
  "sqlx::",
  "PostgresStore",
  "PersistenceStore",
  ".transition_store()",
]) {
  assert(
    !applicationLayer.includes(forbidden),
    `Postmatch Service/Use Case 泄漏基础设施依赖：${forbidden}`,
  );
}
console.log(
  "Postmatch Service 验证通过：7 个公共入口、2 个既有 Ports、资料包状态机与 PostgreSQL adapter 边界完整。",
);
