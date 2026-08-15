pub(in crate::adapters::catalog::teams) fn normalize_team_name(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::normalize_team_name;

    #[test]
    fn normalizes_case_outer_whitespace_and_internal_spacing() {
        assert_eq!(normalize_team_name("  Real   MADRID  "), "real madrid");
    }
}
