use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAcceptanceRequest {
    #[serde(default = "default_performance_window_days")]
    pub performance_window_days: u32,
    #[serde(default = "default_cost_window_days")]
    pub cost_window_days: u32,
    #[serde(default)]
    pub daily_cost_budget_usd: Option<f64>,
    #[serde(default)]
    pub monthly_cost_budget_usd: Option<f64>,
    #[serde(default)]
    pub requested_by: Option<String>,
}

fn default_performance_window_days() -> u32 {
    30
}

fn default_cost_window_days() -> u32 {
    30
}

impl Default for ReleaseAcceptanceRequest {
    fn default() -> Self {
        Self {
            performance_window_days: default_performance_window_days(),
            cost_window_days: default_cost_window_days(),
            daily_cost_budget_usd: None,
            monthly_cost_budget_usd: None,
            requested_by: None,
        }
    }
}
