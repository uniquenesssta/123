fn default_context_type() -> String {
    "general".to_string()
}

fn default_player_page_limit() -> u32 {
    50
}

fn default_registration_status() -> String {
    "registered".to_string()
}

fn default_sample_size() -> i32 {
    1
}
mod ability;
mod availability;
mod catalog;
mod detail;
mod listing;
mod membership;
mod name;
mod position;
mod status;

pub use ability::*;
pub use availability::*;
pub use catalog::*;
pub use detail::*;
pub use listing::*;
pub use membership::*;
pub use name::*;
pub use position::*;
pub use status::*;
