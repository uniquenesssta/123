use football_domain::{CompetitionKind, MatchContext, ModelIdentity, PredictionSummary};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("模型输入无效：{0}")]
    InvalidInput(String),
    #[error("模型参数无效：{0}")]
    InvalidParameters(String),
    #[error("模型计算失败：{0}")]
    Calculation(String),
    #[error("模型序列化失败：{0}")]
    Serialization(String),
    #[error("模型运行时不可用：{0}")]
    Unavailable(String),
}

pub type ModelResult<T> = Result<T, ModelError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub model_id: String,
    pub display_name: String,
    pub engine_version: String,
    pub supported_competitions: Vec<CompetitionKind>,
    pub input_schema_version: String,
    pub output_schema_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    pub context: MatchContext,
    pub identity: ModelIdentity,
    pub snapshot_type: String,
    pub input: Value,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelOutput {
    pub identity: ModelIdentity,
    pub summary: PredictionSummary,
    pub payload: Value,
    pub explanation: Value,
}

pub trait PredictionModel: Send + Sync {
    fn descriptor(&self) -> ModelDescriptor;
    fn supports(&self, context: &MatchContext) -> bool;
    fn validate_input(&self, input: &Value) -> ModelResult<()>;
    fn validate_parameters(&self, parameters: &Value) -> ModelResult<()>;

    fn validate(&self, request: &ModelRequest) -> ModelResult<()> {
        self.validate_input(&request.input)?;
        self.validate_parameters(&request.parameters)
    }

    fn predict(&self, request: &ModelRequest) -> ModelResult<ModelOutput>;
}
