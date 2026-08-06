fn default_coach_role() -> String {
    "head_coach".to_string()
}

fn default_coach_status() -> String {
    "active".to_string()
}
mod catalog;
mod detail;
mod listing;
mod membership;
mod name;

pub use catalog::*;
pub use detail::*;
pub use listing::*;
pub use membership::*;
pub use name::*;
