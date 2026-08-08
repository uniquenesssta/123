pub(crate) fn compact_key_part(value: &str) -> String {
    let normalized: String = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_uppercase())
        .take(12)
        .collect();
    if normalized.is_empty() {
        "TEAM".to_string()
    } else {
        normalized
    }
}
