fn default_tactical_style() -> String {
    "balanced".to_string()
}

fn default_team_profile_confidence() -> f64 {
    0.5
}

fn default_team_type() -> String {
    "club".to_string()
}
mod catalog;
mod deletion;
mod detail;
mod listing;
mod membership;
mod name;
mod profile;

pub use catalog::*;
pub use deletion::*;
pub use detail::*;
pub use listing::*;
pub use membership::*;
pub use name::*;
pub use profile::*;
