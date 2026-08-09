use super::*;

pub(super) fn built_in_gateway() -> Result<OpenAiResearchGateway, GatewayError> {
    OpenAiResearchGateway::new(
        built_in_gateway_config()?,
        Arc::new(ReqwestTransport::new()?),
        Arc::new(DefaultApiKeyProvider),
    )
}

pub(super) fn built_in_gateway_config() -> Result<GatewayConfig, GatewayError> {
    let config: GatewayConfig = serde_json::from_str(include_str!(
        "../../../../../../src-tauri/resources/research/openai_gateway.json"
    ))
    .map_err(|error| {
        GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            format!("内置OpenAI研究网关配置无效：{error}"),
            false,
            "修正版本化网关配置后重新构建应用",
        )
    })?;
    config.validate()?;
    Ok(config)
}
