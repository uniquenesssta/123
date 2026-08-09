use crate::built_in_artifacts::{
    P4_EVIDENCE_SCHEMA_ARTIFACT_VERSION, P4_EVIDENCE_SCHEMA_KEY,
    P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION, P4_SNAPSHOT_SCHEMA_KEY,
};
use crate::ports::research::ResearchArtifactPort;
use crate::ApplicationResult;
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, PromptVersionDraft,
    PromptVersionRecord, SchemaVersionDraft, SchemaVersionRecord, P4_EVIDENCE_SCHEMA_VERSION,
    P4_SNAPSHOT_SCHEMA_VERSION,
};
use serde_json::Value;

pub(crate) async fn register_built_ins<P: ResearchArtifactPort + ?Sized>(
    port: &P,
) -> ApplicationResult<()> {
    for draft in built_in_schema_versions() {
        port.register_schema(&draft).await?;
    }
    Ok(())
}

pub(crate) async fn register_schema<P: ResearchArtifactPort + ?Sized>(
    port: &P,
    draft: SchemaVersionDraft,
) -> ApplicationResult<SchemaVersionRecord> {
    Ok(port.register_schema(&draft).await?)
}

pub(crate) async fn register_prompt<P: ResearchArtifactPort + ?Sized>(
    port: &P,
    draft: PromptVersionDraft,
) -> ApplicationResult<PromptVersionRecord> {
    Ok(port.register_prompt(&draft).await?)
}

pub(crate) async fn register_competition_profile<P: ResearchArtifactPort + ?Sized>(
    port: &P,
    draft: CompetitionProfileVersionDraft,
) -> ApplicationResult<CompetitionProfileVersionRecord> {
    Ok(port.register_competition_profile(&draft).await?)
}

fn built_in_schema_versions() -> Vec<SchemaVersionDraft> {
    vec![
        SchemaVersionDraft {
            schema_key: P4_EVIDENCE_SCHEMA_KEY.to_string(),
            version: P4_EVIDENCE_SCHEMA_ARTIFACT_VERSION.to_string(),
            schema_kind: "structured_evidence".to_string(),
            schema_body: parse_schema(include_str!("../../../../../schemas/evidence.schema.json")),
            description: Some("联网事实证据声明的公开结构契约".to_string()),
            metadata: serde_json::json!({
                "schema_id": P4_EVIDENCE_SCHEMA_VERSION,
                "stage": "C",
                "openai_runtime": false
            }),
        },
        SchemaVersionDraft {
            schema_key: P4_SNAPSHOT_SCHEMA_KEY.to_string(),
            version: P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION.to_string(),
            schema_kind: "immutable_snapshot".to_string(),
            schema_body: parse_schema(include_str!(
                "../../../../../schemas/prematch-snapshot.schema.json"
            )),
            description: Some("外部模型入口使用的不可变赛前快照公开结构契约".to_string()),
            metadata: serde_json::json!({
                "schema_id": P4_SNAPSHOT_SCHEMA_VERSION,
                "stage": "C",
                "feature_field_count": 31,
                "probability_chains": "provider-defined"
            }),
        },
    ]
}

fn parse_schema(content: &str) -> Value {
    serde_json::from_str(content).expect("内置公开持久化 Schema 必须有效")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_persistence_schemas_are_unique_and_strict() {
        let drafts = built_in_schema_versions();
        assert_eq!(drafts.len(), 2);
        assert_ne!(drafts[0].schema_key, drafts[1].schema_key);
        assert_eq!(drafts[0].schema_key, P4_EVIDENCE_SCHEMA_KEY);
        assert_eq!(drafts[0].version, P4_EVIDENCE_SCHEMA_ARTIFACT_VERSION);
        assert_eq!(drafts[1].schema_key, P4_SNAPSHOT_SCHEMA_KEY);
        assert_eq!(drafts[1].version, P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION);
        for draft in drafts {
            assert_eq!(draft.schema_body["additionalProperties"], false);
            assert!(draft.schema_body["$id"].as_str().is_some());
        }
    }
}
