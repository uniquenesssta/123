fn default_formation_alpha() -> f64 {
    3.0
}

fn default_formation_usage_limit() -> u32 {
    100
}

fn default_formation_window_preset() -> String {
    "custom".to_string()
}
mod catalog;
mod resolution;
mod usage;

pub use catalog::*;
pub use resolution::*;
pub use usage::*;
