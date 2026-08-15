pub(crate) mod competition;
mod register_adapters;
mod rules;

pub(crate) use competition::register_model_in_tx;
pub use competition::ModelRegistration;
pub use register_adapters::register_adapters;
