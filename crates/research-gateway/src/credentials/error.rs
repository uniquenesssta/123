use crate::{GatewayError, GatewayErrorCategory};

pub(super) fn missing_key(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::MissingCredential,
        message,
        false,
        "在Windows凭据管理器中保存OpenAI API密钥后重试；不要把密钥写入源码或配置文件",
    )
}
