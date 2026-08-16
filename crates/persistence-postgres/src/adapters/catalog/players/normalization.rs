pub(crate) fn normalize_name(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::normalize_name;

    #[test]
    fn collapses_case_and_spacing() {
        assert_eq!(normalize_name("  Son   Heung-Min  "), "son heung-min");
    }
}
