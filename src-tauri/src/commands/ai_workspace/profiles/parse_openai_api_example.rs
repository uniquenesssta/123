use football_research_gateway::{parse_api_example, ApiExampleParseResult, ApiProtocol};

#[tauri::command]
pub fn parse_openai_api_example(
    example: String,
    preferred_protocol: Option<ApiProtocol>,
) -> Result<ApiExampleParseResult, String> {
    parse_api_example(&example, preferred_protocol).map_err(|error| error.to_string())
}
