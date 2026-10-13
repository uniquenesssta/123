use crate::ApplicationResult;
use football_domain::AnalyticsRefreshRequest;
use serde_json::Value;

pub(super) fn parse_refresh_request(payload: &Value) -> ApplicationResult<AnalyticsRefreshRequest> {
    if payload.is_null() || payload.as_object().is_some_and(|item| item.is_empty()) {
        Ok(AnalyticsRefreshRequest::default())
    } else {
        Ok(serde_json::from_value(payload.clone())?)
    }
}
