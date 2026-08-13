from __future__ import annotations

import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8", newline="\n")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected 1 exact match, found {count}")
    return text.replace(old, new, 1)


def remove_function(text: str, marker: str, label: str) -> str:
    idx = text.find(marker)
    if idx < 0:
        raise RuntimeError(f"{label}: marker missing: {marker}")
    start = text.rfind("\n", 0, idx) + 1
    prev_end = start - 1
    if prev_end >= 0:
        prev_start = text.rfind("\n", 0, prev_end) + 1
        if text[prev_start:prev_end].strip() == "#[test]":
            start = prev_start
    brace = text.find("{", idx)
    if brace < 0:
        raise RuntimeError(f"{label}: opening brace missing")
    depth = 0
    end = None
    for i in range(brace, len(text)):
        ch = text[i]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                end = i + 1
                break
    if end is None:
        raise RuntimeError(f"{label}: closing brace missing")
    while end < len(text) and text[end] in "\r\n":
        end += 1
    return text[:start] + text[end:]


def apply() -> None:
    write(
        "crates/persistence-postgres/src/mapping/mod.rs",
        """mod invalid_state;
mod json;
mod optional;
mod time;
mod uuid;

pub(crate) use invalid_state::invalid_state;
pub(crate) use json::to_json_value;
pub(crate) use time::required_datetime;
pub(crate) use uuid::{optional_uuid, required_uuid};
""",
    )
    write(
        "crates/persistence-postgres/src/mapping/invalid_state.rs",
        """use crate::PersistenceError;

pub(crate) fn invalid_state(message: impl Into<String>) -> PersistenceError {
    PersistenceError::InvalidState(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_invalid_state_message() {
        let error = invalid_state("映射状态无效");
        assert!(matches!(error, PersistenceError::InvalidState(message) if message == "映射状态无效"));
    }
}
""",
    )
    write(
        "crates/persistence-postgres/src/mapping/optional.rs",
        """use super::invalid_state::invalid_state;
use crate::PersistenceResult;
use serde_json::Value;

pub(super) fn optional_trimmed_text<'a>(
    value: Option<&'a Value>,
    key: &str,
    expected_type: &str,
) -> PersistenceResult<Option<&'a str>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let text = value
        .as_str()
        .ok_or_else(|| invalid_state(format!("{key} 必须是 {expected_type}")))?
        .trim();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn normalizes_missing_null_blank_and_present_text() {
        assert_eq!(optional_trimmed_text(None, "id", "字符串").unwrap(), None);
        assert_eq!(optional_trimmed_text(Some(&Value::Null), "id", "字符串").unwrap(), None);
        let blank = json!("   ");
        assert_eq!(optional_trimmed_text(Some(&blank), "id", "字符串").unwrap(), None);
        let text = json!("  value  ");
        assert_eq!(optional_trimmed_text(Some(&text), "id", "字符串").unwrap(), Some("value"));
    }
}
""",
    )
    write(
        "crates/persistence-postgres/src/mapping/uuid.rs",
        """use super::{invalid_state::invalid_state, optional::optional_trimmed_text};
use crate::PersistenceResult;
use serde_json::Value;
use uuid::Uuid;

pub(crate) fn required_uuid(raw: &str, key: &str) -> PersistenceResult<Uuid> {
    Uuid::parse_str(raw)
        .map_err(|error| invalid_state(format!("{key} 不是有效 UUID：{error}")))
}

pub(crate) fn optional_uuid(value: &Value, key: &str) -> PersistenceResult<Option<Uuid>> {
    let Some(raw) = optional_trimmed_text(value.get(key), key, "UUID 字符串")? else {
        return Ok(None);
    };
    required_uuid(raw, key).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn optional_uuid_accepts_missing_null_and_blank_values() {
        assert_eq!(optional_uuid(&json!({}), "id").unwrap(), None);
        assert_eq!(optional_uuid(&json!({"id": null}), "id").unwrap(), None);
        assert_eq!(optional_uuid(&json!({"id": "  "}), "id").unwrap(), None);
    }

    #[test]
    fn optional_uuid_trims_present_values_without_relaxing_required_uuid() {
        let id = Uuid::new_v4();
        assert_eq!(optional_uuid(&json!({"id": format!("  {id}  ")}), "id").unwrap(), Some(id));
        assert!(required_uuid(&format!(" {id} "), "snapshot.snapshot_id").is_err());
    }
}
""",
    )
    write(
        "crates/persistence-postgres/src/mapping/time.rs",
        """use super::invalid_state::invalid_state;
use crate::PersistenceResult;
use chrono::{DateTime, Utc};
use serde_json::Value;

pub(crate) fn required_datetime(
    value: Option<&Value>,
    key: &str,
) -> PersistenceResult<DateTime<Utc>> {
    let raw = value
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_state(format!("{key} 必须是 RFC3339 时间")))?;
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| invalid_state(format!("{key} 时间无效：{error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_rfc3339_into_utc() {
        let value = json!("2026-08-13T11:53:00+08:00");
        let parsed = required_datetime(Some(&value), "snapshot.frozen_at").unwrap();
        assert_eq!(parsed.to_rfc3339(), "2026-08-13T03:53:00+00:00");
    }

    #[test]
    fn rejects_non_string_time_with_existing_error_semantics() {
        let value = json!(123);
        let error = required_datetime(Some(&value), "snapshot.frozen_at").unwrap_err();
        assert!(matches!(error, crate::PersistenceError::InvalidState(message) if message == "snapshot.frozen_at 必须是 RFC3339 时间"));
    }
}
""",
    )
    write(
        "crates/persistence-postgres/src/mapping/json.rs",
        """use crate::PersistenceResult;
use serde::Serialize;
use serde_json::Value;

pub(crate) fn to_json_value<T: Serialize + ?Sized>(value: &T) -> PersistenceResult<Value> {
    Ok(serde_json::to_value(value)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_without_changing_json_shape() {
        let value = vec!["home", "draw", "away"];
        assert_eq!(to_json_value(&value).unwrap(), json!(["home", "draw", "away"]));
    }
}
""",
    )
    write(
        "crates/persistence-postgres/src/competition_kind.rs",
        """use crate::{mapping::invalid_state, PersistenceResult};
use football_domain::CompetitionKind;

pub(crate) fn parse_competition_kind(value: &str) -> PersistenceResult<CompetitionKind> {
    match value {
        "league" => Ok(CompetitionKind::League),
        "group_stage" => Ok(CompetitionKind::GroupStage),
        "knockout_single_leg" => Ok(CompetitionKind::KnockoutSingleLeg),
        "knockout_two_leg" => Ok(CompetitionKind::KnockoutTwoLeg),
        "friendly" => Ok(CompetitionKind::Friendly),
        "custom" => Ok(CompetitionKind::Custom),
        other => Err(invalid_state(format!("未知赛事类型：{other}"))),
    }
}
""",
    )

    lib_path = "crates/persistence-postgres/src/lib.rs"
    lib = read(lib_path)
    lib = replace_once(lib, "mod competitions;\n", "mod competition_kind;\nmod competitions;\n", "lib competition kind module")
    lib = replace_once(lib, "mod lineup_chain;\n", "mod lineup_chain;\nmod mapping;\n", "lib mapping module")
    lib = replace_once(
        lib,
        "pub(crate) use audit::{sha256_json, write_audit_event};\n",
        "pub(crate) use audit::{sha256_json, write_audit_event};\npub(crate) use competition_kind::parse_competition_kind;\n",
        "lib competition kind re-export",
    )
    marker = "\nuse football_domain::CompetitionKind;\n\nfn parse_competition_kind(value: &str) -> PersistenceResult<CompetitionKind> {"
    idx = lib.find(marker)
    if idx < 0:
        raise RuntimeError("lib business enum parser marker missing")
    lib = lib[:idx].rstrip() + "\n"
    write(lib_path, lib)

    model_path = "crates/persistence-postgres/src/model_runs.rs"
    model = read(model_path)
    model = replace_once(
        model,
        "use super::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};\n",
        "use super::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};\nuse crate::mapping::{optional_uuid, required_datetime, required_uuid, to_json_value};\n",
        "model_runs mapping imports",
    )
    model = replace_once(model, "let summary = serde_json::to_value(&output.summary)?;", "let summary = to_json_value(&output.summary)?;", "model summary json")
    model = replace_once(
        model,
        """let snapshot_id = Uuid::parse_str(snapshot_id).map_err(|error| {
        PersistenceError::InvalidState(format!("snapshot.snapshot_id 不是有效 UUID：{error}"))
    })?;""",
        "let snapshot_id = required_uuid(snapshot_id, \"snapshot.snapshot_id\")?;",
        "snapshot uuid",
    )
    model = remove_function(model, "fn required_datetime(", "move required_datetime")
    model = remove_function(model, "fn optional_uuid(", "move optional_uuid")
    model = remove_function(model, "fn optional_uuid_accepts_missing_null_and_blank_values()", "move optional_uuid test")
    write(model_path, model)

    verifier = r'''import fs from "node:fs";
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
const competitions = read("crates/persistence-postgres/src/competitions.rs");
const p4 = read("crates/persistence-postgres/src/p4_orchestration.rs");
const routing = read("crates/persistence-postgres/src/routing.rs");
const modelRuns = read("crates/persistence-postgres/src/model_runs.rs");
check(library.includes("mod mapping;") && library.includes("mod competition_kind;"), "lib.rs 未注册 mapping/competition_kind 模块");
check(library.includes("pub(crate) use competition_kind::parse_competition_kind;"), "lib.rs 未保留共享 CompetitionKind crate 内出口");
check(!library.includes("fn parse_competition_kind"), "lib.rs 仍直接承载业务 CompetitionKind 实现");
check(competitionKind.includes("fn parse_competition_kind") && competitionKind.includes("football_domain::CompetitionKind"), "CompetitionKind 独立业务 owner 不完整");
check(competitionKind.includes("未知赛事类型：{other}"), "CompetitionKind 原错误语义未保持");
for (const [label, source] of [["competitions", competitions], ["p4_orchestration", p4], ["routing", routing]]) check(source.includes("parse_competition_kind"), `${label} 共享 CompetitionKind 调用路径缺失`);
check(modelRuns.includes("use crate::mapping::{optional_uuid, required_datetime, required_uuid, to_json_value};"), "model_runs 未使用 mapping 基础出口");
check(!modelRuns.includes("fn required_datetime(") && !modelRuns.includes("fn optional_uuid("), "model_runs 仍保留重复标量 helper");
check(modelRuns.includes('required_uuid(snapshot_id, "snapshot.snapshot_id")?'), "snapshot UUID 未切换统一 mapping");
check(modelRuns.includes("to_json_value(&output.summary)?"), "Model summary JSON 未切换统一 mapping");
if (failures.length) {
  console.error("R4-03 Row 映射基础规范验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R4-03 Row 映射基础规范验证通过：基础标量形成独立 owner，CompetitionKind 保持独立业务 owner 与共享兼容出口。");
'''
    write("scripts/verify-persistence-mapping.mjs", verifier)

    package_path = "package.json"
    package = read(package_path)
    package = replace_once(
        package,
        '&& node scripts/verify-persistence-audit.mjs",\n    "verify:persistence-foundation"',
        '&& node scripts/verify-persistence-audit.mjs && node scripts/verify-persistence-mapping.mjs",\n    "verify:persistence-foundation"',
        "architecture mapping gate",
    )
    package = replace_once(
        package,
        '    "verify:persistence-audit": "node scripts/verify-persistence-audit.mjs",\n',
        '    "verify:persistence-audit": "node scripts/verify-persistence-audit.mjs",\n    "verify:persistence-mapping": "node scripts/verify-persistence-mapping.mjs",\n',
        "mapping npm script",
    )
    write(package_path, package)


def record() -> None:
    run_id = os.environ["GITHUB_RUN_ID"]
    first_failed = os.environ["R4_03_FAILED_RUN"]
    yaml_failed = os.environ["R4_03_YAML_FAILED_RUN"]
    base = os.environ["R4_03_BASE"]
    record_text = f'''# R4-03 Row 映射基础规范

## 状态

`VERIFYING`

## 实施结果

- 新建 `crates/persistence-postgres/src/mapping/`，按 `time.rs`、`uuid.rs`、`json.rs`、`optional.rs`、`invalid_state.rs` 拆分基础标量映射；`mod.rs` 只负责显式出口。
- `model_runs.rs` 原有 RFC3339 时间解析、optional UUID 解析及对应测试迁入 mapping owner；`snapshot.snapshot_id` UUID 校验与 model summary JSON 序列化切换到统一基础出口，原错误文本和 Serde/UUID 失败传播保持。
- 根 `lib.rs` 的 CompetitionKind 解析实现迁入独立 `competition_kind.rs`；`lib.rs` 只保留 crate 内 re-export，因此 `competitions.rs`、`p4_orchestration.rs`、`routing.rs` 的既有调用路径无需改变。业务 enum 不进入通用 `mapping/`。
- 新增 `verify:persistence-mapping` 并接入 `verify:architecture`，静态拒绝 `mapper.rs`/`queries.rs`、SQLx/Domain 侵入通用 mapping、重复 model_runs helper 及业务枚举回流根实现。
- 未修改 0001–0046 migration SQL、Cargo 生产依赖、公共 Application/Tauri 接口、DTO、数据库 schema、配置、模型保护资产或 R4-04 实现。

## 文件清单

### 新增

- `crates/persistence-postgres/src/mapping/mod.rs`
- `crates/persistence-postgres/src/mapping/time.rs`
- `crates/persistence-postgres/src/mapping/uuid.rs`
- `crates/persistence-postgres/src/mapping/json.rs`
- `crates/persistence-postgres/src/mapping/optional.rs`
- `crates/persistence-postgres/src/mapping/invalid_state.rs`
- `crates/persistence-postgres/src/competition_kind.rs`
- `scripts/verify-persistence-mapping.mjs`
- `docs/modular-rewrite/R04-persistence-foundation/R04-03-row-映射基础规范.md`

### 修改

- `crates/persistence-postgres/src/lib.rs`
- `crates/persistence-postgres/src/model_runs.rs`
- `package.json`
- `architecture/domain-type-inventory.json`（官方生成器同步 Rust 扫描/映射来源清单）
- `README.md`
- `docs/modular-rewrite/R04-persistence-foundation/README.md`

### 删除

- 无生产文件删除。两份临时 R4-03 执行文件在最终实施提交前自删除，不进入最终源码树。

## 验证

- 首次 hard gate run `{first_failed}`：起点范围、源码生成、rustfmt 与 Domain inventory 重建完成；专项 verifier 因生成转义错误语法失败，编译随后暴露 `p4_orchestration.rs`、`routing.rs` 仍依赖共享 CompetitionKind 根出口。该 run 未进入阶段回归、未提交生产源码。
- 第二次调度 run `{yaml_failed}`：修复方案本身未执行，workflow 因嵌套 verifier 源码破坏 YAML block 缩进在调度阶段失败，0 个 job、无生产源码变化。
- 恢复 hard gate run `{run_id}` 在 Windows 2025 / Rust 1.88.0 / Node 22 实际通过：mapping 专项、R4-01/R4-02 专项、数据库冻结/保护资产、rustfmt、Persistence check/tests、完整 architecture/frontend、workspace Clippy `-D warnings` 与 workspace tests。
- Cargo manifests / `Cargo.lock`、历史 migrations 对基线 `{base}` 零 diff。
- 18 个要求专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试未执行；未执行 destructive database reset。

## 兼容性与剩余风险

- `required_datetime` 与 `optional_uuid` 保留原调用语义、空值规则和错误文案；`required_uuid` 对必填 UUID 不新增 trim，保持 `snapshot.snapshot_id` 原有严格解析行为。
- JSON 序列化仍通过 `serde_json::to_value` 并由既有 `PersistenceError::Serialization` 转换传播。
- SQLx row 的原生 `try_get` 类型/NULL 错误语义未改写为其他错误类型，避免 R4-03 造成公共错误语义漂移。
- CompetitionKind 字符串集合、未知值错误文本和三个既有业务调用路径保持。
- 本节点保持 `VERIFYING`，等待 clean Public Platform CI 与正式 PR 合并；R4-04 继续 `BLOCKED`。

## 回退点

- R4-03 基线：`{base}`。
- 实施分支：`agent/r4-03-row-mapping`。
'''
    write("docs/modular-rewrite/R04-persistence-foundation/R04-03-row-映射基础规范.md", record_text)

    stage_path = "docs/modular-rewrite/R04-persistence-foundation/README.md"
    stage = read(stage_path)
    old = "| R4-03 | 通用 Row 映射基础规范 | READY | — |"
    new = "| R4-03 | 通用 Row 映射基础规范 | VERIFYING | [`R04-03-row-映射基础规范.md`](./R04-03-row-映射基础规范.md) |"
    stage = replace_once(stage, old, new, "R4-03 stage row")
    if "## R4-03 实施中" not in stage:
        stage = stage.rstrip() + f'''\n\n## R4-03 实施中\n\n- 从 R4-02 正式收口 HEAD `{base}` 独立建立 `agent/r4-03-row-mapping`。\n- run `{first_failed}` 因 verifier 转义错误与共享调用面漏扫停止；run `{yaml_failed}` 因临时 workflow YAML 缩进错误在调度前停止；两次均未提交生产源码。\n- 恢复 run `{run_id}` 已通过全部最小门禁与阶段回归。\n- R4-03 当前 `VERIFYING`；clean Public Platform CI 与 PR 合并完成前不得标记 `DONE`，R4-04 继续 `BLOCKED`。\n'''
    write(stage_path, stage)

    readme = read("README.md")
    if "- R4-03 Row 映射基础规范" not in readme:
        lines = readme.splitlines()
        anchor = next((i for i, line in enumerate(lines) if line.startswith("- R4-02 Audit 基础设施已将")), None)
        if anchor is None:
            raise RuntimeError("root README R4-02 anchor missing")
        lines.insert(
            anchor + 1,
            f"- R4-03 Row 映射基础规范已建立 `mapping/time.rs`、`uuid.rs`、`json.rs`、`optional.rs`、`invalid_state.rs` 五类基础标量职责；共享 CompetitionKind 解析迁入独立 `competition_kind.rs`，根模块仅保留兼容 re-export，未建立万能动态 mapper。前两次临时执行分别因 verifier 生成错误与 workflow YAML 缩进错误停止且均未提交生产源码；恢复 run `{run_id}` 已通过 mapping 专项、数据库冻结、architecture/frontend、Persistence 与 workspace Rust 回归。18 个专用 PostgreSQL 集成测试与 destructive reset 未执行。当前状态为 `VERIFYING`，等待 clean Public Platform CI 与 PR 合并，R4-04 继续 `BLOCKED`。",
        )
        readme = "\n".join(lines) + "\n"
    write("README.md", readme)


def main() -> None:
    if len(sys.argv) != 2 or sys.argv[1] not in {"apply", "record"}:
        raise SystemExit("usage: r4-03-apply.py apply|record")
    if sys.argv[1] == "apply":
        apply()
    else:
        record()


if __name__ == "__main__":
    main()
