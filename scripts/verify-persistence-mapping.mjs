import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const mappingDir = "crates/persistence-postgres/src/mapping";
const expected = ["invalid_state.rs", "json.rs", "mod.rs", "optional.rs", "time.rs", "uuid.rs"];
check(exists(mappingDir), `R4-03 缺少 mapping 目录：${mappingDir}`);
const actual = exists(mappingDir) ? fs.readdirSync(path.join(root, mappingDir)).filter((name) => name.endsWith(".rs")).sort() : [];
check(JSON.stringify(actual) === JSON.stringify(expected), `mapping 目录职责文件集合不匹配，实际：${actual.join(", ")}`);
const files = Object.fromEntries(expected.map((name) => [name, read(`${mappingDir}/${name}`)]));
const combined = Object.values(files).join("\n");
check(!combined.includes("football_domain"), "通用 mapping 不得依赖业务 Domain 类型");
check(!combined.includes("sqlx::"), "通用 mapping 不得承载 SQL/查询职责");
check(!exists(`${mappingDir}/mapper.rs`) && !exists(`${mappingDir}/queries.rs`), "不得建立万能 mapper.rs 或 queries.rs");
for (const token of ["mod invalid_state;", "mod json;", "mod optional;", "mod time;", "mod uuid;"]) check(files["mod.rs"].includes(token), `mapping/mod.rs 缺少显式声明：${token}`);
check(files["invalid_state.rs"].includes("PersistenceError::InvalidState"), "invalid_state owner 未保持既有错误类型");
for (const token of ["DateTime::parse_from_rfc3339(raw)", "with_timezone(&Utc)", "必须是 RFC3339 时间", "时间无效"]) check(files["time.rs"].includes(token), `time mapping 契约缺失：${token}`);
for (const token of ["Uuid::parse_str(raw)", "optional_trimmed_text", "UUID 字符串", "不是有效 UUID"]) check(files["uuid.rs"].includes(token), `UUID mapping 契约缺失：${token}`);
for (const token of ["value.is_null()", ".trim()", "return Ok(None)"]) check(files["optional.rs"].includes(token), `optional mapping 契约缺失：${token}`);
check(files["json.rs"].includes("serde_json::to_value(value)?"), "JSON mapping 未保持 Serde 错误传播");

const library = read("crates/persistence-postgres/src/lib.rs");
const competitionKind = read("crates/persistence-postgres/src/competition_kind.rs");
const competitionMapper = read("crates/persistence-postgres/src/adapters/competition/detail/record_mapper.rs");
const routeContextMapper = read("crates/persistence-postgres/src/adapters/competition/route_resolution/context/record_mapper.rs");
const p4 = read("crates/persistence-postgres/src/p4_orchestration.rs");
const bindingMapper = read("crates/persistence-postgres/src/adapters/competition/bindings/record_mapper.rs");
const bindingPackageMetadata = read("crates/persistence-postgres/src/adapters/competition/bindings/package_route_metadata.rs");
const rulePackageMapper = read("crates/persistence-postgres/src/adapters/rules/packages/record_mapper.rs");
const modelRuns = read("crates/persistence-postgres/src/model_runs.rs");
check(!exists("crates/persistence-postgres/src/competitions.rs"), "R5-05 后 legacy competitions.rs 不得重新成为 mapping 调用 owner");
check(library.includes("mod mapping;") && library.includes("mod competition_kind;"), "lib.rs 未注册 mapping/competition_kind 模块");
check(library.includes("pub(crate) use competition_kind::parse_competition_kind;"), "lib.rs 未保留共享 CompetitionKind crate 内出口");
check(!library.includes("fn parse_competition_kind"), "lib.rs 仍直接承载业务 CompetitionKind 实现");
check(competitionKind.includes("fn parse_competition_kind") && competitionKind.includes("football_domain::CompetitionKind"), "CompetitionKind 独立业务 owner 不完整");
check(competitionKind.includes("未知赛事类型：{other}"), "CompetitionKind 原错误语义未保持");
for (const [label, source] of [
  ["R5-01 competition mapper", competitionMapper],
  ["R5-05 route context mapper", routeContextMapper],
  ["p4_orchestration", p4],
  ["R5-04 binding mapper", bindingMapper],
  ["R5-04 binding package metadata", bindingPackageMetadata],
  ["R5-03 rule package mapper", rulePackageMapper],
]) check(source.includes("parse_competition_kind"), `${label} 共享 CompetitionKind 调用路径缺失`);
check(!routeContextMapper.includes("sqlx::query") && !routeContextMapper.includes("PgRow"), "R5-05 route context mapper 必须保持纯 typed Row -> Domain mapping");
check(modelRuns.includes("use crate::mapping::{optional_uuid, required_datetime, required_uuid, to_json_value};"), "model_runs 未使用 mapping 基础出口");
check(!modelRuns.includes("fn required_datetime(") && !modelRuns.includes("fn optional_uuid("), "model_runs 仍保留重复标量 helper");
check(modelRuns.includes('required_uuid(snapshot_id, "snapshot.snapshot_id")?'), "snapshot UUID 未切换统一 mapping");
check(modelRuns.includes("to_json_value(&output.summary)?"), "Model summary JSON 未切换统一 mapping");
if (failures.length) {
  console.error("R4-03 Row 映射基础规范验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R4-03 Row 映射基础规范验证通过：基础标量保持独立 owner，CompetitionKind 保持共享业务 owner；R5-01/R5-03/R5-04/R5-05 映射路径继续复用共享解析器，R5-05 context mapper 保持 typed Row -> Domain 边界。");