use super::identity::match_event_key;
use crate::{PersistenceError, PersistenceResult};
use football_domain::{
    MatchEventRevisionStatus, MatchEventType, MatchEventVerificationStatus, MatchReviewDraft,
};
use std::collections::HashSet;

pub(crate) fn validate_review_draft(draft: &MatchReviewDraft) -> PersistenceResult<()> {
    if draft.result.match_id != draft.match_id {
        return Err(PersistenceError::InvalidState(
            "正式赛果与复盘比赛 ID 不一致".to_string(),
        ));
    }
    if draft.result.home_goals_90 < 0 || draft.result.away_goals_90 < 0 {
        return Err(PersistenceError::InvalidState(
            "90 分钟进球不能为负数".to_string(),
        ));
    }
    for value in [
        draft.result.home_goals_extra_time,
        draft.result.away_goals_extra_time,
        draft.result.home_penalties,
        draft.result.away_penalties,
    ]
    .into_iter()
    .flatten()
    {
        if value < 0 {
            return Err(PersistenceError::InvalidState(
                "加时或点球进球不能为负数".to_string(),
            ));
        }
    }
    if draft.result.home_goals_extra_time.is_some() != draft.result.away_goals_extra_time.is_some()
    {
        return Err(PersistenceError::InvalidState(
            "主客队加时进球必须同时填写或同时留空".to_string(),
        ));
    }
    if draft.result.home_penalties.is_some() != draft.result.away_penalties.is_some() {
        return Err(PersistenceError::InvalidState(
            "主客队点球大战进球必须同时填写或同时留空".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&draft.data_coverage) {
        return Err(PersistenceError::InvalidState(
            "数据覆盖率必须位于 0–1".to_string(),
        ));
    }
    if draft.player_observations.is_empty() {
        return Err(PersistenceError::InvalidState(
            "至少需要一条球员赛后表现".to_string(),
        ));
    }
    let mut player_ids = HashSet::new();
    for observation in &draft.player_observations {
        if !player_ids.insert(observation.player_id) {
            return Err(PersistenceError::InvalidState(
                "同一球员存在重复赛后表现".to_string(),
            ));
        }
        if !(0..=150).contains(&observation.minutes_played) {
            return Err(PersistenceError::InvalidState(
                "球员分钟数必须位于 0–150".to_string(),
            ));
        }
        if observation
            .performance_score
            .is_some_and(|value| !(0.0..=100.0).contains(&value))
        {
            return Err(PersistenceError::InvalidState(
                "球员表现分必须位于 0–100".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&observation.input_confidence) {
            return Err(PersistenceError::InvalidState(
                "球员数据可信度必须位于 0–1".to_string(),
            ));
        }
        validate_performance_metrics(&observation.metrics)?;
        if observation.minutes_played > 0
            && observation.performance_score.is_none()
            && observation.metrics.provider_rating.is_none()
            && !performance_metrics_have_signal(&observation.metrics)
        {
            return Err(PersistenceError::InvalidState(
                "出场球员必须提供表现分、供应商评分或至少一项有效事件数据".to_string(),
            ));
        }
    }
    const VALID_PERIODS: [&str; 5] = [
        "first_half",
        "second_half",
        "extra_time_first",
        "extra_time_second",
        "normal_time",
    ];
    for substitution in &draft.substitutions {
        if !(0..=150).contains(&substitution.minute) {
            return Err(PersistenceError::InvalidState(
                "换人分钟必须位于 0–150".to_string(),
            ));
        }
        if substitution.player_in_id.is_none() && substitution.player_out_id.is_none() {
            return Err(PersistenceError::InvalidState(
                "换人记录至少需要一名球员".to_string(),
            ));
        }
        if substitution.player_in_id == substitution.player_out_id
            && substitution.player_in_id.is_some()
        {
            return Err(PersistenceError::InvalidState(
                "换入与换出球员不能相同".to_string(),
            ));
        }
        if !VALID_PERIODS.contains(&substitution.period.trim()) {
            return Err(PersistenceError::InvalidState(
                "换人比赛阶段无效".to_string(),
            ));
        }
    }
    let mut event_keys = HashSet::new();
    let mut event_sequences = HashSet::new();
    let mut ordered_scores = Vec::new();
    for (index, event) in draft.events.iter().enumerate() {
        let event_key = match_event_key(event, draft.match_id, index)?;
        if !event_keys.insert(event_key.clone()) {
            return Err(PersistenceError::InvalidState(format!(
                "比赛事件 event_key 重复：{event_key}"
            )));
        }
        let sequence_no = event.sequence_no.unwrap_or(index as i32 + 1);
        if sequence_no <= 0 || !event_sequences.insert(sequence_no) {
            return Err(PersistenceError::InvalidState(
                "比赛事件 sequence_no 必须大于 0 且不能重复".to_string(),
            ));
        }
        if !(0..=150).contains(&event.minute) {
            return Err(PersistenceError::InvalidState(
                "比赛事件分钟必须位于 0–150".to_string(),
            ));
        }
        if event
            .stoppage_minute
            .is_some_and(|value| !(0..=30).contains(&value))
        {
            return Err(PersistenceError::InvalidState(
                "比赛事件补时分钟必须位于 0–30".to_string(),
            ));
        }
        if !VALID_PERIODS.contains(&event.period.trim()) {
            return Err(PersistenceError::InvalidState(
                "比赛事件阶段无效".to_string(),
            ));
        }
        if !event.confidence.is_finite() || !(0.0..=1.0).contains(&event.confidence) {
            return Err(PersistenceError::InvalidState(
                "比赛事件可信度必须位于 0–1".to_string(),
            ));
        }
        if event.event_type.requires_team() && event.team_id.is_none() {
            return Err(PersistenceError::InvalidState(format!(
                "{} 事件必须关联球队",
                event.event_type.as_str()
            )));
        }
        if event.event_type.requires_player() && event.player_id.is_none() {
            return Err(PersistenceError::InvalidState(format!(
                "{} 事件必须关联球员",
                event.event_type.as_str()
            )));
        }
        if matches!(
            event.event_type,
            MatchEventType::Substitution | MatchEventType::GoalkeeperChange
        ) {
            let label = if event.event_type == MatchEventType::Substitution {
                "换人"
            } else {
                "门将更换"
            };
            if event.player_id.is_none() || event.related_player_id.is_none() {
                return Err(PersistenceError::InvalidState(format!(
                    "{label}事件必须同时关联离场与入场球员"
                )));
            }
            if event.player_id == event.related_player_id {
                return Err(PersistenceError::InvalidState(format!(
                    "{label}事件的离场与入场球员不能相同"
                )));
            }
        }
        if event.event_type == MatchEventType::Assist && event.related_player_id.is_none() {
            return Err(PersistenceError::InvalidState(
                "助攻事件必须关联对应进球球员".to_string(),
            ));
        }
        if event.home_score.is_some() != event.away_score.is_some() {
            return Err(PersistenceError::InvalidState(
                "比赛事件后的主客队比分必须同时存在或同时为空".to_string(),
            ));
        }
        if event.home_score.is_some_and(|value| value < 0)
            || event.away_score.is_some_and(|value| value < 0)
        {
            return Err(PersistenceError::InvalidState(
                "比赛事件后的比分不能为负数".to_string(),
            ));
        }
        if event.verification_status == MatchEventVerificationStatus::Verified
            && event.verified_at.is_none()
        {
            return Err(PersistenceError::InvalidState(
                "已核验比赛事件必须记录核验时间".to_string(),
            ));
        }
        if event.revision_status == MatchEventRevisionStatus::Superseded {
            return Err(PersistenceError::InvalidState(
                "superseded 由系统根据新资料包自动维护，不能作为当前导入状态".to_string(),
            ));
        }
        if event.revision_status.is_effective() {
            if let (Some(home_score), Some(away_score)) = (event.home_score, event.away_score) {
                let is_extra_time = matches!(
                    event.period.trim(),
                    "extra_time_first" | "extra_time_second"
                );
                ordered_scores.push((sequence_no, home_score, away_score, is_extra_time));
            }
        }
    }
    ordered_scores.sort_by_key(|item| item.0);
    let mut previous_score = (0i16, 0i16);
    let mut latest_regulation_score = None;
    let mut latest_overall_score = None;
    let mut has_extra_time_score = false;
    for (_, home_score, away_score, is_extra_time) in &ordered_scores {
        if *home_score < previous_score.0 || *away_score < previous_score.1 {
            return Err(PersistenceError::InvalidState(
                "比赛事件后的比分不能相对前序事件倒退；VAR 取消应使用 cancelled/corrected 修订状态"
                    .to_string(),
            ));
        }
        previous_score = (*home_score, *away_score);
        latest_overall_score = Some(previous_score);
        if *is_extra_time {
            has_extra_time_score = true;
        } else {
            latest_regulation_score = Some(previous_score);
        }
    }
    if let Some((home_score, away_score)) = latest_regulation_score {
        if home_score != draft.result.home_goals_90 || away_score != draft.result.away_goals_90 {
            return Err(PersistenceError::InvalidState(format!(
                "最后一条 90 分钟事件后比分 {home_score}-{away_score} 与正式 90 分钟赛果 {}-{} 不一致",
                draft.result.home_goals_90, draft.result.away_goals_90
            )));
        }
    }
    if has_extra_time_score {
        let (Some(home_extra), Some(away_extra)) = (
            draft.result.home_goals_extra_time,
            draft.result.away_goals_extra_time,
        ) else {
            return Err(PersistenceError::InvalidState(
                "存在加时阶段事件后比分，但正式赛果未同时填写主客队加时进球".to_string(),
            ));
        };
        if let Some((home_score, away_score)) = latest_overall_score {
            let expected_home = draft.result.home_goals_90 + home_extra;
            let expected_away = draft.result.away_goals_90 + away_extra;
            if home_score != expected_home || away_score != expected_away {
                return Err(PersistenceError::InvalidState(format!(
                    "最后一条加时事件后比分 {home_score}-{away_score} 与正式合计赛果 {expected_home}-{expected_away} 不一致"
                )));
            }
        }
    }
    Ok(())
}

fn validate_performance_metrics(
    metrics: &football_domain::PlayerPerformanceMetrics,
) -> PersistenceResult<()> {
    let values = [
        metrics.goals,
        metrics.assists,
        metrics.expected_goals,
        metrics.expected_assists,
        metrics.shots,
        metrics.shots_on_target,
        metrics.key_passes,
        metrics.progressive_actions,
        metrics.tackles,
        metrics.interceptions,
        metrics.clearances,
        metrics.blocks,
        metrics.duels_won,
        metrics.duels_total,
        metrics.fouls,
        metrics.yellow_cards,
        metrics.red_cards,
        metrics.errors_leading_to_shot,
    ];
    if values
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(PersistenceError::InvalidState(
            "球员事件数据必须是有限的非负数".to_string(),
        ));
    }
    if metrics.shots_on_target > metrics.shots {
        return Err(PersistenceError::InvalidState(
            "射正次数不能大于射门次数".to_string(),
        ));
    }
    if metrics.duels_won > metrics.duels_total {
        return Err(PersistenceError::InvalidState(
            "对抗成功次数不能大于对抗总数".to_string(),
        ));
    }
    if let Some(rating) = metrics.provider_rating {
        let valid = rating.is_finite()
            && ((0.0..=10.0).contains(&rating) || (0.0..=100.0).contains(&rating));
        if !valid {
            return Err(PersistenceError::InvalidState(
                "供应商评分必须位于 0–10 或 0–100".to_string(),
            ));
        }
    }
    Ok(())
}

fn performance_metrics_have_signal(metrics: &football_domain::PlayerPerformanceMetrics) -> bool {
    [
        metrics.goals,
        metrics.assists,
        metrics.expected_goals,
        metrics.expected_assists,
        metrics.shots,
        metrics.shots_on_target,
        metrics.key_passes,
        metrics.progressive_actions,
        metrics.tackles,
        metrics.interceptions,
        metrics.clearances,
        metrics.blocks,
        metrics.duels_won,
        metrics.duels_total,
        metrics.fouls,
        metrics.yellow_cards,
        metrics.red_cards,
        metrics.errors_leading_to_shot,
    ]
    .iter()
    .any(|value| *value > 0.0)
}
