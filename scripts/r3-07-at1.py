from pathlib import Path
import os

ROOT = Path(__file__).resolve().parents[1]
RUN_ID = os.environ.get("GITHUB_RUN_ID", "unknown")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8", newline="\n")


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise RuntimeError(f"expected exactly one match in {path}: {old!r}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


write(
    "crates/application/src/ports/research/mod.rs",
    '''use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, ConflictEvaluationDraft,
    ConflictEvaluationRecord, EntityMatchRequest, EntityResolutionDraft, EntityResolutionRecord,
    EvidenceClaimDraft, EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord,
    EvidenceRouteDraft, EvidenceRouteRecord, FactPipelineContext, OpenAiAttemptDraft,
    OpenAiAttemptRecord, OpenAiUsageTotals, PromptVersionDraft, PromptVersionRecord,
    ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord, SchemaVersionDraft,
    SchemaVersionRecord, SourcePolicyVersionDraft, SourcePolicyVersionRecord, TimeAuditDraft,
    TimeAuditRecord, WebCitationDraft, WebSourceDraft,
};
use uuid::Uuid;

#[async_trait]
pub trait ResearchArtifactPort: Send + Sync {
    async fn read_schema(
        &self,
        schema_key: &str,
        version: &str,
    ) -> PortResult<SchemaVersionRecord>;
    async fn register_schema(&self, draft: &SchemaVersionDraft) -> PortResult<SchemaVersionRecord>;
    async fn register_prompt(&self, draft: &PromptVersionDraft) -> PortResult<PromptVersionRecord>;
    async fn register_source_policy(
        &self,
        draft: &SourcePolicyVersionDraft,
    ) -> PortResult<SourcePolicyVersionRecord>;
    async fn register_competition_profile(
        &self,
        draft: &CompetitionProfileVersionDraft,
    ) -> PortResult<CompetitionProfileVersionRecord>;
    async fn create_run(&self, draft: &ResearchRunDraft) -> PortResult<ResearchRunRecord>;
    async fn read_run(&self, run_id: Uuid) -> PortResult<ResearchRunRecord>;
    async fn record_run_event(
        &self,
        draft: &ResearchRunEventDraft,
    ) -> PortResult<ResearchRunRecord>;
}

#[async_trait]
pub trait ResearchEvidenceLedgerPort: Send + Sync {
    async fn append_evidence_claim(
        &self,
        draft: &EvidenceClaimDraft,
    ) -> PortResult<EvidenceClaimRecord>;
    async fn create_evidence_conflict(
        &self,
        draft: &EvidenceConflictDraft,
    ) -> PortResult<EvidenceConflictRecord>;
}

#[async_trait]
pub trait FactPipelinePort: Send + Sync {
    async fn context(&self, match_id: Uuid) -> PortResult<FactPipelineContext>;
    async fn resolve_entity(
        &self,
        request: &EntityMatchRequest,
    ) -> PortResult<EntityResolutionRecord>;
    async fn append_entity_resolution(
        &self,
        draft: &EntityResolutionDraft,
    ) -> PortResult<EntityResolutionRecord>;
    async fn append_time_audit(&self, draft: &TimeAuditDraft) -> PortResult<TimeAuditRecord>;
    async fn append_conflict_evaluation(
        &self,
        draft: &ConflictEvaluationDraft,
    ) -> PortResult<ConflictEvaluationRecord>;
    async fn append_evidence_route(
        &self,
        draft: &EvidenceRouteDraft,
    ) -> PortResult<EvidenceRouteRecord>;
}

#[async_trait]
pub trait ResearchGatewayAuditPort: Send + Sync {
    async fn append_attempt(&self, draft: &OpenAiAttemptDraft) -> PortResult<OpenAiAttemptRecord>;
    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals>;
    async fn append_web_references(
        &self,
        run_id: Uuid,
        sources: &[WebSourceDraft],
        citations: &[WebCitationDraft],
    ) -> PortResult<()>;
}
''',
)

write(
    "crates/application/src/composition/adapters/research.rs",
    '''use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{
    research::{ResearchArtifactPort, ResearchEvidenceLedgerPort},
    PortResult,
};
use async_trait::async_trait;
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, EvidenceClaimDraft,
    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, PromptVersionDraft,
    PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,
    SchemaVersionDraft, SchemaVersionRecord, SourcePolicyVersionDraft, SourcePolicyVersionRecord,
};
use uuid::Uuid;

#[async_trait]
impl ResearchArtifactPort for ActiveDatabase {
    async fn read_schema(
        &self,
        schema_key: &str,
        version: &str,
    ) -> PortResult<SchemaVersionRecord> {
        self.transition_store()
            .read_schema_version_by_key(schema_key, version)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_schema(&self, draft: &SchemaVersionDraft) -> PortResult<SchemaVersionRecord> {
        self.transition_store()
            .register_schema_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_prompt(&self, draft: &PromptVersionDraft) -> PortResult<PromptVersionRecord> {
        self.transition_store()
            .register_prompt_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_source_policy(
        &self,
        draft: &SourcePolicyVersionDraft,
    ) -> PortResult<SourcePolicyVersionRecord> {
        self.transition_store()
            .register_source_policy_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_competition_profile(
        &self,
        draft: &CompetitionProfileVersionDraft,
    ) -> PortResult<CompetitionProfileVersionRecord> {
        self.transition_store()
            .register_competition_profile_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_run(&self, draft: &ResearchRunDraft) -> PortResult<ResearchRunRecord> {
        self.transition_store()
            .create_research_run(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_run(&self, run_id: Uuid) -> PortResult<ResearchRunRecord> {
        self.transition_store()
            .read_research_run(run_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn record_run_event(
        &self,
        draft: &ResearchRunEventDraft,
    ) -> PortResult<ResearchRunRecord> {
        self.transition_store()
            .record_research_run_event(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl ResearchEvidenceLedgerPort for ActiveDatabase {
    async fn append_evidence_claim(
        &self,
        draft: &EvidenceClaimDraft,
    ) -> PortResult<EvidenceClaimRecord> {
        self.transition_store()
            .append_evidence_claim(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_evidence_conflict(
        &self,
        draft: &EvidenceConflictDraft,
    ) -> PortResult<EvidenceConflictRecord> {
        self.transition_store()
            .create_evidence_conflict(draft)
            .await
            .map_err(map_persistence_error)
    }
}
''',
)

write(
    "crates/application/src/use_cases/research/mod.rs",
    "pub(crate) mod artifact_catalog;\npub(crate) mod ledger;\n",
)

write(
    "crates/application/src/use_cases/research/artifact_catalog.rs",
    '''use crate::built_in_artifacts::{
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
''',
)

write(
    "crates/application/src/use_cases/research/ledger.rs",
    '''use crate::ports::research::{ResearchArtifactPort, ResearchEvidenceLedgerPort};
use crate::ApplicationResult;
use football_domain::{
    EvidenceClaimDraft, EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord,
    ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,
};

pub(crate) async fn create_run<P: ResearchArtifactPort + ?Sized>(
    port: &P,
    draft: ResearchRunDraft,
) -> ApplicationResult<ResearchRunRecord> {
    Ok(port.create_run(&draft).await?)
}

pub(crate) async fn record_run_event<P: ResearchArtifactPort + ?Sized>(
    port: &P,
    draft: ResearchRunEventDraft,
) -> ApplicationResult<ResearchRunRecord> {
    Ok(port.record_run_event(&draft).await?)
}

pub(crate) async fn append_evidence_claim<P: ResearchEvidenceLedgerPort + ?Sized>(
    port: &P,
    draft: EvidenceClaimDraft,
) -> ApplicationResult<EvidenceClaimRecord> {
    Ok(port.append_evidence_claim(&draft).await?)
}

pub(crate) async fn create_evidence_conflict<P: ResearchEvidenceLedgerPort + ?Sized>(
    port: &P,
    draft: EvidenceConflictDraft,
) -> ApplicationResult<EvidenceConflictRecord> {
    Ok(port.create_evidence_conflict(&draft).await?)
}
''',
)

write(
    "crates/application/src/services/research/mod.rs",
    "mod facade;\nmod service;\n\npub(crate) use service::ResearchService;\n",
)

write(
    "crates/application/src/services/research/service.rs",
    '''use crate::ports::research::{ResearchArtifactPort, ResearchEvidenceLedgerPort};
use crate::use_cases::research::{artifact_catalog, ledger};
use crate::ApplicationResult;
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, EvidenceClaimDraft,
    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, PromptVersionDraft,
    PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,
    SchemaVersionDraft, SchemaVersionRecord,
};

pub(crate) struct ResearchService;

impl ResearchService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn register_persistence_artifacts<P: ResearchArtifactPort + ?Sized>(
        &self,
        port: &P,
    ) -> ApplicationResult<()> {
        artifact_catalog::register_built_ins(port).await
    }

    pub(crate) async fn register_p4_schema_version<P: ResearchArtifactPort + ?Sized>(
        &self,
        port: &P,
        draft: SchemaVersionDraft,
    ) -> ApplicationResult<SchemaVersionRecord> {
        artifact_catalog::register_schema(port, draft).await
    }

    pub(crate) async fn register_p4_prompt_version<P: ResearchArtifactPort + ?Sized>(
        &self,
        port: &P,
        draft: PromptVersionDraft,
    ) -> ApplicationResult<PromptVersionRecord> {
        artifact_catalog::register_prompt(port, draft).await
    }

    pub(crate) async fn register_p4_competition_profile_version<
        P: ResearchArtifactPort + ?Sized,
    >(
        &self,
        port: &P,
        draft: CompetitionProfileVersionDraft,
    ) -> ApplicationResult<CompetitionProfileVersionRecord> {
        artifact_catalog::register_competition_profile(port, draft).await
    }

    pub(crate) async fn create_p4_research_run<P: ResearchArtifactPort + ?Sized>(
        &self,
        port: &P,
        draft: ResearchRunDraft,
    ) -> ApplicationResult<ResearchRunRecord> {
        ledger::create_run(port, draft).await
    }

    pub(crate) async fn record_p4_research_run_event<P: ResearchArtifactPort + ?Sized>(
        &self,
        port: &P,
        draft: ResearchRunEventDraft,
    ) -> ApplicationResult<ResearchRunRecord> {
        ledger::record_run_event(port, draft).await
    }

    pub(crate) async fn append_p4_evidence_claim<P: ResearchEvidenceLedgerPort + ?Sized>(
        &self,
        port: &P,
        draft: EvidenceClaimDraft,
    ) -> ApplicationResult<EvidenceClaimRecord> {
        ledger::append_evidence_claim(port, draft).await
    }

    pub(crate) async fn create_p4_evidence_conflict<P: ResearchEvidenceLedgerPort + ?Sized>(
        &self,
        port: &P,
        draft: EvidenceConflictDraft,
    ) -> ApplicationResult<EvidenceConflictRecord> {
        ledger::create_evidence_conflict(port, draft).await
    }
}
''',
)

write(
    "crates/application/src/services/research/facade.rs",
    '''use crate::composition::ActiveDatabase;
use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, EvidenceClaimDraft,
    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, PromptVersionDraft,
    PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,
    SchemaVersionDraft, SchemaVersionRecord,
};

impl ApplicationService {
    async fn research_session(&self) -> ApplicationResult<ActiveDatabase> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }

    pub async fn register_p4_schema_version(
        &self,
        draft: SchemaVersionDraft,
    ) -> ApplicationResult<SchemaVersionRecord> {
        let session = self.research_session().await?;
        self.research
            .register_p4_schema_version(&session, draft)
            .await
    }

    pub async fn register_p4_prompt_version(
        &self,
        draft: PromptVersionDraft,
    ) -> ApplicationResult<PromptVersionRecord> {
        let session = self.research_session().await?;
        self.research
            .register_p4_prompt_version(&session, draft)
            .await
    }

    pub async fn register_p4_competition_profile_version(
        &self,
        draft: CompetitionProfileVersionDraft,
    ) -> ApplicationResult<CompetitionProfileVersionRecord> {
        let session = self.research_session().await?;
        self.research
            .register_p4_competition_profile_version(&session, draft)
            .await
    }

    pub async fn create_p4_research_run(
        &self,
        draft: ResearchRunDraft,
    ) -> ApplicationResult<ResearchRunRecord> {
        let session = self.research_session().await?;
        self.research.create_p4_research_run(&session, draft).await
    }

    pub async fn record_p4_research_run_event(
        &self,
        draft: ResearchRunEventDraft,
    ) -> ApplicationResult<ResearchRunRecord> {
        let session = self.research_session().await?;
        self.research
            .record_p4_research_run_event(&session, draft)
            .await
    }

    pub async fn append_p4_evidence_claim(
        &self,
        draft: EvidenceClaimDraft,
    ) -> ApplicationResult<EvidenceClaimRecord> {
        let session = self.research_session().await?;
        self.research.append_p4_evidence_claim(&session, draft).await
    }

    pub async fn create_p4_evidence_conflict(
        &self,
        draft: EvidenceConflictDraft,
    ) -> ApplicationResult<EvidenceConflictRecord> {
        let session = self.research_session().await?;
        self.research
            .create_p4_evidence_conflict(&session, draft)
            .await
    }
}
''',
)

write(
    "crates/application/src/services/mod.rs",
    '''pub(crate) mod competition;
pub(crate) mod database;
pub(crate) mod lineups;
pub(crate) mod players;
pub(crate) mod prediction;
pub(crate) mod research;
pub(crate) mod rules;
pub(crate) mod teams;
''',
)

write(
    "crates/application/src/use_cases/mod.rs",
    '''pub(crate) mod competition;
pub(crate) mod database;
pub(crate) mod lineups;
pub(crate) mod players;
pub(crate) mod prediction;
pub(crate) mod research;
pub(crate) mod rules;
pub(crate) mod teams;
''',
)

write(
    "crates/application/src/service/application_service.rs",
    '''use crate::composition::ApplicationComposition;
use crate::model_registry::ModelRegistry;
use crate::services::{
    competition::CompetitionService, database::DatabaseService, lineups::LineupService,
    players::PlayerService, prediction::PredictionService, research::ResearchService,
    rules::RulesService, teams::TeamService,
};
use std::sync::atomic::AtomicBool;
pub struct ApplicationService {
    pub(crate) registry: ModelRegistry,
    pub(crate) database: DatabaseService,
    pub(crate) competition: CompetitionService,
    pub(crate) rules: RulesService,
    pub(crate) teams: TeamService,
    pub(crate) players: PlayerService,
    pub(crate) lineups: LineupService,
    pub(crate) prediction: PredictionService,
    pub(crate) research: ResearchService,
    pub(crate) p4_worker_running: AtomicBool,
}
impl ApplicationService {
    pub fn new() -> Self {
        let (
            registry,
            database,
            competition,
            rules,
            teams,
            players,
            lineups,
            prediction,
            research,
            p4_worker_running,
        ) = ApplicationComposition::new().into_parts();
        Self {
            registry,
            database,
            competition,
            rules,
            teams,
            players,
            lineups,
            prediction,
            research,
            p4_worker_running,
        }
    }
}
impl Default for ApplicationService {
    fn default() -> Self {
        Self::new()
    }
}
''',
)

write(
    "crates/application/src/composition/application_composition.rs",
    '''use super::PortRegistry;
use crate::model_registry::ModelRegistry;
use crate::model_shell::PublicModelStub;
use crate::services::{
    competition::CompetitionService, database::DatabaseService, lineups::LineupService,
    players::PlayerService, prediction::PredictionService, research::ResearchService,
    rules::RulesService, teams::TeamService,
};
use std::sync::{atomic::AtomicBool, Arc};
pub(crate) struct ApplicationComposition {
    registry: ModelRegistry,
    database: DatabaseService,
    competition: CompetitionService,
    rules: RulesService,
    teams: TeamService,
    players: PlayerService,
    lineups: LineupService,
    prediction: PredictionService,
    research: ResearchService,
    p4_worker_running: AtomicBool,
}
impl ApplicationComposition {
    pub(crate) fn new() -> Self {
        let mut registry = ModelRegistry::new();
        for model in PublicModelStub::built_in_models() {
            registry.register(Arc::new(model));
        }
        let database = DatabaseService::new(PortRegistry::new());
        Self {
            registry,
            database,
            competition: CompetitionService::new(),
            rules: RulesService::new(),
            teams: TeamService::new(),
            players: PlayerService::new(),
            lineups: LineupService::new(),
            prediction: PredictionService::new(),
            research: ResearchService::new(),
            p4_worker_running: AtomicBool::new(false),
        }
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        ModelRegistry,
        DatabaseService,
        CompetitionService,
        RulesService,
        TeamService,
        PlayerService,
        LineupService,
        PredictionService,
        ResearchService,
        AtomicBool,
    ) {
        (
            self.registry,
            self.database,
            self.competition,
            self.rules,
            self.teams,
            self.players,
            self.lineups,
            self.prediction,
            self.research,
            self.p4_worker_running,
        )
    }
}
''',
)

replace_once(
    "crates/application/src/lib.rs",
    "mod p4_orchestration;\nmod p4_persistence;\nmod p4_workbench;",
    "mod p4_orchestration;\nmod p4_workbench;",
)

replace_once(
    "crates/application/src/services/database/facade.rs",
    "        let store = prepared.transition_store();\n        self.register_p4_persistence_artifacts(&store).await?;\n        self.register_openai_research_artifacts(&store).await",
    "        self.research\n            .register_persistence_artifacts(prepared.session())\n            .await?;\n        let store = prepared.transition_store();\n        self.register_openai_research_artifacts(&store).await",
)

p4_persistence = ROOT / "crates/application/src/p4_persistence.rs"
if not p4_persistence.exists():
    raise RuntimeError("p4_persistence.rs missing before R3-07 AT1 migration")
p4_persistence.unlink()

write(
    "scripts/verify-research-service.mjs",
    '''import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8").replaceAll("\\r\\n", "\\n");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
function rustFiles(path) {
  const absolute = join(root, path);
  const result = [];
  for (const entry of readdirSync(absolute, { withFileTypes: true })) {
    const child = join(absolute, entry.name);
    if (entry.isDirectory()) result.push(...rustFiles(relative(root, child)));
    else if (entry.name.endsWith(".rs")) result.push(relative(root, child).replaceAll("\\\\", "/"));
  }
  return result;
}

const publicMethods = [
  "register_p4_schema_version",
  "register_p4_prompt_version",
  "register_p4_competition_profile_version",
  "create_p4_research_run",
  "record_p4_research_run_event",
  "append_p4_evidence_claim",
  "create_p4_evidence_conflict",
];
const required = [
  "crates/application/src/services/research/mod.rs",
  "crates/application/src/services/research/service.rs",
  "crates/application/src/services/research/facade.rs",
  "crates/application/src/use_cases/research/mod.rs",
  "crates/application/src/use_cases/research/artifact_catalog.rs",
  "crates/application/src/use_cases/research/ledger.rs",
  "crates/application/src/composition/adapters/research.rs",
  "crates/application/src/ports/research/mod.rs",
];
for (const path of required) check(existsSync(join(root, path)), `缺少 R3-07 文件：${path}`);
check(!existsSync(join(root, "crates/application/src/p4_persistence.rs")), "旧 p4_persistence.rs 仍残留 Research Artifact/Ledger owner");

const facade = read("crates/application/src/services/research/facade.rs");
const service = read("crates/application/src/services/research/service.rs");
const ports = read("crates/application/src/ports/research/mod.rs");
const adapter = read("crates/application/src/composition/adapters/research.rs");
const databaseFacade = read("crates/application/src/services/database/facade.rs");
const lib = read("crates/application/src/lib.rs");
const packageJson = JSON.parse(read("package.json"));
const frontend = read("scripts/verify-frontend.mjs");

for (const method of publicMethods) {
  check(facade.includes(`fn ${method}`), `Research facade 缺少公共兼容方法：${method}`);
  check(service.includes(`fn ${method}`), `ResearchService 缺少职责：${method}`);
}
check(ports.includes("trait ResearchArtifactPort"), "Research Ports 缺少 ResearchArtifactPort");
check(ports.includes("register_competition_profile"), "ResearchArtifactPort 缺少赛事配置版本写入能力");
check(ports.includes("PortResult<ResearchRunRecord>"), "Research run event 未保留公开返回记录契约");
check(ports.includes("trait ResearchEvidenceLedgerPort"), "Research Ports 缺少 ResearchEvidenceLedgerPort");
check(adapter.includes("impl ResearchEvidenceLedgerPort for ActiveDatabase"), "Research 组合适配器缺少 Evidence Ledger 实现");
check(adapter.includes(".append_evidence_claim(draft)"), "Evidence claim 未复用既有持久化能力");
check(adapter.includes(".create_evidence_conflict(draft)"), "Evidence conflict 未复用既有持久化能力");
check(databaseFacade.includes(".register_persistence_artifacts(prepared.session())"), "数据库初始化未通过 ResearchService 注册内置 Research schema");
check(!databaseFacade.includes("register_p4_persistence_artifacts"), "数据库初始化仍引用旧 p4_persistence owner");
check(!lib.includes("mod p4_persistence;"), "Application 根模块仍登记旧 p4_persistence owner");
for (const path of ["crates/application/src/openai_research.rs", "crates/application/src/fact_pipeline.rs", "crates/application/src/p4_orchestration.rs", "crates/application/src/p4_workbench.rs"]) {
  check(existsSync(join(root, path)), `后续 R3-07 职责被提前删除：${path}`);
}

const researchFiles = [
  ...rustFiles("crates/application/src/services/research"),
  ...rustFiles("crates/application/src/use_cases/research"),
];
for (const path of researchFiles) {
  const source = read(path);
  for (const token of ["football_persistence_postgres", "PostgresStore", "sqlx::", "PgPool", "PersistenceStore"]) {
    check(!source.includes(token), `${path} 泄漏具体持久化实现：${token}`);
  }
}
check(packageJson.scripts?.["verify:research-service"] === "node scripts/verify-research-service.mjs", "package.json 未登记 R3-07 专项门禁");
check(packageJson.scripts?.["verify:architecture"]?.includes("verify-research-service.mjs"), "verify:architecture 未接入 R3-07 门禁");
check(frontend.includes('"verify-research-service.mjs"'), "verify:frontend 未接入 R3-07 门禁");

if (failures.length) throw new Error(`Research Service 验证失败\\n${failures.map((item) => `- ${item}`).join("\\n")}`);
console.log(`Research Service AT1 验证通过：${researchFiles.length} 个 Service/Use Case Rust 文件，7 个公开 Artifact/Ledger API 已迁入 ResearchService/Ports，旧 p4_persistence.rs 已删除。`);
''',
)

replace_once(
    "package.json",
    '"verify:architecture": "node scripts/architecture/verifyModuleBoundaries.mjs && node scripts/architecture/verifyStateOwnership.mjs && node scripts/architecture/verifyProtectedImports.mjs && node scripts/verify-domain-type-inventory.mjs && node scripts/verify-domain-root-exports.mjs && node scripts/verify-application-ports.mjs && node scripts/verify-database-service.mjs && node scripts/verify-competition-rules-service.mjs && node scripts/verify-teams-players-service.mjs && node scripts/verify-lineups-service.mjs && node scripts/verify-prediction-service.mjs"',
    '"verify:architecture": "node scripts/architecture/verifyModuleBoundaries.mjs && node scripts/architecture/verifyStateOwnership.mjs && node scripts/architecture/verifyProtectedImports.mjs && node scripts/verify-domain-type-inventory.mjs && node scripts/verify-domain-root-exports.mjs && node scripts/verify-application-ports.mjs && node scripts/verify-database-service.mjs && node scripts/verify-competition-rules-service.mjs && node scripts/verify-teams-players-service.mjs && node scripts/verify-lineups-service.mjs && node scripts/verify-prediction-service.mjs && node scripts/verify-research-service.mjs"',
)
replace_once(
    "package.json",
    '    "verify:prediction-service": "node scripts/verify-prediction-service.mjs"\n',
    '    "verify:prediction-service": "node scripts/verify-prediction-service.mjs",\n    "verify:research-service": "node scripts/verify-research-service.mjs"\n',
)
replace_once(
    "scripts/verify-frontend.mjs",
    '  "verify-prediction-service.mjs",\n',
    '  "verify-prediction-service.mjs",\n  "verify-research-service.mjs",\n',
)

root_readme = ROOT / "README.md"
root_text = root_readme.read_text(encoding="utf-8")
root_text = root_text.replace(
    "R3-07 Research Service 已开放为 `READY`。",
    "R3-07 Research Service 为 `IN_PROGRESS`。",
    1,
)
if "## R3-07 Research Service（IN_PROGRESS）" not in root_text:
    marker = "\n## R2-04 Lineup 与 Match\n"
    if marker not in root_text:
        raise RuntimeError("root README R3-07 insertion marker missing")
    section = f'''\n## R3-07 Research Service（IN_PROGRESS）\n\n- Atomic Task 1 建立 Research Service / Use Case / Ports 骨架，并迁移原 `p4_persistence.rs` 的 7 个公开 Research Artifact / Ledger API：schema、prompt、赛事配置版本、research run、run event、evidence claim、evidence conflict。数据库初始化中的内置 P4 schema 注册同步改经 ResearchService；旧 `p4_persistence.rs` 删除，不保留空转发层。\n- 新增 `ResearchEvidenceLedgerPort`，并扩展既有 `ResearchArtifactPort` 的赛事配置版本和 run-event 返回契约；具体 PostgreSQL 仍仅由 `composition/adapters/research.rs` 适配，Research Service / Use Case 不直接依赖 PersistenceStore、PostgresStore、SQLx 或 PgPool。\n- `verify:research-service` 已接入 `verify:architecture` 与 `verify:frontend`。Atomic Task 1 Windows hard gate run `{RUN_ID}` 只有在 Research 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests 全部通过后才形成 clean 提交。OpenAI Research、Fact Pipeline、P4 Research worker 和人工冲突裁决留给后续 R3-07 Atomic Tasks。\n\n'''
    root_text = root_text.replace(marker, "\n" + section + "## R2-04 Lineup 与 Match\n", 1)
root_readme.write_text(root_text, encoding="utf-8", newline="\n")

index_path = ROOT / "docs/modular-rewrite/R03-application-services/README.md"
index_text = index_path.read_text(encoding="utf-8")
index_text = index_text.replace(
    "| R3-07 | Research Service | READY |",
    "| R3-07 | Research Service | IN_PROGRESS |",
    1,
)
if "## R3-07 当前结果" not in index_text:
    index_text += f'''\n\n## R3-07 当前结果\n\n- Atomic Task 1 已建立 ResearchService，并按 Artifact Catalog / Research Ledger 两个职责模块迁移原 `p4_persistence.rs` 的 7 个公开写入口；数据库初始化的内置 schema 注册也改经 ResearchService。旧 `p4_persistence.rs` 删除。\n- Research Ports 新增 `ResearchEvidenceLedgerPort`，`ResearchArtifactPort` 补齐赛事配置版本与 run-event 返回记录能力；PostgreSQL 适配保持在 composition 层。\n- `verify:research-service` 已接入 architecture / frontend。Windows hard gate run `{RUN_ID}` 负责 Research 专项、Ports、architecture、Application check/tests、workspace Clippy/tests。OpenAI Research、Fact Pipeline、Research worker 与人工 conflict mutation 尚未迁移，R3-07 保持 `IN_PROGRESS`。\n'''
index_path.write_text(index_text, encoding="utf-8", newline="\n")

print("R3-07 Atomic Task 1 migration generated")
