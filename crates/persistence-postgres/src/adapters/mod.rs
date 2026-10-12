mod ai_workspace;
pub(crate) mod catalog;
pub(crate) mod competition;
pub(crate) mod lineups;
pub(crate) mod matches;
pub(crate) mod p4;
mod postmatch;
mod prediction;
mod register_adapters;
pub(crate) mod review;
mod rules;
pub(crate) mod workbooks;

pub(crate) use competition::register_model_in_tx;
pub use competition::ModelRegistration;
pub use prediction::ModelRunListItem;
pub use register_adapters::register_adapters;
