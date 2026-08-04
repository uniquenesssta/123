use crate::{SpreadsheetError, SpreadsheetResult};
use calamine::{open_workbook_auto, Data, Reader};
use football_domain::{
    MonthlyDataGapRow, PlayerCatalogReferenceData, SpreadsheetAction, SpreadsheetEntityType,
    SpreadsheetExportData, SpreadsheetParsedWorkbook, SpreadsheetRawRow, TeamMonthlyWorkbookData,
    PLAYER_MONTHLY_FORMAT, TEAM_MONTHLY_FORMAT,
};
use rust_xlsxwriter::{
    Color, DataValidation, Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError,
};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[derive(Clone, Copy)]
struct MonthlySheetSpec {
    name: &'static str,
    entity_type: Option<SpreadsheetEntityType>,
    headers: &'static [&'static str],
}

const MONTHLY_TEMPLATE_LAST_ROW: u32 = 99_999;

const TEAM_BASIC: &[&str] = &[
    "action",
    "clear_fields",
    "team_id",
    "official_name",
    "short_name",
    "team_type",
    "country_code",
    "city",
    "founded_year",
    "stadium",
    "is_active",
    "profile_observed_at",
    "data_confidence",
    "source_urls",
    "verified_at",
    "notes",
];
const TEAM_NAMES: &[&str] = &[
    "action",
    "clear_fields",
    "team_id",
    "official_name",
    "name_value",
    "language_code",
    "valid_from",
    "valid_to",
    "source_urls",
    "verified_at",
    "confidence",
    "notes",
];
const COACHES: &[&str] = &[
    "action",
    "clear_fields",
    "coach_id",
    "official_name",
    "nationality_code",
    "coach_status",
    "source_urls",
    "verified_at",
    "confidence",
    "notes",
];
const COACH_PERIODS: &[&str] = &[
    "action",
    "clear_fields",
    "team_id",
    "team_name",
    "coach_id",
    "coach_name",
    "role",
    "valid_from",
    "valid_to",
    "is_interim",
    "confidence",
    "source_urls",
    "verified_at",
    "notes",
];
const FORMATION_USAGE: &[&str] = &[
    "action",
    "scope_type",
    "team_id",
    "team_name",
    "coach_id",
    "coach_name",
    "competition_id",
    "formation_id",
    "formation_code",
    "window_preset",
    "window_start",
    "window_end",
    "observed_matches",
    "usage_count",
    "computed_probability",
    "confidence",
    "alpha",
    "source_urls",
    "verified_at",
    "observed_at",
    "notes",
];
const TACTICAL: &[&str] = &[
    "action",
    "team_id",
    "team_name",
    "coach_id",
    "coach_name",
    "window_start",
    "window_end",
    "build_up_style",
    "progression_style",
    "attacking_width",
    "pressing_intensity",
    "defensive_block",
    "transition_speed",
    "set_piece_tendency",
    "tactical_summary",
    "confidence",
    "source_urls",
    "verified_at",
    "observed_at",
    "notes",
];
const TEAM_ABILITY: &[&str] = &[
    "action",
    "team_id",
    "team_name",
    "observed_at",
    "window_start",
    "window_end",
    "attack_rating",
    "midfield_rating",
    "defence_rating",
    "goalkeeper_rating",
    "squad_depth_rating",
    "stability_rating",
    "sample_size",
    "methodology",
    "confidence",
    "source_urls",
    "verified_at",
    "notes",
];

const PLAYER_BASIC: &[&str] = &[
    "action",
    "clear_fields",
    "player_key",
    "player_id",
    "official_name",
    "birth_date",
    "nationality_code",
    "preferred_foot",
    "height_cm",
    "player_status",
    "source_urls",
    "verified_at",
    "confidence",
    "notes",
];
const PLAYER_NAMES: &[&str] = &[
    "action",
    "clear_fields",
    "player_key",
    "player_id",
    "match_name",
    "match_birth_date",
    "name_value",
    "language_code",
    "is_primary",
    "valid_from",
    "valid_to",
    "source_urls",
    "verified_at",
    "confidence",
    "notes",
];
const PLAYER_PERIODS: &[&str] = &[
    "action",
    "clear_fields",
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
    "source_urls",
    "verified_at",
    "confidence",
    "notes",
];
const PLAYER_POSITIONS: &[&str] = &[
    "action",
    "clear_fields",
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
    "source_urls",
    "verified_at",
    "confidence",
    "notes",
];
const PLAYER_AVAILABILITY: &[&str] = &[
    "action",
    "clear_fields",
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
    "source_urls",
    "verified_at",
    "notes",
];
const PLAYER_ABILITY: &[&str] = &[
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
    "source_urls",
    "verified_at",
    "notes",
];
const PLAYER_TAGS: &[&str] = &[
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
    "source_urls",
    "verified_at",
    "notes",
];
const GAP_HEADERS: &[&str] = &[
    "entity_type",
    "entity_id",
    "entity_name",
    "missing_field",
    "last_observed_at",
    "stale_days",
    "priority",
    "recommended_action",
];

const TEAM_SHEETS: &[MonthlySheetSpec] = &[
    spec(
        "球队基础资料",
        Some(SpreadsheetEntityType::Team),
        TEAM_BASIC,
    ),
    spec(
        "球队别名",
        Some(SpreadsheetEntityType::TeamName),
        TEAM_NAMES,
    ),
    spec("教练目录", Some(SpreadsheetEntityType::Coach), COACHES),
    spec(
        "教练任期",
        Some(SpreadsheetEntityType::TeamCoachPeriod),
        COACH_PERIODS,
    ),
    spec(
        "阵型使用",
        Some(SpreadsheetEntityType::FormationUsage),
        FORMATION_USAGE,
    ),
    spec(
        "战术画像",
        Some(SpreadsheetEntityType::TeamTacticalObservation),
        TACTICAL,
    ),
    spec(
        "球队能力观察",
        Some(SpreadsheetEntityType::TeamAbilityObservation),
        TEAM_ABILITY,
    ),
];
const PLAYER_SHEETS: &[MonthlySheetSpec] = &[
    spec(
        "球员基础资料",
        Some(SpreadsheetEntityType::Player),
        PLAYER_BASIC,
    ),
    spec(
        "球员名称",
        Some(SpreadsheetEntityType::PlayerName),
        PLAYER_NAMES,
    ),
    spec(
        "球队履历",
        Some(SpreadsheetEntityType::PlayerTeamPeriod),
        PLAYER_PERIODS,
    ),
    spec(
        "球员位置",
        Some(SpreadsheetEntityType::PlayerPosition),
        PLAYER_POSITIONS,
    ),
    spec(
        "球员可用性",
        Some(SpreadsheetEntityType::PlayerAvailability),
        PLAYER_AVAILABILITY,
    ),
    spec(
        "能力观察",
        Some(SpreadsheetEntityType::PlayerAbility),
        PLAYER_ABILITY,
    ),
    spec(
        "动态标签",
        Some(SpreadsheetEntityType::PlayerDynamicTag),
        PLAYER_TAGS,
    ),
];

const fn spec(
    name: &'static str,
    entity_type: Option<SpreadsheetEntityType>,
    headers: &'static [&'static str],
) -> MonthlySheetSpec {
    MonthlySheetSpec {
        name,
        entity_type,
        headers,
    }
}

pub fn write_team_monthly_template(
    output_path: &Path,
    references: &PlayerCatalogReferenceData,
) -> SpreadsheetResult<()> {
    write_team_workbook(output_path, references, None)
}

pub fn write_team_monthly_export(
    output_path: &Path,
    references: &PlayerCatalogReferenceData,
    data: &TeamMonthlyWorkbookData,
) -> SpreadsheetResult<()> {
    write_team_workbook(output_path, references, Some(data))
}

pub fn write_player_monthly_template(
    output_path: &Path,
    references: &PlayerCatalogReferenceData,
) -> SpreadsheetResult<()> {
    write_player_workbook(output_path, references, None, &[])
}

pub fn write_player_monthly_export(
    output_path: &Path,
    references: &PlayerCatalogReferenceData,
    data: &SpreadsheetExportData,
    gaps: &[MonthlyDataGapRow],
) -> SpreadsheetResult<()> {
    write_player_workbook(output_path, references, Some(data), gaps)
}

pub fn read_team_monthly_workbook(path: &Path) -> SpreadsheetResult<SpreadsheetParsedWorkbook> {
    read_monthly_workbook(path, TEAM_MONTHLY_FORMAT, TEAM_SHEETS)
}

pub fn read_player_monthly_workbook(path: &Path) -> SpreadsheetResult<SpreadsheetParsedWorkbook> {
    read_monthly_workbook(path, PLAYER_MONTHLY_FORMAT, PLAYER_SHEETS)
}

fn write_team_workbook(
    output_path: &Path,
    references: &PlayerCatalogReferenceData,
    data: Option<&TeamMonthlyWorkbookData>,
) -> SpreadsheetResult<()> {
    let mut workbook = Workbook::new();
    add_instruction_sheet(&mut workbook, "球队月度更新工作簿", TEAM_MONTHLY_FORMAT)?;
    for sheet in TEAM_SHEETS {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(sheet.name)?;
        write_header(worksheet, sheet.headers)?;
        add_monthly_validations(worksheet, sheet.headers)?;
        if let Some(data) = data {
            write_team_rows(worksheet, sheet.entity_type.expect("business sheet"), data)?;
        } else {
            write_example(worksheet, sheet.headers)?;
        }
    }
    add_gap_sheet(
        &mut workbook,
        data.map(|value| value.data_gaps.as_slice()).unwrap_or(&[]),
    )?;
    add_team_reference_sheet(&mut workbook, references)?;
    add_metadata_sheet(&mut workbook, TEAM_MONTHLY_FORMAT, "team_monthly")?;
    workbook.save(output_path)?;
    Ok(())
}

fn write_player_workbook(
    output_path: &Path,
    references: &PlayerCatalogReferenceData,
    data: Option<&SpreadsheetExportData>,
    gaps: &[MonthlyDataGapRow],
) -> SpreadsheetResult<()> {
    let mut workbook = Workbook::new();
    add_instruction_sheet(&mut workbook, "球员月度更新工作簿", PLAYER_MONTHLY_FORMAT)?;
    for sheet in PLAYER_SHEETS {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(sheet.name)?;
        write_header(worksheet, sheet.headers)?;
        add_monthly_validations(worksheet, sheet.headers)?;
        if let Some(data) = data {
            write_player_rows(worksheet, sheet.entity_type.expect("business sheet"), data)?;
        } else {
            write_example(worksheet, sheet.headers)?;
        }
    }
    add_gap_sheet(&mut workbook, gaps)?;
    add_player_team_reference_sheet(&mut workbook, references)?;
    add_player_dictionary_sheet(&mut workbook, references)?;
    add_metadata_sheet(&mut workbook, PLAYER_MONTHLY_FORMAT, "player_monthly")?;
    workbook.save(output_path)?;
    Ok(())
}

fn add_instruction_sheet(
    workbook: &mut Workbook,
    title: &str,
    format_version: &str,
) -> Result<(), XlsxError> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("说明与校验")?;
    worksheet.merge_range(0, 0, 0, 6, title, &title_format())?;
    let mut rows = vec![
        ("格式版本", format_version),
        ("导入流程", "导出或下载模板 → 填写 → 导入预检 → 处理冲突 → 确认提交"),
        ("动作", "add=新增；update=更新；clear=按 clear_fields 清空；skip=忽略"),
        ("空白规则", "空白单元格默认不修改；需要清空时使用 action=clear，并在 clear_fields 填写英文列名，多个字段用逗号分隔"),
        ("稳定标识", "已有实体优先保留 UUID；没有 UUID 时按外部 ID、正式名称/别名及必要辅助字段匹配"),
        ("来源审计", "source_urls 可用换行或分号分隔；verified_at 使用 ISO 8601；confidence 为 0–1"),
        ("安全边界", "工作簿不执行宏、公式或外部链接；所有写入在预检确认后以单事务提交"),
        ("重复导入", "相同文件哈希重复导入返回既有批次，不重复写入"),
    ];
    if format_version == TEAM_MONTHLY_FORMAT {
        rows.push((
            "球队类型",
            "team_type 使用 club/national/reserve/youth/women/other；导入同时兼容 national_team、国家队、俱乐部、预备队、青年队、女足等常见别名",
        ));
    }
    for (index, (label, value)) in rows.iter().enumerate() {
        let row = (index + 2) as u32;
        worksheet.write_string_with_format(row, 0, *label, &section_format())?;
        worksheet.merge_range(row, 1, row, 6, value, &note_format())?;
    }
    worksheet.set_column_width(0, 18)?;
    worksheet.set_column_range_width(1, 6, 22)?;
    worksheet.set_row_height(0, 28)?;
    Ok(())
}

fn write_header(worksheet: &mut Worksheet, headers: &[&str]) -> Result<(), XlsxError> {
    for (column, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, column as u16, *header, &header_format())?;
        let width = if matches!(
            *header,
            "source_urls" | "notes" | "tactical_summary" | "methodology"
        ) {
            30.0
        } else if header.ends_with("_id") {
            38.0
        } else {
            18.0
        };
        worksheet.set_column_width(column as u16, width)?;
    }
    worksheet.set_freeze_panes(1, 0)?;
    worksheet.autofilter(0, 0, 0, headers.len().saturating_sub(1) as u16)?;
    Ok(())
}

fn add_monthly_validations(worksheet: &mut Worksheet, headers: &[&str]) -> Result<(), XlsxError> {
    for (column, header) in headers.iter().enumerate() {
        let values: Option<&[&str]> = match *header {
            "action" => Some(&["add", "update", "clear", "skip"]),
            "team_type" => Some(&["club", "national", "reserve", "youth", "women", "other"]),
            _ => None,
        };
        if let Some(values) = values {
            let validation = DataValidation::new().allow_list_strings(values)?;
            worksheet.add_data_validation(
                1,
                column as u16,
                MONTHLY_TEMPLATE_LAST_ROW,
                column as u16,
                &validation,
            )?;
        }
    }
    Ok(())
}

fn write_example(worksheet: &mut Worksheet, headers: &[&str]) -> Result<(), XlsxError> {
    for (column, header) in headers.iter().enumerate() {
        let value = match *header {
            "action" => "skip",
            "clear_fields" => "",
            "official_name" | "team_name" => "示例球队",
            "coach_name" => "示例教练",
            "match_name" | "name_value" => "示例球员",
            "confidence" | "data_confidence" => "0.5",
            "team_type" => "club",
            "is_active" => "true",
            "is_primary" | "is_interim" => "false",
            "default_role_code" => "组织核心",
            "valid_from" | "window_start" => "2026-07-01",
            "valid_to" | "window_end" => "",
            "verified_at" | "observed_at" => "2026-07-01T00:00:00Z",
            _ => "",
        };
        worksheet.write_string_with_format(1, column as u16, value, &example_format())?;
    }
    Ok(())
}

fn write_team_rows(
    worksheet: &mut Worksheet,
    entity_type: SpreadsheetEntityType,
    data: &TeamMonthlyWorkbookData,
) -> Result<(), XlsxError> {
    match entity_type {
        SpreadsheetEntityType::Team => {
            for (index, item) in data.teams.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        "".into(),
                        item.team_id.to_string(),
                        item.official_name.clone(),
                        opt(&item.short_name),
                        item.team_type.clone(),
                        opt(&item.country_code),
                        opt(&item.city),
                        item.founded_year.map(|v| v.to_string()).unwrap_or_default(),
                        opt(&item.stadium),
                        item.is_active.to_string(),
                        item.profile_observed_at
                            .map(|v| v.to_rfc3339())
                            .unwrap_or_default(),
                        item.data_confidence.to_string(),
                        metadata_text(&item.metadata, "source_urls"),
                        metadata_text(&item.metadata, "verified_at"),
                        opt(&item.notes),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::TeamName => {
            for (index, item) in data.names.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        "".into(),
                        item.team_id.to_string(),
                        item.official_name.clone(),
                        item.name_value.clone(),
                        opt(&item.language_code),
                        date(item.valid_from),
                        date(item.valid_to),
                        metadata_text(&item.metadata, "source_urls"),
                        metadata_text(&item.metadata, "verified_at"),
                        metadata_text(&item.metadata, "confidence"),
                        metadata_text(&item.metadata, "notes"),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::Coach => {
            for (index, item) in data.coaches.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        "".into(),
                        item.coach_id.to_string(),
                        item.official_name.clone(),
                        opt(&item.nationality_code),
                        item.status.clone(),
                        metadata_text(&item.metadata, "source_urls"),
                        metadata_text(&item.metadata, "verified_at"),
                        metadata_text(&item.metadata, "confidence"),
                        metadata_text(&item.metadata, "notes"),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::TeamCoachPeriod => {
            for (index, item) in data.coach_periods.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        "".into(),
                        item.team_id.to_string(),
                        item.team_name.clone(),
                        item.coach_id.to_string(),
                        item.coach_name.clone(),
                        item.role.clone(),
                        item.valid_from.to_string(),
                        date(item.valid_to),
                        item.is_interim.to_string(),
                        item.confidence.to_string(),
                        metadata_text(&item.metadata, "source_urls"),
                        metadata_text(&item.metadata, "verified_at"),
                        metadata_text(&item.metadata, "notes"),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::FormationUsage => {
            for (index, item) in data.formation_usage.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        item.scope_type.clone(),
                        uuid(item.team_id),
                        opt(&item.team_name),
                        uuid(item.coach_id),
                        opt(&item.coach_name),
                        uuid(item.competition_id),
                        item.formation_id.to_string(),
                        item.formation_code.clone(),
                        item.window_preset.clone(),
                        item.window_start.to_string(),
                        item.window_end.to_string(),
                        item.observed_matches.to_string(),
                        item.usage_count.to_string(),
                        item.smoothed_probability.to_string(),
                        item.confidence.to_string(),
                        item.alpha.to_string(),
                        metadata_text(&item.metadata, "source_urls"),
                        metadata_text(&item.metadata, "verified_at"),
                        item.observed_at.to_rfc3339(),
                        metadata_text(&item.metadata, "notes"),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::TeamTacticalObservation => {
            for (index, item) in data.tactical_observations.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        item.team_id.to_string(),
                        item.team_name.clone(),
                        uuid(item.coach_id),
                        opt(&item.coach_name),
                        item.window_start.to_string(),
                        item.window_end.to_string(),
                        opt(&item.build_up_style),
                        opt(&item.progression_style),
                        opt(&item.attacking_width),
                        opt(&item.pressing_intensity),
                        opt(&item.defensive_block),
                        opt(&item.transition_speed),
                        opt(&item.set_piece_tendency),
                        opt(&item.tactical_summary),
                        item.confidence.to_string(),
                        metadata_text(&item.metadata, "source_urls"),
                        metadata_text(&item.metadata, "verified_at"),
                        item.observed_at.to_rfc3339(),
                        metadata_text(&item.metadata, "notes"),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::TeamAbilityObservation => {
            for (index, item) in data.ability_observations.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        item.team_id.to_string(),
                        item.team_name.clone(),
                        item.observed_at.to_rfc3339(),
                        item.window_start.to_string(),
                        item.window_end.to_string(),
                        number(item.attack_rating),
                        number(item.midfield_rating),
                        number(item.defence_rating),
                        number(item.goalkeeper_rating),
                        number(item.squad_depth_rating),
                        number(item.stability_rating),
                        item.sample_size.to_string(),
                        opt(&item.methodology),
                        item.confidence.to_string(),
                        metadata_text(&item.metadata, "source_urls"),
                        metadata_text(&item.metadata, "verified_at"),
                        metadata_text(&item.metadata, "notes"),
                    ],
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn write_player_rows(
    worksheet: &mut Worksheet,
    entity_type: SpreadsheetEntityType,
    data: &SpreadsheetExportData,
) -> Result<(), XlsxError> {
    match entity_type {
        SpreadsheetEntityType::Player => {
            for (index, item) in data.players.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        "".into(),
                        "".into(),
                        item.player_id.to_string(),
                        item.canonical_name.clone(),
                        date(item.date_of_birth),
                        opt(&item.nationality_code),
                        item.preferred_foot.clone(),
                        item.height_cm.map(|v| v.to_string()).unwrap_or_default(),
                        item.status.clone(),
                        "".into(),
                        "".into(),
                        "".into(),
                        "".into(),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerName => {
            for (index, item) in data.names.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        "".into(),
                        "".into(),
                        item.player_id.to_string(),
                        item.player_name.clone(),
                        date(item.player_birth_date),
                        item.name.clone(),
                        opt(&item.language_code),
                        item.is_primary.to_string(),
                        date(item.valid_from),
                        date(item.valid_to),
                        "".into(),
                        "".into(),
                        "".into(),
                        "".into(),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerTeamPeriod => {
            for (index, item) in data.team_periods.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        "".into(),
                        "".into(),
                        item.player_id.to_string(),
                        item.player_name.clone(),
                        date(item.player_birth_date),
                        "".into(),
                        item.team_id.to_string(),
                        item.team_name.clone(),
                        uuid(item.season_id),
                        item.squad_number.map(|v| v.to_string()).unwrap_or_default(),
                        item.valid_from.to_string(),
                        date(item.valid_to),
                        item.registration_status.clone(),
                        "".into(),
                        "".into(),
                        "".into(),
                        "".into(),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerPosition => {
            for (index, item) in data.positions.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        "".into(),
                        "".into(),
                        item.player_id.to_string(),
                        item.player_name.clone(),
                        date(item.player_birth_date),
                        item.position_code.clone(),
                        item.proficiency.to_string(),
                        opt(&item.default_role_code),
                        item.is_primary.to_string(),
                        date(item.valid_from),
                        date(item.valid_to),
                        "".into(),
                        "".into(),
                        "".into(),
                        "".into(),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerAvailability => {
            for (index, item) in data.availability.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        "".into(),
                        "".into(),
                        item.player_id.to_string(),
                        item.player_name.clone(),
                        date(item.player_birth_date),
                        "".into(),
                        uuid(item.team_id),
                        opt(&item.team_name),
                        uuid(item.competition_id),
                        item.status.clone(),
                        opt(&item.reason),
                        item.confidence.to_string(),
                        item.valid_from.to_rfc3339(),
                        item.valid_to.map(|v| v.to_rfc3339()).unwrap_or_default(),
                        "".into(),
                        "".into(),
                        "".into(),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerAbility => {
            for (index, item) in data.abilities.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        "".into(),
                        item.player_id.to_string(),
                        item.player_name.clone(),
                        date(item.player_birth_date),
                        item.dimension_code.clone(),
                        item.context_type.clone(),
                        uuid(item.context_id),
                        item.value.to_string(),
                        item.confidence.to_string(),
                        item.sample_size.to_string(),
                        item.observed_at.to_rfc3339(),
                        item.effective_from.to_rfc3339(),
                        item.effective_to
                            .map(|v| v.to_rfc3339())
                            .unwrap_or_default(),
                        item.calculation_version.clone(),
                        "".into(),
                        "".into(),
                        "".into(),
                    ],
                )?;
            }
        }
        SpreadsheetEntityType::PlayerDynamicTag => {
            for (index, item) in data.dynamic_tags.iter().enumerate() {
                write_row(
                    worksheet,
                    (index + 1) as u32,
                    &[
                        "skip".into(),
                        "".into(),
                        item.player_id.to_string(),
                        item.player_name.clone(),
                        date(item.player_birth_date),
                        item.tag_code.clone(),
                        item.value.to_string(),
                        opt(&item.label),
                        item.confidence.to_string(),
                        item.observed_at.to_rfc3339(),
                        item.valid_from.to_rfc3339(),
                        item.valid_to.to_rfc3339(),
                        uuid(item.competition_id),
                        opt(&item.position_code),
                        uuid(item.opponent_team_id),
                        item.sample_size.to_string(),
                        item.source_type.clone(),
                        item.calculation_version.clone(),
                        "".into(),
                        "".into(),
                        "".into(),
                    ],
                )?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn add_gap_sheet(workbook: &mut Workbook, gaps: &[MonthlyDataGapRow]) -> Result<(), XlsxError> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("数据缺口")?;
    write_header(worksheet, GAP_HEADERS)?;
    for (index, gap) in gaps.iter().enumerate() {
        write_row(
            worksheet,
            (index + 1) as u32,
            &[
                gap.entity_type.clone(),
                gap.entity_id.to_string(),
                gap.entity_name.clone(),
                gap.missing_field.clone(),
                gap.last_observed_at
                    .map(|v| v.to_rfc3339())
                    .unwrap_or_default(),
                gap.stale_days.map(|v| v.to_string()).unwrap_or_default(),
                gap.priority.clone(),
                gap.recommended_action.clone(),
            ],
        )?;
    }
    Ok(())
}

fn add_team_reference_sheet(
    workbook: &mut Workbook,
    references: &PlayerCatalogReferenceData,
) -> Result<(), XlsxError> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("参考字典")?;
    write_header(worksheet, &["dictionary_type", "code", "name", "extra"])?;
    let mut row = 1;
    for formation in &references.formations {
        write_row(
            worksheet,
            row,
            &[
                "formation".into(),
                formation.id.to_string(),
                formation.code.clone(),
                formation.name.clone(),
            ],
        )?;
        row += 1;
    }
    for team in &references.teams {
        write_row(
            worksheet,
            row,
            &[
                "team".into(),
                team.id.to_string(),
                team.canonical_name.clone(),
                opt(&team.country_code),
            ],
        )?;
        row += 1;
    }
    Ok(())
}

fn add_player_team_reference_sheet(
    workbook: &mut Workbook,
    references: &PlayerCatalogReferenceData,
) -> Result<(), XlsxError> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("参考球队")?;
    write_header(worksheet, &["team_id", "official_name", "country_code"])?;
    for (index, team) in references.teams.iter().enumerate() {
        write_row(
            worksheet,
            (index + 1) as u32,
            &[
                team.id.to_string(),
                team.canonical_name.clone(),
                opt(&team.country_code),
            ],
        )?;
    }
    Ok(())
}

fn add_player_dictionary_sheet(
    workbook: &mut Workbook,
    references: &PlayerCatalogReferenceData,
) -> Result<(), XlsxError> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("参考位置与能力维度")?;
    write_header(worksheet, &["dictionary_type", "code", "name", "extra"])?;
    let mut row = 1;
    for position in &references.positions {
        write_row(
            worksheet,
            row,
            &[
                "position".into(),
                position.code.clone(),
                position.name.clone(),
                position.position_group.clone(),
            ],
        )?;
        row += 1;
    }
    for ability in &references.ability_dimensions {
        write_row(
            worksheet,
            row,
            &[
                "ability".into(),
                ability.code.clone(),
                ability.name.clone(),
                format!("{}–{}", ability.minimum_value, ability.maximum_value),
            ],
        )?;
        row += 1;
    }
    Ok(())
}

fn add_metadata_sheet(
    workbook: &mut Workbook,
    format_version: &str,
    workbook_kind: &str,
) -> Result<(), XlsxError> {
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("元数据")?;
    write_row(
        worksheet,
        0,
        &["format_version".into(), format_version.into()],
    )?;
    write_row(
        worksheet,
        1,
        &["workbook_kind".into(), workbook_kind.into()],
    )?;
    write_row(
        worksheet,
        2,
        &["generated_at".into(), chrono::Utc::now().to_rfc3339()],
    )?;
    worksheet.set_column_width(0, 24)?;
    worksheet.set_column_width(1, 48)?;
    Ok(())
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

fn read_monthly_workbook(
    path: &Path,
    expected_format: &str,
    sheets: &[MonthlySheetSpec],
) -> SpreadsheetResult<SpreadsheetParsedWorkbook> {
    let bytes = fs::read(path)?;
    let source_sha256 = digest(&bytes);
    let mut workbook =
        open_workbook_auto(path).map_err(|error| SpreadsheetError::Read(error.to_string()))?;
    let format_version = read_format(&mut workbook)?;
    if format_version != expected_format {
        return Err(SpreadsheetError::InvalidTemplate(format!(
            "模板版本应为 {expected_format}，实际为 {format_version}"
        )));
    }
    let mut rows = Vec::new();
    for sheet in sheets {
        let Some(entity_type) = sheet.entity_type else {
            continue;
        };
        let Ok(range) = workbook.worksheet_range(sheet.name) else {
            continue;
        };
        let mut iter = range.rows();
        let Some(header_row) = iter.next() else {
            continue;
        };
        let headers = header_row
            .iter()
            .map(cell_text)
            .map(|v| v.trim().to_string())
            .collect::<Vec<_>>();
        for required in sheet.headers {
            if !headers.iter().any(|header| header == required)
                && !matches!(
                    (entity_type, *required),
                    (SpreadsheetEntityType::PlayerPosition, "default_role_code")
                )
            {
                return Err(SpreadsheetError::InvalidTemplate(format!(
                    "工作表 {} 缺少固定列 {}",
                    sheet.name, required
                )));
            }
        }
        for (index, row) in iter.enumerate() {
            let mut values = headers
                .iter()
                .enumerate()
                .map(|(column, header)| {
                    let value = row
                        .get(column)
                        .map(|cell| cell_text_for_header(cell, header))
                        .unwrap_or_default();
                    (header.clone(), Value::String(value.trim().to_string()))
                })
                .collect::<Map<String, Value>>();
            if entity_type == SpreadsheetEntityType::PlayerPosition {
                apply_default_role_alias(&mut values);
            }
            if values
                .values()
                .all(|value| value.as_str().unwrap_or_default().is_empty())
            {
                continue;
            }
            let action = parse_action(values.get("action").and_then(Value::as_str))?;
            rows.push(SpreadsheetRawRow {
                sheet_name: sheet.name.to_string(),
                row_number: (index + 2) as u32,
                entity_type,
                action,
                values: Value::Object(values),
            });
        }
    }
    Ok(SpreadsheetParsedWorkbook {
        format_version,
        source_file_name: path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("monthly-workbook.xlsx")
            .to_string(),
        source_sha256,
        rows,
    })
}

fn read_format<R: std::io::Read + std::io::Seek>(
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

fn parse_action(value: Option<&str>) -> SpreadsheetResult<SpreadsheetAction> {
    match value.unwrap_or("add").trim().to_ascii_lowercase().as_str() {
        "" | "add" => Ok(SpreadsheetAction::Add),
        "update" => Ok(SpreadsheetAction::Update),
        "clear" => Ok(SpreadsheetAction::Clear),
        "skip" => Ok(SpreadsheetAction::Skip),
        other => Err(SpreadsheetError::InvalidTemplate(format!(
            "未知 action：{other}"
        ))),
    }
}

fn write_row(worksheet: &mut Worksheet, row: u32, values: &[String]) -> Result<(), XlsxError> {
    for (column, value) in values.iter().enumerate() {
        worksheet.write_string(row, column as u16, value)?;
    }
    Ok(())
}
fn opt(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}
fn uuid(value: Option<uuid::Uuid>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}
fn date(value: Option<chrono::NaiveDate>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}
fn number(value: Option<f64>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}
fn metadata_text(metadata: &Value, key: &str) -> String {
    match metadata.get(key) {
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("\n"),
        Some(Value::String(value)) => value.clone(),
        Some(value) if !value.is_null() => value.to_string(),
        _ => String::new(),
    }
}
fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
fn cell_text_for_header(value: &Data, header: &str) -> String {
    let date_only = matches!(
        header,
        "birth_date"
            | "effective_from"
            | "effective_to"
            | "valid_from"
            | "valid_to"
            | "window_start"
            | "window_end"
    );
    let date_time = matches!(header, "verified_at" | "observed_at");
    match value {
        Data::DateTime(excel) if date_only => excel
            .as_datetime()
            .map(|value| value.date().to_string())
            .unwrap_or_else(|| value.to_string()),
        Data::DateTime(excel) if date_time => excel
            .as_datetime()
            .map(|value| {
                chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(value, chrono::Utc)
                    .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
            })
            .unwrap_or_else(|| value.to_string()),
        Data::DateTimeIso(value) if date_only => value
            .split_once('T')
            .map(|(date, _)| date.to_string())
            .unwrap_or_else(|| value.clone()),
        _ => cell_text(value),
    }
}

fn cell_text(value: &Data) -> String {
    match value {
        Data::Empty => String::new(),
        Data::DateTimeIso(v) | Data::DurationIso(v) => v.clone(),
        _ => value.to_string(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monthly_templates_round_trip() {
        let references = PlayerCatalogReferenceData::default();
        let team = std::env::temp_dir().join("team-monthly-template.xlsx");
        let player = std::env::temp_dir().join("player-monthly-template.xlsx");
        write_team_monthly_template(&team, &references).expect("write team template");
        write_player_monthly_template(&player, &references).expect("write player template");
        assert_eq!(
            read_team_monthly_workbook(&team)
                .expect("read team")
                .format_version,
            TEAM_MONTHLY_FORMAT
        );
        assert_eq!(
            read_player_monthly_workbook(&player)
                .expect("read player")
                .format_version,
            PLAYER_MONTHLY_FORMAT
        );
        let _ = fs::remove_file(team);
        let _ = fs::remove_file(player);
    }

    #[test]
    fn excel_datetime_cells_are_serialized_by_column_semantics() {
        let value = Data::DateTime(calamine::ExcelDateTime::new(
            46204.5,
            calamine::ExcelDateTimeType::DateTime,
            false,
        ));
        assert_eq!(
            cell_text_for_header(&value, "verified_at"),
            "2026-07-01T12:00:00Z"
        );
        assert_eq!(cell_text_for_header(&value, "window_start"), "2026-07-01");
    }
}
