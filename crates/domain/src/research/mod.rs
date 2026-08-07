mod conflict;
mod entity;
mod gateway;
mod pipeline;
mod routing;
mod source_policy;
mod time_audit;

pub use conflict::*;
pub use entity::*;
pub use gateway::*;
pub use pipeline::*;
pub use routing::*;
pub use source_policy::*;
pub use time_audit::*;

pub const P4_FACT_PIPELINE_CONTRACT_VERSION: &str = "football.p4-fact-pipeline.v1";
pub const P4_SOURCE_POLICY_VERSION: &str = "football.p4-source-policy.v1";
pub const P4_EVIDENCE_ROUTE_VERSION: &str = "football.p4-evidence-routes.v1";
pub const P4_RESEARCH_GATEWAY_CONTRACT_VERSION: &str = "football.p4-research-gateway.v1";
pub const P4_RESEARCH_OUTPUT_SCHEMA_VERSION: &str = "football.p4-research-output.v2";
pub const P4_RESEARCH_PROMPT_VERSION: &str = "2.0.0+public.1";
