pub(super) fn normalize_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::normalize_name;

    #[test]
    fn matching_normalization_preserves_existing_case_and_spacing_semantics() {
        assert_eq!(normalize_name("  José   Mourinho  "), "josé mourinho");
    }
}
