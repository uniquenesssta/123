use serde::{Deserialize, Serialize};

pub const TEAM_PACKAGE_FORMAT: &str = "football.team-package.v1";
pub const TEAM_PACKAGE_PREVIEW_EXPORT_FORMAT: &str = "football.team-package-preview.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPackageExportSummary {
    pub output_path: String,
    pub format_version: String,
    pub visible_sheet_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPackagePreviewExportSummary {
    pub output_path: String,
    pub format_version: String,
    pub exported_row_count: u64,
}
