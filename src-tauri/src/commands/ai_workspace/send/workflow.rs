use super::{
    content::{bounded_context_text, bounded_plain_text_history},
    gateway,
    metadata::{metadata_text, metadata_uuid},
    request::SendApiWorkspaceCommand,
    response, session, validation,
};
use crate::bootstrap::AppState;
use football_domain::ApiWorkspaceSessionDetail;

pub(super) async fn execute(
    state: &AppState,
    command: SendApiWorkspaceCommand,
) -> Result<ApiWorkspaceSessionDetail, String> {
    let input = validation::validate(&command)?;
    let request_id = input.request_id;
    let message = input.message;
    let profile_id = input.profile_id;
    let registration = state
        .api_workspace_requests
        .register(request_id)
        .map_err(|error| error.to_string())?;
    let connection = state.openai_profiles.gateway_for(Some(profile_id))?;
    let session = session::resolve(state, &command, &input, &connection.0).await?;
    state
        .service
        .append_api_workspace_user_message(session.id, message.to_string(), &[])
        .await
        .map_err(|error| error.to_string())?;
    let history_detail = state
        .service
        .read_api_workspace_session(session.id)
        .await
        .map_err(|error| error.to_string())?;
    let context_entity_type = metadata_text(&session.metadata, "context_entity_type");
    let context_entity_id = metadata_uuid(&session.metadata, "context_entity_id")?;
    let context_note = if command.include_context {
        let context = state
            .service
            .api_workspace_context(
                session.match_id,
                context_entity_type.as_deref(),
                context_entity_id,
            )
            .await
            .map_err(|error| error.to_string())?;
        format!(
            "\n\nThe following desktop data is read-only context. Use it only to answer the question. Do not propose writes or claim it is current public web information:\n{}",
            bounded_context_text(&context)
        )
    } else {
        String::new()
    };
    let mut messages = bounded_plain_text_history(&history_detail);
    if let Some(last) = messages.last_mut() {
        if last.role == "user" {
            last.content.push_str(&context_note);
        }
    }
    let execution = gateway::execute(
        state,
        &command,
        &input,
        &connection,
        session.id,
        messages,
        registration,
    )
    .await?;
    let assistant_message = response::assistant_message(session.id, execution);
    state
        .service
        .append_api_workspace_assistant_bundle(assistant_message, vec![], vec![])
        .await
        .map_err(|error| error.to_string())
}
