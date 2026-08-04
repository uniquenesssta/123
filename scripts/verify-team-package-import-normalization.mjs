import fs from "node:fs";

function read(path) {
  return fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

const application = read("crates/application/src/spreadsheet.rs");
const persistence = read("crates/persistence-postgres/src/spreadsheet_exchange.rs");
const teamPersistence = read("crates/persistence-postgres/src/monthly_workbooks.rs");
const template = read("crates/spreadsheet-io/src/team_package.rs");

assert(
  application.includes("collect_package_team_references") &&
    application.includes("preview_spreadsheet_import_with_team_references") &&
    application.includes('get("short_name")'),
  "完整资料包预检未把球队简称映射传给球员链",
);
assert(
  persistence.includes("external_team_references") &&
    persistence.includes("DeferredExternal { key: String, name: String }") &&
    persistence.includes("resolve_committed_team_id"),
  "球员链未支持跨批次延迟解析完整资料包球队",
);
assert(
  persistence.includes('"_deferred_team_name"') &&
    persistence.includes("完整资料包球队链提交后仍无法解析球队"),
  "提交阶段未按球队名称解析球队链先前新增的球队",
);
assert(
  persistence.includes("parse_spreadsheet_datetime") &&
    persistence.includes('for format in ["%Y-%m-%d", "%Y/%m/%d", "%Y年%m月%d日"]') &&
    persistence.includes("Excel 时间基准无效"),
  "球员导入时间解析未兼容日期和 Excel 日期序列",
);
assert(
  persistence.includes('Value::String("verified_at".to_string())') &&
    persistence.includes('payload.insert("valid_from".to_string()'),
  "球员球队关系缺少开始日期时未从核验时间生成稳定默认值",
);
assert(
  persistence.includes("default_ttl_hours") &&
    persistence.includes("动态标签默认失效时间超出允许范围") &&
    persistence.includes('payload.insert(\n                    "valid_to".to_string()'),
  "动态标签失效时间留空时未按定义 TTL 自动生成",
);
assert(
  persistence.includes('"official_web_plus_role_model"') &&
    persistence.includes('=> "calculation".to_string()'),
  "联网角色模型来源类型未规范化为数据库允许的 calculation",
);
assert(
  persistence.includes('"cancel_reason": "repreview_same_source"') &&
    persistence.includes("existing_status != \"pending\""),
  "同一文件的旧失败预检批次仍会阻止重新验证",
);
assert(
  !/SpreadsheetEntityType::PlayerDynamicTag\s*=>\s*&\[[\s\S]*?"valid_to"[\s\S]*?\]/.test(
    persistence.match(/fn validate_required_fields[\s\S]*?fn validate_player_fields/)?.[0] ?? "",
  ),
  "动态标签 valid_to 仍被预检硬性要求，无法使用默认 TTL",
);
const childValidation =
  persistence.match(/fn validate_child_fields[\s\S]*?fn validate_date_range/)?.[0] ?? "";
assert(
  childValidation.includes('if let Some(to) = optional_datetime(payload, "valid_to")?') &&
    !childValidation.includes('let to = required_datetime(payload, "valid_to")?'),
  "动态标签子字段校验仍在默认 TTL 补全前强制要求 valid_to",
);
assert(
  persistence.includes("fn dynamic_tag_child_validation_allows_default_ttl") &&
    persistence.includes("fn dynamic_tag_child_validation_rejects_non_increasing_explicit_range"),
  "动态标签默认 TTL 与显式错误区间缺少 Rust 回归测试",
);
assert(
  template.includes("动态标签失效时间留空时按标签默认 TTL 自动生成"),
  "模板说明未同步导入器的日期和动态 TTL 兼容规则",
);
assert(
  teamPersistence.includes("normalize_point_observation_window_payload") &&
    teamPersistence.includes("球队能力或战术观察缺少 window_start/window_end") &&
    teamPersistence.includes("window_end 不能早于 window_start"),
  "球队能力或战术快照缺少单边窗口日期时未自动生成点时窗口",
);
assert(
  teamPersistence.includes("point_observation_window_uses_end_for_missing_start") &&
    teamPersistence.includes("point_observation_window_uses_observed_at_when_both_dates_are_blank") &&
    teamPersistence.includes("point_observation_window_rejects_inverted_range"),
  "球队能力或战术点时窗口缺少 Rust 回归测试",
);
assert(
  template.includes("球队能力/战术快照只填一个窗口日期时自动生成同日点时窗口") &&
    template.includes("阵型使用分布仍须填写真实起止窗口"),
  "模板说明未区分点时快照与阵型真实观察窗口",
);
assert(
  teamPersistence.includes("struct FormationGroupKey") &&
    teamPersistence.includes("formation_entity_reference") &&
    teamPersistence.includes('text(values, &format!("{prefix}_name"))'),
  "阵型分组仍只依赖空白 UUID，未使用球队/教练名称作为稳定后备身份",
);
assert(
  teamPersistence.includes("validate_formation_group_rows(&mut preview_rows)") &&
    teamPersistence.includes("阵型使用次数合计超过观察场数（使用"),
  "阵型分布合计仍要到正式提交阶段才发现，预检未执行分组级校验",
);
assert(
  teamPersistence.includes("formation_group_key_separates_blank_ids_by_team_and_coach_names") &&
    teamPersistence.includes("formation_group_preview_keeps_distinct_teams_separate") &&
    teamPersistence.includes("formation_group_preview_rejects_aggregate_overflow"),
  "阵型分组名称后备、跨球队隔离和合计超限缺少 Rust 回归测试",
);
assert(
  teamPersistence.includes("validate_import_formation_reference") &&
    teamPersistence.includes("resolve_or_register_import_formation_tx") &&
    teamPersistence.includes("canonical_formation_code") &&
    teamPersistence.includes("is_valid_custom_formation_code") &&
    teamPersistence.includes("登记为自定义阵型"),
  "阵型目录预检、Unicode 规范化或自定义阵型登记链缺失",
);
assert(
  teamPersistence.includes("工作表“{}”第 {} 行") &&
    teamPersistence.includes("formation_code_normalizes_excel_unicode_before_lookup") &&
    teamPersistence.includes("custom_formation_requires_ten_outfield_players") &&
    !teamPersistence.includes(
      '"SELECT id FROM football.formations WHERE lower(code)=lower($1) AND is_active"',
    ),
  "阵型提交仍可能以无行上下文的 fetch_one 暴露 SQLx RowNotFound",
);
assert(
  application.includes("完整资料包球队、教练与阵型链提交失败") &&
    application.includes("完整资料包球员、评分与动态状态链提交失败"),
  "统一资料包提交错误未标明球队链或球员链阶段",
);
assert(
  template.includes("目录外但各线人数合计为 10 的标准代码") &&
    template.includes("会作为自定义阵型保留并登记"),
  "模板说明未同步阵型规范化与自定义登记规则",
);

console.log("球队完整资料包时间、TTL、跨批次球队关联、阵型分组与重复预检契约验证通过。");
