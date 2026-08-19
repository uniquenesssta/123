use super::normalization::normalize_query;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NameSearch {
    tokens: Vec<String>,
}

impl NameSearch {
    pub(crate) fn parse(value: Option<&str>) -> Option<Self> {
        let normalized = value.map(normalize_query).unwrap_or_default();
        let tokens = normalized
            .split_whitespace()
            .map(str::to_string)
            .filter(|token| !token.is_empty())
            .collect::<Vec<_>>();
        (!tokens.is_empty()).then_some(Self { tokens })
    }

    pub(crate) fn tokens(&self) -> &[String] {
        &self.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chinese_and_english_partial_terms() {
        let search = NameSearch::parse(Some("  Marlon · 索萨  ")).unwrap();
        assert_eq!(search.tokens(), &["marlon".to_string(), "索萨".to_string()]);
    }

    #[test]
    fn empty_query_is_ignored() {
        assert!(NameSearch::parse(Some("  --  ")).is_none());
        assert!(NameSearch::parse(None).is_none());
    }
}
