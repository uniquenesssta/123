use super::parse::parse_success_response;
use crate::formal_research::schema::{validate_research_output, ValidationContext};
use crate::{GatewayError, GatewayRequest, GatewayResponse, SourcePolicy, TransportResponse};

pub(crate) fn parse_and_validate(
    source_policy: &SourcePolicy,
    request: &GatewayRequest,
    response: TransportResponse,
) -> Result<GatewayResponse, GatewayError> {
    let parsed =
        parse_success_response(response.status, response.provider_request_id, response.body)?;
    validate_research_output(
        &parsed.output,
        &ValidationContext {
            match_key: &request.match_key,
            schema_version: &request.schema_version,
            data_cutoff_at: request.data_cutoff_at,
            requested_fact_keys: &request.requested_fact_keys,
            source_policy,
            citations: &parsed.citations,
            sources: &parsed.sources,
        },
    )?;
    Ok(parsed)
}
