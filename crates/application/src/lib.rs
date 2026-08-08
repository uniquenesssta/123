mod analytics;
mod api_workspace;
mod built_in_artifacts;
mod competition;
mod composition;
mod exchange;
mod fact_pipeline;
mod match_review_package;
mod model_registry;
mod model_shell;
mod openai_research;
mod p4_orchestration;
mod p4_persistence;
mod p4_workbench;
mod player_catalog;
pub mod ports;
mod postmatch;
mod prediction;
mod release_acceptance;
mod review;
mod rule_packages;
mod service;
mod services;
mod spreadsheet;
mod use_cases;

use football_domain::{
    CompetitionBindingSummary, CompetitionKind, CompetitionRecord, PredictionInputAuditSummary,
    RoundRecord, RouteDecision, RulePackageDraft, RulePackageSummary, SeasonRecord, StageRecord,
};
use football_model_api::{ModelDescriptor, ModelOutput};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

pub(crate) use composition::{
    DatabaseHealth, DatabaseStats, ModelRunListItem, PersistenceError, PersistenceStore,
};
pub use model_registry::ModelRegistry;
pub use service::ApplicationService;

pub use api_workspace::{
    api_workspace_preset_spec, api_workspace_preset_specs, api_workspace_presets,
    read_api_workspace_attachments, ApiWorkspacePresetSpec,
};
pub use fact_pipeline::ProcessResearchEvidenceCommand;
pub use model_shell::{default_match, default_parameters, p4_default_match, p4_default_parameters};
pub use openai_research::OpenAiResearchCommand;
pub use rule_packages::default_rule_package_template;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("数据库尚未连接，请先完成 PostgreSQL 配置")]
    DatabaseNotConnected,
    #[error("未注册模型：{0}")]
    ModelNotFound(String),
    #[error("比赛开球时间无效：{0}")]
    InvalidKickoff(String),
    #[error("赛事或规则包输入无效：{0}")]
    Validation(String),
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
    #[error(transparent)]
    Port(#[from] ports::PortError),
    #[error("模型执行失败：{0}")]
    Model(String),
    #[error("Excel 处理失败：{0}")]
    Spreadsheet(#[from] football_spreadsheet_io::SpreadsheetError),
    #[error("AI 分析包处理失败：{0}")]
    AnalysisPackage(#[from] football_analysis_package::AnalysisPackageError),
    #[error("OpenAI联网事实研究失败：{0}")]
    ResearchGateway(#[from] football_research_gateway::GatewayError),
    #[error("JSON 解析失败：{0}")]
    Json(#[from] serde_json::Error),
    #[error("文件系统错误：{0}")]
    Io(#[from] std::io::Error),
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapData {
    pub app_version: String,
    pub database_configured: bool,
    pub database_url: Option<String>,
    pub database_health: Option<DatabaseHealth>,
    pub stats: Option<DatabaseStats>,
    pub models: Vec<ModelDescriptor>,
    pub competitions: Vec<CompetitionRecord>,
    pub seasons: Vec<SeasonRecord>,
    pub stages: Vec<StageRecord>,
    pub rounds: Vec<RoundRecord>,
    pub rule_packages: Vec<RulePackageSummary>,
    pub competition_bindings: Vec<CompetitionBindingSummary>,
    pub recent_runs: Vec<ModelRunListItem>,
    pub default_match: Value,
    pub default_rule_package: RulePackageDraft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionCommand {
    pub match_input: Value,
    #[serde(default = "default_snapshot_type")]
    pub snapshot_type: String,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub season_id: Option<Uuid>,
    #[serde(default)]
    pub stage_id: Option<Uuid>,
    #[serde(default)]
    pub competition_kind: CompetitionKind,
    #[serde(default = "default_model_family")]
    pub model_family: String,
    #[serde(default)]
    pub explicit_rule_package_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMatchPredictionCommand {
    pub match_id: Uuid,
    #[serde(default = "default_snapshot_type")]
    pub snapshot_type: String,
    #[serde(default = "default_model_family")]
    pub model_family: String,
    #[serde(default)]
    pub explicit_rule_package_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePreviewCommand {
    pub kickoff_time: String,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub season_id: Option<Uuid>,
    #[serde(default)]
    pub stage_id: Option<Uuid>,
    #[serde(default)]
    pub competition_kind: CompetitionKind,
    #[serde(default = "default_model_family")]
    pub model_family: String,
    #[serde(default)]
    pub explicit_rule_package_id: Option<Uuid>,
}

fn default_snapshot_type() -> String {
    "T-N".to_string()
}

fn default_model_family() -> String {
    "p4".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionExecution {
    pub run_id: Uuid,
    pub duration_ms: i64,
    pub route: RouteDecision,
    pub output: ModelOutput,
    #[serde(default)]
    pub input_audit: Option<PredictionInputAuditSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashSet;

    #[test]
    fn built_in_rule_packages_are_unique_and_valid() {
        let service = ApplicationService::new();
        let mut package_keys = HashSet::new();
        let mut model_ids = HashSet::new();
        let expected_version = format!("{}+public.1", env!("CARGO_PKG_VERSION"));

        for draft in rule_packages::built_in_rule_packages() {
            assert!(package_keys.insert(draft.package_key.clone()));
            model_ids.insert(draft.routing.model_id.clone());
            assert_eq!(draft.version, expected_version);
            rule_packages::validate_rule_package_shape(&draft).expect("内置规则包结构无效");
            rule_packages::validate_parameter_identity(&draft).expect("内置规则包版本无效");
            let model = service
                .registry
                .get(&draft.routing.model_id)
                .expect("内置规则包引用了未注册模型");
            assert!(model
                .descriptor()
                .supported_competitions
                .contains(&draft.competition_profile.competition_kind));
            model
                .validate_parameters(&draft.parameters)
                .expect("内置规则包参数无效");
        }

        assert_eq!(package_keys.len(), CompetitionKind::ALL.len() * 2);
        assert_eq!(model_ids.len(), CompetitionKind::ALL.len() * 2);
    }

    #[test]
    fn public_model_entries_are_external_provider_stubs() {
        let service = ApplicationService::new();
        assert!(service
            .registry
            .descriptors()
            .iter()
            .all(|descriptor| descriptor.engine_version == "external-provider"));
    }

    #[test]
    fn generated_match_key_is_written_back_to_manual_input() {
        let input = json!({"kickoff_time": "2026-07-20T12:00:00Z"});
        let normalized = prediction::ensure_match_input_id(input, "SIM-20260720-TEAM-A-TEAM-B")
            .expect("手工输入应能补齐比赛键");
        assert_eq!(
            normalized.get("match_id").and_then(Value::as_str),
            Some("SIM-20260720-TEAM-A-TEAM-B")
        );
    }

    #[test]
    fn user_rule_package_template_is_self_consistent() {
        let draft = default_rule_package_template();
        rule_packages::validate_rule_package_shape(&draft).expect("用户规则包模板结构无效");
        rule_packages::validate_parameter_identity(&draft).expect("用户规则包版本无效");
        assert_eq!(draft.format_version, "football.rule-package.v1");
        assert_eq!(draft.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(
            draft.competition_profile.competition_kind,
            CompetitionKind::Custom
        );
    }
}
