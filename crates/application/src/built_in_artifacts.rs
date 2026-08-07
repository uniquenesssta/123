pub(crate) const P4_EVIDENCE_SCHEMA_KEY: &str = "p4-evidence";
pub(crate) const P4_EVIDENCE_SCHEMA_ARTIFACT_VERSION: &str = "1.0.0+public.1";
pub(crate) const P4_SNAPSHOT_SCHEMA_KEY: &str = "p4-prematch-snapshot";
pub(crate) const P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION: &str = "1.0.0+public.1";
pub(crate) const P4_RESEARCH_SCHEMA_KEY: &str = "p4-openai-research-output";
pub(crate) const P4_RESEARCH_SCHEMA_ARTIFACT_VERSION: &str = "2.0.0+public.1";
pub(crate) const P4_RESEARCH_PROMPT_KEY: &str = "p4-openai-research-system";
pub(crate) const P4_RESEARCH_PROMPT_ARTIFACT_VERSION: &str =
    football_domain::P4_RESEARCH_PROMPT_VERSION;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewritten_public_artifacts_have_distinct_immutable_versions() {
        assert_eq!(P4_EVIDENCE_SCHEMA_ARTIFACT_VERSION, "1.0.0+public.1");
        assert_eq!(P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION, "1.0.0+public.1");
        assert_eq!(P4_RESEARCH_SCHEMA_ARTIFACT_VERSION, "2.0.0+public.1");
        assert_eq!(P4_RESEARCH_PROMPT_ARTIFACT_VERSION, "2.0.0+public.1");
    }
}
