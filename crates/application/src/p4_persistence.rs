use super::{ApplicationResult, ApplicationService};
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, EvidenceClaimDraft,
    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, PrematchSnapshotBundle,
    PrematchSnapshotDraft, PrematchSnapshotRecord, PromptVersionDraft, PromptVersionRecord,
    ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord, SchemaVersionDraft,
    SchemaVersionRecord, P4_EVIDENCE_SCHEMA_VERSION, P4_SNAPSHOT_SCHEMA_VERSION,
};
use serde_json::Value;
use uuid::Uuid;

use crate::PersistenceStore;

impl ApplicationService {
    pub(super) async fn register_p4_persistence_artifacts(
        &self,
        store: &PersistenceStore,
    ) -> ApplicationResult<()> {
        for draft in built_in_schema_versions() {
            store.register_schema_version(&draft).await?;
        }
        Ok(())
    }

    pub async fn register_p4_schema_version(
        &self,
        draft: SchemaVersionDraft,
    ) -> ApplicationResult<SchemaVersionRecord> {
        let store = self.active_store().await?;
        Ok(store.register_schema_version(&draft).await?)
    }

    pub async fn register_p4_prompt_version(
        &self,
        draft: PromptVersionDraft,
    ) -> ApplicationResult<PromptVersionRecord> {
        let store = self.active_store().await?;
        Ok(store.register_prompt_version(&draft).await?)
    }

    pub async fn register_p4_competition_profile_version(
        &self,
        draft: CompetitionProfileVersionDraft,
    ) -> ApplicationResult<CompetitionProfileVersionRecord> {
        let store = self.active_store().await?;
        Ok(store.register_competition_profile_version(&draft).await?)
    }

    pub async fn create_p4_research_run(
        &self,
        draft: ResearchRunDraft,
    ) -> ApplicationResult<ResearchRunRecord> {
        let store = self.active_store().await?;
        Ok(store.create_research_run(&draft).await?)
    }

    pub async fn record_p4_research_run_event(
        &self,
        draft: ResearchRunEventDraft,
    ) -> ApplicationResult<ResearchRunRecord> {
        let store = self.active_store().await?;
        Ok(store.record_research_run_event(&draft).await?)
    }

    pub async fn append_p4_evidence_claim(
        &self,
        draft: EvidenceClaimDraft,
    ) -> ApplicationResult<EvidenceClaimRecord> {
        let store = self.active_store().await?;
        Ok(store.append_evidence_claim(&draft).await?)
    }

    pub async fn create_p4_evidence_conflict(
        &self,
        draft: EvidenceConflictDraft,
    ) -> ApplicationResult<EvidenceConflictRecord> {
        let store = self.active_store().await?;
        Ok(store.create_evidence_conflict(&draft).await?)
    }

    pub async fn freeze_p4_prematch_snapshot(
        &self,
        draft: PrematchSnapshotDraft,
    ) -> ApplicationResult<PrematchSnapshotRecord> {
        let store = self.active_store().await?;
        Ok(store.freeze_prematch_snapshot(&draft).await?)
    }

    pub async fn read_p4_prematch_snapshot(
        &self,
        snapshot_id: Uuid,
    ) -> ApplicationResult<PrematchSnapshotBundle> {
        let store = self.active_store().await?;
        Ok(store.read_prematch_snapshot(snapshot_id).await?)
    }
}

fn built_in_schema_versions() -> Vec<SchemaVersionDraft> {
    vec![
        SchemaVersionDraft {
            schema_key: "p4-evidence".to_string(),
            version: "1.0.0".to_string(),
            schema_kind: "structured_evidence".to_string(),
            schema_body: parse_schema(include_str!("../../../schemas/evidence.schema.json")),
            description: Some("联网事实证据声明的公开结构契约".to_string()),
            metadata: serde_json::json!({
                "schema_id": P4_EVIDENCE_SCHEMA_VERSION,
                "stage": "C",
                "openai_runtime": false
            }),
        },
        SchemaVersionDraft {
            schema_key: "p4-prematch-snapshot".to_string(),
            version: "1.0.0".to_string(),
            schema_kind: "immutable_snapshot".to_string(),
            schema_body: parse_schema(include_str!(
                "../../../schemas/prematch-snapshot.schema.json"
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
        for draft in drafts {
            assert_eq!(draft.schema_body["additionalProperties"], false);
            assert!(draft.schema_body["$id"].as_str().is_some());
        }
    }
}
