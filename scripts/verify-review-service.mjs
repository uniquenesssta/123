import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const failures = [];
const required = [
  "crates/application/src/services/review/mod.rs",
  "crates/application/src/services/review/service.rs",
  "crates/application/src/services/review/facade.rs",
  "crates/application/src/use_cases/review/mod.rs",
  "crates/application/src/use_cases/review/generate_match_review/mod.rs",
  "crates/application/src/use_cases/review/list_reviewable_matches/mod.rs",
  "crates/application/src/use_cases/review/list_match_reviews/mod.rs",
  "crates/application/src/use_cases/review/read_match_review/mod.rs",
  "crates/application/src/use_cases/review/list_ability_candidates/mod.rs",
  "crates/application/src/use_cases/review/decide_ability_candidate/mod.rs",
  "crates/application/src/composition/adapters/review.rs",
];
for (const relative of required) {
  if (!fs.existsSync(path.join(root, relative))) failures.push(`缺少 Review 模块：${relative}`);
}
if (fs.existsSync(path.join(root, "crates/application/src/review.rs"))) {
  failures.push("旧 Review owner crates/application/src/review.rs 仍存在");
}
const lib = fs.readFileSync(path.join(root, "crates/application/src/lib.rs"), "utf8");
if (/^mod review;$/m.test(lib)) failures.push("lib.rs 仍直接挂载旧 review.rs");
const services = fs.readFileSync(path.join(root, "crates/application/src/services/mod.rs"), "utf8");
const useCases = fs.readFileSync(path.join(root, "crates/application/src/use_cases/mod.rs"), "utf8");
const adapters = fs.readFileSync(path.join(root, "crates/application/src/composition/adapters/mod.rs"), "utf8");
if (!services.includes("pub(crate) mod review;")) failures.push("services 未登记 review");
if (!useCases.includes("pub(crate) mod review;")) failures.push("use_cases 未登记 review");
if (!adapters.includes("mod review;")) failures.push("composition adapters 未登记 review");

const facade = fs.readFileSync(path.join(root, "crates/application/src/services/review/facade.rs"), "utf8");
const service = fs.readFileSync(path.join(root, "crates/application/src/services/review/service.rs"), "utf8");
const port = fs.readFileSync(path.join(root, "crates/application/src/ports/review/mod.rs"), "utf8");
const adapter = fs.readFileSync(path.join(root, "crates/application/src/composition/adapters/review.rs"), "utf8");
const tauri = fs.readFileSync(path.join(root, "src-tauri/src/commands/review.rs"), "utf8");
const methods = [
  "generate_match_review",
  "list_reviewable_matches",
  "list_match_reviews",
  "read_match_review",
  "list_ability_candidates",
  "decide_ability_candidate",
];
for (const method of methods) {
  if (!facade.includes(`pub async fn ${method}`)) failures.push(`Review facade 缺少公共方法 ${method}`);
  if (!service.includes(`fn ${method}`)) failures.push(`ReviewService 缺少职责 ${method}`);
  if (!tauri.includes(`pub async fn ${method}`)) failures.push(`Tauri Review 公共命令缺少 ${method}`);
}
if (!port.includes("pub trait MatchReviewPort")) failures.push("缺少 MatchReviewPort");
if (!adapter.includes("impl MatchReviewPort for ActiveDatabase")) failures.push("ActiveDatabase 未实现 MatchReviewPort");
if (!port.includes("status: Option<AbilityCandidateStatus>")) failures.push("能力候选状态过滤未进入 Review Port");
if (!port.includes("match_review_id: Option<Uuid>")) failures.push("能力候选复盘过滤未进入 Review Port");
if (!facade.includes("self.database") || !facade.includes("active_session()")) failures.push("Review facade 未经 DatabaseService 活动会话取得端口实现");

for (const relative of required.filter((item) => item.includes("/services/review/") || item.includes("/use_cases/review/"))) {
  const text = fs.readFileSync(path.join(root, relative), "utf8");
  for (const token of ["football_persistence_postgres", "sqlx::", "PostgresStore", "PersistenceStore", ".transition_store()"] ) {
    if (text.includes(token)) failures.push(`${relative} 泄漏具体持久化符号：${token}`);
  }
}

const composition = fs.readFileSync(path.join(root, "crates/application/src/composition/application_composition.rs"), "utf8");
const appService = fs.readFileSync(path.join(root, "crates/application/src/service/application_service.rs"), "utf8");
if (!composition.includes("review: ReviewService")) failures.push("ApplicationComposition 未持有唯一 ReviewService");
if (!appService.includes("pub(crate) review: ReviewService")) failures.push("ApplicationService 未持有 ReviewService");

if (failures.length) {
  console.error("Review Service 验证失败：
- " + failures.join("
- "));
  process.exit(1);
}
console.log("Review Service 验证通过：6 个 Review Core 公共职责已迁入 Service/Use Case/Port，旧 review.rs 已退出，Tauri 公共命令保持原名。 ");
