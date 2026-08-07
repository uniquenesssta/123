pub mod coach;
pub mod competition;
pub mod formation;
pub mod lineup;
pub mod match_record;
pub mod player;
pub mod prediction;
pub mod research;
pub mod routing;
pub mod shared;
pub mod team;

mod analytics;
mod api_workspace;
mod exchange;
mod match_event;
mod match_review_package;
mod match_review_workflow;
mod monthly_workbook;
mod postmatch;
mod release_acceptance;
mod review;
mod spreadsheet;
mod team_package;
pub use coach::*;
pub use competition::*;
pub use formation::*;
pub use lineup::*;
pub use match_record::*;
pub use player::*;
pub use prediction::*;
pub use research::*;
pub use routing::*;
pub use shared::*;
pub use team::*;

pub use analytics::*;
pub use api_workspace::*;
pub use exchange::*;
pub use match_event::*;
pub use match_review_package::*;
pub use match_review_workflow::*;
pub use monthly_workbook::*;
pub use postmatch::*;
pub use release_acceptance::*;
pub use review::*;
pub use spreadsheet::*;
pub use team_package::*;

fn default_true() -> bool {
    true
}

fn default_team_page_limit() -> u32 {
    50
}

fn default_confidence() -> f64 {
    1.0
}
