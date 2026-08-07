mod evidence;
mod research;
mod snapshot;
mod version;

pub use evidence::*;
pub use research::*;
pub use snapshot::*;
pub use version::*;

pub const P4_PERSISTENCE_CONTRACT_VERSION: &str = "football.p4-persistence.v1";
pub const P4_EVIDENCE_SCHEMA_VERSION: &str = "football.p4-evidence.v1";
pub const P4_SNAPSHOT_SCHEMA_VERSION: &str = "football.p4-prematch-snapshot.v1";
pub const P4_FEATURE_FIELD_COUNT: usize = 31;
