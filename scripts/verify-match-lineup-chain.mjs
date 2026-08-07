import crypto from "node:crypto";
import fs from "node:fs";
import { isVersionAtLeast } from "./version.mjs";

const read = (path) =>
  fs
    .readFileSync(new URL(`../${path}`, import.meta.url), "utf8")
    .replace(/\r\n?/g, "\n");
const requireTrue = (condition, message) => { if (!condition) throw new Error(message); };

const contractText = read("contracts/match-lineup-chain-contract.json");
const contract = JSON.parse(contractText);
const migration = read("crates/persistence-postgres/migrations/0024_match_lineup_chain.sql");
const domain = read("crates/domain/src/lineup/chain.rs") + read("crates/domain/src/exchange.rs") + (read("crates/domain/src/lib.rs") + read("crates/domain/src/lineup/kind.rs") + read("crates/domain/src/lineup/player.rs") + read("crates/domain/src/lineup/snapshot.rs") + read("crates/domain/src/lineup/preset.rs") + read("crates/domain/src/lineup/chain.rs") + read("crates/domain/src/match_record/status.rs") + read("crates/domain/src/match_record/catalog.rs"));
const persistence = read("crates/persistence-postgres/src/lineup_chain.rs");
const catalog = read("crates/persistence-postgres/src/player_catalog.rs");
const exchange = read("crates/persistence-postgres/src/match_exchange.rs");
const prediction = read("crates/persistence-postgres/src/match_prediction.rs");
const workbook = read("crates/spreadsheet-io/src/match_workbook.rs");
const application = read("crates/application/src/player_catalog.rs");
const commands = read("src-tauri/src/commands/catalog.rs");
const registry = read("src-tauri/src/bootstrap/command_registry.rs");
const client = read("src/api/client.ts");
const lineupsPage = read("src/pages/lineups.ts");
const pageLoaders = read("src/controllers/pageLoaders.ts");
const teamsPage = read("src/pages/teams.ts");
const main = read("src/main.ts");
const integrationTests = read("crates/persistence-postgres/tests/postgres_integration.rs");
const pkg = JSON.parse(read("package.json"));
const hash = crypto.createHash("sha256").update(contractText, "utf8").digest("hex");

requireTrue(isVersionAtLeast(pkg.version, contract.release_version), "当前项目版本早于阶段5版本0.18.0");
requireTrue(contract.release_version === "0.18.0", "阶段5契约发布版本错误");
requireTrue(contract.delivery_phase === "H_PRE_STAGE_5" && contract.integration_point_h_started === false, "阶段5越过H前置边界");
requireTrue(contract.workbook_format === "football.match-lineup.v2" && contract.legacy_workbook_readable === "football.match-lineup.v1", "比赛阵容工作簿兼容契约错误");
requireTrue(migration.includes(hash), "数据库迁移中的阶段5契约哈希不匹配");
requireTrue(migration.includes("DROP INDEX IF EXISTS football.lineups_one_active_revision_idx"), "旧单活动阵容索引未受控迁移");
requireTrue(migration.includes("lineups_active_horizon_version_uq") && migration.includes("snapshot_type, lineup_type"), "四时点版本唯一索引缺失");
requireTrue(migration.includes("model_eligible") && migration.includes("supersedes_lineup_id"), "阵容模型门禁或历史版本字段缺失");
requireTrue(migration.includes("bench_order BETWEEN 1 AND 99"), "替补顺序数据库约束缺失");
requireTrue(domain.includes('MATCH_LINEUP_IMPORT_FORMAT: &str = "football.match-lineup.v2"'), "比赛阵容v2格式常量缺失");
requireTrue(domain.includes('MATCH_LINEUP_IMPORT_LEGACY_FORMAT: &str = "football.match-lineup.v1"'), "比赛阵容v1兼容常量缺失");
for (const horizon of contract.formal_horizons) {
  requireTrue(migration.includes(`'${horizon}'`), `历史阵容契约时点缺失：${horizon}`);
}
for (const horizon of ["T-N", "T-24h", "T-6h", "T-1h"]) {
  requireTrue(domain.includes(`"${horizon}"`) && workbook.includes(`"${horizon}"`), `当前阵容数据窗口缺失：${horizon}`);
}
requireTrue(persistence.includes("preferred_lineup_id") && persistence.includes("model_eligible"), "模型阵容选择链缺失");
requireTrue(persistence.includes("lineup_type IN ('confirmed','expected')") && persistence.includes("ORDER BY lineup.captured_at DESC") && persistence.includes("WHEN 'confirmed' THEN 2"), "确认阵容优先规则缺失");
requireTrue(persistence.includes("lineup_type == \"actual\"") && persistence.includes("实际阵容只用于赛后复盘"), "实际阵容隔离规则缺失");
requireTrue(persistence.includes("starter_count != 11") && persistence.includes("formation_id.is_none()"), "阵容完整性门禁缺失");
requireTrue(persistence.includes("reference_time.min(kickoff_time - Duration::seconds(1))"), "赛前安全截止时间缺失");
requireTrue(persistence.includes('"T-N" => None') && persistence.includes("lineup.captured_at <= $3"), "T-N 任意赛前时间或最新阵容选择规则缺失");
requireTrue(!persistence.includes("模型链只支持 T-24h / T-6h / T-90m / T-1h"), "阵容链仍拒绝 T-N");
requireTrue(!catalog.includes("T-N 仅用于旧数据兼容") && !exchange.includes("T-N 仅用于旧数据兼容"), "阵容创建或导入仍拒绝 T-N");
requireTrue(/let\s+snapshot_type\s*=\s*crate::lineup_chain::normalize_lineup_snapshot_type\(&draft\.snapshot_type\)\?\.to_string\(\);/.test(catalog), "阵容创建未把规范化 snapshot_type 转换为持久化 String");
requireTrue(workbook.includes('["T-N", "T-24h", "T-6h", "T-1h"]'), "比赛阵容工作簿窗口选项不正确");
requireTrue(!workbook.includes('["T-N", "T-24h", "T-6h", "T-90m", "T-1h"]'), "比赛阵容工作簿仍开放T-90m新输入");
requireTrue(catalog.includes("替补顺序必须位于 1–99") && catalog.includes("refresh_lineup_validation_in_tx"), "客户端创建阵容校验链缺失");
requireTrue(exchange.includes("ready_end_previous") || exchange.includes("ended_previous"), "阵容导入版本结束结果缺失");
requireTrue(exchange.includes("refresh_lineup_validation_in_tx"), "工作簿提交后未刷新阵容门禁");
requireTrue(prediction.includes("阵容冻结门禁未通过") && prediction.includes("preferred_lineup_id"), "模型冻结门禁缺失");
requireTrue(prediction.includes('"formation_id": lineup.formation_id') && prediction.includes('"player_id": item.player_id'), "模型输入未保留阵型或球员UUID");
for (const field of [...contract.workbook_fields.lineup, ...contract.workbook_fields.lineup_player]) {
  requireTrue(workbook.includes(`"${field}"`), `比赛阵容工作簿字段缺失：${field}`);
}
requireTrue(workbook.includes("参考阵型") && workbook.includes("参考教练"), "比赛阵容参考字典缺失");
for (const command of contract.commands) {
  requireTrue(application.includes(`fn ${command}`), `应用服务缺失：${command}`);
  requireTrue(commands.includes(`fn ${command}`), `Tauri命令缺失：${command}`);
  requireTrue(registry.includes(command), `Tauri注册缺失：${command}`);
  requireTrue(client.includes(command), `前端API缺失：${command}`);
}
requireTrue(lineupsPage.includes("检查模型链路") && lineupsPage.includes("P4 输入"), "比赛中心阵容链可视化缺失");
requireTrue(lineupsPage.includes("references?.managed_matches ?? []"), "阵容编排未覆盖全部可管理比赛");
requireTrue(main.includes('renderedPage === "lineups"') && main.includes("capturePairedLineupFromDom"), "阵容页面重绘前未同步双方业务状态");
requireTrue(lineupsPage.includes('data-workspace-persist="false"') && lineupsPage.includes("paired-lineup-workflow"), "双方阵容业务表单仍可能被通用工作区快照覆盖");
requireTrue(pageLoaders.includes("export async function fetchLineups(): Promise<LineupsLoadResult>") && pageLoaders.includes("api.playerCatalogReferenceData()"), "阵容中心仍复用可能过期的比赛引用");
requireTrue(main.includes('controls: {}') && main.includes('active_section: "builder"'), "程序化进入阵容编排时未清理旧工作区控件快照");
requireTrue(lineupsPage.includes("open-player-from-lineup"), "阵容球员跳转链缺失");
requireTrue(teamsPage.includes("比赛阵容版本链"), "球队中心阵容历史缺失");
requireTrue(main.includes("inspectPairedLineupChain") && main.includes("selectedMatchLineupChain"), "客户端双方阵容链状态缺失");
requireTrue(lineupsPage.includes('value="T-N"') && lineupsPage.includes("任意赛前时间"), "阵容编排未提供 T-N");
requireTrue(lineupsPage.includes("continue-lineup-prediction"), "阵容链就绪后缺少返回正式推演入口");
requireTrue(main.includes("readSelectedPredictionLineupChain") && main.includes("openMissingPredictionLineup"), "正式推演缺少阵容预检与修复路由");
requireTrue(main.includes("loadBothPairedLineupSides") && main.includes("createPairedLineups"), "正式推演阻断未进入双方阵容闭环");
requireTrue(main.includes("requestSequence !== pairedLineupLoadSequence[side]") && main.includes("pairedSide(side).team_id !== teamId"), "双方球队名单异步加载缺少过期响应隔离");
requireTrue(main.includes("data_window_start_time") && main.includes("记录时间不能晚于窗口截止"), "时间窗口记录前端门禁缺失");
requireTrue(lineupsPage.includes("主队和客队相对编排") && lineupsPage.includes("一次提交"), "阵容编排仍不是主客队相对输入");
requireTrue(main.includes('workspaceState.patchModule("prediction", { active_section: "formal" })'), "双方阵容就绪后未闭环返回正式推演");
requireTrue(integrationTests.includes("match_lineup_chain_versions_model_selection_and_freeze_gate_are_consistent"), "阶段5 PostgreSQL端到端测试缺失");
requireTrue(integrationTests.includes("assert!(!invalid_home.model_eligible)"), "无效阵容冻结门禁测试缺失");
requireTrue(integrationTests.includes('snapshot_type: "T-N"') && integrationTests.includes("T-6h 数据窗口应读取窗口内最新 T-N 阵容"), "数据窗口最新记录 PostgreSQL 回归测试缺失");
console.log("阶段5比赛、阵容与模型输入闭环契约验证通过。");
