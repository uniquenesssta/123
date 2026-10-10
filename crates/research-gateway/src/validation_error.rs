use crate::{GatewayError, GatewayErrorCategory};

pub(crate) fn schema_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::SchemaValidation,
        message,
        false,
        "保留原始响应并将任务标记为结构校验失败；修正Prompt或Schema版本后重试",
    )
}

pub(crate) fn source_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::SourcePolicy,
        message,
        false,
        "检查来源策略、引用完整性和禁用内容后重新执行研究任务",
    )
}
