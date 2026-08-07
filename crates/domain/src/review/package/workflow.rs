use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{MatchReviewPackagePreview, MatchReviewPackageSnapshotSummary};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MatchReviewPackageWorkflowStatus { Exported, PreviewBlocked, PreviewValid, Confirmed, FactsCommitted, ReviewCreated, Settled, Superseded }
impl MatchReviewPackageWorkflowStatus {
    pub const fn as_str(self) -> &'static str { match self { Self::Exported=>"exported", Self::PreviewBlocked=>"preview_blocked", Self::PreviewValid=>"preview_valid", Self::Confirmed=>"confirmed", Self::FactsCommitted=>"facts_committed", Self::ReviewCreated=>"review_created", Self::Settled=>"settled", Self::Superseded=>"superseded" } }
    pub fn parse(value: &str) -> Result<Self, String> { match value { "exported"=>Ok(Self::Exported), "preview_blocked"=>Ok(Self::PreviewBlocked), "preview_valid"=>Ok(Self::PreviewValid), "confirmed"=>Ok(Self::Confirmed), "facts_committed"=>Ok(Self::FactsCommitted), "review_created"=>Ok(Self::ReviewCreated), "settled"=>Ok(Self::Settled), "superseded"=>Ok(Self::Superseded), other=>Err(format!("未知赛后复盘资料包状态：{other}")) } }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MatchReviewPackageWorkflowStep { ExportPackage, CompleteExternalData, PreviewImport, ConfirmImport, CommitFacts, GenerateReview, SettleReview, OpenAnalytics }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MatchReviewPackageWorkflowAction { ExportPackage, PreviewImport, ConfirmImport, CommitFacts, GenerateReview, InspectSettlementReadiness, SettleReview, OpenAnalytics }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchReviewPackageActionBlocker { pub action: MatchReviewPackageWorkflowAction, pub reason: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageWorkflowRecord {
    pub package_id: Uuid, pub match_id: Uuid, pub match_key: String, pub status: MatchReviewPackageWorkflowStatus,
    #[serde(default)] pub completed_steps: Vec<MatchReviewPackageWorkflowStep>,
    #[serde(default)] pub allowed_actions: Vec<MatchReviewPackageWorkflowAction>,
    #[serde(default)] pub blocking_reasons: Vec<MatchReviewPackageActionBlocker>,
    #[serde(default)] pub next_action: Option<MatchReviewPackageWorkflowAction>,
    pub export_path: String, pub export_sha256: String, pub pre_match_snapshot: MatchReviewPackageSnapshotSummary,
    pub export_database_snapshot: MatchReviewPackageSnapshotSummary, pub import_path: Option<String>, pub import_sha256: Option<String>,
    pub preview_ready: bool, #[serde(default)] pub preview: Option<MatchReviewPackagePreview>, pub confirmed_by: Option<String>,
    pub confirmation_note: Option<String>, pub review_id: Option<Uuid>, pub exported_at: DateTime<Utc>,
    pub previewed_at: Option<DateTime<Utc>>, pub confirmed_at: Option<DateTime<Utc>>, pub facts_committed_at: Option<DateTime<Utc>>,
    pub review_created_at: Option<DateTime<Utc>>, pub settled_at: Option<DateTime<Utc>>, pub updated_at: DateTime<Utc>,
}
impl MatchReviewPackageWorkflowRecord {
    pub fn with_capabilities(mut self) -> Self { self.completed_steps=completed_steps(self.status,self.previewed_at.is_some()); self.allowed_actions=allowed_actions(self.status); self.blocking_reasons=all_workflow_actions().into_iter().filter(|action| !self.allowed_actions.contains(action)).map(|action| MatchReviewPackageActionBlocker { action, reason:blocking_reason(self.status,action).to_string() }).collect(); self.next_action=next_action(self.status); self }
    pub fn allows(&self, action: MatchReviewPackageWorkflowAction) -> bool { self.allowed_actions.contains(&action) }
    pub fn blocking_reason(&self, action: MatchReviewPackageWorkflowAction) -> Option<&str> { self.blocking_reasons.iter().find(|item| item.action==action).map(|item| item.reason.as_str()) }
    pub fn require_action(&self, action: MatchReviewPackageWorkflowAction) -> Result<(), String> { if self.allows(action) { return Ok(()); } Err(self.blocking_reason(action).unwrap_or("当前工作流状态不允许执行该操作").to_string()) }
}
fn all_workflow_actions() -> [MatchReviewPackageWorkflowAction;8] { use MatchReviewPackageWorkflowAction as A; [A::ExportPackage,A::PreviewImport,A::ConfirmImport,A::CommitFacts,A::GenerateReview,A::InspectSettlementReadiness,A::SettleReview,A::OpenAnalytics] }
fn allowed_actions(status: MatchReviewPackageWorkflowStatus) -> Vec<MatchReviewPackageWorkflowAction> { use MatchReviewPackageWorkflowAction as A; use MatchReviewPackageWorkflowStatus as S; let mut actions=vec![A::ExportPackage]; match status { S::Exported|S::PreviewBlocked=>actions.push(A::PreviewImport), S::PreviewValid=>{actions.push(A::PreviewImport);actions.push(A::ConfirmImport)}, S::Confirmed=>actions.push(A::CommitFacts), S::FactsCommitted=>actions.push(A::GenerateReview), S::ReviewCreated=>{actions.push(A::InspectSettlementReadiness);actions.push(A::SettleReview)}, S::Settled=>actions.push(A::OpenAnalytics), S::Superseded=>actions.clear() } actions }
fn completed_steps(status: MatchReviewPackageWorkflowStatus, previewed: bool) -> Vec<MatchReviewPackageWorkflowStep> { use MatchReviewPackageWorkflowStatus as S; use MatchReviewPackageWorkflowStep as T; if status==S::Superseded{return Vec::new()} let mut steps=vec![T::ExportPackage]; if previewed{steps.push(T::CompleteExternalData)} if matches!(status,S::PreviewValid|S::Confirmed|S::FactsCommitted|S::ReviewCreated|S::Settled){steps.push(T::PreviewImport)} if matches!(status,S::Confirmed|S::FactsCommitted|S::ReviewCreated|S::Settled){steps.push(T::ConfirmImport)} if matches!(status,S::FactsCommitted|S::ReviewCreated|S::Settled){steps.push(T::CommitFacts)} if matches!(status,S::ReviewCreated|S::Settled){steps.push(T::GenerateReview)} if status==S::Settled{steps.push(T::SettleReview)} steps }
fn next_action(status: MatchReviewPackageWorkflowStatus) -> Option<MatchReviewPackageWorkflowAction> { use MatchReviewPackageWorkflowAction as A; use MatchReviewPackageWorkflowStatus as S; match status { S::Exported|S::PreviewBlocked=>Some(A::PreviewImport), S::PreviewValid=>Some(A::ConfirmImport), S::Confirmed=>Some(A::CommitFacts), S::FactsCommitted=>Some(A::GenerateReview), S::ReviewCreated=>Some(A::InspectSettlementReadiness), S::Settled=>Some(A::OpenAnalytics), S::Superseded=>None } }
fn blocking_reason(status: MatchReviewPackageWorkflowStatus, action: MatchReviewPackageWorkflowAction) -> &'static str { use MatchReviewPackageWorkflowAction as A; use MatchReviewPackageWorkflowStatus as S; if status==S::Superseded{return "该资料包已被新一轮导出替代"} match action { A::ExportPackage=>"当前资料包不能重新导出", A::PreviewImport=>match status{S::Confirmed|S::FactsCommitted|S::ReviewCreated|S::Settled=>"资料包已人工确认，不能覆盖本轮预检结果", _=>"尚未导出可供预检的资料包"}, A::ConfirmImport=>match status{S::Exported=>"尚未导入并预检填写后的资料包",S::PreviewBlocked=>"预检仍有阻断错误",S::Confirmed|S::FactsCommitted|S::ReviewCreated|S::Settled=>"资料包已经人工确认",_=>"资料包尚未通过预检"}, A::CommitFacts=>match status{S::Exported|S::PreviewBlocked|S::PreviewValid=>"资料包尚未人工确认",S::FactsCommitted|S::ReviewCreated|S::Settled=>"真实赛后事实已经写入",_=>"当前资料包不能写入赛后事实"}, A::GenerateReview=>match status{S::Exported|S::PreviewBlocked|S::PreviewValid|S::Confirmed=>"真实赛后事实尚未写入",S::ReviewCreated|S::Settled=>"正式复盘已经生成",_=>"当前资料包不能生成正式复盘"}, A::InspectSettlementReadiness|A::SettleReview=>match status{S::Settled=>"正式结算已经完成",_=>"正式复盘尚未生成"}, A::OpenAnalytics=>"正式结算尚未完成" } }

#[cfg(test)]
mod workflow_tests { use super::*; #[test] fn workflow_state_exposes_one_authoritative_next_action(){use MatchReviewPackageWorkflowAction as A;use MatchReviewPackageWorkflowStatus as S;for (status,expected) in [(S::Exported,Some(A::PreviewImport)),(S::PreviewBlocked,Some(A::PreviewImport)),(S::PreviewValid,Some(A::ConfirmImport)),(S::Confirmed,Some(A::CommitFacts)),(S::FactsCommitted,Some(A::GenerateReview)),(S::ReviewCreated,Some(A::InspectSettlementReadiness)),(S::Settled,Some(A::OpenAnalytics)),(S::Superseded,None)]{assert_eq!(next_action(status),expected)}} #[test] fn blocked_preview_does_not_unlock_confirmation(){use MatchReviewPackageWorkflowAction as A;use MatchReviewPackageWorkflowStatus as S;let actions=allowed_actions(S::PreviewBlocked);assert!(actions.contains(&A::PreviewImport));assert!(!actions.contains(&A::ConfirmImport));assert_eq!(blocking_reason(S::PreviewBlocked,A::ConfirmImport),"预检仍有阻断错误")} #[test] fn settled_workflow_only_opens_analysis_or_starts_new_export(){use MatchReviewPackageWorkflowAction as A;use MatchReviewPackageWorkflowStatus as S;let actions=allowed_actions(S::Settled);assert_eq!(actions,vec![A::ExportPackage,A::OpenAnalytics]);assert!(completed_steps(S::Settled,true).contains(&MatchReviewPackageWorkflowStep::SettleReview))} #[test] fn persisted_status_round_trips_without_stringly_typed_logic(){use MatchReviewPackageWorkflowStatus as S;for status in [S::Exported,S::PreviewBlocked,S::PreviewValid,S::Confirmed,S::FactsCommitted,S::ReviewCreated,S::Settled,S::Superseded]{assert_eq!(S::parse(status.as_str()),Ok(status));assert_eq!(serde_json::to_string(&status).expect("序列化工作流状态"),format!("\"{}\"",status.as_str()))}} }
