use super::*;

pub(super) async fn register(port: &dyn ResearchArtifactPort) -> ApplicationResult<()> {
    port.register_schema(&built_in_research_schema()).await?;
    port.register_prompt(&built_in_research_prompt()).await?;
    fact_pipeline::register_fact_pipeline_artifacts(port).await?;
    Ok(())
}

pub(super) fn built_in_research_schema() -> SchemaVersionDraft {
    SchemaVersionDraft {
        schema_key: RESEARCH_SCHEMA_KEY.to_string(),
        version: P4_RESEARCH_SCHEMA_ARTIFACT_VERSION.to_string(),
        schema_kind: "openai_structured_output".to_string(),
        schema_body: serde_json::from_str(include_str!(
            "../../../../../../schemas/research-output.schema.json"
        ))
        .expect("内置公开研究输出Schema必须有效"),
        description: Some("OpenAI Web Search事实研究严格输出契约".to_string()),
        metadata: json!({
            "schema_id": P4_RESEARCH_OUTPUT_SCHEMA_VERSION,
            "stage": "E",
            "strict": true,
            "rust_second_validation": true,
            "atomic_claims": true,
            "entity_time_conflict_pipeline": true
        }),
    }
}

pub(super) fn built_in_research_prompt() -> PromptVersionDraft {
    PromptVersionDraft {
        prompt_key: RESEARCH_PROMPT_KEY.to_string(),
        version: P4_RESEARCH_PROMPT_ARTIFACT_VERSION.to_string(),
        prompt_role: "research_system".to_string(),
        content: include_str!(
            "../../../../../../src-tauri/resources/research/public_research_prompt.txt"
        )
        .to_string(),
        metadata: json!({
            "stage": "E",
            "static_prefix": true,
            "prediction_prohibited": true,
            "atomic_source_backed_claims": true,
            "entity_resolution_stage": "E"
        }),
    }
}
