use football_domain::{AnalyticsOverview, DataQualitySummary, QueryPerformanceSummary};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct ParameterDefinition {
    value: Value,
}

impl ParameterDefinition {
    pub(crate) fn from_value(value: Value) -> Self {
        Self { value }
    }

    pub(crate) fn as_value(&self) -> &Value {
        &self.value
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AnalyticsJobProgressPayload {
    Empty,
    SampleSize(u64),
    OpenFindings(i64),
}

impl AnalyticsJobProgressPayload {
    pub(crate) fn into_value(self) -> Value {
        match self {
            Self::Empty => json!({}),
            Self::SampleSize(sample_size) => json!({ "sample_size": sample_size }),
            Self::OpenFindings(open_findings) => json!({ "open_findings": open_findings }),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AnalyticsJobResult {
    AnalyticsOverview(Box<AnalyticsOverview>),
    DataQuality(DataQualitySummary),
    QueryPerformance(QueryPerformanceSummary),
    FullAnalysisRefresh {
        sample_size: u64,
        expected_calibration_error: Option<f64>,
        quality_findings: i64,
        database_size_bytes: i64,
    },
}

impl AnalyticsJobResult {
    pub(crate) fn into_value(self) -> serde_json::Result<Value> {
        match self {
            Self::AnalyticsOverview(value) => serde_json::to_value(value),
            Self::DataQuality(value) => serde_json::to_value(value),
            Self::QueryPerformance(value) => serde_json::to_value(value),
            Self::FullAnalysisRefresh {
                sample_size,
                expected_calibration_error,
                quality_findings,
                database_size_bytes,
            } => Ok(json!({
                "sample_size": sample_size,
                "expected_calibration_error": expected_calibration_error,
                "quality_findings": quality_findings,
                "database_size_bytes": database_size_bytes,
            })),
        }
    }
}
