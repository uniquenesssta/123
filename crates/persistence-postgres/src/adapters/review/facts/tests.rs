use super::{
    identity::{match_event_key, match_review_package_id},
    summarize_match_events, validate_review_draft,
};
use crate::PersistenceError;
use football_domain::{
    MatchEventRevisionStatus, MatchEventVerificationStatus, MatchReviewDraft,
    MatchReviewEventDraft, MatchReviewEventRecord,
};
use serde_json::json;
use uuid::Uuid;

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn draft() -> MatchReviewDraft {
    serde_json::from_value(json!({
        "match_id": id(1), "result": {"match_id": id(1), "home_goals_90": 1,
            "away_goals_90": 0, "finalized_at": "2026-10-12T00:00:00Z"},
        "player_observations": [{"player_id": id(5), "team_id": id(3), "started": true,
            "minutes_played": 90, "performance_score": 70.0}]
    }))
    .unwrap()
}

fn event(sequence: i32) -> MatchReviewEventDraft {
    serde_json::from_value(json!({"event_type": "goal", "sequence_no": sequence,
        "team_id": id(3), "player_id": id(5), "minute": 12}))
    .unwrap()
}

fn invalid(draft: &MatchReviewDraft, expected: &str) {
    match validate_review_draft(draft).unwrap_err() {
        PersistenceError::InvalidState(message) => assert!(message.contains(expected), "{message}"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn event_identity_keeps_explicit_keys_and_excludes_descriptive_updates() {
    let original = event(1);
    let key = match_event_key(&original, id(1), 0).unwrap();
    assert!(key.starts_with("generated:"));
    assert_eq!(key.len(), 74);
    assert!(key[10..]
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));
    let mut changed = original.clone();
    changed.event_key = Some("  ".into());
    changed.description = Some("corrected description".into());
    changed.home_score = Some(1);
    changed.away_score = Some(0);
    changed.verification_status = MatchEventVerificationStatus::Verified;
    changed.verified_at = Some("2026-10-12T00:00:00Z".parse().unwrap());
    changed.confidence = 0.8;
    changed.metadata = json!({"revision": 2});
    changed.source_urls = vec!["https://example.com/event".into()];
    assert_eq!(match_event_key(&changed, id(1), 9).unwrap(), key);
    changed.event_key = Some("  manual:event  ".into());
    assert_eq!(
        match_event_key(&changed, id(99), 9).unwrap(),
        "manual:event"
    );
    changed.event_key = None;
    changed.minute += 1;
    assert_ne!(match_event_key(&changed, id(1), 0).unwrap(), key);
    assert_ne!(match_event_key(&original, id(2), 0).unwrap(), key);
    let mut implicit = original;
    implicit.sequence_no = None;
    assert_eq!(match_event_key(&implicit, id(1), 0).unwrap(), key);
    assert_ne!(match_event_key(&implicit, id(1), 1).unwrap(), key);
}

#[test]
fn source_package_identity_preserves_optional_top_level_uuid_contract() {
    assert_eq!(
        match_review_package_id(&json!({"package_id": id(7)})),
        Some(id(7))
    );
    for value in [
        json!(null),
        json!({}),
        json!({"package_id": "invalid"}),
        json!({"package_id": 7}),
        json!({"review_package_confirmation": {"package_id": id(7)}}),
    ] {
        assert_eq!(match_review_package_id(&value), None);
    }
}

#[test]
fn draft_result_coverage_and_duplicate_observations_are_rejected() {
    assert!(validate_review_draft(&draft()).is_ok());
    for case in 0..8 {
        let mut changed = draft();
        let expected = match case {
            0 => {
                changed.result.match_id = id(2);
                "正式赛果与复盘比赛 ID 不一致"
            }
            1 => {
                changed.result.home_goals_90 = -1;
                "90 分钟进球不能为负数"
            }
            2 => {
                changed.result.home_penalties = Some(-1);
                "加时或点球进球不能为负数"
            }
            3 => {
                changed.result.home_goals_extra_time = Some(0);
                "主客队加时进球必须同时填写或同时留空"
            }
            4 => {
                changed.result.home_penalties = Some(0);
                "主客队点球大战进球必须同时填写或同时留空"
            }
            5 => {
                changed.data_coverage = f64::NAN;
                "数据覆盖率必须位于 0–1"
            }
            6 => {
                changed.player_observations.clear();
                "至少需要一条球员赛后表现"
            }
            _ => {
                changed
                    .player_observations
                    .push(changed.player_observations[0].clone());
                "同一球员存在重复赛后表现"
            }
        };
        invalid(&changed, expected);
    }
}

#[test]
fn player_metrics_ranges_and_required_playing_signal_preserve_boundaries() {
    for case in 0..8 {
        let mut changed = draft();
        let observation = &mut changed.player_observations[0];
        let expected = match case {
            0 => {
                observation.minutes_played = 151;
                "球员分钟数必须位于 0–150"
            }
            1 => {
                observation.performance_score = Some(f64::INFINITY);
                "球员表现分必须位于 0–100"
            }
            2 => {
                observation.input_confidence = -0.1;
                "球员数据可信度必须位于 0–1"
            }
            3 => {
                observation.metrics.goals = f64::NAN;
                "球员事件数据必须是有限的非负数"
            }
            4 => {
                observation.metrics.shots_on_target = 1.0;
                "射正次数不能大于射门次数"
            }
            5 => {
                observation.metrics.duels_won = 1.0;
                "对抗成功次数不能大于对抗总数"
            }
            6 => {
                observation.metrics.provider_rating = Some(101.0);
                "供应商评分必须位于 0–10 或 0–100"
            }
            _ => {
                observation.performance_score = None;
                "出场球员必须提供表现分、供应商评分或至少一项有效事件数据"
            }
        };
        invalid(&changed, expected);
    }
    for (minutes, score, confidence) in [(0, 0.0, 0.0), (150, 100.0, 1.0)] {
        let mut valid = draft();
        let observation = &mut valid.player_observations[0];
        observation.minutes_played = minutes;
        observation.performance_score = Some(score);
        observation.input_confidence = confidence;
        assert!(validate_review_draft(&valid).is_ok());
    }
    for provider_rating in [0.0, 10.0, 100.0] {
        let mut valid = draft();
        valid.player_observations[0].performance_score = None;
        valid.player_observations[0].metrics.provider_rating = Some(provider_rating);
        assert!(validate_review_draft(&valid).is_ok());
    }
}

#[test]
fn substitution_and_event_player_links_are_validated() {
    let substitution: football_domain::SubstitutionDraft = serde_json::from_value(
        json!({"team_id": id(3), "player_out_id": id(5), "player_in_id": id(6), "minute": 60}),
    )
    .unwrap();
    for case in 0..5 {
        let mut changed = draft();
        changed.substitutions.push(substitution.clone());
        let item = &mut changed.substitutions[0];
        let expected = match case {
            0 => {
                item.minute = -1;
                "换人分钟必须位于 0–150"
            }
            1 => {
                item.player_out_id = None;
                item.player_in_id = None;
                "换人记录至少需要一名球员"
            }
            2 => {
                item.player_in_id = item.player_out_id;
                "换入与换出球员不能相同"
            }
            3 => {
                item.period = "invalid".into();
                "换人比赛阶段无效"
            }
            _ => {
                changed.events.push(serde_json::from_value(json!({"event_type": "goalkeeper_change", "team_id": id(3), "player_id": id(5), "minute": 60})).unwrap());
                "门将更换事件必须同时关联离场与入场球员"
            }
        };
        invalid(&changed, expected);
    }
    let mut valid = draft();
    valid.substitutions.push(substitution);
    valid.events.push(serde_json::from_value(json!({"event_type": "goalkeeper_change", "team_id": id(3), "player_id": id(5), "related_player_id": id(6), "minute": 60})).unwrap());
    assert!(validate_review_draft(&valid).is_ok());
}

#[test]
fn event_sequence_verification_and_import_status_guards_are_preserved() {
    for case in 0..8 {
        let mut changed = draft();
        changed.events.push(event(1));
        let item = &mut changed.events[0];
        let expected = match case {
            0 => {
                item.sequence_no = Some(0);
                "sequence_no 必须大于 0 且不能重复"
            }
            1 => {
                item.event_key = Some("same".into());
                let mut duplicate = event(2);
                duplicate.event_key = Some("same".into());
                changed.events.push(duplicate);
                "event_key 重复"
            }
            2 => {
                let mut duplicate = event(1);
                duplicate.event_key = Some("other".into());
                changed.events.push(duplicate);
                "sequence_no 必须大于 0 且不能重复"
            }
            3 => {
                item.stoppage_minute = Some(31);
                "比赛事件补时分钟必须位于 0–30"
            }
            4 => {
                item.confidence = f64::NAN;
                "比赛事件可信度必须位于 0–1"
            }
            5 => {
                item.verification_status = MatchEventVerificationStatus::Verified;
                "已核验比赛事件必须记录核验时间"
            }
            6 => {
                item.revision_status = MatchEventRevisionStatus::Superseded;
                "不能作为当前导入状态"
            }
            _ => {
                item.home_score = Some(1);
                "主客队比分必须同时存在或同时为空"
            }
        };
        invalid(&changed, expected);
    }
}

#[test]
fn event_scores_check_regulation_extra_time_and_ignore_cancelled_revisions() {
    let mut valid = draft();
    let mut first = event(1);
    first.home_score = Some(1);
    first.away_score = Some(0);
    valid.events.push(first);
    let mut extra = event(2);
    extra.period = "extra_time_second".into();
    extra.minute = 110;
    extra.home_score = Some(2);
    extra.away_score = Some(0);
    valid.events.push(extra);
    invalid(&valid, "正式赛果未同时填写主客队加时进球");
    valid.result.home_goals_extra_time = Some(1);
    valid.result.away_goals_extra_time = Some(0);
    assert!(validate_review_draft(&valid).is_ok());
    let mut mismatch = valid.clone();
    mismatch.events[1].home_score = Some(3);
    invalid(&mismatch, "与正式合计赛果 2-0 不一致");
    let mut backwards = valid.clone();
    backwards.events[1].home_score = Some(0);
    invalid(&backwards, "比分不能相对前序事件倒退");
    backwards.events[1].revision_status = MatchEventRevisionStatus::Cancelled;
    assert!(validate_review_draft(&backwards).is_ok());
    let mut regulation = valid;
    regulation.events[0].home_score = Some(2);
    regulation.events[1].home_score = Some(3);
    invalid(&regulation, "与正式 90 分钟赛果 1-0 不一致");
}

fn record(
    sequence: i32,
    status: MatchEventRevisionStatus,
    verification: MatchEventVerificationStatus,
) -> MatchReviewEventRecord {
    let mut value = serde_json::to_value(event(sequence)).unwrap();
    let object = value.as_object_mut().unwrap();
    object.insert("id".into(), json!(id(sequence as u128 + 10)));
    object.insert("match_id".into(), json!(id(1)));
    object.insert("event_key".into(), json!(format!("event:{sequence}")));
    object.insert("revision_status".into(), json!(status));
    object.insert("verification_status".into(), json!(verification));
    for field in ["verified_at", "recorded_at", "updated_at"] {
        object.insert(field.into(), json!("2026-10-12T00:00:00Z"));
    }
    object.insert("home_score".into(), json!(sequence));
    object.insert("away_score".into(), json!(0));
    object.insert("minute".into(), json!(sequence * 10));
    serde_json::from_value(value).unwrap()
}

#[test]
fn event_summary_counts_effective_and_audit_statuses_separately() {
    let empty = summarize_match_events(&[]);
    assert_eq!(empty.total_count, 0);
    assert_eq!(empty.latest_home_score, None);
    let events = [
        record(
            1,
            MatchEventRevisionStatus::Active,
            MatchEventVerificationStatus::Verified,
        ),
        record(
            2,
            MatchEventRevisionStatus::Corrected,
            MatchEventVerificationStatus::Disputed,
        ),
        record(
            3,
            MatchEventRevisionStatus::Cancelled,
            MatchEventVerificationStatus::Verified,
        ),
        record(
            4,
            MatchEventRevisionStatus::Superseded,
            MatchEventVerificationStatus::Unverified,
        ),
    ];
    let summary = summarize_match_events(&events);
    assert_eq!(summary.total_count, 4);
    assert_eq!(summary.effective_count, 2);
    assert_eq!(summary.cancelled_count, 1);
    assert_eq!(summary.disputed_count, 1);
    assert_eq!(summary.verified_count, 2);
    assert_eq!(summary.event_type_counts.get("goal"), Some(&2));
    assert_eq!(summary.last_event_minute, Some(20));
    assert_eq!(
        (summary.latest_home_score, summary.latest_away_score),
        (Some(2), Some(0))
    );
}
