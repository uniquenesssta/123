use football_domain::{
    LineupPairDraft, LineupRecord, LineupType, MatchResultDraft, MatchResultRecord,
    MatchReviewPackagePreview, MatchReviewPackageSnapshotSummary, MatchReviewPackageWorkflowRecord,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use uuid::Uuid;

pub(super) fn apply_confirmation_metadata(
    preview: &mut MatchReviewPackagePreview,
    workflow: &MatchReviewPackageWorkflowRecord,
) {
    let confirmation = json!({
        "package_id": workflow.package_id,
        "source_sha256": workflow.import_sha256.as_deref(),
        "confirmed_by": workflow.confirmed_by.as_deref(),
        "confirmation_note": workflow.confirmation_note.as_deref(),
        "confirmed_at": workflow.confirmed_at.as_ref(),
    });
    merge_metadata(
        &mut preview.review.result.metadata,
        "review_package_confirmation",
        confirmation.clone(),
    );
    merge_metadata(
        &mut preview.lineup_pair.home.metadata,
        "review_package_confirmation",
        confirmation.clone(),
    );
    merge_metadata(
        &mut preview.lineup_pair.away.metadata,
        "review_package_confirmation",
        confirmation,
    );
    if let Some(note) = workflow
        .confirmation_note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        preview.review.notes = Some(match preview.review.notes.take() {
            Some(existing) if !existing.trim().is_empty() => {
                format!("{}\n人工确认：{}", existing.trim(), note)
            }
            _ => format!("人工确认：{note}"),
        });
    }
}

pub(super) fn validate_membership(
    players: &[football_domain::LineupPlayerDraft],
    registered: &HashSet<Uuid>,
    label: &str,
    errors: &mut Vec<String>,
) {
    for player in players {
        if !registered.contains(&player.player_id) && !player.membership_override {
            errors.push(format!(
                "{label}球员 {} 不在当前球队登记名单；确认真实出场时需将 membership_override 设为 true 并说明",
                player.player_id
            ));
        }
    }
}

pub(super) fn validate_event_identities(
    lineup_pair: &LineupPairDraft,
    events: &[football_domain::MatchReviewEventDraft],
    errors: &mut Vec<String>,
) {
    let player_teams = lineup_pair
        .home
        .players
        .iter()
        .map(|player| (player.player_id, lineup_pair.home.team_id))
        .chain(
            lineup_pair
                .away
                .players
                .iter()
                .map(|player| (player.player_id, lineup_pair.away.team_id)),
        )
        .collect::<std::collections::HashMap<_, _>>();
    for (index, event) in events.iter().enumerate() {
        for (role, player_id) in [
            ("player_id", event.player_id),
            ("related_player_id", event.related_player_id),
        ] {
            let Some(player_id) = player_id else { continue };
            let Some(player_team_id) = player_teams.get(&player_id) else {
                errors.push(format!(
                    "比赛事件第 {} 行的 {role} 不在准备导入的比赛名单中",
                    index + 2
                ));
                continue;
            };
            let team_matches_event = event.team_id.is_none_or(|team_id| {
                if event.event_type == football_domain::MatchEventType::OwnGoal
                    && role == "player_id"
                {
                    team_id != *player_team_id
                } else {
                    team_id == *player_team_id
                }
            });
            if !team_matches_event {
                let expected = if event.event_type == football_domain::MatchEventType::OwnGoal
                    && role == "player_id"
                {
                    "乌龙球球员应属于事件受益球队的对手"
                } else {
                    "球员应属于事件球队"
                };
                errors.push(format!(
                    "比赛事件第 {} 行的 {role} 与 team_id 不一致：{expected}",
                    index + 2
                ));
            }
        }
    }
}

pub(super) fn snapshot_from_lineups(
    lineups: &[LineupRecord],
    home_team_id: Uuid,
    away_team_id: Uuid,
    actual: bool,
    result: Option<&MatchResultRecord>,
) -> MatchReviewPackageSnapshotSummary {
    let preferred = |team_id| {
        lineups
            .iter()
            .filter(|lineup| {
                lineup.team_id == team_id
                    && matches!(lineup.lineup_type, LineupType::Actual) == actual
            })
            .min_by_key(|lineup| match lineup.lineup_type {
                LineupType::Confirmed => 0,
                LineupType::Expected => 1,
                LineupType::Actual => 2,
            })
    };
    let home = preferred(home_team_id);
    let away = preferred(away_team_id);
    MatchReviewPackageSnapshotSummary {
        home_goals_90: result.map(|value| value.home_goals_90),
        away_goals_90: result.map(|value| value.away_goals_90),
        home_player_count: home.map_or(0, |value| value.players.len() as u64),
        away_player_count: away.map_or(0, |value| value.players.len() as u64),
        home_starter_count: home.map_or(0, |value| {
            value
                .players
                .iter()
                .filter(|player| player.is_starter)
                .count() as u64
        }),
        away_starter_count: away.map_or(0, |value| {
            value
                .players
                .iter()
                .filter(|player| player.is_starter)
                .count() as u64
        }),
    }
}

pub(super) fn snapshot_from_pair(
    pair: &LineupPairDraft,
    result: &MatchResultDraft,
) -> MatchReviewPackageSnapshotSummary {
    MatchReviewPackageSnapshotSummary {
        home_goals_90: Some(result.home_goals_90),
        away_goals_90: Some(result.away_goals_90),
        home_player_count: pair.home.players.len() as u64,
        away_player_count: pair.away.players.len() as u64,
        home_starter_count: pair
            .home
            .players
            .iter()
            .filter(|player| player.is_starter)
            .count() as u64,
        away_starter_count: pair
            .away
            .players
            .iter()
            .filter(|player| player.is_starter)
            .count() as u64,
    }
}

fn merge_metadata(target: &mut Value, key: &str, value: Value) {
    if !target.is_object() {
        *target = json!({});
    }
    if let Some(object) = target.as_object_mut() {
        object.insert(key.to_string(), value);
    }
}
