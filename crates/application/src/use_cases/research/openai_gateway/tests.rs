use super::artifacts::{built_in_research_prompt, built_in_research_schema};
use super::gateway::built_in_gateway_config;
use super::*;

#[test]
fn built_in_gateway_is_strict_and_has_no_secret() {
    let config = built_in_gateway_config().expect("config");
    assert!(!config.research_model.is_empty());
    assert!(!config.extraction_model.is_empty());
    assert!(!config.store);
    let serialized =
        include_str!("../../../../../../src-tauri/resources/research/openai_gateway.json");
    assert!(!serialized.contains("sk-"));
    assert!(!serialized.contains("api_key"));
    let schema = built_in_research_schema();
    assert_eq!(schema.version, P4_RESEARCH_SCHEMA_ARTIFACT_VERSION);
    assert_eq!(
        built_in_research_prompt().version,
        P4_RESEARCH_PROMPT_ARTIFACT_VERSION
    );
    assert_eq!(schema.schema_body["additionalProperties"], false);
    assert!(built_in_research_prompt()
        .content
        .to_ascii_lowercase()
        .contains("do not calculate probabilities"));
}
