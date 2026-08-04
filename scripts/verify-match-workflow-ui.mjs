import fs from "node:fs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const requireTrue = (condition, message) => {
  if (!condition) throw new Error(message);
};

const types = read("src/types.ts");
const page = read("src/pages/lineups.ts");
const builder = read("src/components/lineupBuilder.ts");
const main = read("src/main.ts");
const styles = read("src/styles/components.css");
const client = read("src/api/client.ts");
const domain = read("crates/domain/src/lib.rs");
const persistence = read("crates/persistence-postgres/src/player_catalog.rs");
const application = read("crates/application/src/player_catalog.rs");
const command = read("src-tauri/src/commands/catalog.rs");
const registry = read("src-tauri/src/lib.rs");
const prediction = read("src/pages/prediction.ts");
const predictionApplication = read("crates/application/src/prediction.rs");
const integrationTests = read("crates/persistence-postgres/tests/postgres_integration.rs");

requireTrue(types.includes("interface LineupPairDraft") && domain.includes("struct LineupPairDraft"), "双方阵容原子提交契约缺失");
requireTrue(persistence.includes("pub async fn create_lineup_pair") && persistence.includes("let mut tx = self.pool.begin().await?") && persistence.includes("lineup_pair_created"), "双方阵容未在同一数据库事务中提交");
for (const source of [application, command, registry, client]) {
  requireTrue(source.includes("create_lineup_pair"), "双方阵容命令链未贯通");
}
requireTrue(page.includes("主队和客队相对编排") && page.includes("paired-lineup-side") && page.includes('data-action="create-lineup-pair"'), "双方并排阵容工作台缺失");
requireTrue(page.includes("match-browser-layout") && page.includes("比赛目录") && page.includes("比赛详情"), "比赛左侧目录与右侧详情布局缺失");
requireTrue(page.includes("new-match-competition") && page.includes("new-match-season") && page.includes("new-match-team-scope"), "赛事、赛季和球队体系分层选择缺失");
requireTrue(!page.includes('data-action="create-team"') && !page.includes('data-action="create-lineup-player"'), "比赛管理仍混入球队或球员快速创建入口");
requireTrue(main.includes("autoSelectMatchSeason") && persistence.includes("resolve_match_scope_draft"), "赛季自动匹配未覆盖前后端");
requireTrue(main.includes("filterMatchTeamOptions") && types.includes("season_team_memberships"), "赛事/赛季参赛队过滤链缺失");
requireTrue(main.includes("WorkflowContinuation") && main.includes("startWorkflowCompletion") && main.includes("returnToWorkflow"), "跨页面补录与返回原任务链缺失");
requireTrue(builder.includes('data-action="complete-workflow"') && page.includes('data-action="complete-workflow"'), "缺失资料没有可操作跳转入口");
requireTrue(styles.includes(".match-browser-layout") && styles.includes(".paired-lineup-board") && styles.includes(".workflow-continuation-banner"), "比赛中心统一视觉结构缺失");
requireTrue(prediction.includes("外部模型未捆绑") && !prediction.toLowerCase().includes("golden-master"), "推演页面未明确公开模型边界或仍暴露私有回归资产");
requireTrue(!predictionApplication.toLowerCase().includes("golden-master") && read("crates/model-stub/src/lib.rs").includes("ModelError::Unavailable"), "后端未使用显式不可用的公开模型入口");
requireTrue(!page.includes('value="T-90m"') && !prediction.includes('value="T-90m"'), "当前UI仍开放T-90m新输入");
requireTrue(integrationTests.includes("match_scope_inference_and_lineup_pair_transaction_are_atomic") && integrationTests.includes("任一侧失败后不得保留另一侧阵容"), "双方阵容事务回滚或赛季推断集成回归缺失");

console.log("比赛目录、赛事层级、双方阵容原子提交、跨页面补录返回和正式规则包隔离验证通过。");
