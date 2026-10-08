use super::command::ProcessResearchEvidenceCommand;
use super::evidence::process_fact_group;
use super::prepare::prepare_fact;
use super::routing::{built_in_route_registry, process_missing_field};
use super::source_policy::{build_source_index, built_in_source_policy};
use super::types::PreparedFact;
use super::validation::{validate_pipeline_command, validate_pipeline_context};
use crate::ports::research::{FactPipelinePort, ResearchEvidenceLedgerPort};
use crate::ApplicationResult;
use football_domain::FactPipelineSummary;
use std::collections::BTreeMap;

pub(crate) trait FactPipelineAccess: FactPipelinePort + ResearchEvidenceLedgerPort {}
impl<T> FactPipelineAccess for T where T: FactPipelinePort + ResearchEvidenceLedgerPort + ?Sized {}

pub(crate) async fn process_p4_research_evidence(
    port: &dyn FactPipelineAccess,
    command: ProcessResearchEvidenceCommand,
) -> ApplicationResult<FactPipelineSummary> {
    validate_pipeline_command(&command)?;
    let context = port.context(command.research_run_id).await?;
    validate_pipeline_context(&command, &context)?;
    let policy = built_in_source_policy().definition;
    let registry = built_in_route_registry();
    let source_index = build_source_index(&command, &policy)?;

    let mut summary = FactPipelineSummary {
        fact_count: u32::try_from(command.output.facts.len()).unwrap_or(u32::MAX),
        missing_field_count: u32::try_from(command.output.missing_fields.len()).unwrap_or(u32::MAX),
        ..FactPipelineSummary::default()
    };
    let mut groups: BTreeMap<String, Vec<PreparedFact>> = BTreeMap::new();

    for fact in command.output.facts {
        let (group_key, prepared) = prepare_fact(
            port,
            &context,
            &registry,
            &source_index,
            fact,
            command.retrieved_at,
            &mut summary,
        )
        .await?;
        groups.entry(group_key).or_default().push(prepared);
    }

    for prepared_group in groups.into_values() {
        process_fact_group(port, &context, &registry, prepared_group, &mut summary).await?;
    }

    for missing in command.output.missing_fields {
        process_missing_field(
            port,
            &context,
            &registry,
            &missing,
            command.retrieved_at,
            &mut summary,
        )
        .await?;
    }
    Ok(summary)
}
