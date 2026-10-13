use crate::{PersistenceError, PersistenceResult};

pub(crate) const FORMAL_HORIZONS: [&str; 4] = ["T-N", "T-24h", "T-6h", "T-1h"];

pub(crate) fn non_negative_u64(value: i64, label: &str) -> PersistenceResult<u64> {
    u64::try_from(value).map_err(|_| PersistenceError::InvalidState(format!("{label}不能为负数")))
}

pub(crate) fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
