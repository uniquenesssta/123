use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiResearchCommand {
    pub research_run_id: Uuid,
    pub trace_id: Uuid,
    pub match_key: String,
    pub data_cutoff_at: DateTime<Utc>,
    #[serde(default = "default_operation")]
    pub operation: GatewayOperation,
    pub dynamic_context: Value,
    pub requested_fact_keys: Vec<String>,
}

fn default_operation() -> GatewayOperation {
    GatewayOperation::Research
}
