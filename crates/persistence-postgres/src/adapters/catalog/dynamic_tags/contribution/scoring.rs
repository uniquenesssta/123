use football_domain::{ContributionComponent, PlayerDynamicTagRecord};
use std::collections::HashMap;

pub(super) const CONTRIBUTION_VERSION: &str = "match-contribution-v2-role-context";

pub(super) fn availability_multiplier(status: Option<&str>) -> f64 {
    match status.unwrap_or("unknown") {
        "available" => 1.0,
        "doubtful" => 0.75,
        "unavailable" => 0.0,
        "injured" => 0.15,
        "suspended" => 0.0,
        "rested" => 0.85,
        "returning" => 0.70,
        _ => 0.80,
    }
}

pub(super) fn tag_value(
    tags: &HashMap<&str, &PlayerDynamicTagRecord>,
    code: &str,
    default: f64,
) -> f64 {
    tags.get(code).map(|tag| tag.value).unwrap_or(default)
}

pub(super) fn component(
    code: &str,
    label: &str,
    value: f64,
    confidence: f64,
    source: impl Into<String>,
) -> ContributionComponent {
    ContributionComponent {
        code: code.to_string(),
        label: label.to_string(),
        value,
        confidence,
        source: source.into(),
    }
}

pub(super) fn component_from_tag(
    tags: &HashMap<&str, &PlayerDynamicTagRecord>,
    code: &str,
    label: &str,
    value: f64,
) -> ContributionComponent {
    if let Some(tag) = tags.get(code) {
        component(code, label, value, tag.confidence, tag.source_type.clone())
    } else {
        component(code, label, value, 0.5, "default")
    }
}
