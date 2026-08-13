use crate::composition::DatabaseSession;
use crate::{ApplicationResult, ApplicationService};

pub(crate) async fn execute(
    application: &ApplicationService,
    session: &DatabaseSession,
) -> ApplicationResult<()> {
    application
        .rules
        .register_built_ins(&application.registry, session)
        .await?;
    application
        .research
        .register_persistence_artifacts(session)
        .await?;
    application
        .research
        .register_openai_research_artifacts(session)
        .await
}
