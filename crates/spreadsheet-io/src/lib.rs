mod match_review_workbook;
pub use match_review_workbook::{read_match_review_package, write_match_review_package};

mod team_package;
pub use team_package::{read_team_package_workbook, write_team_package_template};

mod ai_package;
pub use ai_package::{extract_ai_match_workbook, write_ai_match_package};

mod monthly_workbook;
pub use monthly_workbook::{
    read_player_monthly_workbook, read_team_monthly_workbook, write_player_monthly_export,
    write_player_monthly_template, write_team_monthly_export, write_team_monthly_template,
};

mod match_workbook;
pub use match_workbook::{
    read_match_lineup_workbook, write_match_lineup_export, write_match_lineup_template,
};

use calamine::{open_workbook_auto, Data, DataType, Reader};
use football_domain::{
    PlayerCatalogReferenceData, SpreadsheetAction, SpreadsheetEntityType, SpreadsheetExportData,
    SpreadsheetParsedWorkbook, SpreadsheetRawRow, PLAYER_IMPORT_FORMAT,
};
use rust_xlsxwriter::{
    Color, DataValidation, Format, FormatAlign, FormatBorder, Formula, Workbook, Worksheet,
    XlsxError,
};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpreadsheetError {
    #[error("无法读取 Excel 文件：{0}")]
    Read(String),
    #[error("无法写入 Excel 文件：{0}")]
    Write(#[from] XlsxError),
    #[error("Excel 模板无效：{0}")]
    InvalidTemplate(String),
    #[error("文件系统错误：{0}")]
    Io(#[from] std::io::Error),
}

pub type SpreadsheetResult<T> = Result<T, SpreadsheetError>;

const TEMPLATE_LAST_ROW: u32 = 99_999;

struct SheetSpec {
    name: &'static str,
    entity_type: SpreadsheetEntityType,
    headers: &'static [&'static str],
    required: &'static [&'static str],
    widths: &'static [f64],
    example: &'static [&'static str],
}

const TEAM_HEADERS: &[&str] = &[
    "action",
    "team_key",
    "team_id",
    "official_name",
    "country_code",
    "is_active",
    "notes",
];
const PLAYER_HEADERS: &[&str] = &[
    "action",
    "player_key",
    "player_id",
    "official_name",
    "birth_date",
    "nationality_code",
    "preferred_foot",
    "height_cm",
    "player_status",
    "notes",
];
const NAME_HEADERS: &[&str] = &[
    "action",
    "player_key",
    "player_id",
    "match_name",
    "match_birth_date",
    "name_value",
    "language_code",
    "is_primary",
    "valid_from",
    "valid_to",
];
const POSITION_HEADERS: &[&str] = &[
    "action",
    "player_key",
    "player_id",
    "match_name",
    "match_birth_date",
    "position_code",
    "proficiency",
    "default_role_code",
    "is_primary",
    "valid_from",
    "valid_to",
];
const TEAM_PERIOD_HEADERS: &[&str] = &[
    "action",
    "player_key",
    "player_id",
    "match_name",
    "match_birth_date",
    "team_key",
    "team_id",
    "team_name",
    "season_id",
    "squad_number",
    "valid_from",
    "valid_to",
    "registration_status",
];
const ABILITY_HEADERS: &[&str] = &[
    "action",
    "player_key",
    "player_id",
    "match_name",
    "match_birth_date",
    "dimension_code",
    "context_type",
    "context_id",
    "value",
    "confidence",
    "sample_size",
    "observed_at",
    "effective_from",
    "effective_to",
    "calculation_version",
];
const AVAILABILITY_HEADERS: &[&str] = &[
    "action",
    "player_key",
    "player_id",
    "match_name",
    "match_birth_date",
    "team_key",
    "team_id",
    "team_name",
    "competition_id",
    "availability_status",
    "reason",
    "confidence",
    "valid_from",
    "valid_to",
];
const DYNAMIC_TAG_HEADERS: &[&str] = &[
    "action",
    "player_key",
    "player_id",
    "match_name",
    "match_birth_date",
    "tag_code",
    "tag_value",
    "label",
    "confidence",
    "observed_at",
    "valid_from",
    "valid_to",
    "competition_id",
    "position_code",
    "opponent_team_id",
    "sample_size",
    "source_type",
    "calculation_version",
];

const EXTERNAL_ID_HEADERS: &[&str] = &[
    "action",
    "provider_code",
    "entity_type",
    "entity_key",
    "entity_id",
    "entity_name",
    "entity_birth_date",
    "external_id",
];

const SHEETS: &[SheetSpec] = &[
    SheetSpec {
        name: "球队资料",
        entity_type: SpreadsheetEntityType::Team,
        headers: TEAM_HEADERS,
        required: &["official_name"],
        widths: &[10.0, 16.0, 38.0, 26.0, 14.0, 12.0, 30.0],
        example: &[
            "skip",
            "T001",
            "",
            "示例足球俱乐部",
            "KOR",
            "true",
            "示例行可删除",
        ],
    },
    SheetSpec {
        name: "球员基础资料",
        entity_type: SpreadsheetEntityType::Player,
        headers: PLAYER_HEADERS,
        required: &["official_name"],
        widths: &[10.0, 16.0, 38.0, 24.0, 14.0, 16.0, 16.0, 12.0, 16.0, 30.0],
        example: &[
            "skip",
            "P001",
            "",
            "示例球员",
            "2000-01-01",
            "KOR",
            "right",
            "180",
            "active",
            "示例行可删除",
        ],
    },
    SheetSpec {
        name: "球员名称",
        entity_type: SpreadsheetEntityType::PlayerName,
        headers: NAME_HEADERS,
        required: &["name_value"],
        widths: &[10.0, 16.0, 38.0, 24.0, 16.0, 24.0, 14.0, 12.0, 14.0, 14.0],
        example: &[
            "skip",
            "P001",
            "",
            "示例球员",
            "2000-01-01",
            "Example Player",
            "en",
            "false",
            "2026-01-01",
            "",
        ],
    },
    SheetSpec {
        name: "球员位置",
        entity_type: SpreadsheetEntityType::PlayerPosition,
        headers: POSITION_HEADERS,
        required: &["position_code", "proficiency"],
        widths: &[
            10.0, 16.0, 38.0, 24.0, 16.0, 16.0, 14.0, 22.0, 12.0, 14.0, 14.0,
        ],
        example: &[
            "skip",
            "P001",
            "",
            "示例球员",
            "2000-01-01",
            "ST",
            "0.85",
            "抢点中锋",
            "true",
            "2026-01-01",
            "",
        ],
    },
    SheetSpec {
        name: "球队履历",
        entity_type: SpreadsheetEntityType::PlayerTeamPeriod,
        headers: TEAM_PERIOD_HEADERS,
        required: &["valid_from", "registration_status"],
        widths: &[
            10.0, 16.0, 38.0, 24.0, 16.0, 16.0, 38.0, 24.0, 38.0, 12.0, 14.0, 14.0, 20.0,
        ],
        example: &[
            "skip",
            "P001",
            "",
            "示例球员",
            "2000-01-01",
            "T001",
            "",
            "示例足球俱乐部",
            "",
            "10",
            "2026-01-01",
            "",
            "registered",
        ],
    },
    SheetSpec {
        name: "球员能力",
        entity_type: SpreadsheetEntityType::PlayerAbility,
        headers: ABILITY_HEADERS,
        required: &[
            "dimension_code",
            "value",
            "observed_at",
            "effective_from",
            "calculation_version",
        ],
        widths: &[
            10.0, 16.0, 38.0, 24.0, 16.0, 20.0, 18.0, 38.0, 12.0, 12.0, 12.0, 24.0, 24.0, 24.0,
            24.0,
        ],
        example: &[
            "skip",
            "P001",
            "",
            "示例球员",
            "2000-01-01",
            "finishing",
            "general",
            "",
            "72",
            "0.8",
            "10",
            "2026-07-12T00:00:00Z",
            "2026-07-12T00:00:00Z",
            "",
            "manual-v1",
        ],
    },
    SheetSpec {
        name: "伤停状态",
        entity_type: SpreadsheetEntityType::PlayerAvailability,
        headers: AVAILABILITY_HEADERS,
        required: &["availability_status", "valid_from"],
        widths: &[
            10.0, 16.0, 38.0, 24.0, 16.0, 16.0, 38.0, 24.0, 38.0, 22.0, 28.0, 12.0, 24.0, 24.0,
        ],
        example: &[
            "skip",
            "P001",
            "",
            "示例球员",
            "2000-01-01",
            "T001",
            "",
            "示例足球俱乐部",
            "",
            "available",
            "",
            "1",
            "2026-07-12T00:00:00Z",
            "",
        ],
    },
    SheetSpec {
        name: "动态标签",
        entity_type: SpreadsheetEntityType::PlayerDynamicTag,
        headers: DYNAMIC_TAG_HEADERS,
        required: &[
            "tag_code",
            "tag_value",
            "observed_at",
            "valid_from",
            "valid_to",
            "calculation_version",
        ],
        widths: &[
            10.0, 16.0, 38.0, 24.0, 16.0, 22.0, 14.0, 24.0, 12.0, 24.0, 24.0, 24.0, 38.0, 16.0,
            38.0, 12.0, 18.0, 24.0,
        ],
        example: &[
            "skip",
            "P001",
            "",
            "示例球员",
            "2000-01-01",
            "match_readiness",
            "0.92",
            "状态良好",
            "0.8",
            "2026-07-12T00:00:00Z",
            "2026-07-12T00:00:00Z",
            "2026-07-19T00:00:00Z",
            "",
            "",
            "",
            "5",
            "manual",
            "manual-v1",
        ],
    },
    SheetSpec {
        name: "外部数据ID",
        entity_type: SpreadsheetEntityType::ExternalEntityId,
        headers: EXTERNAL_ID_HEADERS,
        required: &["provider_code", "entity_type", "external_id"],
        widths: &[10.0, 20.0, 16.0, 16.0, 38.0, 24.0, 16.0, 28.0],
        example: &[
            "skip",
            "provider_code",
            "player",
            "P001",
            "",
            "示例球员",
            "2000-01-01",
            "external-123",
        ],
    },
];

pub fn write_player_catalog_template(
    output_path: &Path,
    references: &PlayerCatalogReferenceData,
) -> SpreadsheetResult<()> {
    let mut workbook = Workbook::new();
    add_instruction_sheet(&mut workbook)?;
    add_field_definition_sheet(&mut workbook)?;
    add_enum_sheet(&mut workbook, references)?;
    add_metadata_sheet(&mut workbook)?;
    for spec in SHEETS {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(spec.name)?;
        write_sheet_header(worksheet, spec)?;
        write_example_row(worksheet, spec)?;
        add_validations(worksheet, spec, references)?;
    }
    workbook.save(output_path)?;
    Ok(())
}

pub fn write_player_catalog_export(
    output_path: &Path,
    references: &PlayerCatalogReferenceData,
    data: &SpreadsheetExportData,
) -> SpreadsheetResult<()> {
    let mut workbook = Workbook::new();
    add_instruction_sheet(&mut workbook)?;
    add_field_definition_sheet(&mut workbook)?;
    add_enum_sheet(&mut workbook, references)?;
    add_metadata_sheet(&mut workbook)?;
    for spec in SHEETS {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(spec.name)?;
        write_sheet_header(worksheet, spec)?;
        add_validations(worksheet, spec, references)?;
        write_export_rows(worksheet, spec.entity_type, data)?;
    }
    workbook.save(output_path)?;
    Ok(())
}

pub fn read_player_catalog_workbook(path: &Path) -> SpreadsheetResult<SpreadsheetParsedWorkbook> {
    let bytes = fs::read(path)?;
    let source_sha256 = hex_digest(&bytes);
    let mut workbook =
        open_workbook_auto(path).map_err(|error| SpreadsheetError::Read(error.to_string()))?;
    let format_version = read_format_version(&mut workbook)?;
    if format_version != PLAYER_IMPORT_FORMAT {
        return Err(SpreadsheetError::InvalidTemplate(format!(
            "模板版本应为 {PLAYER_IMPORT_FORMAT}，实际为 {format_version}"
        )));
    }
    let mut rows = Vec::new();
    for spec in SHEETS {
        let Ok(range) = workbook.worksheet_range(spec.name) else {
            continue;
        };
        let mut range_rows = range.rows();
        let Some(header_row) = range_rows.next() else {
            continue;
        };
        let headers = header_row
            .iter()
            .map(cell_text)
            .map(|value| value.trim().to_string())
            .collect::<Vec<_>>();
        validate_headers(spec, &headers)?;
        for (index, row) in range_rows.enumerate() {
            let row_number = (index + 2) as u32;
            let mut values = headers
                .iter()
                .enumerate()
                .map(|(column, header)| {
                    let value = row
                        .get(column)
                        .map(|cell| row_cell_text(cell, spec.entity_type, header))
                        .unwrap_or_default();
                    (header.clone(), Value::String(value.trim().to_string()))
                })
                .collect::<Map<String, Value>>();
            if spec.entity_type == SpreadsheetEntityType::PlayerPosition {
                apply_default_role_alias(&mut values);
            }
            if values
                .values()
                .all(|value| value.as_str().unwrap_or("").is_empty())
            {
                continue;
            }
            let action = parse_action(values.get("action").and_then(Value::as_str))?;
            rows.push(SpreadsheetRawRow {
                sheet_name: spec.name.to_string(),
                row_number,
                entity_type: spec.entity_type,
                action,
                values: Value::Object(values),
            });
        }
    }
    let source_file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("player-import.xlsx")
        .to_string();
    Ok(SpreadsheetParsedWorkbook {
        format_version,
        source_file_name,
        source_sha256,
        rows,
    })
}

fn add_instruction_sheet(workbook: &mut Workbook) -> Result<(), XlsxError> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("使用说明")?;
    worksheet.merge_range(
        0,
        0,
        0,
        5,
        "足球赛事模型平台 · 球员数据交换",
        &title_format(),
    )?;
    let rows = [
        ("模板版本", PLAYER_IMPORT_FORMAT),
        (
            "使用流程",
            "软件导出 → 填写或交给 ChatGPT 整理 → 软件预检 → 处理冲突 → 确认写入 PostgreSQL",
        ),
        (
            "稳定标识",
            "已有数据保留 UUID；新数据使用 player_key / team_key 在多个工作表间关联",
        ),
        (
            "日期格式",
            "日期使用 YYYY-MM-DD；时间使用 ISO 8601，例如 2026-07-12T00:00:00Z",
        ),
        (
            "动作",
            "add=新增；update=更新；skip=跳过。示例行默认 skip，不会写入数据库",
        ),
        (
            "安全规则",
            "Excel 永不直接写库；错误和冲突行必须先处理，确认后使用单一事务提交",
        ),
        (
            "能力结构",
            "球员能力使用长表结构：一名球员 × 一个能力维度 × 一条观察记录",
        ),
        (
            "动态标签",
            "短期状态必须提供 valid_to；过期后不会参与后续比赛计算",
        ),
        (
            "球队自动关联",
            "在球队履历页仅填写 team_name 且数据库中不存在同名球队时，预检会明确标注并在提交事务内自动创建球队后建立球员归属；同名多条仍必须人工处理",
        ),
    ];
    for (offset, (label, text)) in rows.iter().enumerate() {
        let row = (offset + 2) as u32;
        worksheet.write_string_with_format(row, 0, *label, &section_format())?;
        worksheet.merge_range(row, 1, row, 5, text, &note_format())?;
    }
    worksheet.set_column_width(0, 18)?;
    worksheet.set_column_range_width(1, 5, 22)?;
    worksheet.set_row_height(0, 28)?;
    worksheet.set_freeze_panes(1, 0)?;
    Ok(())
}

fn add_field_definition_sheet(workbook: &mut Workbook) -> Result<(), XlsxError> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("字段说明")?;
    let headers = [
        "工作表",
        "字段",
        "中文含义",
        "必填",
        "格式/范围",
        "写入规则",
        "示例",
    ];
    for (column, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, column as u16, *header, &header_format())?;
    }
    let mut row = 1_u32;
    for spec in SHEETS {
        for (column, field) in spec.headers.iter().enumerate() {
            let required = spec.required.contains(field);
            let example = spec.example.get(column).copied().unwrap_or("");
            let values = [
                spec.name,
                *field,
                field_description(field),
                if required { "是" } else { "否" },
                field_format(field),
                field_write_rule(spec.entity_type, field),
                example,
            ];
            for (output_column, value) in values.iter().enumerate() {
                worksheet.write_string(row, output_column as u16, *value)?;
            }
            row += 1;
        }
    }
    worksheet.set_column_width(0, 18)?;
    worksheet.set_column_width(1, 24)?;
    worksheet.set_column_width(2, 26)?;
    worksheet.set_column_width(3, 10)?;
    worksheet.set_column_width(4, 30)?;
    worksheet.set_column_width(5, 42)?;
    worksheet.set_column_width(6, 28)?;
    worksheet.set_freeze_panes(1, 0)?;
    worksheet.autofilter(0, 0, row.saturating_sub(1), 6)?;
    Ok(())
}

fn field_description(field: &str) -> &'static str {
    match field {
        "action" => "本行导入动作",
        "team_key" => "工作簿内球队临时键",
        "team_id" => "数据库球队 UUID",
        "official_name" => "正式显示名称",
        "country_code" => "国家或地区代码",
        "is_active" => "是否仍为有效球队",
        "notes" => "人工备注，不参与实体匹配",
        "player_key" => "工作簿内球员临时键",
        "player_id" => "数据库球员 UUID",
        "birth_date" | "match_birth_date" | "entity_birth_date" => "出生日期",
        "nationality_code" => "国籍代码",
        "preferred_foot" => "惯用脚",
        "height_cm" => "身高（厘米）",
        "player_status" => "球员职业状态",
        "match_name" => "用于匹配已有球员的姓名",
        "name_value" => "新增球员名称或别名",
        "language_code" => "名称语言代码",
        "is_primary" => "是否作为当前主名称或主位置",
        "valid_from" => "生效起始日期或时间",
        "valid_to" => "失效日期或时间",
        "position_code" => "位置代码",
        "proficiency" => "位置熟练度",
        "default_role_code" => "该位置下自动继承到比赛阵容的默认战术角色",
        "team_name" => "用于匹配已有球队的名称",
        "season_id" => "数据库赛季 UUID",
        "squad_number" => "球衣号码",
        "registration_status" => "球队注册关系状态",
        "dimension_code" => "球员能力维度代码",
        "context_type" => "能力观察上下文类型",
        "context_id" => "能力观察上下文 UUID",
        "value" => "能力观察值",
        "confidence" => "可信度",
        "sample_size" => "支撑该观察的样本量",
        "observed_at" => "实际观察时间",
        "effective_from" => "能力生效时间",
        "effective_to" => "能力失效时间",
        "calculation_version" => "能力计算或人工录入版本",
        "competition_id" => "数据库赛事 UUID",
        "availability_status" => "球员伤停或可用状态",
        "reason" => "状态原因",
        "provider_code" => "数据供应商代码",
        "entity_type" => "外部 ID 所属实体类型",
        "entity_key" => "工作簿内实体临时键",
        "entity_id" => "数据库实体 UUID",
        "entity_name" => "用于匹配已有实体的名称",
        "external_id" => "供应商系统中的稳定 ID",
        "tag_code" => "动态标签代码",
        "tag_value" => "动态标签数值",
        "label" => "界面显示标签",
        "opponent_team_id" => "仅对指定对手生效的球队 UUID",
        "source_type" => "动态标签数据来源类型",
        _ => "扩展字段",
    }
}

fn field_format(field: &str) -> &'static str {
    match field {
        "action" => "add / update / skip",
        "team_id" | "player_id" | "season_id" | "context_id" | "competition_id" | "entity_id"
        | "opponent_team_id" => "UUID；新实体可留空",
        "birth_date" | "match_birth_date" | "entity_birth_date" => "YYYY-MM-DD",
        "valid_from" | "valid_to" => "按工作表要求使用 YYYY-MM-DD 或 ISO 8601",
        "observed_at" | "effective_from" | "effective_to" => "ISO 8601，例如 2026-07-12T00:00:00Z",
        "tag_value" => "按动态标签定义的最小/最大范围填写",
        "height_cm" => "120–230 的整数",
        "squad_number" => "0–99 的整数",
        "proficiency" | "confidence" => "0–1 的小数",
        "default_role_code" => "自由文本，例如：组织核心、单后腰、抢点中锋",
        "sample_size" => "非负整数",
        "is_active" | "is_primary" => "true / false",
        "position_code" | "dimension_code" | "provider_code" | "tag_code" => "从枚举值工作表选择",
        _ => "文本",
    }
}

fn field_write_rule(entity_type: SpreadsheetEntityType, field: &str) -> &'static str {
    if field == "action" {
        return "球队/球员基础资料支持 update；历史型工作表使用 add 或 skip";
    }
    if matches!(field, "team_id" | "player_id" | "entity_id") {
        return "存在 UUID 时优先按 UUID 匹配；不存在时再使用临时键或姓名";
    }
    if matches!(field, "team_key" | "player_key" | "entity_key") {
        return "只在当前工作簿中使用，用于跨工作表关联新实体";
    }
    match entity_type {
        SpreadsheetEntityType::Team | SpreadsheetEntityType::Player => {
            "add 创建；update 更新当前基础资料；skip 不处理"
        }
        SpreadsheetEntityType::ExternalEntityId => {
            "add 新增稳定关联；已绑定到其他实体时禁止自动改绑"
        }
        SpreadsheetEntityType::PlayerTeamPeriod if field == "team_name" => {
            "优先匹配已有球队；未填 team_id/team_key 且没有同名球队时，预检后可自动创建球队并绑定球员"
        }
        _ => "历史记录只追加；导出现有数据时默认 skip，明确新增时改为 add",
    }
}

fn add_metadata_sheet(workbook: &mut Workbook) -> Result<(), XlsxError> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("元数据")?;
    worksheet.write_string(0, 0, "format_version")?;
    worksheet.write_string(0, 1, PLAYER_IMPORT_FORMAT)?;
    worksheet.write_string(1, 0, "generated_by")?;
    worksheet.write_string(1, 1, "football-match-model-platform")?;
    worksheet.set_hidden(true);
    Ok(())
}

fn add_enum_sheet(
    workbook: &mut Workbook,
    references: &PlayerCatalogReferenceData,
) -> Result<(), XlsxError> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("枚举值")?;
    let fixed: &[(&str, &[&str])] = &[
        ("action", &["add", "update", "skip"]),
        (
            "player_status",
            &["active", "inactive", "retired", "unknown"],
        ),
        ("preferred_foot", &["left", "right", "both", "unknown"]),
        (
            "registration_status",
            &["registered", "loan", "trial", "released", "unknown"],
        ),
        (
            "availability_status",
            &[
                "available",
                "doubtful",
                "unavailable",
                "injured",
                "suspended",
                "rested",
                "returning",
                "unknown",
            ],
        ),
        ("entity_type", &["player", "team"]),
    ];
    for (column, (header, values)) in fixed.iter().enumerate() {
        let column = column as u16;
        worksheet.write_string_with_format(0, column, *header, &header_format())?;
        for (row, value) in values.iter().enumerate() {
            worksheet.write_string((row + 1) as u32, column, *value)?;
        }
        worksheet.set_column_width(column, 22)?;
    }
    let position_column = fixed.len() as u16;
    worksheet.write_string_with_format(0, position_column, "position_code", &header_format())?;
    for (row, position) in references.positions.iter().enumerate() {
        worksheet.write_string((row + 1) as u32, position_column, &position.code)?;
    }
    let ability_column = position_column + 1;
    worksheet.write_string_with_format(0, ability_column, "ability_dimension", &header_format())?;
    for (row, ability) in references.ability_dimensions.iter().enumerate() {
        worksheet.write_string((row + 1) as u32, ability_column, &ability.code)?;
    }
    let provider_column = ability_column + 1;
    worksheet.write_string_with_format(0, provider_column, "provider_code", &header_format())?;
    for (row, provider) in references.providers.iter().enumerate() {
        worksheet.write_string((row + 1) as u32, provider_column, &provider.code)?;
    }
    let dynamic_tag_column = provider_column + 1;
    worksheet.write_string_with_format(
        0,
        dynamic_tag_column,
        "dynamic_tag_code",
        &header_format(),
    )?;
    for (row, tag) in references.dynamic_tag_definitions.iter().enumerate() {
        worksheet.write_string((row + 1) as u32, dynamic_tag_column, &tag.code)?;
    }
    worksheet.set_column_range_width(position_column, dynamic_tag_column, 24)?;
    worksheet.set_freeze_panes(1, 0)?;
    Ok(())
}

fn write_sheet_header(worksheet: &mut Worksheet, spec: &SheetSpec) -> Result<(), XlsxError> {
    for (column, header) in spec.headers.iter().enumerate() {
        let format = if spec.required.contains(header) {
            required_header_format()
        } else {
            header_format()
        };
        worksheet.write_string_with_format(0, column as u16, *header, &format)?;
        worksheet.set_column_width(column as u16, spec.widths[column])?;
    }
    worksheet.set_row_height(0, 32)?;
    worksheet.set_freeze_panes(1, 0)?;
    worksheet.autofilter(0, 0, TEMPLATE_LAST_ROW, (spec.headers.len() - 1) as u16)?;
    Ok(())
}

fn write_example_row(worksheet: &mut Worksheet, spec: &SheetSpec) -> Result<(), XlsxError> {
    for (column, value) in spec.example.iter().enumerate() {
        worksheet.write_string_with_format(1, column as u16, *value, &example_format())?;
    }
    Ok(())
}

fn add_validations(
    worksheet: &mut Worksheet,
    spec: &SheetSpec,
    references: &PlayerCatalogReferenceData,
) -> Result<(), XlsxError> {
    for (column, header) in spec.headers.iter().enumerate() {
        let fixed_values: Option<&[&str]> = match *header {
            "action" => Some(&["add", "update", "skip"]),
            "player_status" => Some(&["active", "inactive", "retired", "unknown"]),
            "preferred_foot" => Some(&["left", "right", "both", "unknown"]),
            "registration_status" => Some(&["registered", "loan", "trial", "released", "unknown"]),
            "availability_status" => Some(&[
                "available",
                "doubtful",
                "injured",
                "suspended",
                "rested",
                "returning",
                "unknown",
            ]),
            "entity_type" => Some(&["player", "team"]),
            "source_type" => Some(&[
                "manual",
                "provider",
                "lineup_import",
                "ai_analysis",
                "match_review",
                "calculation",
            ]),
            _ => None,
        };
        if let Some(values) = fixed_values {
            let validation = DataValidation::new().allow_list_strings(values)?;
            worksheet.add_data_validation(
                1,
                column as u16,
                TEMPLATE_LAST_ROW,
                column as u16,
                &validation,
            )?;
            continue;
        }

        let dynamic = match *header {
            "position_code" => Some((6usize, references.positions.len())),
            "dimension_code" => Some((7usize, references.ability_dimensions.len())),
            "provider_code" => Some((8usize, references.providers.len())),
            "tag_code" => Some((9usize, references.dynamic_tag_definitions.len())),
            _ => None,
        };
        if let Some((enum_column, length)) = dynamic.filter(|(_, length)| *length > 0) {
            let column_name = excel_column_name(enum_column);
            let range = format!("'枚举值'!${column_name}$2:${column_name}${}", length + 1);
            let formula = format!("=INDIRECT(\"{range}\")");
            let validation = DataValidation::new().allow_list_formula(Formula::new(formula));
            worksheet.add_data_validation(
                1,
                column as u16,
                TEMPLATE_LAST_ROW,
                column as u16,
                &validation,
            )?;
        }
    }
    Ok(())
}

fn excel_column_name(mut zero_based: usize) -> String {
    let mut output = String::new();
    loop {
        let remainder = zero_based % 26;
        output.insert(0, (b'A' + remainder as u8) as char);
        if zero_based < 26 {
            break;
        }
        zero_based = zero_based / 26 - 1;
    }
    output
}

fn write_export_rows(
    worksheet: &mut Worksheet,
    entity_type: SpreadsheetEntityType,
    data: &SpreadsheetExportData,
) -> Result<(), XlsxError> {
    match entity_type {
        SpreadsheetEntityType::Team => {
            for (index, item) in data.teams.iter().enumerate() {
                let row = (index + 1) as u32;
                write_strings(
                    worksheet,
                    row,
                    &[
                        "update",
                        "",
                        &item.team_id.to_string(),
                        &item.canonical_name,
                        item.country_code.as_deref().unwrap_or(""),
                        if item.is_active { "true" } else { "false" },
                        "",
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::Player => {
            for (index, item) in data.players.iter().enumerate() {
                let row = (index + 1) as u32;
                write_strings(
                    worksheet,
                    row,
                    &[
                        "update",
                        "",
                        &item.player_id.to_string(),
                        &item.canonical_name,
                        &item
                            .date_of_birth
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        item.nationality_code.as_deref().unwrap_or(""),
                        &item.preferred_foot,
                        &item
                            .height_cm
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        &item.status,
                        "",
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerName => {
            for (index, item) in data.names.iter().enumerate() {
                let row = (index + 1) as u32;
                write_strings(
                    worksheet,
                    row,
                    &[
                        "skip",
                        "",
                        &item.player_id.to_string(),
                        &item.player_name,
                        &item
                            .player_birth_date
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        &item.name,
                        item.language_code.as_deref().unwrap_or(""),
                        if item.is_primary { "true" } else { "false" },
                        &item
                            .valid_from
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        &item
                            .valid_to
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerPosition => {
            for (index, item) in data.positions.iter().enumerate() {
                let row = (index + 1) as u32;
                write_strings(
                    worksheet,
                    row,
                    &[
                        "skip",
                        "",
                        &item.player_id.to_string(),
                        &item.player_name,
                        &item
                            .player_birth_date
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        &item.position_code,
                        &item.proficiency.to_string(),
                        item.default_role_code.as_deref().unwrap_or(""),
                        if item.is_primary { "true" } else { "false" },
                        &item
                            .valid_from
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        &item
                            .valid_to
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerTeamPeriod => {
            for (index, item) in data.team_periods.iter().enumerate() {
                let row = (index + 1) as u32;
                write_strings(
                    worksheet,
                    row,
                    &[
                        "skip",
                        "",
                        &item.player_id.to_string(),
                        &item.player_name,
                        &item
                            .player_birth_date
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        "",
                        &item.team_id.to_string(),
                        &item.team_name,
                        &item
                            .season_id
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        &item
                            .squad_number
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        &item.valid_from.to_string(),
                        &item
                            .valid_to
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        &item.registration_status,
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerAbility => {
            for (index, item) in data.abilities.iter().enumerate() {
                let row = (index + 1) as u32;
                write_strings(
                    worksheet,
                    row,
                    &[
                        "skip",
                        "",
                        &item.player_id.to_string(),
                        &item.player_name,
                        &item
                            .player_birth_date
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        &item.dimension_code,
                        &item.context_type,
                        &item
                            .context_id
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        &item.value.to_string(),
                        &item.confidence.to_string(),
                        &item.sample_size.to_string(),
                        &item.observed_at.to_rfc3339(),
                        &item.effective_from.to_rfc3339(),
                        &item
                            .effective_to
                            .map(|value| value.to_rfc3339())
                            .unwrap_or_default(),
                        &item.calculation_version,
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerAvailability => {
            for (index, item) in data.availability.iter().enumerate() {
                let row = (index + 1) as u32;
                write_strings(
                    worksheet,
                    row,
                    &[
                        "skip",
                        "",
                        &item.player_id.to_string(),
                        &item.player_name,
                        &item
                            .player_birth_date
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        "",
                        &item
                            .team_id
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        item.team_name.as_deref().unwrap_or(""),
                        &item
                            .competition_id
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        &item.status,
                        item.reason.as_deref().unwrap_or(""),
                        &item.confidence.to_string(),
                        &item.valid_from.to_rfc3339(),
                        &item
                            .valid_to
                            .map(|value| value.to_rfc3339())
                            .unwrap_or_default(),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::ExternalEntityId => {
            for (index, item) in data.external_ids.iter().enumerate() {
                let row = (index + 1) as u32;
                write_strings(
                    worksheet,
                    row,
                    &[
                        "skip",
                        &item.provider_code,
                        &item.entity_type,
                        "",
                        &item.entity_id.to_string(),
                        &item.entity_name,
                        "",
                        &item.external_id,
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerDynamicTag => {
            for (index, item) in data.dynamic_tags.iter().enumerate() {
                let row = (index + 1) as u32;
                let values = vec![
                    "skip".to_string(),
                    String::new(),
                    item.player_id.to_string(),
                    item.player_name.clone(),
                    item.player_birth_date
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    item.tag_code.clone(),
                    item.value.to_string(),
                    item.label.clone().unwrap_or_default(),
                    item.confidence.to_string(),
                    item.observed_at.to_rfc3339(),
                    item.valid_from.to_rfc3339(),
                    item.valid_to.to_rfc3339(),
                    item.competition_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    item.position_code.clone().unwrap_or_default(),
                    item.opponent_team_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    item.sample_size.to_string(),
                    item.source_type.clone(),
                    item.calculation_version.clone(),
                ];
                write_owned_strings(worksheet, row, &values)?;
            }
        }
        SpreadsheetEntityType::TeamName
        | SpreadsheetEntityType::Coach
        | SpreadsheetEntityType::CoachName
        | SpreadsheetEntityType::TeamCoachPeriod
        | SpreadsheetEntityType::FormationUsage
        | SpreadsheetEntityType::TeamTacticalObservation
        | SpreadsheetEntityType::TeamAbilityObservation
        | SpreadsheetEntityType::Match
        | SpreadsheetEntityType::Lineup
        | SpreadsheetEntityType::LineupPlayer => {}
    }
    Ok(())
}

fn write_strings(worksheet: &mut Worksheet, row: u32, values: &[&str]) -> Result<(), XlsxError> {
    for (column, value) in values.iter().enumerate() {
        worksheet.write_string(row, column as u16, *value)?;
    }
    Ok(())
}

fn write_owned_strings(
    worksheet: &mut Worksheet,
    row: u32,
    values: &[String],
) -> Result<(), XlsxError> {
    for (column, value) in values.iter().enumerate() {
        worksheet.write_string(row, column as u16, value)?;
    }
    Ok(())
}

fn read_format_version<R: std::io::Read + std::io::Seek>(
    workbook: &mut calamine::Sheets<R>,
) -> SpreadsheetResult<String> {
    let range = workbook
        .worksheet_range("元数据")
        .map_err(|error| SpreadsheetError::Read(error.to_string()))?;
    let value = range.get((0, 1)).map(cell_text).unwrap_or_default();
    if value.trim().is_empty() {
        return Err(SpreadsheetError::InvalidTemplate(
            "缺少元数据工作表或 format_version".to_string(),
        ));
    }
    Ok(value.trim().to_string())
}

fn apply_default_role_alias(values: &mut Map<String, Value>) {
    if values
        .get("default_role_code")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
    {
        return;
    }
    for alias in [
        "role_code",
        "tactical_role_code",
        "default_tactical_role",
        "player_role_code",
        "战术角色",
        "默认战术角色",
        "球员战术角色",
    ] {
        if let Some(role_code) = values
            .get(alias)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
        {
            values.insert("default_role_code".to_string(), Value::String(role_code));
            break;
        }
    }
}

fn validate_headers(spec: &SheetSpec, headers: &[String]) -> SpreadsheetResult<()> {
    let index = headers
        .iter()
        .enumerate()
        .map(|(position, header)| (header.as_str(), position))
        .collect::<HashMap<_, _>>();
    for header in spec.headers {
        if !index.contains_key(header)
            && !matches!(
                (spec.entity_type, *header),
                (SpreadsheetEntityType::PlayerPosition, "default_role_code")
            )
        {
            return Err(SpreadsheetError::InvalidTemplate(format!(
                "工作表 {} 缺少固定列 {}",
                spec.name, header
            )));
        }
    }
    Ok(())
}

fn parse_action(value: Option<&str>) -> SpreadsheetResult<SpreadsheetAction> {
    match value.unwrap_or("add").trim().to_lowercase().as_str() {
        "" | "add" => Ok(SpreadsheetAction::Add),
        "update" => Ok(SpreadsheetAction::Update),
        "clear" => Ok(SpreadsheetAction::Clear),
        "skip" => Ok(SpreadsheetAction::Skip),
        other => Err(SpreadsheetError::InvalidTemplate(format!(
            "未知 action：{other}"
        ))),
    }
}

fn row_cell_text(value: &Data, entity_type: SpreadsheetEntityType, header: &str) -> String {
    let date_only = matches!(
        header,
        "birth_date" | "match_birth_date" | "entity_birth_date"
    ) || matches!(
        (entity_type, header),
        (SpreadsheetEntityType::PlayerName, "valid_from" | "valid_to")
            | (
                SpreadsheetEntityType::PlayerPosition,
                "valid_from" | "valid_to"
            )
            | (
                SpreadsheetEntityType::PlayerTeamPeriod,
                "valid_from" | "valid_to"
            )
    );
    if date_only {
        if let Some(date) = value.as_date() {
            return date.format("%Y-%m-%d").to_string();
        }
    }
    let date_time = matches!(
        (entity_type, header),
        (
            SpreadsheetEntityType::PlayerAbility,
            "observed_at" | "effective_from" | "effective_to"
        ) | (
            SpreadsheetEntityType::PlayerAvailability,
            "valid_from" | "valid_to"
        ) | (
            SpreadsheetEntityType::PlayerDynamicTag,
            "observed_at" | "valid_from" | "valid_to"
        )
    );
    if date_time {
        if let Some(date_time) = value.as_datetime() {
            return format!("{}Z", date_time.format("%Y-%m-%dT%H:%M:%S"));
        }
    }
    cell_text(value)
}

fn cell_text(value: &Data) -> String {
    match value {
        Data::Empty => String::new(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn title_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_size(16.0)
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x163A5F))
        .set_align(FormatAlign::VerticalCenter)
}

fn header_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x2563A6))
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_text_wrap()
}

fn required_header_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0xB45309))
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_text_wrap()
}

fn section_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(Color::RGB(0x163A5F))
        .set_background_color(Color::RGB(0xDCEAF7))
        .set_border(FormatBorder::Thin)
}

fn note_format() -> Format {
    Format::new()
        .set_font_color(Color::RGB(0x475569))
        .set_background_color(Color::RGB(0xF3F6FA))
        .set_border(FormatBorder::Thin)
        .set_text_wrap()
}

fn example_format() -> Format {
    Format::new()
        .set_font_color(Color::RGB(0x7C2D12))
        .set_background_color(Color::RGB(0xFFF7ED))
        .set_border(FormatBorder::Thin)
}

/// Reads a workbook into a bounded plain-text representation for the API workspace.
/// Formula results are treated as untrusted attachment content and never executed.
pub fn read_workbook_for_api(path: &Path) -> SpreadsheetResult<String> {
    const MAX_SHEETS: usize = 20;
    const MAX_ROWS_PER_SHEET: usize = 5_000;
    const MAX_COLUMNS_PER_ROW: usize = 100;
    const MAX_OUTPUT_BYTES: usize = 1_500_000;

    let mut workbook =
        open_workbook_auto(path).map_err(|error| SpreadsheetError::Read(error.to_string()))?;
    let sheet_names = workbook.sheet_names().to_vec();
    let mut output = String::new();
    for sheet_name in sheet_names.into_iter().take(MAX_SHEETS) {
        if output.len() >= MAX_OUTPUT_BYTES {
            break;
        }
        let range = workbook
            .worksheet_range(&sheet_name)
            .map_err(|error| SpreadsheetError::Read(error.to_string()))?;
        output.push_str("\n## Sheet: ");
        output.push_str(&sheet_name.replace(['\r', '\n'], " "));
        output.push('\n');
        for row in range.rows().take(MAX_ROWS_PER_SHEET) {
            let line = row
                .iter()
                .take(MAX_COLUMNS_PER_ROW)
                .map(|cell| cell.to_string().replace(['\r', '\n', '\t'], " "))
                .collect::<Vec<_>>()
                .join("\t");
            if output.len().saturating_add(line.len()).saturating_add(1) > MAX_OUTPUT_BYTES {
                output.push_str("\n[workbook content truncated]\n");
                return Ok(output);
            }
            output.push_str(&line);
            output.push('\n');
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::PlayerCatalogReferenceData;

    #[test]
    fn sheet_contracts_are_aligned_and_unique() {
        for sheet in SHEETS {
            assert_eq!(
                sheet.headers.len(),
                sheet.widths.len(),
                "{} widths",
                sheet.name
            );
            assert_eq!(
                sheet.headers.len(),
                sheet.example.len(),
                "{} example",
                sheet.name
            );
            let mut seen = std::collections::HashSet::new();
            for header in sheet.headers {
                assert!(
                    seen.insert(header),
                    "{} contains duplicate {header}",
                    sheet.name
                );
            }
        }
    }

    #[test]
    fn converts_zero_based_excel_columns() {
        assert_eq!(excel_column_name(0), "A");
        assert_eq!(excel_column_name(25), "Z");
        assert_eq!(excel_column_name(26), "AA");
        assert_eq!(excel_column_name(52), "BA");
    }

    #[test]
    fn writes_and_reads_empty_template() {
        let path = std::env::temp_dir().join("football-player-import-test.xlsx");
        write_player_catalog_template(&path, &PlayerCatalogReferenceData::default())
            .expect("write template");
        let parsed = read_player_catalog_workbook(&path).expect("read template");
        assert_eq!(parsed.format_version, PLAYER_IMPORT_FORMAT);
        assert_eq!(parsed.rows.len(), SHEETS.len());
        assert!(parsed
            .rows
            .iter()
            .all(|row| row.action == SpreadsheetAction::Skip));
        let _ = std::fs::remove_file(path);
    }
}
