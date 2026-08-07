use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAiUsageTotals {
    pub today_cost_usd: f64,
    pub month_cost_usd: f64,
    pub today_request_count: u64,
    pub month_request_count: u64,
}
