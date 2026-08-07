pub mod ai_workspace;
pub mod analytics;
pub mod coach;
pub mod competition;
pub mod exchange;
pub mod formation;
pub mod lineup;
pub mod match_record;
pub mod player;
pub mod postmatch;
pub mod prediction;
pub mod release;
pub mod research;
pub mod review;
pub mod routing;
pub mod shared;
pub mod team;

pub use ai_workspace::*;
pub use analytics::*;
pub use coach::*;
pub use competition::*;
pub use exchange::*;
pub use formation::*;
pub use lineup::*;
pub use match_record::*;
pub use player::*;
pub use postmatch::*;
pub use prediction::*;
pub use release::*;
pub use research::*;
pub use review::*;
pub use routing::*;
pub use shared::*;
pub use team::*;

fn default_true() -> bool {
    true
}
fn default_team_page_limit() -> u32 {
    50
}
fn default_confidence() -> f64 {
    1.0
}
