mod command;
mod conflict;
mod entity_resolution;
mod evidence;
mod prepare;
mod process;
mod routing;
mod source_policy;
mod time_audit;
mod types;
mod validation;

pub use command::ProcessResearchEvidenceCommand;
pub(crate) use process::{process_p4_research_evidence, FactPipelineAccess};
pub(crate) use source_policy::register_fact_pipeline_artifacts;

#[cfg(test)]
mod tests;
