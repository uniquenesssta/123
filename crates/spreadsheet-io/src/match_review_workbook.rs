use crate::{SpreadsheetError, SpreadsheetResult};
use calamine::{open_workbook_auto, Data, Reader};
use chrono::{DateTime, Utc};
use football_domain::{
    AvailabilityStatus, LineupDraft, LineupPairDraft, LineupPlayerDraft, LineupRecord, LineupType,
    MatchEventRevisionStatus, MatchEventType, MatchEventVerificationStatus, MatchResultDraft,
    MatchReviewDraft, MatchReviewEventDraft, MatchReviewPackageComparison, MatchReviewPackageData,
    MatchReviewPackageDiffSummary, MatchReviewPackagePreview, PlayerMatchObservationDraft,
    PlayerPerformanceMetrics, SubstitutionDraft, MATCH_REVIEW_PACKAGE_FORMAT,
};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use uuid::Uuid;

const RESULT_HEADERS: &[&str] = &[
    "package_id",
    "match_id",
    "match_key",
    "competition_name",
    "kickoff_time",
    "home_team_id",
    "home_team_name",
    "away_team_id",
    "away_team_name",
    "source_run_id",
    "home_formation",
    "home_formation_id",
    "home_coach_id",
    "away_formation",
    "away_formation_id",
    "away_coach_id",
    "home_goals_90",
    "away_goals_90",
    "home_goals_extra_time",
    "away_goals_extra_time",
    "home_penalties",
    "away_penalties",
    "finalized_at",
    "data_coverage",
    "review_version",
    "source_urls",
    "notes",
];

const LINEUP_HEADERS: &[&str] = &[
    "team_side",
    "team_id",
    "team_name",
    "player_id",
    "player_name",
    "pre_match_in_squad",
    "pre_match_started",
    "in_matchday_squad",
    "started",
    "position_code",
    "role_code",
    "shirt_number",
    "minutes_played",
    "sequence_no",
    "bench_order",
    "membership_override",
    "entry_minute",
    "exit_minute",
    "exit_reason",
    "source_urls",
    "confidence",
    "notes",
];

const EVENT_HEADERS: &[&str] = &[
    "event_key",
    "sequence_no",
    "event_type",
    "team_id",
    "team_name",
    "player_id",
    "player_name",
    "related_player_id",
    "related_player_name",
    "minute",
    "stoppage_minute",
    "period",
    "home_score",
    "away_score",
    "verification_status",
    "revision_status",
    "verified_at",
    "source_document_id",
    "revision_of_event_id",
    "description",
    "source_urls",
    "confidence",
    "notes",
];

const PERFORMANCE_HEADERS: &[&str] = &[
    "team_id",
    "team_name",
    "player_id",
    "player_name",
    "started",
    "minutes_played",
    "provider_rating",
    "goals",
    "assists",
    "expected_goals",
    "expected_assists",
    "shots",
    "shots_on_target",
    "key_passes",
    "progressive_actions",
    "tackles",
    "interceptions",
    "clearances",
    "blocks",
    "duels_won",
    "duels_total",
    "fouls",
    "yellow_cards",
    "red_cards",
    "errors_leading_to_shot",
    "attack_contribution",
    "defence_contribution",
    "progression_organization",
    "chance_creation",
    "finishing",
    "positional_duty",
    "tactical_execution",
    "physical_condition",
    "key_event_impact",
    "confidence",
    "source_urls",
    "notes",
];

const PLAYER_REFERENCE_HEADERS: &[&str] = &[
    "team_id",
    "team_name",
    "player_id",
    "player_name",
    "localized_name",
    "position_code",
    "squad_number",
    "registration_status",
    "availability_status",
    "ability_average",
];

const SNAPSHOT_HEADERS: &[&str] = &[
    "snapshot_kind",
    "team_id",
    "team_name",
    "player_id",
    "player_name",
    "is_starter",
    "position_code",
    "role_code",
    "expected_minutes",
    "starting_probability",
    "detail",
];

pub fn write_match_review_package(
    output_path: &Path,
    data: &MatchReviewPackageData,
) -> SpreadsheetResult<()> {
    let mut workbook = Workbook::new();
    add_instructions(&mut workbook, data)?;
    add_metadata(&mut workbook, data)?;
    add_result_sheet(&mut workbook, data)?;
    add_actual_lineup_sheet(&mut workbook, data)?;
    add_event_sheet(&mut workbook)?;
    add_performance_sheet(&mut workbook, data)?;
    add_player_reference_sheet(&mut workbook, data)?;
    add_snapshot_sheet(&mut workbook, data)?;
    add_dictionary_sheet(&mut workbook)?;
    workbook.save(output_path)?;
    Ok(())
}

pub fn read_match_review_package(path: &Path) -> SpreadsheetResult<MatchReviewPackagePreview> {
    let bytes = fs::read(path)?;
    let source_sha256 = hex_digest(&bytes);
    let source_path = path.to_string_lossy().to_string();
    let source_file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let mut workbook =
        open_workbook_auto(path).map_err(|error| SpreadsheetError::Read(error.to_string()))?;
    let metadata = read_key_value_sheet(&mut workbook, "元数据")?;
    let format_version = required_text(&metadata, "format_version")?.to_string();
    if format_version != MATCH_REVIEW_PACKAGE_FORMAT {
        return Err(SpreadsheetError::InvalidTemplate(format!(
            "模板版本应为 {MATCH_REVIEW_PACKAGE_FORMAT}，实际为 {format_version}"
        )));
    }
    let package_id = parse_uuid(required_text(&metadata, "package_id")?, "package_id")?;
    let match_id = parse_uuid(required_text(&metadata, "match_id")?, "match_id")?;
    let match_key = required_text(&metadata, "match_key")?.to_string();
    let home_team_id = parse_uuid(required_text(&metadata, "home_team_id")?, "home_team_id")?;
    let away_team_id = parse_uuid(required_text(&metadata, "away_team_id")?, "away_team_id")?;

    let result_rows = read_table_sheet(&mut workbook, "比赛与赛果")?;
    let result = result_rows.first().ok_or_else(|| {
        SpreadsheetError::InvalidTemplate("比赛与赛果工作表没有数据行".to_string())
    })?;
    ensure_identity(result, "package_id", package_id)?;
    ensure_identity(result, "match_id", match_id)?;
    let home_team_name = text(result, "home_team_name").unwrap_or("主队").to_string();
    let away_team_name = text(result, "away_team_name").unwrap_or("客队").to_string();
    let finalized_at = parse_datetime(required(result, "finalized_at")?, "finalized_at")?;
    let data_coverage = parse_f64(text(result, "data_coverage"), 1.0, "data_coverage")?;
    let source_run_id = optional_uuid(text(result, "source_run_id"), "source_run_id")?;

    let lineup_rows = read_table_sheet(&mut workbook, "实际阵容")?;
    let performance_rows = read_table_sheet(&mut workbook, "球员表现")?;
    let event_rows = read_table_sheet(&mut workbook, "换人与事件")?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    if !(0.0..=1.0).contains(&data_coverage) {
        errors.push("数据覆盖率必须位于 0–1".to_string());
    }

    let home_formation = optional_string(text(result, "home_formation"));
    let away_formation = optional_string(text(result, "away_formation"));
    let home_formation_id = optional_uuid(text(result, "home_formation_id"), "home_formation_id")?;
    let away_formation_id = optional_uuid(text(result, "away_formation_id"), "away_formation_id")?;
    let home_coach_id = optional_uuid(text(result, "home_coach_id"), "home_coach_id")?;
    let away_coach_id = optional_uuid(text(result, "away_coach_id"), "away_coach_id")?;

    let home_players = parse_lineup_players(
        &lineup_rows,
        "home",
        home_team_id,
        &home_team_name,
        &mut errors,
    )?;
    let away_players = parse_lineup_players(
        &lineup_rows,
        "away",
        away_team_id,
        &away_team_name,
        &mut errors,
    )?;
    validate_lineup("主队", &home_players, &mut errors, &mut warnings);
    validate_lineup("客队", &away_players, &mut errors, &mut warnings);

    let captured_at = finalized_at;
    let lineup_pair = LineupPairDraft {
        home: LineupDraft {
            match_id,
            team_id: home_team_id,
            lineup_type: LineupType::Actual,
            snapshot_type: "T-1h".to_string(),
            formation: home_formation,
            formation_id: home_formation_id,
            coach_id: home_coach_id,
            captured_at,
            source_document_id: None,
            source_urls: split_urls(text(result, "source_urls")),
            quality_score: Some(data_coverage),
            metadata: json!({"source": "match_review_package", "package_id": package_id}),
            players: home_players.clone(),
        },
        away: LineupDraft {
            match_id,
            team_id: away_team_id,
            lineup_type: LineupType::Actual,
            snapshot_type: "T-1h".to_string(),
            formation: away_formation,
            formation_id: away_formation_id,
            coach_id: away_coach_id,
            captured_at,
            source_document_id: None,
            source_urls: split_urls(text(result, "source_urls")),
            quality_score: Some(data_coverage),
            metadata: json!({"source": "match_review_package", "package_id": package_id}),
            players: away_players.clone(),
        },
    };

    let all_players = home_players
        .iter()
        .chain(away_players.iter())
        .collect::<Vec<_>>();
    let events = parse_events(
        &event_rows,
        package_id,
        home_team_id,
        away_team_id,
        &home_players,
        &away_players,
        &mut errors,
    )?;
    let substitutions = events
        .iter()
        .filter(|event| event.event_type == MatchEventType::Substitution)
        .map(|event| SubstitutionDraft {
            team_id: event.team_id.unwrap_or(Uuid::nil()),
            player_out_id: event.player_id,
            player_in_id: event.related_player_id,
            minute: event.minute,
            period: event.period.clone(),
            reason: event.description.clone(),
            source_document_id: event.source_document_id,
            metadata: json!({
                "stoppage_minute": event.stoppage_minute,
                "source_urls": event.source_urls,
                "confidence": event.confidence,
                "package_id": package_id,
            }),
        })
        .collect::<Vec<_>>();
    for substitution in &substitutions {
        if substitution.team_id == Uuid::nil() {
            errors.push("换人事件必须填写 team_id".to_string());
        }
        if substitution.player_in_id.is_none() || substitution.player_out_id.is_none() {
            errors.push("换人事件必须同时填写换上和换下球员".to_string());
        }
    }

    let observations = parse_performance_rows(
        &performance_rows,
        &all_players,
        home_team_id,
        away_team_id,
        &mut errors,
    )?;
    validate_substitute_events(&all_players, &substitutions, &mut errors);
    let home_goals_90 = parse_i16(text(result, "home_goals_90"), 0, "home_goals_90")?;
    let away_goals_90 = parse_i16(text(result, "away_goals_90"), 0, "away_goals_90")?;
    let home_goals_extra_time = optional_i16(
        text(result, "home_goals_extra_time"),
        "home_goals_extra_time",
    )?;
    let away_goals_extra_time = optional_i16(
        text(result, "away_goals_extra_time"),
        "away_goals_extra_time",
    )?;
    validate_event_score_consistency(
        &events,
        home_goals_90,
        away_goals_90,
        home_goals_extra_time,
        away_goals_extra_time,
        home_team_id,
        away_team_id,
        &mut errors,
        &mut warnings,
    );

    let event_json = serde_json::to_value(&events)
        .map_err(|error| SpreadsheetError::InvalidTemplate(error.to_string()))?;
    let review = MatchReviewDraft {
        match_id,
        review_version: optional_string(text(result, "review_version")),
        data_coverage,
        source_run_id,
        result: MatchResultDraft {
            match_id,
            home_goals_90,
            away_goals_90,
            home_goals_extra_time,
            away_goals_extra_time,
            home_penalties: optional_i16(text(result, "home_penalties"), "home_penalties")?,
            away_penalties: optional_i16(text(result, "away_penalties"), "away_penalties")?,
            finalized_at,
            source_document_id: None,
            metadata: json!({
                "entry_mode": "match_review_package",
                "package_id": package_id,
                "source_urls": split_urls(text(result, "source_urls")),
                "events": event_json,
            }),
        },
        substitutions,
        events: events.clone(),
        player_observations: observations,
        notes: optional_string(text(result, "notes")),
    };

    let diff = build_diff(&lineup_rows, &home_players, &away_players);
    let home_starter_count = home_players.iter().filter(|item| item.is_starter).count() as u64;
    let away_starter_count = away_players.iter().filter(|item| item.is_starter).count() as u64;
    let preview = MatchReviewPackagePreview {
        source_path,
        source_file_name,
        source_sha256,
        format_version,
        package_id,
        match_id,
        match_key,
        home_team_name,
        away_team_name,
        lineup_pair,
        review,
        events,
        comparison: MatchReviewPackageComparison::default(),
        diff,
        warnings,
        ready: errors.is_empty(),
        errors,
        home_player_count: home_players.len() as u64,
        away_player_count: away_players.len() as u64,
        home_starter_count,
        away_starter_count,
        substitution_count: event_rows
            .iter()
            .filter(|row| text(row, "event_type") == Some("substitution"))
            .count() as u64,
        observation_count: performance_rows.len() as u64,
    };
    Ok(preview)
}

fn add_instructions(
    workbook: &mut Workbook,
    data: &MatchReviewPackageData,
) -> SpreadsheetResult<()> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("说明与校验")?;
    sheet.set_column_width(0, 24.0)?;
    sheet.set_column_width(1, 100.0)?;
    let title = Format::new()
        .set_bold()
        .set_font_size(18)
        .set_font_color(Color::RGB(0x0F766E));
    let head = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xE6F5F1))
        .set_border(FormatBorder::Thin);
    let wrap = Format::new().set_text_wrap();
    sheet.write_string_with_format(0, 0, "赛后复盘资料包", &title)?;
    let rows = [
        ("用途", "本资料包冻结赛前设定，并由外部助手补充真实比分、实际阵容、换人事件和球员表现。导回客户端后必须预检和人工确认。"),
        ("不可修改", "package_id、match_id、match_key、球队 ID、球员 ID、source_run_id 和赛前快照。需要新增实际球员时，从“球员参考”复制 player_id。"),
        ("第一步", "核对“比赛与赛果”，填写正式比分、确认时间、数据覆盖率、来源和备注。"),
        ("第二步", "在“实际阵容”修正比赛名单、首发、位置、分钟和号码。每队必须恰好 11 名首发。"),
        ("第三步", "在“换人与事件”填写所有换人；进球、红黄牌、伤退、VAR 和阵型变化也可记录。"),
        ("第四步", "在“球员表现”填写出场球员评分与可获取的技术数据。无法确认的字段留空，不得猜测。"),
        ("第五步", "回到客户端选择“导入并预检”，确认赛前与赛后差异、阻断错误和来源可信度，再正式写入。"),
    ];
    for (index, (label, value)) in rows.iter().enumerate() {
        let row = (index + 2) as u32;
        sheet.write_string_with_format(row, 0, *label, &head)?;
        sheet.write_string_with_format(row, 1, *value, &wrap)?;
    }
    let match_row = rows.len() as u32 + 2;
    sheet.write_string_with_format(match_row, 0, "比赛", &head)?;
    let match_label = format!(
        "{} vs {} · {}",
        data.match_record.home_team_name,
        data.match_record.away_team_name,
        data.match_record.kickoff_time
    );
    sheet.write_string_with_format(match_row, 1, &match_label, &wrap)?;
    Ok(())
}

fn add_metadata(workbook: &mut Workbook, data: &MatchReviewPackageData) -> SpreadsheetResult<()> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("元数据")?;
    sheet.set_column_width(0, 30.0)?;
    sheet.set_column_width(1, 80.0)?;
    let run_id = data
        .latest_model_run
        .as_ref()
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let rows = [
        ("format_version", MATCH_REVIEW_PACKAGE_FORMAT.to_string()),
        ("package_id", data.package_id.to_string()),
        ("match_id", data.match_record.id.to_string()),
        ("match_key", data.match_record.external_key.clone()),
        ("home_team_id", data.match_record.home_team_id.to_string()),
        ("away_team_id", data.match_record.away_team_id.to_string()),
        ("exported_at", data.exported_at.to_rfc3339()),
        ("source_run_id", run_id.to_string()),
        (
            "immutable_notice",
            "系统身份字段不可修改；导入时会再次核对。".to_string(),
        ),
    ];
    for (row, (key, value)) in rows.iter().enumerate() {
        sheet.write_string(row as u32, 0, *key)?;
        sheet.write_string(row as u32, 1, value)?;
    }
    Ok(())
}

fn add_result_sheet(
    workbook: &mut Workbook,
    data: &MatchReviewPackageData,
) -> SpreadsheetResult<()> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("比赛与赛果")?;
    write_headers(sheet, RESULT_HEADERS)?;
    let home = preferred_lineup(data, data.match_record.home_team_id);
    let away = preferred_lineup(data, data.match_record.away_team_id);
    let existing = data.existing_result.as_ref();
    let source_run_id = data
        .latest_model_run
        .as_ref()
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let values = vec![
        data.package_id.to_string(),
        data.match_record.id.to_string(),
        data.match_record.external_key.clone(),
        data.match_record
            .competition_name
            .clone()
            .unwrap_or_default(),
        data.match_record.kickoff_time.to_rfc3339(),
        data.match_record.home_team_id.to_string(),
        data.match_record.home_team_name.clone(),
        data.match_record.away_team_id.to_string(),
        data.match_record.away_team_name.clone(),
        source_run_id.to_string(),
        home.and_then(|item| {
            item.formation_code
                .clone()
                .or_else(|| item.formation.clone())
        })
        .unwrap_or_default(),
        home.and_then(|item| item.formation_id)
            .map(|value| value.to_string())
            .unwrap_or_default(),
        home.and_then(|item| item.coach_id)
            .map(|value| value.to_string())
            .unwrap_or_default(),
        away.and_then(|item| {
            item.formation_code
                .clone()
                .or_else(|| item.formation.clone())
        })
        .unwrap_or_default(),
        away.and_then(|item| item.formation_id)
            .map(|value| value.to_string())
            .unwrap_or_default(),
        away.and_then(|item| item.coach_id)
            .map(|value| value.to_string())
            .unwrap_or_default(),
        existing
            .map(|item| item.home_goals_90.to_string())
            .unwrap_or_default(),
        existing
            .map(|item| item.away_goals_90.to_string())
            .unwrap_or_default(),
        existing
            .and_then(|item| item.home_goals_extra_time)
            .map(|value| value.to_string())
            .unwrap_or_default(),
        existing
            .and_then(|item| item.away_goals_extra_time)
            .map(|value| value.to_string())
            .unwrap_or_default(),
        existing
            .and_then(|item| item.home_penalties)
            .map(|value| value.to_string())
            .unwrap_or_default(),
        existing
            .and_then(|item| item.away_penalties)
            .map(|value| value.to_string())
            .unwrap_or_default(),
        existing
            .map(|item| item.finalized_at.to_rfc3339())
            .unwrap_or_default(),
        data.latest_review
            .as_ref()
            .map(|item| item.data_coverage.to_string())
            .unwrap_or_else(|| "1".to_string()),
        data.latest_review
            .as_ref()
            .map(|item| item.review_version.clone())
            .unwrap_or_default(),
        String::new(),
        data.latest_review
            .as_ref()
            .and_then(|item| item.conclusions.get("notes"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
    ];
    write_row(sheet, 1, &values)?;
    Ok(())
}

fn add_actual_lineup_sheet(
    workbook: &mut Workbook,
    data: &MatchReviewPackageData,
) -> SpreadsheetResult<()> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("实际阵容")?;
    write_headers(sheet, LINEUP_HEADERS)?;
    let mut row = 1u32;
    for (side, team_id, team_name) in [
        (
            "home",
            data.match_record.home_team_id,
            data.match_record.home_team_name.as_str(),
        ),
        (
            "away",
            data.match_record.away_team_id,
            data.match_record.away_team_name.as_str(),
        ),
    ] {
        if let Some(lineup) = preferred_lineup(data, team_id) {
            for player in &lineup.players {
                let values = vec![
                    side.to_string(),
                    team_id.to_string(),
                    team_name.to_string(),
                    player.player_id.to_string(),
                    player.player_name.clone(),
                    "true".to_string(),
                    player.is_starter.to_string(),
                    "true".to_string(),
                    player.is_starter.to_string(),
                    player.position_code.clone().unwrap_or_default(),
                    player.role_code.clone().unwrap_or_default(),
                    player
                        .shirt_number
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    player
                        .actual_minutes
                        .or(player.expected_minutes)
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    player.sequence_no.to_string(),
                    player
                        .bench_order
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    player.membership_override.to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                    player.source_urls.join(" | "),
                    lineup.quality_score.unwrap_or(0.8).to_string(),
                    String::new(),
                ];
                write_row(sheet, row, &values)?;
                row += 1;
            }
        }
    }
    Ok(())
}

fn add_event_sheet(workbook: &mut Workbook) -> SpreadsheetResult<()> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("换人与事件")?;
    write_headers(sheet, EVENT_HEADERS)?;
    Ok(())
}

fn add_performance_sheet(
    workbook: &mut Workbook,
    data: &MatchReviewPackageData,
) -> SpreadsheetResult<()> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("球员表现")?;
    write_headers(sheet, PERFORMANCE_HEADERS)?;
    let mut row = 1u32;
    for team_id in [
        data.match_record.home_team_id,
        data.match_record.away_team_id,
    ] {
        if let Some(lineup) = preferred_lineup(data, team_id) {
            for player in &lineup.players {
                let values = vec![
                    team_id.to_string(),
                    lineup.team_name.clone(),
                    player.player_id.to_string(),
                    player.player_name.clone(),
                    player.is_starter.to_string(),
                    player
                        .actual_minutes
                        .or(player.expected_minutes)
                        .map(|value| value.to_string())
                        .unwrap_or_default(),
                    String::new(),
                    "0".to_string(),
                    "0".to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    "0.9".to_string(),
                    String::new(),
                    String::new(),
                ];
                write_row(sheet, row, &values)?;
                row += 1;
            }
        }
    }
    Ok(())
}

fn add_player_reference_sheet(
    workbook: &mut Workbook,
    data: &MatchReviewPackageData,
) -> SpreadsheetResult<()> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("球员参考")?;
    write_headers(sheet, PLAYER_REFERENCE_HEADERS)?;
    let mut row = 1u32;
    for (team, team_name) in [
        (&data.home_team, data.match_record.home_team_name.as_str()),
        (&data.away_team, data.match_record.away_team_name.as_str()),
    ] {
        for player in &team.squad {
            let values = vec![
                team.team.id.to_string(),
                team_name.to_string(),
                player.player_id.to_string(),
                player.player_name.clone(),
                player.localized_name.clone().unwrap_or_default(),
                player.position_code.clone().unwrap_or_default(),
                player
                    .squad_number
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                player.registration_status.clone(),
                player
                    .availability_status
                    .map(|value| value.as_str().to_string())
                    .unwrap_or_default(),
                player
                    .ability_average
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            ];
            write_row(sheet, row, &values)?;
            row += 1;
        }
    }
    Ok(())
}

fn add_snapshot_sheet(
    workbook: &mut Workbook,
    data: &MatchReviewPackageData,
) -> SpreadsheetResult<()> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("赛前快照")?;
    write_headers(sheet, SNAPSHOT_HEADERS)?;
    let mut row = 1u32;
    for lineup in &data.pre_match_lineups {
        if lineup.lineup_type == LineupType::Actual {
            continue;
        }
        for player in &lineup.players {
            let values = vec![
                format!("{} / {}", lineup.lineup_type.as_str(), lineup.snapshot_type),
                lineup.team_id.to_string(),
                lineup.team_name.clone(),
                player.player_id.to_string(),
                player.player_name.clone(),
                player.is_starter.to_string(),
                player.position_code.clone().unwrap_or_default(),
                player.role_code.clone().unwrap_or_default(),
                player
                    .expected_minutes
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                player
                    .starting_probability
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                format!(
                    "阵型 {}；捕获 {}",
                    lineup
                        .formation_code
                        .clone()
                        .or_else(|| lineup.formation.clone())
                        .unwrap_or_else(|| "未设置".to_string()),
                    lineup.captured_at
                ),
            ];
            write_row(sheet, row, &values)?;
            row += 1;
        }
    }
    if let Some(run) = &data.latest_model_run {
        let values = vec![
            "model_run".to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            serde_json::to_string(run).unwrap_or_default(),
        ];
        write_row(sheet, row, &values)?;
    }
    Ok(())
}

fn add_dictionary_sheet(workbook: &mut Workbook) -> SpreadsheetResult<()> {
    let sheet = workbook.add_worksheet();
    sheet.set_name("字段字典")?;
    sheet.set_column_width(0, 28.0)?;
    sheet.set_column_width(1, 110.0)?;
    let rows = [
        ("in_matchday_squad", "是否进入真实比赛名单。false 的行不会写入实际阵容。"),
        ("started", "是否真实首发；每队必须恰好 11 名。"),
        ("minutes_played", "真实出场分钟，0–150。替补出场者必须在换人事件中出现。"),
        ("provider_rating", "外部来源的 0–10 评分；出场球员必须填写。"),
        ("confidence", "字段可信度 0–1；无法确认时留空数据并降低可信度，不得猜测。"),
        ("event_key", "事件稳定身份。留空时按本资料包行号自动生成；修订同一事件时应保留原 event_key。"),
        ("sequence_no", "事件在本场比赛中的顺序，必须大于 0。"),
        ("event_type", "substitution / goal / own_goal / assist / penalty_goal / penalty_missed / yellow_card / second_yellow_card / red_card / injury / var / formation_change / goalkeeper_change / other。"),
        ("substitution", "换人事件中 player_id=换下球员，related_player_id=换上球员；team_id 必填。"),
        ("own_goal", "乌龙球的 team_id 填受益球队，player_id 填打入乌龙球的对方球员。"),
        ("assist", "助攻事件中 player_id=助攻球员，related_player_id=对应进球球员。"),
        ("home_score / away_score", "该事件确认后的主客队比分；必须同时填写或同时留空。"),
        ("verification_status", "unverified / verified / disputed。verified 时应填写 verified_at。"),
        ("revision_status", "active / corrected / cancelled；系统会把新资料包中缺失的旧事件标记为 superseded。"),
        ("period", "normal_time / first_half / second_half / extra_time_first / extra_time_second；补时分钟单独填 stoppage_minute。"),
        ("source_urls", "多个来源使用 | 分隔；如已建立正式来源文档，可同时填写 source_document_id。"),
        ("membership_override", "球员不在当前球队登记名单但确实出场时设为 true，并在 notes 说明。"),
    ];
    for (row, (key, description)) in rows.iter().enumerate() {
        sheet.write_string(row as u32, 0, *key)?;
        sheet.write_string(row as u32, 1, *description)?;
    }
    Ok(())
}

fn preferred_lineup(data: &MatchReviewPackageData, team_id: Uuid) -> Option<&LineupRecord> {
    data.pre_match_lineups
        .iter()
        .filter(|item| item.team_id == team_id)
        .min_by_key(|item| match item.lineup_type {
            LineupType::Actual => 0,
            LineupType::Confirmed => 1,
            LineupType::Expected => 2,
        })
}

fn write_headers(sheet: &mut Worksheet, headers: &[&str]) -> SpreadsheetResult<()> {
    let format = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x0F766E))
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);
    for (column, header) in headers.iter().enumerate() {
        sheet.write_string_with_format(0, column as u16, *header, &format)?;
        sheet.set_column_width(
            column as u16,
            if header.contains("notes")
                || header.contains("source")
                || header.contains("description")
            {
                30.0
            } else {
                18.0
            },
        )?;
    }
    sheet.set_freeze_panes(1, 0)?;
    Ok(())
}

fn write_row(sheet: &mut Worksheet, row: u32, values: &[String]) -> SpreadsheetResult<()> {
    for (column, value) in values.iter().enumerate() {
        sheet.write_string(row, column as u16, value)?;
    }
    Ok(())
}

fn read_key_value_sheet<R: std::io::Read + std::io::Seek>(
    workbook: &mut calamine::Sheets<R>,
    sheet_name: &str,
) -> SpreadsheetResult<HashMap<String, String>> {
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|error| SpreadsheetError::Read(error.to_string()))?;
    let mut values = HashMap::new();
    for row in range.rows() {
        let key = row.first().map(cell_text).unwrap_or_default();
        if key.trim().is_empty() {
            continue;
        }
        let value = row.get(1).map(cell_text).unwrap_or_default();
        values.insert(key.trim().to_string(), value.trim().to_string());
    }
    Ok(values)
}

fn read_table_sheet<R: std::io::Read + std::io::Seek>(
    workbook: &mut calamine::Sheets<R>,
    sheet_name: &str,
) -> SpreadsheetResult<Vec<HashMap<String, String>>> {
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|error| SpreadsheetError::Read(error.to_string()))?;
    let mut rows = range.rows();
    let headers = rows
        .next()
        .ok_or_else(|| SpreadsheetError::InvalidTemplate(format!("{sheet_name} 工作表缺少表头")))?
        .iter()
        .map(cell_text)
        .collect::<Vec<_>>();
    let mut result = Vec::new();
    for row in rows {
        let mut item = HashMap::new();
        let mut has_value = false;
        for (index, header) in headers.iter().enumerate() {
            if header.trim().is_empty() {
                continue;
            }
            let value = row.get(index).map(cell_text).unwrap_or_default();
            if !value.trim().is_empty() {
                has_value = true;
            }
            item.insert(header.trim().to_string(), value.trim().to_string());
        }
        if has_value {
            result.push(item);
        }
    }
    Ok(result)
}

fn cell_text(value: &Data) -> String {
    match value {
        Data::Empty => String::new(),
        Data::String(value) => value.clone(),
        Data::Float(value) => {
            if value.fract() == 0.0 {
                format!("{value:.0}")
            } else {
                value.to_string()
            }
        }
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::Error(value) => format!("{value:?}"),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) => value.clone(),
        Data::DurationIso(value) => value.clone(),
    }
}

fn required_text<'a>(values: &'a HashMap<String, String>, key: &str) -> SpreadsheetResult<&'a str> {
    values
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| SpreadsheetError::InvalidTemplate(format!("元数据缺少 {key}")))
}

fn text<'a>(row: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    row.get(key)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn required<'a>(row: &'a HashMap<String, String>, key: &str) -> SpreadsheetResult<&'a str> {
    text(row, key).ok_or_else(|| SpreadsheetError::InvalidTemplate(format!("字段 {key} 不能为空")))
}

fn parse_uuid(value: &str, field: &str) -> SpreadsheetResult<Uuid> {
    Uuid::parse_str(value.trim())
        .map_err(|_| SpreadsheetError::InvalidTemplate(format!("{field} 不是有效 UUID：{value}")))
}

fn optional_uuid(value: Option<&str>, field: &str) -> SpreadsheetResult<Option<Uuid>> {
    value.map(|value| parse_uuid(value, field)).transpose()
}

fn ensure_identity(
    row: &HashMap<String, String>,
    field: &str,
    expected: Uuid,
) -> SpreadsheetResult<()> {
    let actual = parse_uuid(required(row, field)?, field)?;
    if actual != expected {
        return Err(SpreadsheetError::InvalidTemplate(format!(
            "{field} 与元数据不一致，资料包身份可能被修改"
        )));
    }
    Ok(())
}

fn parse_datetime(value: &str, field: &str) -> SpreadsheetResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value.trim())
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| {
            SpreadsheetError::InvalidTemplate(format!(
                "{field} 必须使用 RFC3339 时间，例如 2026-07-21T23:00:00+09:00"
            ))
        })
}

fn parse_bool(value: Option<&str>, default: bool) -> bool {
    match value.map(|item| item.trim().to_ascii_lowercase()) {
        Some(value) if matches!(value.as_str(), "true" | "1" | "yes" | "是" | "y") => true,
        Some(value) if matches!(value.as_str(), "false" | "0" | "no" | "否" | "n") => false,
        _ => default,
    }
}

fn parse_i16(value: Option<&str>, default: i16, field: &str) -> SpreadsheetResult<i16> {
    match value {
        Some(value) => value
            .parse::<i16>()
            .map_err(|_| SpreadsheetError::InvalidTemplate(format!("{field} 必须是整数：{value}"))),
        None => Ok(default),
    }
}

fn optional_i16(value: Option<&str>, field: &str) -> SpreadsheetResult<Option<i16>> {
    value
        .map(|value| {
            value.parse::<i16>().map_err(|_| {
                SpreadsheetError::InvalidTemplate(format!("{field} 必须是整数：{value}"))
            })
        })
        .transpose()
}

fn parse_f64(value: Option<&str>, default: f64, field: &str) -> SpreadsheetResult<f64> {
    match value {
        Some(value) => value
            .parse::<f64>()
            .map_err(|_| SpreadsheetError::InvalidTemplate(format!("{field} 必须是数字：{value}"))),
        None => Ok(default),
    }
}

fn optional_f64(value: Option<&str>, field: &str) -> SpreadsheetResult<Option<f64>> {
    value
        .map(|value| {
            value.parse::<f64>().map_err(|_| {
                SpreadsheetError::InvalidTemplate(format!("{field} 必须是数字：{value}"))
            })
        })
        .transpose()
}

fn optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn split_urls(value: Option<&str>) -> Vec<String> {
    value
        .unwrap_or_default()
        .split(['|', '\n', ';'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn availability(value: Option<&str>) -> Option<AvailabilityStatus> {
    match value.unwrap_or_default().trim() {
        "available" => Some(AvailabilityStatus::Available),
        "doubtful" => Some(AvailabilityStatus::Doubtful),
        "unavailable" => Some(AvailabilityStatus::Unavailable),
        "injured" => Some(AvailabilityStatus::Injured),
        "suspended" => Some(AvailabilityStatus::Suspended),
        "rested" => Some(AvailabilityStatus::Rested),
        "returning" => Some(AvailabilityStatus::Returning),
        "unknown" => Some(AvailabilityStatus::Unknown),
        _ => None,
    }
}

fn parse_lineup_players(
    rows: &[HashMap<String, String>],
    side: &str,
    team_id: Uuid,
    team_name: &str,
    errors: &mut Vec<String>,
) -> SpreadsheetResult<Vec<LineupPlayerDraft>> {
    let mut players = Vec::new();
    let mut ids = HashSet::new();
    for row in rows
        .iter()
        .filter(|row| text(row, "team_side") == Some(side))
    {
        if !parse_bool(text(row, "in_matchday_squad"), true) {
            continue;
        }
        let row_team_id = optional_uuid(text(row, "team_id"), "team_id")?.unwrap_or(team_id);
        if row_team_id != team_id {
            errors.push(format!("{team_name} 阵容中存在不属于该侧的 team_id"));
            continue;
        }
        let player_id = parse_uuid(required(row, "player_id")?, "player_id")?;
        if !ids.insert(player_id) {
            errors.push(format!("{team_name} 阵容中球员 {player_id} 重复"));
            continue;
        }
        let minutes = optional_i16(text(row, "minutes_played"), "minutes_played")?;
        let is_starter = parse_bool(text(row, "started"), false);
        players.push(LineupPlayerDraft {
            player_id,
            position_code: optional_string(text(row, "position_code")),
            role_code: optional_string(text(row, "role_code")),
            is_starter,
            shirt_number: optional_i16(text(row, "shirt_number"), "shirt_number")?,
            expected_minutes: None,
            actual_minutes: minutes,
            sequence_no: parse_i16(
                text(row, "sequence_no"),
                players.len() as i16 + 1,
                "sequence_no",
            )?,
            bench_order: optional_i16(text(row, "bench_order"), "bench_order")?,
            availability_status: availability(text(row, "availability_status")),
            starting_probability: Some(if is_starter { 1.0 } else { 0.0 }),
            membership_override: parse_bool(text(row, "membership_override"), false),
            source_urls: split_urls(text(row, "source_urls")),
            metadata: json!({
                "team_side": side,
                "entry_minute": optional_i16(text(row, "entry_minute"), "entry_minute")?,
                "exit_minute": optional_i16(text(row, "exit_minute"), "exit_minute")?,
                "exit_reason": optional_string(text(row, "exit_reason")),
                "confidence": parse_f64(text(row, "confidence"), 0.9, "confidence")?,
                "notes": optional_string(text(row, "notes")),
            }),
        });
    }
    Ok(players)
}

fn validate_lineup(
    label: &str,
    players: &[LineupPlayerDraft],
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    if !(11..=30).contains(&players.len()) {
        errors.push(format!(
            "{label}比赛名单必须为 11–30 人，当前 {} 人",
            players.len()
        ));
    }
    let starters = players.iter().filter(|item| item.is_starter).count();
    if starters != 11 {
        errors.push(format!("{label}必须恰好 11 名首发，当前 {starters} 名"));
    }
    for player in players {
        if player
            .actual_minutes
            .is_some_and(|value| !(0..=150).contains(&value))
        {
            errors.push(format!(
                "{label}球员 {} 出场分钟必须位于 0–150",
                player.player_id
            ));
        }
    }
    let total_minutes: i32 = players
        .iter()
        .filter_map(|item| item.actual_minutes)
        .map(i32::from)
        .sum();
    if total_minutes > 0 && !(850..=1200).contains(&total_minutes) {
        warnings.push(format!(
            "{label}总出场分钟为 {total_minutes}，请核对加时、红牌和分钟口径"
        ));
    }
}

fn parse_events(
    rows: &[HashMap<String, String>],
    package_id: Uuid,
    home_team_id: Uuid,
    away_team_id: Uuid,
    home_players: &[LineupPlayerDraft],
    away_players: &[LineupPlayerDraft],
    errors: &mut Vec<String>,
) -> SpreadsheetResult<Vec<MatchReviewEventDraft>> {
    let mut events = Vec::new();
    let mut event_keys = HashSet::new();
    let mut event_sequences = HashSet::new();
    let valid_periods = [
        "normal_time",
        "first_half",
        "second_half",
        "extra_time_first",
        "extra_time_second",
    ];
    let player_teams = home_players
        .iter()
        .map(|player| (player.player_id, home_team_id))
        .chain(
            away_players
                .iter()
                .map(|player| (player.player_id, away_team_id)),
        )
        .collect::<HashMap<_, _>>();

    for (index, row) in rows.iter().enumerate() {
        let spreadsheet_row = index + 2;
        let event_type = required(row, "event_type")?
            .parse::<MatchEventType>()
            .map_err(SpreadsheetError::InvalidTemplate)?;
        let confidence = parse_f64(text(row, "confidence"), 1.0, "confidence")?;
        if !(0.0..=1.0).contains(&confidence) {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行可信度必须位于 0–1"
            ));
        }
        let minute = parse_i16(text(row, "minute"), 0, "minute")?;
        if !(0..=150).contains(&minute) {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行分钟必须位于 0–150"
            ));
        }
        let stoppage_minute = optional_i16(text(row, "stoppage_minute"), "stoppage_minute")?;
        if stoppage_minute.is_some_and(|value| !(0..=30).contains(&value)) {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行补时分钟必须位于 0–30"
            ));
        }
        let period =
            optional_string(text(row, "period")).unwrap_or_else(|| "normal_time".to_string());
        if !valid_periods.contains(&period.as_str()) {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行 period 无效：{period}"
            ));
        }
        let team_id = optional_uuid(text(row, "team_id"), "team_id")?;
        if event_type.requires_team() && team_id.is_none() {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行 {} 必须填写 team_id",
                event_type.as_str()
            ));
        }
        if team_id.is_some_and(|value| value != home_team_id && value != away_team_id) {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行 team_id 不属于本场主客队"
            ));
        }

        let player_id = optional_uuid(text(row, "player_id"), "player_id")?;
        let related_player_id = optional_uuid(text(row, "related_player_id"), "related_player_id")?;
        if event_type.requires_player() && player_id.is_none() {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行 {} 必须填写 player_id",
                event_type.as_str()
            ));
        }
        for (label, player) in [
            ("player_id", player_id),
            ("related_player_id", related_player_id),
        ] {
            if player.is_some_and(|value| !player_teams.contains_key(&value)) {
                errors.push(format!(
                    "换人与事件第 {spreadsheet_row} 行 {label} 不在本场实际名单中"
                ));
            }
        }
        if matches!(
            event_type,
            MatchEventType::Substitution | MatchEventType::GoalkeeperChange
        ) {
            if player_id.is_none() || related_player_id.is_none() {
                let label = if event_type == MatchEventType::Substitution {
                    "换人"
                } else {
                    "门将更换"
                };
                errors.push(format!(
                    "换人与事件第 {spreadsheet_row} 行{label}必须同时填写离场与入场球员"
                ));
            }
            if player_id.is_some() && player_id == related_player_id {
                errors.push(format!(
                    "换人与事件第 {spreadsheet_row} 行离场与入场球员不能相同"
                ));
            }
        }
        if let (Some(team_id), Some(player_id)) = (team_id, player_id) {
            if let Some(player_team_id) = player_teams.get(&player_id) {
                let expected_same_team = event_type != MatchEventType::OwnGoal;
                if (expected_same_team && *player_team_id != team_id)
                    || (!expected_same_team && *player_team_id == team_id)
                {
                    errors.push(format!(
                        "换人与事件第 {spreadsheet_row} 行球员与事件球队不匹配"
                    ));
                }
            }
        }
        if event_type == MatchEventType::Assist && related_player_id.is_none() {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行助攻事件应在 related_player_id 填写进球球员"
            ));
        }

        let home_score = optional_i16(text(row, "home_score"), "home_score")?;
        let away_score = optional_i16(text(row, "away_score"), "away_score")?;
        if home_score.is_some() != away_score.is_some() {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行主客队事件后比分必须同时填写或同时留空"
            ));
        }
        if home_score.is_some_and(|value| value < 0) || away_score.is_some_and(|value| value < 0) {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行事件后比分不能为负数"
            ));
        }

        let verification_status = optional_string(text(row, "verification_status"))
            .unwrap_or_else(|| "unverified".to_string())
            .parse::<MatchEventVerificationStatus>()
            .map_err(SpreadsheetError::InvalidTemplate)?;
        let revision_status = optional_string(text(row, "revision_status"))
            .unwrap_or_else(|| "active".to_string())
            .parse::<MatchEventRevisionStatus>()
            .map_err(SpreadsheetError::InvalidTemplate)?;
        let verified_at = text(row, "verified_at")
            .map(|value| parse_datetime(value, "verified_at"))
            .transpose()?;
        if verification_status == MatchEventVerificationStatus::Verified && verified_at.is_none() {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行已核验事件必须填写 verified_at"
            ));
        }

        if revision_status == MatchEventRevisionStatus::Superseded {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行不能手工填写 superseded；该状态由系统维护"
            ));
        }

        let event_key = optional_string(text(row, "event_key"))
            .unwrap_or_else(|| format!("{package_id}:event:{spreadsheet_row}"));
        if !event_keys.insert(event_key.clone()) {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行 event_key 重复：{event_key}"
            ));
        }
        let sequence_no = optional_i16(text(row, "sequence_no"), "sequence_no")?
            .map(i32::from)
            .unwrap_or(index as i32 + 1);
        if sequence_no <= 0 {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行 sequence_no 必须大于 0"
            ));
        } else if !event_sequences.insert(sequence_no) {
            errors.push(format!(
                "换人与事件第 {spreadsheet_row} 行 sequence_no 重复：{sequence_no}"
            ));
        }

        events.push(MatchReviewEventDraft {
            event_key: Some(event_key),
            sequence_no: Some(sequence_no),
            event_type,
            team_id,
            player_id,
            related_player_id,
            minute,
            stoppage_minute,
            period,
            home_score,
            away_score,
            verification_status,
            revision_status,
            verified_at,
            source_document_id: optional_uuid(
                text(row, "source_document_id"),
                "source_document_id",
            )?,
            source_package_id: Some(package_id),
            revision_of_event_id: optional_uuid(
                text(row, "revision_of_event_id"),
                "revision_of_event_id",
            )?,
            description: optional_string(text(row, "description")),
            source_urls: split_urls(text(row, "source_urls")),
            confidence,
            metadata: json!({"notes": optional_string(text(row, "notes"))}),
        });
    }
    Ok(events)
}

fn parse_performance_rows(
    rows: &[HashMap<String, String>],
    lineup_players: &[&LineupPlayerDraft],
    home_team_id: Uuid,
    away_team_id: Uuid,
    errors: &mut Vec<String>,
) -> SpreadsheetResult<Vec<PlayerMatchObservationDraft>> {
    let by_player = rows
        .iter()
        .filter_map(|row| {
            optional_uuid(text(row, "player_id"), "player_id")
                .ok()
                .flatten()
                .map(|id| (id, row))
        })
        .collect::<HashMap<_, _>>();
    let mut observations = Vec::new();
    for player in lineup_players {
        let row = by_player.get(&player.player_id).copied();
        let minutes = player.actual_minutes.unwrap_or(0);
        let rating = row
            .and_then(|value| text(value, "provider_rating"))
            .map(|value| value.parse::<f64>())
            .transpose()
            .map_err(|_| {
                SpreadsheetError::InvalidTemplate(format!(
                    "球员 {} 的 provider_rating 必须是数字",
                    player.player_id
                ))
            })?;
        if minutes > 0 && rating.is_none() {
            errors.push(format!("出场球员 {} 缺少 0–10 评分", player.player_id));
        }
        if rating.is_some_and(|value| !(0.0..=10.0).contains(&value)) {
            errors.push(format!("球员 {} 的评分必须位于 0–10", player.player_id));
        }
        let team_id = row
            .and_then(|value| text(value, "team_id"))
            .map(|value| parse_uuid(value, "team_id"))
            .transpose()?
            .unwrap_or_else(|| {
                if player.metadata.get("team_side").and_then(Value::as_str) == Some("away") {
                    away_team_id
                } else {
                    home_team_id
                }
            });
        let source_urls = row
            .map(|value| split_urls(text(value, "source_urls")))
            .unwrap_or_default();
        let confidence = row
            .map(|value| {
                parse_f64(
                    text(value, "confidence"),
                    if minutes > 0 { 0.9 } else { 0.6 },
                    "confidence",
                )
            })
            .transpose()?
            .unwrap_or(if minutes > 0 { 0.9 } else { 0.6 });
        let metric = |key: &str| -> SpreadsheetResult<f64> {
            row.map(|value| parse_f64(text(value, key), 0.0, key))
                .transpose()
                .map(|value| value.unwrap_or(0.0))
        };
        let dimension = |key: &str| -> SpreadsheetResult<Option<f64>> {
            row.map(|value| optional_f64(text(value, key), key))
                .transpose()
                .map(|value| value.flatten())
        };
        let extra = json!({
            "attack_contribution": dimension("attack_contribution")?,
            "defence_contribution": dimension("defence_contribution")?,
            "progression_organization": dimension("progression_organization")?,
            "chance_creation": dimension("chance_creation")?,
            "finishing": dimension("finishing")?,
            "positional_duty": dimension("positional_duty")?,
            "tactical_execution": dimension("tactical_execution")?,
            "physical_condition": dimension("physical_condition")?,
            "key_event_impact": dimension("key_event_impact")?,
            "source_urls": source_urls,
            "notes": row.and_then(|value| optional_string(text(value, "notes"))),
        });
        observations.push(PlayerMatchObservationDraft {
            player_id: player.player_id,
            team_id,
            position_code: player.position_code.clone(),
            role_code: player.role_code.clone(),
            started: player.is_starter,
            minutes_played: minutes,
            performance_score: None,
            input_confidence: confidence,
            metrics: PlayerPerformanceMetrics {
                goals: metric("goals")?,
                assists: metric("assists")?,
                expected_goals: metric("expected_goals")?,
                expected_assists: metric("expected_assists")?,
                shots: metric("shots")?,
                shots_on_target: metric("shots_on_target")?,
                key_passes: metric("key_passes")?,
                progressive_actions: metric("progressive_actions")?,
                tackles: metric("tackles")?,
                interceptions: metric("interceptions")?,
                clearances: metric("clearances")?,
                blocks: metric("blocks")?,
                duels_won: metric("duels_won")?,
                duels_total: metric("duels_total")?,
                fouls: metric("fouls")?,
                yellow_cards: metric("yellow_cards")?,
                red_cards: metric("red_cards")?,
                errors_leading_to_shot: metric("errors_leading_to_shot")?,
                provider_rating: rating,
                extra,
            },
            source_document_id: None,
        });
    }
    Ok(observations)
}

fn validate_substitute_events(
    players: &[&LineupPlayerDraft],
    substitutions: &[SubstitutionDraft],
    errors: &mut Vec<String>,
) {
    let incoming = substitutions
        .iter()
        .filter_map(|item| item.player_in_id)
        .collect::<HashSet<_>>();
    for player in players
        .iter()
        .filter(|item| !item.is_starter && item.actual_minutes.unwrap_or(0) > 0)
    {
        if !incoming.contains(&player.player_id) {
            errors.push(format!(
                "替补出场球员 {} 缺少对应换人事件",
                player.player_id
            ));
        }
    }
}

fn validate_event_score_consistency(
    events: &[MatchReviewEventDraft],
    home_goals: i16,
    away_goals: i16,
    home_goals_extra_time: Option<i16>,
    away_goals_extra_time: Option<i16>,
    home_team_id: Uuid,
    away_team_id: Uuid,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let effective_events = events
        .iter()
        .filter(|event| event.revision_status.is_effective())
        .collect::<Vec<_>>();
    let scoring_events = effective_events
        .iter()
        .copied()
        .filter(|event| event.event_type.counts_toward_score())
        .collect::<Vec<_>>();
    let is_extra_time = |period: &str| matches!(period, "extra_time_first" | "extra_time_second");

    let regulation_scoring_events = scoring_events
        .iter()
        .copied()
        .filter(|event| !is_extra_time(event.period.trim()))
        .collect::<Vec<_>>();
    let home_count = regulation_scoring_events
        .iter()
        .filter(|event| event.team_id == Some(home_team_id))
        .count() as i16;
    let away_count = regulation_scoring_events
        .iter()
        .filter(|event| event.team_id == Some(away_team_id))
        .count() as i16;
    if !regulation_scoring_events.is_empty()
        && (home_count != home_goals || away_count != away_goals)
    {
        warnings.push(format!(
            "90 分钟有效进球事件计数为 {home_count}-{away_count}，与 90 分钟比分 {home_goals}-{away_goals} 不一致；请检查乌龙球或事件遗漏"
        ));
    }

    let extra_scoring_events = scoring_events
        .iter()
        .copied()
        .filter(|event| is_extra_time(event.period.trim()))
        .collect::<Vec<_>>();
    if !extra_scoring_events.is_empty() {
        match (home_goals_extra_time, away_goals_extra_time) {
            (Some(home_extra), Some(away_extra)) => {
                let home_extra_count = extra_scoring_events
                    .iter()
                    .filter(|event| event.team_id == Some(home_team_id))
                    .count() as i16;
                let away_extra_count = extra_scoring_events
                    .iter()
                    .filter(|event| event.team_id == Some(away_team_id))
                    .count() as i16;
                if home_extra_count != home_extra || away_extra_count != away_extra {
                    warnings.push(format!(
                        "加时有效进球事件计数为 {home_extra_count}-{away_extra_count}，与加时进球 {home_extra}-{away_extra} 不一致"
                    ));
                }
            }
            _ => errors.push(
                "存在加时阶段的进球或比分事件，但比赛与赛果未同时填写主客队加时进球".to_string(),
            ),
        }
    }

    let mut previous_score = (0i16, 0i16);
    let mut latest_regulation_score = None;
    let mut latest_overall_score = None;
    let mut has_extra_time_score = false;
    let mut ordered = effective_events;
    ordered.sort_by_key(|event| {
        (
            event.sequence_no.unwrap_or(i32::MAX),
            event.minute,
            event.stoppage_minute.unwrap_or_default(),
        )
    });
    for event in ordered {
        let (Some(home_score), Some(away_score)) = (event.home_score, event.away_score) else {
            continue;
        };
        if home_score < previous_score.0 || away_score < previous_score.1 {
            errors.push(format!(
                "事件 {} 的比分 {}-{} 早于前序比分 {}-{}；请修正事件顺序，或将被取消事件标记为 cancelled/corrected",
                event.event_key.as_deref().unwrap_or("未命名事件"),
                home_score,
                away_score,
                previous_score.0,
                previous_score.1,
            ));
        }
        previous_score = (home_score, away_score);
        latest_overall_score = Some(previous_score);
        if is_extra_time(event.period.trim()) {
            has_extra_time_score = true;
        } else {
            latest_regulation_score = Some(previous_score);
        }
    }
    if let Some((latest_home, latest_away)) = latest_regulation_score {
        if latest_home != home_goals || latest_away != away_goals {
            errors.push(format!(
                "最后一条 90 分钟有效事件后比分为 {latest_home}-{latest_away}，与 90 分钟赛果 {home_goals}-{away_goals} 不一致"
            ));
        }
    }
    if has_extra_time_score {
        if let (Some(home_extra), Some(away_extra), Some((latest_home, latest_away))) = (
            home_goals_extra_time,
            away_goals_extra_time,
            latest_overall_score,
        ) {
            let expected_home = home_goals + home_extra;
            let expected_away = away_goals + away_extra;
            if latest_home != expected_home || latest_away != expected_away {
                errors.push(format!(
                    "最后一条加时有效事件后比分为 {latest_home}-{latest_away}，与 90 分钟及加时合计赛果 {expected_home}-{expected_away} 不一致"
                ));
            }
        }
    } else if latest_regulation_score.is_none() {
        if let Some((latest_home, latest_away)) = latest_overall_score {
            if latest_home != home_goals || latest_away != away_goals {
                errors.push(format!(
                    "最后一条有效事件后比分为 {latest_home}-{latest_away}，与 90 分钟赛果 {home_goals}-{away_goals} 不一致"
                ));
            }
        }
    }
}

fn build_diff(
    rows: &[HashMap<String, String>],
    home_players: &[LineupPlayerDraft],
    away_players: &[LineupPlayerDraft],
) -> MatchReviewPackageDiffSummary {
    let current = home_players
        .iter()
        .chain(away_players.iter())
        .map(|item| item.player_id)
        .collect::<HashSet<_>>();
    let pre_match = rows
        .iter()
        .filter(|row| parse_bool(text(row, "pre_match_in_squad"), false))
        .filter_map(|row| {
            optional_uuid(text(row, "player_id"), "player_id")
                .ok()
                .flatten()
        })
        .collect::<HashSet<_>>();
    let names = rows
        .iter()
        .filter_map(|row| {
            optional_uuid(text(row, "player_id"), "player_id")
                .ok()
                .flatten()
                .map(|id| {
                    (
                        id,
                        text(row, "player_name").unwrap_or("未命名球员").to_string(),
                    )
                })
        })
        .collect::<HashMap<_, _>>();
    let side_names = |side: &str, added: bool| -> Vec<String> {
        rows.iter()
            .filter(|row| text(row, "team_side") == Some(side))
            .filter_map(|row| {
                let id = optional_uuid(text(row, "player_id"), "player_id")
                    .ok()
                    .flatten()?;
                let pre = parse_bool(text(row, "pre_match_started"), false);
                let actual = if side == "home" {
                    home_players
                } else {
                    away_players
                }
                .iter()
                .find(|item| item.player_id == id)
                .is_some_and(|item| item.is_starter);
                if (added && actual && !pre) || (!added && pre && !actual) {
                    Some(names.get(&id).cloned().unwrap_or_else(|| id.to_string()))
                } else {
                    None
                }
            })
            .collect()
    };
    MatchReviewPackageDiffSummary {
        home_added_starters: side_names("home", true),
        home_removed_starters: side_names("home", false),
        away_added_starters: side_names("away", true),
        away_removed_starters: side_names("away", false),
        added_matchday_players: current
            .difference(&pre_match)
            .map(|id| names.get(id).cloned().unwrap_or_else(|| id.to_string()))
            .collect(),
        removed_matchday_players: pre_match
            .difference(&current)
            .map(|id| names.get(id).cloned().unwrap_or_else(|| id.to_string()))
            .collect(),
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
