use football_domain::{
    CompetitionDraft, CompetitionKind, MatchDraft, MatchStatus, PlayerDraft, PlayerStatus,
    PreferredFoot, SeasonDraft, TeamListQuery,
};
use serde_json::{json, Value};

#[test]
fn draft_defaults_and_optional_semantics_are_stable() {
    let competition: CompetitionDraft = serde_json::from_value(json!({
        "code": "EPL",
        "name": "Premier League",
        "competition_kind": "league"
    }))
    .expect("competition draft should deserialize");
    assert_eq!(competition.timezone, "UTC");
    assert!(competition.country_code.is_none());
    assert_eq!(competition.metadata, Value::Null);

    let season: SeasonDraft = serde_json::from_value(json!({
        "competition_id": "00000000-0000-0000-0000-000000000001",
        "name": "2026"
    }))
    .expect("season draft should deserialize");
    assert_eq!(season.status, "planned");
    assert!(season.starts_on.is_none());
    assert!(season.ends_on.is_none());

    let player: PlayerDraft = serde_json::from_value(json!({
        "canonical_name": "Contract Player"
    }))
    .expect("player draft should deserialize");
    assert_eq!(player.preferred_foot, PreferredFoot::Unknown);
    assert_eq!(player.status, PlayerStatus::Active);
    assert_eq!(player.metadata, Value::Null);

    let match_draft: MatchDraft = serde_json::from_value(json!({
        "home_team_id": "00000000-0000-0000-0000-000000000010",
        "away_team_id": "00000000-0000-0000-0000-000000000011",
        "kickoff_time": "2026-08-06T12:00:00Z"
    }))
    .expect("match draft should deserialize");
    assert_eq!(match_draft.external_key, "");
    assert_eq!(match_draft.status, MatchStatus::Scheduled);
    assert_eq!(match_draft.metadata, Value::Null);

    let query: TeamListQuery = serde_json::from_value(json!({})).expect("query should deserialize");
    assert!(query.active_only);
    assert_eq!(query.limit, 50);
    assert!(query.search.is_none());

    assert_eq!(CompetitionKind::default(), CompetitionKind::Custom);
}
