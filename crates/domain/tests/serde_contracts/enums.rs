use football_domain::{
    AvailabilityStatus, CompetitionKind, LineupType, MatchStatus, PlayerStatus, PreferredFoot,
    RouteSource,
};
use serde::{de::DeserializeOwned, Serialize};

fn assert_json_name<T>(value: T, expected: &str)
where
    T: Serialize + DeserializeOwned,
{
    let serialized = serde_json::to_string(&value).expect("enum should serialize");
    assert_eq!(serialized, format!("\"{expected}\""));
    let round_trip: T = serde_json::from_str(&serialized).expect("enum should deserialize");
    let reserialized =
        serde_json::to_string(&round_trip).expect("round-tripped enum should serialize");
    assert_eq!(reserialized, serialized);
}

#[test]
fn snake_case_enum_names_are_stable() {
    assert_json_name(CompetitionKind::KnockoutTwoLeg, "knockout_two_leg");
    assert_json_name(PreferredFoot::Both, "both");
    assert_json_name(PlayerStatus::Retired, "retired");
    assert_json_name(AvailabilityStatus::Returning, "returning");
    assert_json_name(MatchStatus::Postponed, "postponed");
    assert_json_name(LineupType::Confirmed, "confirmed");
    assert_json_name(
        RouteSource::CompetitionKindDefault,
        "competition_kind_default",
    );
}

#[test]
fn enum_defaults_are_stable() {
    assert_eq!(CompetitionKind::default(), CompetitionKind::Custom);
    assert_eq!(PreferredFoot::default(), PreferredFoot::Unknown);
    assert_eq!(PlayerStatus::default(), PlayerStatus::Active);
    assert_eq!(MatchStatus::default(), MatchStatus::Scheduled);
}
