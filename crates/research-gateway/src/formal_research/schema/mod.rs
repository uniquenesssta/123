mod decode;
mod validation;

pub(crate) use decode::decode_output_text;
pub use validation::{validate_research_output, ValidationContext};
