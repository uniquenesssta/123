use crate::{SpreadsheetError, SpreadsheetResult};
use calamine::{open_workbook_auto, Data, Reader};
use football_domain::{
    PlayerCatalogReferenceData, SpreadsheetAction, SpreadsheetEntityType,
    SpreadsheetParsedWorkbook, SpreadsheetRawRow, TEAM_PACKAGE_FORMAT,
};
use rust_xlsxwriter::{
    Color, DataValidation, Format, FormatAlign, FormatBorder, Workbook, Worksheet, XlsxError,
};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, path::Path};

const LAST_DATA_ROW: u32 = 99_999;
const KEY_ROW: usize = 2;
const FIRST_DATA_ROW: usize = 3;

const TEAM_NAME_KEYS: &[&str] = &[
    "action",
    "clear_fields",
    "team_id",
    "team_name",
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

const TEAM_KEYS: &[&str] = &[
    "action",
    "clear_fields",
    "team_id",
    "team_name",
    "short_name",
    "team_type",
    "country_code",
    "city",
    "founded_year",
    "stadium",
    "is_active",
    "profile_observed_at",
    "data_confidence",
    "window_start",
    "window_end",
    "team_attack_rating",
    "team_midfield_rating",
    "team_defence_rating",
    "team_goalkeeper_rating",
    "team_squad_depth_rating",
    "team_stability_rating",
    "team_sample_size",
    "team_methodology",
    "build_up_style",
    "progression_style",
    "attacking_width",
    "pressing_intensity",
    "defensive_block",
    "transition_speed",
    "set_piece_tendency",
    "tactical_summary",
    "observation_confidence",
    "observed_at",
    "source_urls",
    "verified_at",
    "notes",
];

const PLAYER_NAME_KEYS: &[&str] = &[
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

const PLAYER_KEYS: &[&str] = &[
    "action",
    "clear_fields",
    "player_key",
    "player_id",
    "official_name",
    "english_name",
    "birth_date",
    "nationality_code",
    "preferred_foot",
    "height_cm",
    "player_status",
    "team_key",
    "team_id",
    "team_name",
    "squad_number",
    "registration_status",
    "roster_valid_from",
    "roster_valid_to",
    "club_team_key",
    "club_team_id",
    "club_team_name",
    "club_country_code",
    "club_squad_number",
    "club_registration_status",
    "club_valid_from",
    "club_valid_to",
    "position_code",
    "position_proficiency",
    "position_is_primary",
    "default_role_code",
    "availability_status",
    "availability_reason",
    "availability_valid_from",
    "availability_valid_to",
    "ability_attack",
    "ability_defence",
    "ability_creation",
    "ability_progression",
    "ability_finishing",
    "ability_physical",
    "ability_stamina",
    "ability_stability",
    "ability_discipline",
    "ability_tactical_execution",
    "ability_versatility",
    "ability_substitute_impact",
    "ability_confidence",
    "ability_sample_size",
    "ability_observed_at",
    "ability_effective_from",
    "ability_effective_to",
    "ability_calculation_version",
    "tag_match_readiness",
    "tag_form_multiplier",
    "tag_fatigue_multiplier",
    "tag_position_fit",
    "tag_tactical_fit",
    "tag_chemistry_fit",
    "tag_starting_probability",
    "tag_expected_minutes_share",
    "tag_realization_multiplier",
    "tag_volatility",
    "tag_confidence",
    "tag_sample_size",
    "tag_observed_at",
    "tag_valid_from",
    "tag_valid_to",
    "tag_source_type",
    "tag_calculation_version",
    "source_urls",
    "verified_at",
    "notes",
];

const COACH_KEYS: &[&str] = &[
    "action",
    "clear_fields",
    "coach_id",
    "coach_name",
    "nationality_code",
    "coach_status",
    "team_id",
    "team_name",
    "role",
    "valid_from",
    "valid_to",
    "is_interim",
    "formation_id",
    "formation_code",
    "scope_type",
    "window_preset",
    "window_start",
    "window_end",
    "observed_matches",
    "usage_count",
    "formation_familiarity",
    "confidence",
    "alpha",
    "observed_at",
    "source_urls",
    "verified_at",
    "notes",
];

const ABILITY_COLUMNS: &[(&str, &str)] = &[
    ("ability_attack", "attack"),
    ("ability_defence", "defence"),
    ("ability_creation", "creation"),
    ("ability_progression", "progression"),
    ("ability_finishing", "finishing"),
    ("ability_physical", "physical"),
    ("ability_stamina", "stamina"),
    ("ability_stability", "stability"),
    ("ability_discipline", "discipline"),
    ("ability_tactical_execution", "tactical_execution"),
    ("ability_versatility", "versatility"),
    ("ability_substitute_impact", "substitute_impact"),
];

const TAG_COLUMNS: &[(&str, &str)] = &[
    ("tag_match_readiness", "match_readiness"),
    ("tag_form_multiplier", "form_multiplier"),
    ("tag_fatigue_multiplier", "fatigue_multiplier"),
    ("tag_position_fit", "position_fit"),
    ("tag_tactical_fit", "tactical_fit"),
    ("tag_chemistry_fit", "chemistry_fit"),
    ("tag_starting_probability", "starting_probability"),
    ("tag_expected_minutes_share", "expected_minutes_share"),
    ("tag_realization_multiplier", "realization_multiplier"),
    ("tag_volatility", "volatility"),
];

pub fn write_team_package_template(
    output_path: &Path,
    references: &PlayerCatalogReferenceData,
) -> SpreadsheetResult<()> {
    let mut workbook = Workbook::new();
    write_instruction_sheet(&mut workbook)?;
    write_team_sheet(&mut workbook)?;
    write_team_name_sheet(&mut workbook)?;
    write_player_sheet(&mut workbook)?;
    write_player_name_sheet(&mut workbook)?;
    write_coach_sheet(&mut workbook)?;
    write_dictionary_sheet(&mut workbook, references)?;
    workbook.save(output_path)?;
    Ok(())
}

pub fn read_team_package_workbook(path: &Path) -> SpreadsheetResult<SpreadsheetParsedWorkbook> {
    let bytes = fs::read(path)?;
    let source_sha256 = digest(&bytes);
    let mut workbook =
        open_workbook_auto(path).map_err(|error| SpreadsheetError::Read(error.to_string()))?;
    let format = workbook
        .worksheet_range("说明与校验")
        .map_err(|error| SpreadsheetError::Read(error.to_string()))?
        .get((1, 1))
        .map(cell_text)
        .unwrap_or_default();
    if format.trim() != TEAM_PACKAGE_FORMAT {
        return Err(SpreadsheetError::InvalidTemplate(format!(
            "球队完整资料包版本应为 {TEAM_PACKAGE_FORMAT}，实际为 {}",
            format.trim()
        )));
    }

    let mut rows = Vec::new();
    parse_team_sheet(&mut workbook, &mut rows)?;
    if workbook.sheet_names().iter().any(|name| name == "球队名称") {
        parse_team_name_sheet(&mut workbook, &mut rows)?;
    }
    parse_player_sheet(&mut workbook, &mut rows)?;
    if workbook.sheet_names().iter().any(|name| name == "球员名称") {
        parse_player_name_sheet(&mut workbook, &mut rows)?;
    }
    parse_coach_sheet(&mut workbook, &mut rows)?;

    Ok(SpreadsheetParsedWorkbook {
        format_version: TEAM_PACKAGE_FORMAT.to_string(),
        source_file_name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("team-package.xlsx")
            .to_string(),
        source_sha256,
        rows,
    })
}

fn write_instruction_sheet(workbook: &mut Workbook) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("说明与校验")?;
    sheet.merge_range(
        0,
        0,
        0,
        6,
        "球队完整资料包 · P4 输入工作簿",
        &title_format(),
    )?;
    let rows = [
        ("格式版本", TEAM_PACKAGE_FORMAT),
        ("核心用途", "一次导入一支或多支球队、全部球员、主教练/历史教练、阵型分布、基础评分和动态评分，导入后直接进入现有 P4 数据链路。"),
        ("可见工作表", "球队总览、球队名称、球员与评分、球员名称、教练与阵型；名称表用于同时维护中文名、英文名、官方名、主显示名和历史名称。"),
        ("填写规则", "每张业务表第 2 行是中文字段，第 3 行是不可修改的固定字段键；action 留空或填写 upsert 时，预检会按数据库匹配结果自动转为 add/update；action=clear 时才按 clear_fields 清空。球队名称/球员名称可按 language_code 填 zh-CN、en、fr 等；is_primary=true 会同步为主显示名。"),
        ("评分边界", "基础能力可填写 0–100，或沿用 0–10000 评分（系统自动除以 100）；动态标签按字段字典范围填写。基础能力和动态标签分开保存，动态值不会覆盖长期能力。"),
        ("P4 链路", "球队能力 → 球员能力 → 球员位置与默认战术角色 → 动态标签 → 国家队与俱乐部双重球员关系 → 教练任期 → 阵型分布；比赛阵容优先使用本场覆盖，否则自动继承默认角色。"),
        ("时间要求", "日期和时间字段均可填写 YYYY-MM-DD、ISO 8601 或 Excel 日期单元格；仅填写日期时按 UTC 00:00:00 处理。球队能力/战术快照只填一个窗口日期时自动生成同日点时窗口；阵型使用分布仍须填写真实起止窗口。动态标签失效时间留空时按标签默认 TTL 自动生成。"),
        ("阵型规则", "阵型代码会统一全角数字、空格和不同横线字符；目录外但各线人数合计为 10 的标准代码（如 3-4-1-2）会作为自定义阵型保留并登记，无法识别的文本会在预检行直接阻断。"),
        ("导入流程", "选择文件 → 统一预检 → 处理冲突/错误 → 一次确认提交。重复文件不会重复写入。"),
    ];
    for (index, (label, value)) in rows.iter().enumerate() {
        let row = (index + 1) as u32;
        sheet.write_string_with_format(row, 0, *label, &section_format())?;
        sheet.merge_range(row, 1, row, 6, *value, &note_format())?;
    }
    sheet.set_column_width(0, 18)?;
    for column in 1..=6 {
        sheet.set_column_width(column, 18)?;
    }
    sheet.set_row_height(0, 30)?;
    Ok(())
}

fn write_team_sheet(workbook: &mut Workbook) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("球队总览")?;
    write_group_row(
        sheet,
        &[
            (0, 1, "操作"),
            (2, 12, "球队身份"),
            (13, 22, "球队能力（0–100）"),
            (23, 30, "战术画像"),
            (31, 35, "观察与审计"),
        ],
    )?;
    write_key_row(sheet, TEAM_KEYS)?;
    write_example_row(sheet, TEAM_KEYS, "team")?;
    apply_common_sheet_layout(sheet, TEAM_KEYS)?;
    add_validation(
        sheet,
        TEAM_KEYS,
        "action",
        &["upsert", "add", "update", "clear", "skip"],
    )?;
    add_validation(
        sheet,
        TEAM_KEYS,
        "team_type",
        &["club", "national", "reserve", "youth", "women", "other"],
    )?;
    Ok(())
}

fn write_team_name_sheet(workbook: &mut Workbook) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("球队名称")?;
    write_group_row(
        sheet,
        &[
            (0, 1, "操作"),
            (2, 3, "球队匹配"),
            (4, 8, "多语言名称"),
            (9, 12, "来源与备注"),
        ],
    )?;
    write_key_row(sheet, TEAM_NAME_KEYS)?;
    write_example_row(sheet, TEAM_NAME_KEYS, "team_name")?;
    apply_common_sheet_layout(sheet, TEAM_NAME_KEYS)?;
    add_validation(
        sheet,
        TEAM_NAME_KEYS,
        "action",
        &["upsert", "add", "update", "clear", "skip"],
    )?;
    add_validation(sheet, TEAM_NAME_KEYS, "is_primary", &["true", "false"])?;
    Ok(())
}

fn write_player_sheet(workbook: &mut Workbook) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("球员与评分")?;
    write_group_row(
        sheet,
        &[
            (0, 1, "操作"),
            (2, 10, "球员身份"),
            (11, 17, "国家队/主球队关系"),
            (18, 25, "俱乐部关系"),
            (26, 33, "位置、默认角色与可用性"),
            (34, 51, "基础能力评分（0–100）"),
            (52, 68, "动态评分与比赛适配"),
            (69, 71, "来源与备注"),
        ],
    )?;
    write_key_row(sheet, PLAYER_KEYS)?;
    write_example_row(sheet, PLAYER_KEYS, "player")?;
    apply_common_sheet_layout(sheet, PLAYER_KEYS)?;
    add_validation(
        sheet,
        PLAYER_KEYS,
        "action",
        &["upsert", "add", "update", "clear", "skip"],
    )?;
    add_validation(
        sheet,
        PLAYER_KEYS,
        "preferred_foot",
        &["right", "left", "both", "unknown"],
    )?;
    add_validation(
        sheet,
        PLAYER_KEYS,
        "player_status",
        &["active", "inactive", "retired", "unknown"],
    )?;
    add_validation(
        sheet,
        PLAYER_KEYS,
        "registration_status",
        &["registered", "loan", "trial", "unknown"],
    )?;
    add_validation(
        sheet,
        PLAYER_KEYS,
        "club_registration_status",
        &["registered", "loan", "trial", "unknown"],
    )?;
    add_validation(
        sheet,
        PLAYER_KEYS,
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
    )?;
    Ok(())
}

fn write_player_name_sheet(workbook: &mut Workbook) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("球员名称")?;
    write_group_row(
        sheet,
        &[
            (0, 1, "操作"),
            (2, 5, "球员匹配"),
            (6, 10, "多语言名称"),
            (11, 14, "来源与备注"),
        ],
    )?;
    write_key_row(sheet, PLAYER_NAME_KEYS)?;
    write_example_row(sheet, PLAYER_NAME_KEYS, "player_name")?;
    apply_common_sheet_layout(sheet, PLAYER_NAME_KEYS)?;
    add_validation(
        sheet,
        PLAYER_NAME_KEYS,
        "action",
        &["upsert", "add", "update", "clear", "skip"],
    )?;
    add_validation(sheet, PLAYER_NAME_KEYS, "is_primary", &["true", "false"])?;
    Ok(())
}

fn write_coach_sheet(workbook: &mut Workbook) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("教练与阵型")?;
    write_group_row(
        sheet,
        &[
            (0, 1, "操作"),
            (2, 5, "教练身份"),
            (6, 11, "球队任期"),
            (12, 23, "阵型熟悉度与使用分布"),
            (24, 26, "来源与备注"),
        ],
    )?;
    write_key_row(sheet, COACH_KEYS)?;
    write_example_row(sheet, COACH_KEYS, "coach")?;
    apply_common_sheet_layout(sheet, COACH_KEYS)?;
    add_validation(
        sheet,
        COACH_KEYS,
        "action",
        &["upsert", "add", "update", "clear", "skip"],
    )?;
    add_validation(
        sheet,
        COACH_KEYS,
        "coach_status",
        &["active", "inactive", "retired", "unknown"],
    )?;
    add_validation(
        sheet,
        COACH_KEYS,
        "role",
        &[
            "head_coach",
            "interim_head_coach",
            "caretaker",
            "assistant_coach",
            "other",
        ],
    )?;
    add_validation(
        sheet,
        COACH_KEYS,
        "scope_type",
        &[
            "team_coach",
            "team",
            "coach",
            "competition_default",
            "system_default",
        ],
    )?;
    add_validation(
        sheet,
        COACH_KEYS,
        "window_preset",
        &[
            "custom",
            "last_5",
            "last_10",
            "last_20",
            "current_coach_term",
            "current_season",
        ],
    )?;
    Ok(())
}

fn write_dictionary_sheet(
    workbook: &mut Workbook,
    references: &PlayerCatalogReferenceData,
) -> Result<(), XlsxError> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("字段字典")?;
    sheet.write_string_with_format(0, 0, "类型", &header_format())?;
    sheet.write_string_with_format(0, 1, "代码", &header_format())?;
    sheet.write_string_with_format(0, 2, "名称", &header_format())?;
    sheet.write_string_with_format(0, 3, "范围/说明", &header_format())?;
    let mut row = 1_u32;
    for position in &references.positions {
        write_text_row(
            sheet,
            row,
            &[
                "position",
                &position.code,
                &position.name,
                &position.position_group,
            ],
        )?;
        row += 1;
    }
    for ability in &references.ability_dimensions {
        let range = format!("{}–{}", ability.minimum_value, ability.maximum_value);
        write_text_row(
            sheet,
            row,
            &[
                "ability",
                ability.code.as_str(),
                ability.name.as_str(),
                range.as_str(),
            ],
        )?;
        row += 1;
    }
    for tag in &references.dynamic_tag_definitions {
        let range = format!("{}–{}", tag.minimum_value, tag.maximum_value);
        write_text_row(
            sheet,
            row,
            &[
                "dynamic_tag",
                tag.code.as_str(),
                tag.name.as_str(),
                range.as_str(),
            ],
        )?;
        row += 1;
    }
    sheet.set_column_width(0, 18)?;
    sheet.set_column_width(1, 28)?;
    sheet.set_column_width(2, 24)?;
    sheet.set_column_width(3, 42)?;
    sheet.set_freeze_panes(1, 0)?;
    Ok(())
}

fn parse_team_sheet<R: std::io::Read + std::io::Seek>(
    workbook: &mut calamine::Sheets<R>,
    output: &mut Vec<SpreadsheetRawRow>,
) -> SpreadsheetResult<()> {
    for (row_number, values) in read_business_rows(workbook, "球队总览", TEAM_KEYS)? {
        let action = parse_action(text(&values, "action"))?;
        if action == SpreadsheetAction::Skip {
            output.push(raw(
                "球队总览",
                row_number,
                SpreadsheetEntityType::Team,
                action,
                values,
            ));
            continue;
        }
        let team = mapped(
            &values,
            &[
                ("action", "action"),
                ("clear_fields", "clear_fields"),
                ("team_id", "team_id"),
                ("team_name", "official_name"),
                ("short_name", "short_name"),
                ("team_type", "team_type"),
                ("country_code", "country_code"),
                ("city", "city"),
                ("founded_year", "founded_year"),
                ("stadium", "stadium"),
                ("is_active", "is_active"),
                ("profile_observed_at", "profile_observed_at"),
                ("data_confidence", "data_confidence"),
                ("source_urls", "source_urls"),
                ("verified_at", "verified_at"),
                ("notes", "notes"),
            ],
        );
        output.push(raw(
            "球队总览",
            row_number,
            SpreadsheetEntityType::Team,
            action,
            team,
        ));

        if any_nonempty(
            &values,
            &[
                "team_attack_rating",
                "team_midfield_rating",
                "team_defence_rating",
                "team_goalkeeper_rating",
                "team_squad_depth_rating",
                "team_stability_rating",
            ],
        ) {
            let mut ability = mapped(
                &values,
                &[
                    ("action", "action"),
                    ("team_id", "team_id"),
                    ("team_name", "team_name"),
                    ("observed_at", "observed_at"),
                    ("window_start", "window_start"),
                    ("window_end", "window_end"),
                    ("team_sample_size", "sample_size"),
                    ("team_methodology", "methodology"),
                    ("observation_confidence", "confidence"),
                    ("source_urls", "source_urls"),
                    ("verified_at", "verified_at"),
                    ("notes", "notes"),
                ],
            );
            for (source, target) in [
                ("team_attack_rating", "attack_rating"),
                ("team_midfield_rating", "midfield_rating"),
                ("team_defence_rating", "defence_rating"),
                ("team_goalkeeper_rating", "goalkeeper_rating"),
                ("team_squad_depth_rating", "squad_depth_rating"),
                ("team_stability_rating", "stability_rating"),
            ] {
                ability.insert(
                    target.to_string(),
                    Value::String(normalized_rating_text(&values, source)?),
                );
            }
            output.push(raw(
                "球队总览",
                row_number,
                SpreadsheetEntityType::TeamAbilityObservation,
                action,
                ability,
            ));
        }
        if any_nonempty(
            &values,
            &[
                "build_up_style",
                "progression_style",
                "attacking_width",
                "pressing_intensity",
                "defensive_block",
                "transition_speed",
                "set_piece_tendency",
                "tactical_summary",
            ],
        ) {
            let tactical = mapped(
                &values,
                &[
                    ("action", "action"),
                    ("team_id", "team_id"),
                    ("team_name", "team_name"),
                    ("window_start", "window_start"),
                    ("window_end", "window_end"),
                    ("build_up_style", "build_up_style"),
                    ("progression_style", "progression_style"),
                    ("attacking_width", "attacking_width"),
                    ("pressing_intensity", "pressing_intensity"),
                    ("defensive_block", "defensive_block"),
                    ("transition_speed", "transition_speed"),
                    ("set_piece_tendency", "set_piece_tendency"),
                    ("tactical_summary", "tactical_summary"),
                    ("observation_confidence", "confidence"),
                    ("source_urls", "source_urls"),
                    ("verified_at", "verified_at"),
                    ("observed_at", "observed_at"),
                    ("notes", "notes"),
                ],
            );
            output.push(raw(
                "球队总览",
                row_number,
                SpreadsheetEntityType::TeamTacticalObservation,
                action,
                tactical,
            ));
        }
    }
    Ok(())
}

fn parse_team_name_sheet<R: std::io::Read + std::io::Seek>(
    workbook: &mut calamine::Sheets<R>,
    output: &mut Vec<SpreadsheetRawRow>,
) -> SpreadsheetResult<()> {
    for (row_number, values) in read_business_rows(workbook, "球队名称", TEAM_NAME_KEYS)? {
        let action = parse_action(text(&values, "action"))?;
        let name = mapped(
            &values,
            &[
                ("action", "action"),
                ("clear_fields", "clear_fields"),
                ("team_id", "team_id"),
                ("team_name", "team_name"),
                ("name_value", "name_value"),
                ("language_code", "language_code"),
                ("is_primary", "is_primary"),
                ("valid_from", "valid_from"),
                ("valid_to", "valid_to"),
                ("source_urls", "source_urls"),
                ("verified_at", "verified_at"),
                ("confidence", "confidence"),
                ("notes", "notes"),
            ],
        );
        output.push(raw(
            "球队名称",
            row_number,
            SpreadsheetEntityType::TeamName,
            action,
            name,
        ));
    }
    Ok(())
}

fn parse_player_sheet<R: std::io::Read + std::io::Seek>(
    workbook: &mut calamine::Sheets<R>,
    output: &mut Vec<SpreadsheetRawRow>,
) -> SpreadsheetResult<()> {
    let mut emitted_club_teams = HashSet::new();
    for row in output.iter().filter(|row| {
        row.entity_type == SpreadsheetEntityType::Team && row.action != SpreadsheetAction::Skip
    }) {
        if let Some(values) = row.values.as_object() {
            emitted_club_teams.extend(team_entity_identity_aliases(values));
        }
    }
    for (row_number, values) in read_business_rows(workbook, "球员与评分", PLAYER_KEYS)? {
        let action = parse_action(text(&values, "action"))?;
        let base = mapped(
            &values,
            &[
                ("action", "action"),
                ("clear_fields", "clear_fields"),
                ("player_key", "player_key"),
                ("player_id", "player_id"),
                ("official_name", "official_name"),
                ("birth_date", "birth_date"),
                ("nationality_code", "nationality_code"),
                ("preferred_foot", "preferred_foot"),
                ("height_cm", "height_cm"),
                ("player_status", "player_status"),
                ("source_urls", "source_urls"),
                ("verified_at", "verified_at"),
                ("ability_confidence", "confidence"),
                ("notes", "notes"),
            ],
        );
        output.push(raw(
            "球员与评分",
            row_number,
            SpreadsheetEntityType::Player,
            action,
            base,
        ));
        if action == SpreadsheetAction::Skip {
            continue;
        }

        let mut emitted_team_period_identities = HashSet::new();
        if nonempty(&values, "team_name")
            || nonempty(&values, "team_id")
            || nonempty(&values, "team_key")
        {
            let period = mapped(
                &values,
                &[
                    ("action", "action"),
                    ("clear_fields", "clear_fields"),
                    ("player_key", "player_key"),
                    ("player_id", "player_id"),
                    ("official_name", "match_name"),
                    ("birth_date", "match_birth_date"),
                    ("team_key", "team_key"),
                    ("team_id", "team_id"),
                    ("team_name", "team_name"),
                    ("squad_number", "squad_number"),
                    ("roster_valid_from", "valid_from"),
                    ("roster_valid_to", "valid_to"),
                    ("registration_status", "registration_status"),
                    ("source_urls", "source_urls"),
                    ("verified_at", "verified_at"),
                    ("ability_confidence", "confidence"),
                    ("notes", "notes"),
                ],
            );
            push_unique_team_period(
                output,
                row_number,
                action,
                period,
                &mut emitted_team_period_identities,
            );
        }

        if let Some(club_name) = text(&values, "club_team_name") {
            let mut club = Map::new();
            club.insert("action".into(), Value::String("upsert".into()));
            club.insert("official_name".into(), Value::String(club_name.clone()));
            club.insert("team_type".into(), Value::String("club".into()));
            if let Some(team_key) = text(&values, "club_team_key") {
                club.insert("team_key".into(), Value::String(team_key));
            }
            if let Some(country_code) = text(&values, "club_country_code") {
                club.insert("country_code".into(), Value::String(country_code));
            }
            for key in ["source_urls", "verified_at", "notes"] {
                if let Some(value) = text(&values, key) {
                    club.insert(key.into(), Value::String(value));
                }
            }
            push_unique_team_entity(output, row_number, club, &mut emitted_club_teams);
            let mut club_period = mapped(
                &values,
                &[
                    ("action", "action"),
                    ("clear_fields", "clear_fields"),
                    ("player_key", "player_key"),
                    ("player_id", "player_id"),
                    ("official_name", "match_name"),
                    ("birth_date", "match_birth_date"),
                    ("club_team_key", "team_key"),
                    ("club_team_id", "team_id"),
                    ("club_team_name", "team_name"),
                    ("club_squad_number", "squad_number"),
                    ("club_registration_status", "registration_status"),
                    ("club_valid_from", "valid_from"),
                    ("club_valid_to", "valid_to"),
                    ("source_urls", "source_urls"),
                    ("verified_at", "verified_at"),
                    ("ability_confidence", "confidence"),
                    ("notes", "notes"),
                ],
            );
            if club_period
                .get("registration_status")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .is_empty()
            {
                club_period.insert(
                    "registration_status".into(),
                    Value::String("registered".into()),
                );
            }
            if club_period
                .get("valid_from")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .is_empty()
            {
                if let Some(value) = first_date_text(
                    &values,
                    &[
                        "roster_valid_from",
                        "verified_at",
                        "ability_observed_at",
                        "tag_observed_at",
                    ],
                ) {
                    club_period.insert("valid_from".into(), Value::String(value));
                }
            }
            push_unique_team_period(
                output,
                row_number,
                action,
                club_period,
                &mut emitted_team_period_identities,
            );
        }
        if nonempty(&values, "english_name") {
            let mut name = mapped(
                &values,
                &[
                    ("action", "action"),
                    ("player_key", "player_key"),
                    ("player_id", "player_id"),
                    ("official_name", "match_name"),
                    ("birth_date", "match_birth_date"),
                    ("english_name", "name_value"),
                    ("roster_valid_from", "valid_from"),
                    ("roster_valid_to", "valid_to"),
                    ("source_urls", "source_urls"),
                    ("verified_at", "verified_at"),
                    ("ability_confidence", "confidence"),
                    ("notes", "notes"),
                ],
            );
            name.insert("language_code".into(), Value::String("en".into()));
            name.insert("is_primary".into(), Value::String("false".into()));
            output.push(raw(
                "球员与评分",
                row_number,
                SpreadsheetEntityType::PlayerName,
                action,
                name,
            ));
        }
        if nonempty(&values, "position_code") {
            let position = mapped(
                &values,
                &[
                    ("action", "action"),
                    ("clear_fields", "clear_fields"),
                    ("player_key", "player_key"),
                    ("player_id", "player_id"),
                    ("official_name", "match_name"),
                    ("birth_date", "match_birth_date"),
                    ("position_code", "position_code"),
                    ("position_proficiency", "proficiency"),
                    ("position_is_primary", "is_primary"),
                    ("default_role_code", "default_role_code"),
                    ("roster_valid_from", "valid_from"),
                    ("roster_valid_to", "valid_to"),
                    ("source_urls", "source_urls"),
                    ("verified_at", "verified_at"),
                    ("ability_confidence", "confidence"),
                    ("notes", "notes"),
                ],
            );
            output.push(raw(
                "球员与评分",
                row_number,
                SpreadsheetEntityType::PlayerPosition,
                action,
                position,
            ));
        }
        if nonempty(&values, "availability_status") {
            let availability = mapped(
                &values,
                &[
                    ("action", "action"),
                    ("clear_fields", "clear_fields"),
                    ("player_key", "player_key"),
                    ("player_id", "player_id"),
                    ("official_name", "match_name"),
                    ("birth_date", "match_birth_date"),
                    ("team_key", "team_key"),
                    ("team_id", "team_id"),
                    ("team_name", "team_name"),
                    ("availability_status", "availability_status"),
                    ("availability_reason", "reason"),
                    ("ability_confidence", "confidence"),
                    ("availability_valid_from", "valid_from"),
                    ("availability_valid_to", "valid_to"),
                    ("source_urls", "source_urls"),
                    ("verified_at", "verified_at"),
                    ("notes", "notes"),
                ],
            );
            output.push(raw(
                "球员与评分",
                row_number,
                SpreadsheetEntityType::PlayerAvailability,
                action,
                availability,
            ));
        }
        for (column, dimension) in ABILITY_COLUMNS {
            if !nonempty(&values, column) {
                continue;
            }
            let mut ability = mapped(
                &values,
                &[
                    ("action", "action"),
                    ("player_key", "player_key"),
                    ("player_id", "player_id"),
                    ("official_name", "match_name"),
                    ("birth_date", "match_birth_date"),
                    ("ability_confidence", "confidence"),
                    ("ability_sample_size", "sample_size"),
                    ("ability_observed_at", "observed_at"),
                    ("ability_effective_from", "effective_from"),
                    ("ability_effective_to", "effective_to"),
                    ("ability_calculation_version", "calculation_version"),
                    ("source_urls", "source_urls"),
                    ("verified_at", "verified_at"),
                    ("notes", "notes"),
                ],
            );
            ability.insert("dimension_code".into(), Value::String((*dimension).into()));
            ability.insert("context_type".into(), Value::String("general".into()));
            ability.insert(
                "value".into(),
                Value::String(normalized_rating_text(&values, column)?),
            );
            output.push(raw(
                "球员与评分",
                row_number,
                SpreadsheetEntityType::PlayerAbility,
                action,
                ability,
            ));
        }
        for (column, tag_code) in TAG_COLUMNS {
            if !nonempty(&values, column) {
                continue;
            }
            let mut tag = mapped(
                &values,
                &[
                    ("action", "action"),
                    ("player_key", "player_key"),
                    ("player_id", "player_id"),
                    ("official_name", "match_name"),
                    ("birth_date", "match_birth_date"),
                    ("tag_confidence", "confidence"),
                    ("tag_sample_size", "sample_size"),
                    ("tag_observed_at", "observed_at"),
                    ("tag_valid_from", "valid_from"),
                    ("tag_valid_to", "valid_to"),
                    ("position_code", "position_code"),
                    ("tag_source_type", "source_type"),
                    ("tag_calculation_version", "calculation_version"),
                    ("source_urls", "source_urls"),
                    ("verified_at", "verified_at"),
                    ("notes", "notes"),
                ],
            );
            tag.insert("tag_code".into(), Value::String((*tag_code).into()));
            tag.insert(
                "tag_value".into(),
                Value::String(text(&values, column).unwrap_or_default()),
            );
            output.push(raw(
                "球员与评分",
                row_number,
                SpreadsheetEntityType::PlayerDynamicTag,
                action,
                tag,
            ));
        }
    }
    Ok(())
}

fn parse_player_name_sheet<R: std::io::Read + std::io::Seek>(
    workbook: &mut calamine::Sheets<R>,
    output: &mut Vec<SpreadsheetRawRow>,
) -> SpreadsheetResult<()> {
    for (row_number, values) in read_business_rows(workbook, "球员名称", PLAYER_NAME_KEYS)? {
        let action = parse_action(text(&values, "action"))?;
        let name = mapped(
            &values,
            &[
                ("action", "action"),
                ("clear_fields", "clear_fields"),
                ("player_key", "player_key"),
                ("player_id", "player_id"),
                ("match_name", "match_name"),
                ("match_birth_date", "match_birth_date"),
                ("name_value", "name_value"),
                ("language_code", "language_code"),
                ("is_primary", "is_primary"),
                ("valid_from", "valid_from"),
                ("valid_to", "valid_to"),
                ("source_urls", "source_urls"),
                ("verified_at", "verified_at"),
                ("confidence", "confidence"),
                ("notes", "notes"),
            ],
        );
        output.push(raw(
            "球员名称",
            row_number,
            SpreadsheetEntityType::PlayerName,
            action,
            name,
        ));
    }
    Ok(())
}

fn parse_coach_sheet<R: std::io::Read + std::io::Seek>(
    workbook: &mut calamine::Sheets<R>,
    output: &mut Vec<SpreadsheetRawRow>,
) -> SpreadsheetResult<()> {
    let mut coaches = HashSet::new();
    let mut periods = HashSet::new();
    for (row_number, values) in read_business_rows(workbook, "教练与阵型", COACH_KEYS)? {
        let action = parse_action(text(&values, "action"))?;
        let coach_identity = text(&values, "coach_id")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| {
                text(&values, "coach_name")
                    .unwrap_or_default()
                    .to_lowercase()
            });
        if !coach_identity.is_empty() && coaches.insert(coach_identity.clone()) {
            let coach = mapped(
                &values,
                &[
                    ("action", "action"),
                    ("clear_fields", "clear_fields"),
                    ("coach_id", "coach_id"),
                    ("coach_name", "official_name"),
                    ("nationality_code", "nationality_code"),
                    ("coach_status", "coach_status"),
                    ("source_urls", "source_urls"),
                    ("verified_at", "verified_at"),
                    ("confidence", "confidence"),
                    ("notes", "notes"),
                ],
            );
            output.push(raw(
                "教练与阵型",
                row_number,
                SpreadsheetEntityType::Coach,
                action,
                coach,
            ));
        }
        if action == SpreadsheetAction::Skip {
            continue;
        }
        let has_team = nonempty(&values, "team_id") || nonempty(&values, "team_name");
        let derived_valid_from = text(&values, "valid_from")
            .or_else(|| first_date_text(&values, &["window_start", "observed_at", "verified_at"]));
        if has_team && !coach_identity.is_empty() && derived_valid_from.is_some() {
            let role = text(&values, "role").unwrap_or_else(|| "head_coach".to_string());
            let valid_from = derived_valid_from.unwrap_or_default();
            let period_key = format!(
                "{}|{}|{}|{}",
                text(&values, "team_id")
                    .unwrap_or_else(|| text(&values, "team_name").unwrap_or_default()),
                text(&values, "coach_id")
                    .unwrap_or_else(|| text(&values, "coach_name").unwrap_or_default()),
                role,
                valid_from,
            );
            if periods.insert(period_key) {
                let mut period = mapped(
                    &values,
                    &[
                        ("action", "action"),
                        ("clear_fields", "clear_fields"),
                        ("team_id", "team_id"),
                        ("team_name", "team_name"),
                        ("coach_id", "coach_id"),
                        ("coach_name", "coach_name"),
                        ("role", "role"),
                        ("valid_from", "valid_from"),
                        ("valid_to", "valid_to"),
                        ("is_interim", "is_interim"),
                        ("confidence", "confidence"),
                        ("source_urls", "source_urls"),
                        ("verified_at", "verified_at"),
                        ("notes", "notes"),
                    ],
                );
                if period
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .is_empty()
                {
                    period.insert("role".into(), Value::String("head_coach".into()));
                }
                if period
                    .get("valid_from")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .is_empty()
                {
                    period.insert("valid_from".into(), Value::String(valid_from));
                }
                output.push(raw(
                    "教练与阵型",
                    row_number,
                    SpreadsheetEntityType::TeamCoachPeriod,
                    action,
                    period,
                ));
            }
        }
        if nonempty(&values, "formation_code") || nonempty(&values, "formation_id") {
            let formation = mapped(
                &values,
                &[
                    ("action", "action"),
                    ("scope_type", "scope_type"),
                    ("team_id", "team_id"),
                    ("team_name", "team_name"),
                    ("coach_id", "coach_id"),
                    ("coach_name", "coach_name"),
                    ("formation_id", "formation_id"),
                    ("formation_code", "formation_code"),
                    ("window_preset", "window_preset"),
                    ("window_start", "window_start"),
                    ("window_end", "window_end"),
                    ("observed_matches", "observed_matches"),
                    ("usage_count", "usage_count"),
                    ("confidence", "confidence"),
                    ("alpha", "alpha"),
                    ("observed_at", "observed_at"),
                    ("source_urls", "source_urls"),
                    ("verified_at", "verified_at"),
                    ("notes", "notes"),
                    ("formation_familiarity", "formation_familiarity"),
                ],
            );
            output.push(raw(
                "教练与阵型",
                row_number,
                SpreadsheetEntityType::FormationUsage,
                action,
                formation,
            ));
        }
    }
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

fn is_backward_compatible_optional_key(key: &str) -> bool {
    matches!(key, "default_role_code")
}

fn read_business_rows<R: std::io::Read + std::io::Seek>(
    workbook: &mut calamine::Sheets<R>,
    sheet_name: &str,
    required_keys: &[&str],
) -> SpreadsheetResult<Vec<(u32, Map<String, Value>)>> {
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|error| SpreadsheetError::Read(error.to_string()))?;
    let key_row = range.rows().nth(KEY_ROW).ok_or_else(|| {
        SpreadsheetError::InvalidTemplate(format!("工作表 {sheet_name} 缺少固定字段键行"))
    })?;
    let keys = key_row
        .iter()
        .map(cell_text)
        .map(|value| value.trim().to_string())
        .collect::<Vec<_>>();
    for required in required_keys {
        if !keys.iter().any(|key| key == required)
            && !required.starts_with("club_")
            && !is_backward_compatible_optional_key(required)
        {
            return Err(SpreadsheetError::InvalidTemplate(format!(
                "工作表 {sheet_name} 缺少固定字段 {required}"
            )));
        }
    }
    let mut rows = Vec::new();
    for (index, row) in range.rows().skip(FIRST_DATA_ROW).enumerate() {
        let mut values = keys
            .iter()
            .enumerate()
            .map(|(column, key)| {
                let value = row
                    .get(column)
                    .map(|cell| cell_text_for_key(cell, key))
                    .unwrap_or_default();
                (key.clone(), Value::String(value.trim().to_string()))
            })
            .collect::<Map<_, _>>();
        if sheet_name == "球员与评分" {
            apply_default_role_alias(&mut values);
        }
        if values
            .values()
            .all(|value| value.as_str().unwrap_or_default().is_empty())
        {
            continue;
        }
        rows.push(((index + FIRST_DATA_ROW + 1) as u32, values));
    }
    Ok(rows)
}

fn mapped(values: &Map<String, Value>, fields: &[(&str, &str)]) -> Map<String, Value> {
    fields
        .iter()
        .map(|(source, target)| {
            (
                (*target).to_string(),
                Value::String(text(values, source).unwrap_or_default()),
            )
        })
        .collect()
}

fn push_unique_team_entity(
    output: &mut Vec<SpreadsheetRawRow>,
    row_number: u32,
    payload: Map<String, Value>,
    emitted_identities: &mut HashSet<String>,
) {
    let identities = team_entity_identity_aliases(&payload);
    if identities
        .iter()
        .any(|identity| emitted_identities.contains(identity))
    {
        return;
    }
    emitted_identities.extend(identities);
    output.push(raw(
        "球员与评分",
        row_number,
        SpreadsheetEntityType::Team,
        SpreadsheetAction::Upsert,
        payload,
    ));
}

fn team_entity_identity_aliases(payload: &Map<String, Value>) -> Vec<String> {
    [
        ("team_id", "id"),
        ("team_key", "key"),
        ("official_name", "name"),
    ]
    .into_iter()
    .filter_map(|(field, prefix)| {
        text(payload, field).and_then(|value| {
            let normalized = normalized_identity_value(&value);
            (!normalized.is_empty()).then(|| format!("{prefix}:{normalized}"))
        })
    })
    .collect()
}

fn normalized_identity_value(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn push_unique_team_period(
    output: &mut Vec<SpreadsheetRawRow>,
    row_number: u32,
    action: SpreadsheetAction,
    payload: Map<String, Value>,
    emitted_identities: &mut HashSet<String>,
) {
    let identities = team_period_identity_aliases(&payload);
    if identities
        .iter()
        .any(|identity| emitted_identities.contains(identity))
    {
        return;
    }
    emitted_identities.extend(identities);
    output.push(raw(
        "球员与评分",
        row_number,
        SpreadsheetEntityType::PlayerTeamPeriod,
        action,
        payload,
    ));
}

fn team_period_identity_aliases(payload: &Map<String, Value>) -> Vec<String> {
    [
        ("team_id", "id"),
        ("team_key", "key"),
        ("team_name", "name"),
    ]
    .into_iter()
    .filter_map(|(field, prefix)| {
        text(payload, field).and_then(|value| {
            let normalized = normalized_identity_value(&value);
            (!normalized.is_empty()).then(|| format!("{prefix}:{normalized}"))
        })
    })
    .collect()
}

fn normalized_rating_text(values: &Map<String, Value>, key: &str) -> SpreadsheetResult<String> {
    let Some(raw) = text(values, key) else {
        return Ok(String::new());
    };
    let value = raw.parse::<f64>().map_err(|_| {
        SpreadsheetError::InvalidTemplate(format!("字段 {key} 的评分不是有效数字：{raw}"))
    })?;
    let normalized = if (0.0..=100.0).contains(&value) {
        value
    } else if (0.0..=10_000.0).contains(&value) {
        value / 100.0
    } else {
        return Err(SpreadsheetError::InvalidTemplate(format!(
            "字段 {key} 的评分必须位于 0–100 或 0–10000：{raw}"
        )));
    };
    Ok(format!("{normalized:.4}"))
}

fn key_label(key: &str) -> &str {
    match key {
        "action" => "操作",
        "clear_fields" => "明确清空字段",
        "team_id" => "球队ID",
        "team_key" => "球队临时键",
        "team_name" => "球队名称",
        "match_name" => "匹配姓名",
        "match_birth_date" => "匹配出生日期",
        "name_value" => "名称内容",
        "language_code" => "语言代码",
        "is_primary" => "设为主显示名",
        "short_name" => "球队简称",
        "team_type" => "球队类型",
        "country_code" => "国家/地区代码",
        "city" => "城市",
        "founded_year" => "成立年份",
        "stadium" => "主场",
        "is_active" => "是否活跃",
        "profile_observed_at" => "球队档案观察时间",
        "data_confidence" => "球队档案可信度",
        "window_start" => "观察窗口开始",
        "window_end" => "观察窗口结束",
        "team_attack_rating" => "球队进攻评分",
        "team_midfield_rating" => "球队中场评分",
        "team_defence_rating" => "球队防守评分",
        "team_goalkeeper_rating" => "球队门将评分",
        "team_squad_depth_rating" => "球队阵容厚度",
        "team_stability_rating" => "球队稳定性",
        "team_sample_size" => "球队能力样本量",
        "team_methodology" => "球队评分方法",
        "build_up_style" => "后场组织方式",
        "progression_style" => "推进方式",
        "attacking_width" => "进攻宽度",
        "pressing_intensity" => "压迫强度",
        "defensive_block" => "防守落位",
        "transition_speed" => "攻防转换速度",
        "set_piece_tendency" => "定位球倾向",
        "tactical_summary" => "战术摘要",
        "observation_confidence" => "观察可信度",
        "observed_at" => "观察时间",
        "source_urls" => "来源链接",
        "verified_at" => "核验时间",
        "notes" => "备注",
        "player_key" => "球员临时键",
        "player_id" => "球员ID",
        "official_name" => "球员正式姓名",
        "english_name" => "英文名/别名",
        "birth_date" => "出生日期",
        "nationality_code" => "国籍代码",
        "preferred_foot" => "惯用脚",
        "height_cm" => "身高(cm)",
        "player_status" => "球员档案状态",
        "squad_number" => "球衣号码",
        "registration_status" => "注册关系",
        "roster_valid_from" => "入队/征召日期",
        "roster_valid_to" => "离队/结束日期",
        "club_team_key" => "俱乐部临时键",
        "club_team_id" => "俱乐部球队ID",
        "club_team_name" => "俱乐部名称",
        "club_country_code" => "俱乐部国家代码",
        "club_squad_number" => "俱乐部号码",
        "club_registration_status" => "俱乐部注册关系",
        "club_valid_from" => "俱乐部效力开始",
        "club_valid_to" => "俱乐部效力结束",
        "position_code" => "位置代码",
        "position_proficiency" => "位置熟练度",
        "position_is_primary" => "是否主位置",
        "default_role_code" => "默认战术角色",
        "availability_status" => "当前可用性",
        "availability_reason" => "不可用原因",
        "availability_valid_from" => "状态开始时间",
        "availability_valid_to" => "状态结束时间",
        "ability_attack" => "进攻能力",
        "ability_defence" => "防守能力",
        "ability_creation" => "创造能力",
        "ability_progression" => "推进能力",
        "ability_finishing" => "终结能力",
        "ability_physical" => "身体对抗",
        "ability_stamina" => "体能耐力",
        "ability_stability" => "稳定性",
        "ability_discipline" => "纪律性",
        "ability_tactical_execution" => "战术执行",
        "ability_versatility" => "多面性",
        "ability_substitute_impact" => "替补冲击",
        "ability_confidence" => "基础评分可信度",
        "ability_sample_size" => "基础评分样本量",
        "ability_observed_at" => "基础评分观察时间",
        "ability_effective_from" => "基础评分生效时间",
        "ability_effective_to" => "基础评分结束时间",
        "ability_calculation_version" => "基础评分版本",
        "tag_match_readiness" => "比赛准备度",
        "tag_form_multiplier" => "近期状态系数",
        "tag_fatigue_multiplier" => "体能负荷系数",
        "tag_position_fit" => "位置适配系数",
        "tag_tactical_fit" => "战术适配系数",
        "tag_chemistry_fit" => "组合熟悉度",
        "tag_starting_probability" => "首发概率",
        "tag_expected_minutes_share" => "预计分钟比例",
        "tag_realization_multiplier" => "兑现率修正",
        "tag_volatility" => "状态波动度",
        "tag_confidence" => "动态评分可信度",
        "tag_sample_size" => "动态评分样本量",
        "tag_observed_at" => "动态评分观察时间",
        "tag_valid_from" => "动态评分生效时间",
        "tag_valid_to" => "动态评分失效时间",
        "tag_source_type" => "动态评分来源类型",
        "tag_calculation_version" => "动态评分版本",
        "coach_id" => "教练ID",
        "coach_name" => "教练姓名",
        "coach_status" => "教练状态",
        "role" => "教练职务",
        "valid_from" => "任期开始",
        "valid_to" => "任期结束",
        "is_interim" => "是否临时教练",
        "formation_id" => "阵型ID",
        "formation_code" => "阵型代码",
        "scope_type" => "阵型观察范围",
        "window_preset" => "观察窗口预设",
        "observed_matches" => "观察场次",
        "usage_count" => "使用场次",
        "formation_familiarity" => "教练阵型熟悉度",
        "confidence" => "可信度",
        "alpha" => "平滑强度",
        _ => key,
    }
}

fn raw(
    sheet_name: &str,
    row_number: u32,
    entity_type: SpreadsheetEntityType,
    action: SpreadsheetAction,
    values: Map<String, Value>,
) -> SpreadsheetRawRow {
    SpreadsheetRawRow {
        sheet_name: sheet_name.to_string(),
        row_number,
        entity_type,
        action,
        values: Value::Object(values),
    }
}

fn parse_action(value: Option<String>) -> SpreadsheetResult<SpreadsheetAction> {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .replace(['-', ' '], "_")
        .as_str()
    {
        "" | "upsert" | "merge" | "add_or_update" | "insert_or_update" => {
            Ok(SpreadsheetAction::Upsert)
        }
        "add" | "insert" => Ok(SpreadsheetAction::Add),
        "update" => Ok(SpreadsheetAction::Update),
        "clear" => Ok(SpreadsheetAction::Clear),
        "skip" => Ok(SpreadsheetAction::Skip),
        other => Err(SpreadsheetError::InvalidTemplate(format!(
            "未知 action：{other}；允许 upsert/add/update/clear/skip，留空等同 upsert"
        ))),
    }
}

fn text(values: &Map<String, Value>, key: &str) -> Option<String> {
    values
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn first_date_text(values: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        text(values, key).map(|value| {
            value
                .split(['T', ' '])
                .next()
                .unwrap_or(value.as_str())
                .to_string()
        })
    })
}

fn nonempty(values: &Map<String, Value>, key: &str) -> bool {
    text(values, key).is_some()
}

fn any_nonempty(values: &Map<String, Value>, keys: &[&str]) -> bool {
    keys.iter().any(|key| nonempty(values, key))
}

fn write_group_row(sheet: &mut Worksheet, groups: &[(u16, u16, &str)]) -> Result<(), XlsxError> {
    for (start, end, label) in groups {
        if start == end {
            sheet.write_string_with_format(0, *start, *label, &group_format())?;
        } else {
            sheet.merge_range(0, *start, 0, *end, *label, &group_format())?;
        }
    }
    sheet.set_row_height(0, 24)?;
    Ok(())
}

fn write_key_row(sheet: &mut Worksheet, keys: &[&str]) -> Result<(), XlsxError> {
    for (column, key) in keys.iter().enumerate() {
        sheet.write_string_with_format(1, column as u16, key_label(key), &header_format())?;
        sheet.write_string_with_format(2, column as u16, *key, &machine_key_format())?;
    }
    sheet.set_row_height(1, 38)?;
    sheet.set_row_height(2, 12)?;
    Ok(())
}

fn write_example_row(sheet: &mut Worksheet, keys: &[&str], kind: &str) -> Result<(), XlsxError> {
    for (column, key) in keys.iter().enumerate() {
        let value = match (*key, kind) {
            ("action", _) => "skip",
            ("team_name", "team") => "示例国家队",
            ("team_name", "team_name") => "France",
            ("name_value", "team_name") => "法国",
            ("language_code", "team_name") => "zh-CN",
            ("is_primary", "team_name") => "true",
            ("team_type", "team") => "national",
            ("country_code", "team") => "FRA",
            ("official_name", "player") => "示例球员",
            ("player_key", "player_name") => "P001",
            ("match_name", "player_name") => "Kylian Mbappé",
            ("match_birth_date", "player_name") => "1998-12-20",
            ("name_value", "player_name") => "基利安·姆巴佩",
            ("language_code", "player_name") => "zh-CN",
            ("is_primary", "player_name") => "true",
            ("player_key", "player") => "P001",
            ("birth_date", "player") => "2000-01-01",
            ("team_name", "player") => "示例国家队",
            ("club_team_name", "player") => "示例俱乐部",
            ("club_country_code", "player") => "FRA",
            ("club_registration_status", "player") => "registered",
            ("position_code", "player") => "ST",
            ("default_role_code", "player") => "抢点中锋",
            ("coach_name", "coach") => "示例主教练",
            ("team_name", "coach") => "示例国家队",
            ("role", "coach") => "head_coach",
            ("formation_code", "coach") => "4-2-3-1",
            ("scope_type", "coach") => "team_coach",
            ("notes", _) => "示例行可删除",
            _ => "",
        };
        sheet.write_string_with_format(3, column as u16, value, &example_format())?;
    }
    Ok(())
}

fn apply_common_sheet_layout(sheet: &mut Worksheet, keys: &[&str]) -> Result<(), XlsxError> {
    for (column, key) in keys.iter().enumerate() {
        let width = if key.ends_with("_id") {
            38.0
        } else if matches!(*key, "source_urls" | "notes" | "tactical_summary") {
            30.0
        } else if matches!(
            *key,
            "official_name"
                | "team_name"
                | "club_team_name"
                | "coach_name"
                | "english_name"
                | "match_name"
                | "name_value"
        ) {
            22.0
        } else {
            16.0
        };
        sheet.set_column_width(column as u16, width)?;
    }
    sheet.set_freeze_panes(3, 0)?;
    sheet.autofilter(1, 0, 1, keys.len().saturating_sub(1) as u16)?;
    Ok(())
}

fn add_validation(
    sheet: &mut Worksheet,
    keys: &[&str],
    key: &str,
    values: &[&str],
) -> Result<(), XlsxError> {
    let Some(column) = keys.iter().position(|value| *value == key) else {
        return Ok(());
    };
    let validation = DataValidation::new().allow_list_strings(values)?;
    sheet.add_data_validation(3, column as u16, LAST_DATA_ROW, column as u16, &validation)?;
    Ok(())
}

fn write_text_row(sheet: &mut Worksheet, row: u32, values: &[&str]) -> Result<(), XlsxError> {
    for (column, value) in values.iter().enumerate() {
        sheet.write_string(row, column as u16, *value)?;
    }
    Ok(())
}

fn cell_text_for_key(value: &Data, key: &str) -> String {
    let date_only = matches!(
        key,
        "birth_date"
            | "roster_valid_from"
            | "roster_valid_to"
            | "club_valid_from"
            | "club_valid_to"
            | "availability_valid_from"
            | "availability_valid_to"
            | "valid_from"
            | "valid_to"
            | "window_start"
            | "window_end"
    );
    let date_time = key.ends_with("_at")
        || matches!(
            key,
            "ability_effective_from" | "ability_effective_to" | "tag_valid_from" | "tag_valid_to"
        );
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
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.clone(),
        _ => value.to_string(),
    }
}

fn digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn title_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_size(16.0)
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x153B5B))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
}
fn group_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x315E7D))
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
}
fn header_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_size(9.0)
        .set_font_color(Color::RGB(0x17324D))
        .set_background_color(Color::RGB(0xDCEAF4))
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap()
}
fn machine_key_format() -> Format {
    Format::new()
        .set_font_size(8.0)
        .set_font_color(Color::RGB(0x64748B))
        .set_background_color(Color::RGB(0xF1F5F9))
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
}
fn section_format() -> Format {
    Format::new()
        .set_bold()
        .set_font_color(Color::RGB(0x17324D))
        .set_background_color(Color::RGB(0xDCEAF4))
        .set_border(FormatBorder::Thin)
}
fn note_format() -> Format {
    Format::new()
        .set_font_color(Color::RGB(0x475569))
        .set_background_color(Color::RGB(0xF5F8FA))
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
    fn action_aliases_are_normalized() {
        assert_eq!(
            parse_action(None).expect("blank action"),
            SpreadsheetAction::Upsert
        );
        assert_eq!(
            parse_action(Some("upsert".into())).expect("upsert action"),
            SpreadsheetAction::Upsert
        );
        assert_eq!(
            parse_action(Some("insert-or-update".into())).expect("insert-or-update action"),
            SpreadsheetAction::Upsert
        );
        assert_eq!(
            parse_action(Some("add".into())).expect("add action"),
            SpreadsheetAction::Add
        );
    }

    #[test]
    fn duplicate_main_and_club_team_period_is_emitted_once() {
        let mut output = Vec::new();
        let mut identities = HashSet::new();

        let mut main_period = Map::new();
        main_period.insert(
            "team_name".to_string(),
            Value::String("Atlético Mineiro".to_string()),
        );
        main_period.insert("squad_number".to_string(), Value::String("1".to_string()));
        push_unique_team_period(
            &mut output,
            4,
            SpreadsheetAction::Upsert,
            main_period,
            &mut identities,
        );

        let mut club_period = Map::new();
        club_period.insert(
            "team_name".to_string(),
            Value::String("  ATLÉTICO   MINEIRO  ".to_string()),
        );
        club_period.insert("squad_number".to_string(), Value::String("1".to_string()));
        push_unique_team_period(
            &mut output,
            4,
            SpreadsheetAction::Upsert,
            club_period,
            &mut identities,
        );

        assert_eq!(output.len(), 1);
        assert_eq!(
            output[0].entity_type,
            SpreadsheetEntityType::PlayerTeamPeriod
        );
    }

    #[test]
    fn distinct_main_and_club_team_periods_are_both_emitted() {
        let mut output = Vec::new();
        let mut identities = HashSet::new();

        for team_name in ["Algeria", "Manchester City"] {
            let mut period = Map::new();
            period.insert(
                "team_name".to_string(),
                Value::String(team_name.to_string()),
            );
            push_unique_team_period(
                &mut output,
                4,
                SpreadsheetAction::Upsert,
                period,
                &mut identities,
            );
        }

        assert_eq!(output.len(), 2);
    }

    #[test]
    fn explicit_team_overview_suppresses_implicit_club_team() {
        let mut output = Vec::new();
        let explicit = json!({
            "official_name": "Atlético Mineiro",
            "country_code": "BRA",
            "team_type": "club",
            "stadium": "Arena MRV"
        });
        output.push(SpreadsheetRawRow {
            sheet_name: "球队总览".into(),
            row_number: 4,
            entity_type: SpreadsheetEntityType::Team,
            action: SpreadsheetAction::Upsert,
            values: explicit,
        });
        let mut identities = HashSet::new();
        identities.extend(team_entity_identity_aliases(
            output[0].values.as_object().expect("team payload"),
        ));
        let implicit = serde_json::from_value::<Map<String, Value>>(json!({
            "official_name": "  ATLÉTICO   MINEIRO  ",
            "country_code": "BRA",
            "team_type": "club"
        }))
        .expect("implicit team");
        push_unique_team_entity(&mut output, 4, implicit, &mut identities);
        assert_eq!(
            output
                .iter()
                .filter(|row| row.entity_type == SpreadsheetEntityType::Team)
                .count(),
            1
        );
    }

    #[test]
    fn distinct_implicit_club_team_is_preserved() {
        let mut output = Vec::new();
        let mut identities = HashSet::new();
        for name in ["Atlético Mineiro", "Esporte Clube Bahia"] {
            let payload = serde_json::from_value::<Map<String, Value>>(json!({
                "official_name": name,
                "team_type": "club"
            }))
            .expect("team payload");
            push_unique_team_entity(&mut output, 4, payload, &mut identities);
        }
        assert_eq!(output.len(), 2);
    }

    #[test]
    fn physical_worksheet_row_number_survives_blank_rows() {
        let path = std::env::temp_dir().join(format!(
            "football-team-package-row-identity-{}.xlsx",
            std::process::id()
        ));
        let mut workbook = Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.set_name("球员与评分").expect("set player sheet name");
        write_key_row(sheet, PLAYER_KEYS).expect("write player keys");

        let zero_based_physical_row = 7_u32;
        let official_name_column = PLAYER_KEYS
            .iter()
            .position(|key| *key == "official_name")
            .expect("official_name column") as u16;
        sheet
            .write_string(
                zero_based_physical_row,
                official_name_column,
                "物理行号测试球员",
            )
            .expect("write player row");
        workbook.save(&path).expect("save row identity workbook");

        let mut workbook = open_workbook_auto(&path).expect("open row identity workbook");
        let rows =
            read_business_rows(&mut workbook, "球员与评分", PLAYER_KEYS).expect("read player rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, zero_based_physical_row + 1);
        assert_eq!(
            rows[0].1.get("official_name").and_then(Value::as_str),
            Some("物理行号测试球员")
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn team_package_template_round_trip() {
        let references = PlayerCatalogReferenceData {
            teams: Vec::new(),
            season_team_memberships: Vec::new(),
            formations: Vec::new(),
            providers: Vec::new(),
            positions: Vec::new(),
            ability_dimensions: Vec::new(),
            dynamic_tag_definitions: Vec::new(),
            upcoming_matches: Vec::new(),
            managed_matches: Vec::new(),
        };
        let path = std::env::temp_dir().join("football-team-package.xlsx");
        write_team_package_template(&path, &references).expect("write team package");
        let parsed = read_team_package_workbook(&path).expect("read team package");
        assert_eq!(parsed.format_version, TEAM_PACKAGE_FORMAT);
        assert!(parsed
            .rows
            .iter()
            .any(|row| row.entity_type == SpreadsheetEntityType::Team));
        assert!(parsed
            .rows
            .iter()
            .any(|row| row.entity_type == SpreadsheetEntityType::TeamName));
        assert!(parsed
            .rows
            .iter()
            .any(|row| row.entity_type == SpreadsheetEntityType::Player));
        assert!(parsed
            .rows
            .iter()
            .any(|row| row.entity_type == SpreadsheetEntityType::PlayerName));
        assert!(parsed
            .rows
            .iter()
            .any(|row| row.entity_type == SpreadsheetEntityType::Coach));
        let _ = fs::remove_file(path);
    }
}
