use crate::{SpreadsheetError, SpreadsheetResult};
use calamine::{open_workbook_auto, Data, DataType, Reader};
use football_domain::{
    MatchLineupExportData, SpreadsheetAction, SpreadsheetEntityType, SpreadsheetParsedWorkbook,
    SpreadsheetRawRow, MATCH_LINEUP_IMPORT_FORMAT, MATCH_LINEUP_IMPORT_LEGACY_FORMAT,
};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

const MATCH_HEADERS: &[&str] = &[
    "action",
    "match_key",
    "match_id",
    "competition_code",
    "competition_id",
    "season_id",
    "stage_id",
    "round_id",
    "kickoff_time",
    "home_team_id",
    "home_team_name",
    "away_team_id",
    "away_team_name",
    "status",
    "venue",
    "neutral_venue",
    "snapshot_type",
    "weather",
    "surface",
    "travel_distance_home_km",
    "travel_distance_away_km",
    "rest_days_home",
    "rest_days_away",
    "schedule_density_home",
    "schedule_density_away",
    "importance",
    "tactical_notes",
    "source_urls",
];
const LINEUP_HEADERS: &[&str] = &[
    "action",
    "lineup_key",
    "lineup_id",
    "match_key",
    "match_id",
    "team_side",
    "team_id",
    "team_name",
    "lineup_type",
    "snapshot_type",
    "formation",
    "formation_id",
    "coach_id",
    "coach_name",
    "captured_at",
    "quality_score",
    "source_urls",
    "notes",
];
const LINEUP_PLAYER_HEADERS: &[&str] = &[
    "action",
    "lineup_key",
    "lineup_id",
    "match_key",
    "match_id",
    "team_side",
    "team_id",
    "team_name",
    "player_id",
    "player_name",
    "birth_date",
    "position_code",
    "role_code",
    "is_starter",
    "shirt_number",
    "expected_minutes",
    "actual_minutes",
    "sequence_no",
    "bench_order",
    "availability_status",
    "starting_probability",
    "membership_override",
    "source_urls",
    "notes",
];
const TAG_HEADERS: &[&str] = &[
    "action",
    "match_key",
    "player_id",
    "player_name",
    "birth_date",
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

struct Spec {
    name: &'static str,
    entity_type: SpreadsheetEntityType,
    headers: &'static [&'static str],
    required: &'static [&'static str],
    example: &'static [&'static str],
}

const SPECS: &[Spec] = &[
    Spec {
        name: "比赛资料",
        entity_type: SpreadsheetEntityType::Match,
        headers: MATCH_HEADERS,
        required: &[
            "match_key",
            "competition_code",
            "kickoff_time",
            "home_team_name",
            "away_team_name",
        ],
        example: &[
            "skip",
            "MATCH-001",
            "",
            "KR-KLEAGUE1",
            "",
            "",
            "",
            "",
            "2026-07-20T19:00:00+09:00",
            "",
            "示例主队",
            "",
            "示例客队",
            "scheduled",
            "",
            "false",
            "T-1h",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "normal",
            "",
            "",
        ],
    },
    Spec {
        name: "阵容信息",
        entity_type: SpreadsheetEntityType::Lineup,
        headers: LINEUP_HEADERS,
        required: &[
            "lineup_key",
            "match_key",
            "team_side",
            "lineup_type",
            "snapshot_type",
            "formation_id",
            "captured_at",
        ],
        example: &[
            "skip",
            "HOME-LINEUP",
            "",
            "MATCH-001",
            "",
            "home",
            "",
            "示例主队",
            "expected",
            "T-6h",
            "4-2-3-1",
            "",
            "",
            "",
            "2026-07-20T13:00:00+09:00",
            "0.8",
            "",
            "",
        ],
    },
    Spec {
        name: "阵容球员",
        entity_type: SpreadsheetEntityType::LineupPlayer,
        headers: LINEUP_PLAYER_HEADERS,
        required: &[
            "lineup_key",
            "match_key",
            "team_side",
            "player_name",
            "is_starter",
            "sequence_no",
        ],
        example: &[
            "skip",
            "HOME-LINEUP",
            "",
            "MATCH-001",
            "",
            "home",
            "",
            "示例主队",
            "",
            "示例球员",
            "2000-01-01",
            "ST",
            "forward",
            "true",
            "9",
            "90",
            "",
            "1",
            "",
            "available",
            "0.9",
            "false",
            "",
            "",
        ],
    },
    Spec {
        name: "动态标签",
        entity_type: SpreadsheetEntityType::PlayerDynamicTag,
        headers: TAG_HEADERS,
        required: &[
            "player_name",
            "tag_code",
            "tag_value",
            "observed_at",
            "valid_from",
            "valid_to",
            "calculation_version",
        ],
        example: &[
            "skip",
            "MATCH-001",
            "",
            "示例球员",
            "2000-01-01",
            "match_readiness",
            "0.92",
            "状态良好",
            "0.8",
            "2026-07-20T10:00:00+09:00",
            "2026-07-20T10:00:00+09:00",
            "2026-07-27T10:00:00+09:00",
            "",
            "ST",
            "",
            "5",
            "lineup_import",
            "manual-v1",
        ],
    },
];

pub fn write_match_lineup_template(
    output_path: &Path,
    data: &MatchLineupExportData,
) -> SpreadsheetResult<()> {
    write_match_lineup_workbook(output_path, data, true)
}

pub fn write_match_lineup_export(
    output_path: &Path,
    data: &MatchLineupExportData,
) -> SpreadsheetResult<()> {
    write_match_lineup_workbook(output_path, data, false)
}

fn write_match_lineup_workbook(
    output_path: &Path,
    data: &MatchLineupExportData,
    include_examples: bool,
) -> SpreadsheetResult<()> {
    let mut workbook = Workbook::new();
    add_instructions(&mut workbook)?;
    add_field_definitions(&mut workbook)?;
    add_enums(&mut workbook, data)?;
    add_reference_sheets(&mut workbook, data)?;
    add_metadata(&mut workbook)?;
    for spec in SPECS {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(spec.name)?;
        write_header(worksheet, spec)?;
        if include_examples {
            write_values(worksheet, 1, spec.example)?;
        } else {
            write_export_rows(worksheet, spec.entity_type, data)?;
        }
    }
    workbook.save(output_path)?;
    Ok(())
}

pub fn read_match_lineup_workbook(path: &Path) -> SpreadsheetResult<SpreadsheetParsedWorkbook> {
    let bytes = fs::read(path)?;
    let source_sha256 = hex_digest(&bytes);
    let mut workbook =
        open_workbook_auto(path).map_err(|error| SpreadsheetError::Read(error.to_string()))?;
    let format_version = read_format_version(&mut workbook)?;
    if !matches!(
        format_version.as_str(),
        MATCH_LINEUP_IMPORT_FORMAT | MATCH_LINEUP_IMPORT_LEGACY_FORMAT
    ) {
        return Err(SpreadsheetError::InvalidTemplate(format!(
            "模板版本应为 {MATCH_LINEUP_IMPORT_FORMAT} 或 {MATCH_LINEUP_IMPORT_LEGACY_FORMAT}，实际为 {format_version}"
        )));
    }
    let mut rows = Vec::new();
    for spec in SPECS {
        let Ok(range) = workbook.worksheet_range(spec.name) else {
            continue;
        };
        let mut iterator = range.rows();
        let Some(header_row) = iterator.next() else {
            continue;
        };
        let headers = header_row.iter().map(cell_text).collect::<Vec<_>>();
        validate_headers(spec, &headers, &format_version)?;
        for (offset, row) in iterator.enumerate() {
            let values = headers
                .iter()
                .enumerate()
                .filter(|(_, header)| !header.trim().is_empty())
                .map(|(index, header)| {
                    (
                        header.clone(),
                        Value::String(
                            row.get(index)
                                .map(|cell| row_cell_text(cell, header))
                                .unwrap_or_default(),
                        ),
                    )
                })
                .collect::<Map<_, _>>();
            let mut values = values;
            if format_version == MATCH_LINEUP_IMPORT_LEGACY_FORMAT
                && spec.entity_type == SpreadsheetEntityType::Lineup
            {
                values
                    .entry("snapshot_type".to_string())
                    .or_insert_with(|| Value::String("T-1h".to_string()));
            }
            if values
                .values()
                .all(|value| value.as_str().unwrap_or("").trim().is_empty())
            {
                continue;
            }
            let action = match values
                .get("action")
                .and_then(Value::as_str)
                .unwrap_or("skip")
                .trim()
                .to_lowercase()
                .as_str()
            {
                "add" => SpreadsheetAction::Add,
                "update" => SpreadsheetAction::Update,
                "skip" | "" => SpreadsheetAction::Skip,
                other => {
                    return Err(SpreadsheetError::InvalidTemplate(format!(
                        "{} 第 {} 行 action 无效：{other}",
                        spec.name,
                        offset + 2
                    )))
                }
            };
            rows.push(SpreadsheetRawRow {
                sheet_name: spec.name.to_string(),
                row_number: (offset + 2) as u32,
                entity_type: spec.entity_type,
                action,
                values: Value::Object(values),
            });
        }
    }
    Ok(SpreadsheetParsedWorkbook {
        format_version,
        source_file_name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("match.xlsx")
            .to_string(),
        source_sha256,
        rows,
    })
}

fn add_instructions(workbook: &mut Workbook) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("使用说明")?;
    sheet.merge_range(0, 0, 0, 5, "比赛与阵容推演输入", &title_format())?;
    let notes = [
        ("模板版本", MATCH_LINEUP_IMPORT_FORMAT),
        ("推荐流程", "软件导出 → 交给 ChatGPT 整理 → 软件预检 → 处理冲突 → 确认导入 → 模型推演"),
        ("比赛资料", "一行一场比赛。已有比赛保留 match_id；新比赛使用稳定 match_key。"),
        ("阵容", "阵容信息定义 T-N、T-24h、T-6h、T-1h 四个数据窗口版本，阵容球员通过 lineup_key 关联；同一时点同一类型的新版本会 supersede 旧版本，但历史仍可回看。"),
        ("战术角色", "role_code 留空时，导入会按本场 position_code 优先继承球员位置档案的默认战术角色；填写不同内容时仅覆盖本场，不回写球员长期档案。"),
        ("模型门禁", "正式模型只读取当前时点之前、状态 active、完整 11 名首发并绑定 formation_id 的预计或确认阵容。"),
        ("履历例外", "球员在开球时点不属于该队会阻断模型资格；确认是临时注册、国家队征召或数据修正时，将 membership_override 设为 true。"),
        ("动态标签", "短期状态必须设置 valid_to，过期后不再参与后续比赛计算。"),
        ("安全规则", "Excel 永不直接写库，必须通过预检和 PostgreSQL 事务确认。"),
    ];
    for (index, (label, value)) in notes.iter().enumerate() {
        let row = (index + 2) as u32;
        sheet.write_string_with_format(row, 0, *label, &section_format())?;
        sheet.merge_range(row, 1, row, 5, value, &note_format())?;
    }
    sheet.set_column_width(0, 18)?;
    sheet.set_column_width(1, 28)?;
    sheet.set_column_width(2, 24)?;
    sheet.set_column_width(3, 24)?;
    sheet.set_column_width(4, 24)?;
    sheet.set_column_width(5, 24)?;
    Ok(())
}

fn add_field_definitions(workbook: &mut Workbook) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("字段说明")?;
    for (col, label) in ["工作表", "字段", "必填", "说明"].iter().enumerate() {
        sheet.write_string_with_format(0, col as u16, *label, &header_format())?;
    }
    let mut row = 1u32;
    for spec in SPECS {
        for field in spec.headers {
            sheet.write_string(row, 0, spec.name)?;
            sheet.write_string(row, 1, *field)?;
            sheet.write_string(
                row,
                2,
                if spec.required.contains(field) {
                    "是"
                } else {
                    "否"
                },
            )?;
            sheet.write_string(row, 3, field_description(field))?;
            row += 1;
        }
    }
    sheet.set_column_width(0, 16)?;
    sheet.set_column_width(1, 28)?;
    sheet.set_column_width(2, 10)?;
    sheet.set_column_width(3, 62)?;
    sheet.autofilter(0, 0, row.saturating_sub(1), 3)?;
    sheet.set_freeze_panes(1, 0)?;
    Ok(())
}

fn add_enums(
    workbook: &mut Workbook,
    data: &MatchLineupExportData,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("枚举值")?;
    let fixed: &[(&str, &[&str])] = &[
        ("action", &["add", "update", "skip"]),
        ("team_side", &["home", "away"]),
        ("lineup_type", &["expected", "confirmed", "actual"]),
        ("snapshot_type", &["T-N", "T-24h", "T-6h", "T-1h"]),
        (
            "match_status",
            &["scheduled", "live", "finished", "postponed", "cancelled"],
        ),
        (
            "availability_status",
            &[
                "available",
                "doubtful",
                "injured",
                "suspended",
                "rested",
                "returning",
                "unknown",
            ],
        ),
        (
            "source_type",
            &[
                "manual",
                "provider",
                "lineup_import",
                "ai_analysis",
                "match_review",
                "calculation",
            ],
        ),
    ];
    for (column, (label, values)) in fixed.iter().enumerate() {
        sheet.write_string_with_format(0, column as u16, *label, &header_format())?;
        for (row, value) in values.iter().enumerate() {
            sheet.write_string((row + 1) as u32, column as u16, *value)?;
        }
    }
    let columns = [
        (
            7u16,
            "competition_code",
            data.competitions
                .iter()
                .map(|value| value.code.clone())
                .collect::<Vec<_>>(),
        ),
        (
            8u16,
            "position_code",
            data.positions
                .iter()
                .map(|value| value.code.clone())
                .collect::<Vec<_>>(),
        ),
        (
            9u16,
            "dynamic_tag_code",
            data.dynamic_tag_definitions
                .iter()
                .map(|value| value.code.clone())
                .collect::<Vec<_>>(),
        ),
    ];
    for (column, label, values) in columns {
        sheet.write_string_with_format(0, column, label, &header_format())?;
        for (row, value) in values.iter().enumerate() {
            sheet.write_string((row + 1) as u32, column, value)?;
        }
    }
    for column in 0..=9u16 {
        sheet.set_column_width(column, 22)?;
    }
    sheet.set_freeze_panes(1, 0)?;
    Ok(())
}

fn add_reference_sheets(
    workbook: &mut Workbook,
    data: &MatchLineupExportData,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let team_sheet = workbook.add_worksheet();
    team_sheet.set_name("参考球队")?;
    for (column, label) in ["team_id", "team_name", "country_code"].iter().enumerate() {
        team_sheet.write_string_with_format(0, column as u16, *label, &header_format())?;
    }
    for (index, team) in data.teams.iter().enumerate() {
        let row = (index + 1) as u32;
        team_sheet.write_string(row, 0, team.id.to_string())?;
        team_sheet.write_string(row, 1, &team.canonical_name)?;
        team_sheet.write_string(row, 2, team.country_code.as_deref().unwrap_or(""))?;
    }
    team_sheet.set_column_width(0, 38)?;
    team_sheet.set_column_width(1, 26)?;
    team_sheet.set_column_width(2, 16)?;
    team_sheet.set_freeze_panes(1, 0)?;

    let player_sheet = workbook.add_worksheet();
    player_sheet.set_name("参考球员")?;
    for (column, label) in [
        "player_id",
        "player_name",
        "birth_date",
        "current_team_id",
        "current_team_name",
        "primary_position_code",
        "primary_role_code",
        "availability_status",
    ]
    .iter()
    .enumerate()
    {
        player_sheet.write_string_with_format(0, column as u16, *label, &header_format())?;
    }
    for (index, player) in data.players.iter().enumerate() {
        let row = (index + 1) as u32;
        let values = [
            player.player_id.to_string(),
            player.canonical_name.clone(),
            player
                .date_of_birth
                .map(|value| value.to_string())
                .unwrap_or_default(),
            player
                .current_team_id
                .map(|value| value.to_string())
                .unwrap_or_default(),
            player.current_team_name.clone().unwrap_or_default(),
            player.primary_position_code.clone().unwrap_or_default(),
            player.primary_role_code.clone().unwrap_or_default(),
            player
                .availability_status
                .map(|value| value.as_str().to_string())
                .unwrap_or_default(),
        ];
        write_owned(player_sheet, row, &values)?;
    }
    for column in 0..=7u16 {
        player_sheet.set_column_width(column, if matches!(column, 0 | 3) { 38.0 } else { 24.0 })?;
    }
    player_sheet.set_freeze_panes(1, 0)?;

    let formation_sheet = workbook.add_worksheet();
    formation_sheet.set_name("参考阵型")?;
    for (column, label) in [
        "formation_id",
        "formation_code",
        "formation_name",
        "is_builtin",
    ]
    .iter()
    .enumerate()
    {
        formation_sheet.write_string_with_format(0, column as u16, *label, &header_format())?;
    }
    for (index, formation) in data.formations.iter().enumerate() {
        let row = (index + 1) as u32;
        formation_sheet.write_string(row, 0, formation.id.to_string())?;
        formation_sheet.write_string(row, 1, &formation.code)?;
        formation_sheet.write_string(row, 2, &formation.name)?;
        formation_sheet.write_boolean(row, 3, formation.is_builtin)?;
    }
    formation_sheet.set_column_width(0, 38)?;
    formation_sheet.set_column_width(1, 18)?;
    formation_sheet.set_column_width(2, 24)?;
    formation_sheet.set_column_width(3, 12)?;
    formation_sheet.set_freeze_panes(1, 0)?;

    let coach_sheet = workbook.add_worksheet();
    coach_sheet.set_name("参考教练")?;
    for (column, label) in [
        "coach_id",
        "coach_name",
        "nationality_code",
        "current_team_name",
    ]
    .iter()
    .enumerate()
    {
        coach_sheet.write_string_with_format(0, column as u16, *label, &header_format())?;
    }
    for (index, coach) in data.coaches.iter().enumerate() {
        let row = (index + 1) as u32;
        coach_sheet.write_string(row, 0, coach.id.to_string())?;
        coach_sheet.write_string(row, 1, &coach.canonical_name)?;
        coach_sheet.write_string(row, 2, coach.nationality_code.as_deref().unwrap_or(""))?;
        coach_sheet.write_string(row, 3, coach.current_team_name.as_deref().unwrap_or(""))?;
    }
    coach_sheet.set_column_width(0, 38)?;
    coach_sheet.set_column_width(1, 24)?;
    coach_sheet.set_column_width(2, 18)?;
    coach_sheet.set_column_width(3, 24)?;
    coach_sheet.set_freeze_panes(1, 0)?;
    Ok(())
}

fn add_metadata(workbook: &mut Workbook) -> Result<(), rust_xlsxwriter::XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("元数据")?;
    sheet.write_string(0, 0, "format_version")?;
    sheet.write_string(0, 1, MATCH_LINEUP_IMPORT_FORMAT)?;
    sheet.write_string(1, 0, "generated_by")?;
    sheet.write_string(1, 1, "football-match-model-platform")?;
    sheet.set_hidden(true);
    Ok(())
}

fn write_header(sheet: &mut Worksheet, spec: &Spec) -> Result<(), rust_xlsxwriter::XlsxError> {
    for (column, header) in spec.headers.iter().enumerate() {
        let format = if spec.required.contains(header) {
            required_header_format()
        } else {
            header_format()
        };
        sheet.write_string_with_format(0, column as u16, *header, &format)?;
        sheet.set_column_width(column as u16, header_width(header))?;
    }
    sheet.autofilter(0, 0, 99_999, (spec.headers.len() - 1) as u16)?;
    sheet.set_freeze_panes(1, 0)?;
    Ok(())
}

fn write_export_rows(
    sheet: &mut Worksheet,
    entity: SpreadsheetEntityType,
    data: &MatchLineupExportData,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    match entity {
        SpreadsheetEntityType::Match => {
            if let Some(item) = &data.selected_match {
                let values = vec![
                    "update".to_string(),
                    item.external_key.clone(),
                    item.id.to_string(),
                    data.competitions
                        .iter()
                        .find(|competition| Some(competition.id) == item.competition_id)
                        .map(|value| value.code.clone())
                        .unwrap_or_default(),
                    item.competition_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    item.season_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    item.stage_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    item.round_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    item.kickoff_time.to_rfc3339(),
                    item.home_team_id.to_string(),
                    item.home_team_name.clone(),
                    item.away_team_id.to_string(),
                    item.away_team_name.clone(),
                    item.status.as_str().to_string(),
                    item.venue.clone().unwrap_or_default(),
                    "false".to_string(),
                    "T-1h".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "normal".to_string(),
                    "".to_string(),
                    "".to_string(),
                ];
                write_owned(sheet, 1, &values)?;
            }
        }
        SpreadsheetEntityType::Lineup => {
            for (index, lineup) in data.lineups.iter().enumerate() {
                let team_side = data
                    .selected_match
                    .as_ref()
                    .map(|item| {
                        if item.home_team_id == lineup.team_id {
                            "home"
                        } else {
                            "away"
                        }
                    })
                    .unwrap_or("");
                let team_name = lineup.team_name.clone();
                let values = vec![
                    "skip".to_string(),
                    format!("LINEUP-{}", index + 1),
                    lineup.id.to_string(),
                    lineup.match_id.to_string(),
                    lineup.match_id.to_string(),
                    team_side.to_string(),
                    lineup.team_id.to_string(),
                    team_name,
                    lineup.lineup_type.as_str().to_string(),
                    lineup.snapshot_type.clone(),
                    lineup.formation.clone().unwrap_or_default(),
                    lineup
                        .formation_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    lineup
                        .coach_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    lineup.coach_name.clone().unwrap_or_default(),
                    lineup.captured_at.to_rfc3339(),
                    lineup
                        .quality_score
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    lineup.source_urls.join(";"),
                    "".to_string(),
                ];
                write_owned(sheet, (index + 1) as u32, &values)?;
            }
        }
        SpreadsheetEntityType::LineupPlayer => {
            let mut row = 1u32;
            for (lineup_index, lineup) in data.lineups.iter().enumerate() {
                let team_side = data
                    .selected_match
                    .as_ref()
                    .map(|item| {
                        if item.home_team_id == lineup.team_id {
                            "home"
                        } else {
                            "away"
                        }
                    })
                    .unwrap_or("");
                let team_name = lineup.team_name.clone();
                for player in &lineup.players {
                    let player_ref = data
                        .players
                        .iter()
                        .find(|item| item.player_id == player.player_id);
                    let values = vec![
                        "skip".to_string(),
                        format!("LINEUP-{}", lineup_index + 1),
                        lineup.id.to_string(),
                        lineup.match_id.to_string(),
                        lineup.match_id.to_string(),
                        team_side.to_string(),
                        lineup.team_id.to_string(),
                        team_name.clone(),
                        player.player_id.to_string(),
                        player.player_name.clone(),
                        player_ref
                            .and_then(|item| item.date_of_birth)
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        player.position_code.clone().unwrap_or_default(),
                        player.role_code.clone().unwrap_or_default(),
                        player.is_starter.to_string(),
                        player
                            .shirt_number
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        player
                            .expected_minutes
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        player
                            .actual_minutes
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        player.sequence_no.to_string(),
                        player
                            .bench_order
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        player
                            .availability_status
                            .map(|value| value.as_str().to_string())
                            .unwrap_or_default(),
                        player
                            .starting_probability
                            .map(|value| value.to_string())
                            .unwrap_or_default(),
                        player.membership_override.to_string(),
                        player.source_urls.join(";"),
                        player.validation_warning.clone().unwrap_or_default(),
                    ];
                    write_owned(sheet, row, &values)?;
                    row += 1;
                }
            }
        }
        SpreadsheetEntityType::PlayerDynamicTag => {
            for (index, tag) in data.dynamic_tags.iter().enumerate() {
                let values = vec![
                    "skip".to_string(),
                    data.selected_match
                        .as_ref()
                        .map(|item| item.external_key.clone())
                        .unwrap_or_default(),
                    tag.player_id.to_string(),
                    data.players
                        .iter()
                        .find(|item| item.player_id == tag.player_id)
                        .map(|item| item.canonical_name.clone())
                        .unwrap_or_default(),
                    data.players
                        .iter()
                        .find(|item| item.player_id == tag.player_id)
                        .and_then(|item| item.date_of_birth)
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    tag.tag_code.clone(),
                    tag.value.to_string(),
                    tag.label.clone().unwrap_or_default(),
                    tag.confidence.to_string(),
                    tag.observed_at.to_rfc3339(),
                    tag.valid_from.to_rfc3339(),
                    tag.valid_to.to_rfc3339(),
                    tag.competition_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    tag.position_code.clone().unwrap_or_default(),
                    tag.opponent_team_id
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    tag.sample_size.to_string(),
                    tag.source_type.clone(),
                    tag.calculation_version.clone(),
                ];
                write_owned(sheet, (index + 1) as u32, &values)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn write_values(
    sheet: &mut Worksheet,
    row: u32,
    values: &[&str],
) -> Result<(), rust_xlsxwriter::XlsxError> {
    for (column, value) in values.iter().enumerate() {
        sheet.write_string(row, column as u16, *value)?;
    }
    Ok(())
}
fn write_owned(
    sheet: &mut Worksheet,
    row: u32,
    values: &[String],
) -> Result<(), rust_xlsxwriter::XlsxError> {
    for (column, value) in values.iter().enumerate() {
        sheet.write_string(row, column as u16, value)?;
    }
    Ok(())
}

fn validate_headers(
    spec: &Spec,
    headers: &[String],
    format_version: &str,
) -> SpreadsheetResult<()> {
    let set = headers
        .iter()
        .map(|value| value.trim())
        .collect::<std::collections::HashSet<_>>();
    let legacy_required: &[&str] = match spec.entity_type {
        SpreadsheetEntityType::Lineup => &[
            "lineup_key",
            "match_key",
            "team_side",
            "lineup_type",
            "captured_at",
        ],
        _ => spec.required,
    };
    let required_fields = if format_version == MATCH_LINEUP_IMPORT_LEGACY_FORMAT {
        legacy_required
    } else {
        spec.required
    };
    for required in required_fields.iter().chain([&"action"]) {
        if !set.contains(required) {
            return Err(SpreadsheetError::InvalidTemplate(format!(
                "{} 缺少必需列：{required}",
                spec.name
            )));
        }
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
    if value.is_empty() {
        return Err(SpreadsheetError::InvalidTemplate(
            "缺少元数据工作表".to_string(),
        ));
    }
    Ok(value)
}

fn row_cell_text(cell: &Data, header: &str) -> String {
    if header == "birth_date" {
        if let Some(date) = cell.as_date() {
            return date.format("%Y-%m-%d").to_string();
        }
    }
    if matches!(
        header,
        "kickoff_time" | "captured_at" | "observed_at" | "valid_from" | "valid_to"
    ) {
        if let Some(date_time) = cell.as_datetime() {
            return format!("{}Z", date_time.format("%Y-%m-%dT%H:%M:%S"));
        }
    }
    cell_text(cell)
}

fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.trim().to_string(),
        Data::Float(value) => {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                value.to_string()
            }
        }
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
        Data::DateTime(_) => cell.as_string().unwrap_or_default(),
        Data::Error(value) => format!("{value:?}"),
    }
}
fn hex_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn field_description(field: &str) -> &'static str {
    match field {
        "action" => "add 新增、update 更新、skip 跳过",
        "match_key" => "工作簿和 AI 交换包中的稳定比赛键",
        "lineup_key" => "当前工作簿中关联阵容和阵容球员的临时键",
        "competition_code" => "软件内置或自定义赛事代码",
        "kickoff_time" | "captured_at" | "observed_at" | "valid_from" | "valid_to" => {
            "ISO 8601 时间，必须带时区"
        }
        "team_side" => "home 或 away",
        "lineup_type" => "expected、confirmed 或 actual",
        "snapshot_type" => "T-N、T-24h、T-6h 或 T-1h；T-90m 仅保留历史数据",
        "formation_id" => "内置阵型 UUID；正式模型输入必须填写",
        "role_code" => "本场战术角色覆盖；留空时按球员位置档案自动继承默认角色",
        "bench_order" => "替补顺序，1–99，仅替补填写",
        "membership_override" => "履历不一致时由用户明确确认 true",
        "source_urls" => "多个来源网址使用换行或分号分隔",
        "is_starter" | "neutral_venue" => "true 或 false",
        "starting_probability" | "quality_score" | "confidence" => "0–1 小数",
        "tag_value" => "按标签定义的允许范围填写",
        _ => "结构化推演输入字段",
    }
}
fn header_width(header: &str) -> f64 {
    match header {
        "match_id" | "competition_id" | "season_id" | "stage_id" | "round_id" | "team_id"
        | "player_id" | "lineup_id" | "opponent_team_id" | "formation_id" | "coach_id" => 38.0,
        "kickoff_time" | "captured_at" | "observed_at" | "valid_from" | "valid_to" => 25.0,
        "home_team_name" | "away_team_name" | "team_name" | "player_name" | "tactical_notes"
        | "notes" => 24.0,
        _ => 17.0,
    }
}
fn title_format() -> Format {
    Format::new()
        .set_background_color(Color::RGB(0x163A5F))
        .set_font_color(Color::White)
        .set_bold()
        .set_font_size(16)
        .set_align(FormatAlign::Left)
        .set_border(FormatBorder::Thin)
}
fn header_format() -> Format {
    Format::new()
        .set_background_color(Color::RGB(0x2563A6))
        .set_font_color(Color::White)
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
}
fn required_header_format() -> Format {
    Format::new()
        .set_background_color(Color::RGB(0xB45309))
        .set_font_color(Color::White)
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
}
fn section_format() -> Format {
    Format::new()
        .set_background_color(Color::RGB(0xDCEAF7))
        .set_font_color(Color::RGB(0x163A5F))
        .set_bold()
        .set_border(FormatBorder::Thin)
}
fn note_format() -> Format {
    Format::new()
        .set_background_color(Color::RGB(0xF3F6FA))
        .set_font_color(Color::RGB(0x475569))
        .set_text_wrap()
        .set_border(FormatBorder::Thin)
}
