use super::validation::validate_monitoring_request;
use crate::PersistenceError;
use football_domain::PostmatchMonitoringRequest;
use uuid::Uuid;

fn request(horizon: &str, baseline_size: usize, current_size: usize) -> PostmatchMonitoringRequest {
    PostmatchMonitoringRequest {
        competition_id: Uuid::nil(),
        horizon: horizon.into(),
        baseline_size,
        current_size,
    }
}

#[test]
fn monitoring_accepts_original_four_horizons_and_five_sample_boundary() {
    for horizon in ["T-N", "T-24h", "T-6h", "T-1h"] {
        validate_monitoring_request(&request(horizon, 5, 5)).unwrap();
        validate_monitoring_request(&request(horizon, usize::MAX, usize::MAX)).unwrap();
    }
}

#[test]
fn monitoring_rejects_horizon_before_window_validation() {
    for horizon in ["T-90m", "t-1h", " T-1h ", "", "unknown"] {
        assert!(
            matches!(validate_monitoring_request(&request(horizon, 0, 0)).unwrap_err(), PersistenceError::InvalidState(message) if message == "正式赛后监控只支持 T-N、T-24h、T-6h、T-1h")
        );
    }
}

#[test]
fn monitoring_rejects_either_short_window_with_original_error() {
    for (baseline_size, current_size) in [(0, 5), (4, 5), (5, 0), (5, 4), (4, 4)] {
        assert!(
            matches!(validate_monitoring_request(&request("T-1h", baseline_size, current_size)).unwrap_err(), PersistenceError::InvalidState(message) if message == "漂移基线窗口和当前窗口都不能少于 5 场")
        );
    }
}
