use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{MatchReviewPackagePreview, MatchReviewPackageWorkflowRecord};
use crate::review::MatchReviewDetail;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageConfirmationRequest { pub package_id: Uuid, #[serde(default)] pub confirmed_by: Option<String>, #[serde(default)] pub confirmation_note: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageFactsCommitResult { pub home_lineup_id: Uuid, pub away_lineup_id: Uuid, pub workflow: MatchReviewPackageWorkflowRecord }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageReviewResult { pub review: MatchReviewDetail, pub workflow: MatchReviewPackageWorkflowRecord }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageCommitRequest { pub preview: MatchReviewPackagePreview, #[serde(default)] pub confirmed_by: Option<String>, #[serde(default)] pub confirmation_note: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageCommitResult { pub home_lineup_id: Uuid, pub away_lineup_id: Uuid, pub review: MatchReviewDetail }
