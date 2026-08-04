use serde::{Deserialize, Serialize};

use crate::match_review_package::MatchReviewPackageWorkflowRecord;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MatchReviewPackageWorkflowStatus {
    Exported,
    PreviewBlocked,
    PreviewValid,
    Confirmed,
    FactsCommitted,
    ReviewCreated,
    Settled,
    Superseded,
}

impl MatchReviewPackageWorkflowStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exported => "exported",
            Self::PreviewBlocked => "preview_blocked",
            Self::PreviewValid => "preview_valid",
            Self::Confirmed => "confirmed",
            Self::FactsCommitted => "facts_committed",
            Self::ReviewCreated => "review_created",
            Self::Settled => "settled",
            Self::Superseded => "superseded",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "exported" => Ok(Self::Exported),
            "preview_blocked" => Ok(Self::PreviewBlocked),
            "preview_valid" => Ok(Self::PreviewValid),
            "confirmed" => Ok(Self::Confirmed),
            "facts_committed" => Ok(Self::FactsCommitted),
            "review_created" => Ok(Self::ReviewCreated),
            "settled" => Ok(Self::Settled),
            "superseded" => Ok(Self::Superseded),
            other => Err(format!("未知赛后复盘资料包状态：{other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MatchReviewPackageWorkflowStep {
    ExportPackage,
    CompleteExternalData,
    PreviewImport,
    ConfirmImport,
    CommitFacts,
    GenerateReview,
    SettleReview,
    OpenAnalytics,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MatchReviewPackageWorkflowAction {
    ExportPackage,
    PreviewImport,
    ConfirmImport,
    CommitFacts,
    GenerateReview,
    InspectSettlementReadiness,
    SettleReview,
    OpenAnalytics,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchReviewPackageActionBlocker {
    pub action: MatchReviewPackageWorkflowAction,
    pub reason: String,
}

impl MatchReviewPackageWorkflowRecord {
    pub fn with_capabilities(mut self) -> Self {
        self.completed_steps = completed_steps(self.status, self.previewed_at.is_some());
        self.allowed_actions = allowed_actions(self.status);
        self.blocking_reasons = all_workflow_actions()
            .into_iter()
            .filter(|action| !self.allowed_actions.contains(action))
            .map(|action| MatchReviewPackageActionBlocker {
                action,
                reason: blocking_reason(self.status, action).to_string(),
            })
            .collect();
        self.next_action = next_action(self.status);
        self
    }

    pub fn allows(&self, action: MatchReviewPackageWorkflowAction) -> bool {
        self.allowed_actions.contains(&action)
    }

    pub fn blocking_reason(&self, action: MatchReviewPackageWorkflowAction) -> Option<&str> {
        self.blocking_reasons
            .iter()
            .find(|item| item.action == action)
            .map(|item| item.reason.as_str())
    }

    pub fn require_action(&self, action: MatchReviewPackageWorkflowAction) -> Result<(), String> {
        if self.allows(action) {
            return Ok(());
        }
        Err(self
            .blocking_reason(action)
            .unwrap_or("当前工作流状态不允许执行该操作")
            .to_string())
    }
}

fn all_workflow_actions() -> [MatchReviewPackageWorkflowAction; 8] {
    use MatchReviewPackageWorkflowAction as Action;
    [
        Action::ExportPackage,
        Action::PreviewImport,
        Action::ConfirmImport,
        Action::CommitFacts,
        Action::GenerateReview,
        Action::InspectSettlementReadiness,
        Action::SettleReview,
        Action::OpenAnalytics,
    ]
}

fn allowed_actions(
    status: MatchReviewPackageWorkflowStatus,
) -> Vec<MatchReviewPackageWorkflowAction> {
    use MatchReviewPackageWorkflowAction as Action;
    use MatchReviewPackageWorkflowStatus as Status;

    let mut actions = vec![Action::ExportPackage];
    match status {
        Status::Exported | Status::PreviewBlocked => actions.push(Action::PreviewImport),
        Status::PreviewValid => {
            actions.push(Action::PreviewImport);
            actions.push(Action::ConfirmImport);
        }
        Status::Confirmed => actions.push(Action::CommitFacts),
        Status::FactsCommitted => actions.push(Action::GenerateReview),
        Status::ReviewCreated => {
            actions.push(Action::InspectSettlementReadiness);
            actions.push(Action::SettleReview);
        }
        Status::Settled => actions.push(Action::OpenAnalytics),
        Status::Superseded => actions.clear(),
    }
    actions
}

fn completed_steps(
    status: MatchReviewPackageWorkflowStatus,
    previewed: bool,
) -> Vec<MatchReviewPackageWorkflowStep> {
    use MatchReviewPackageWorkflowStatus as Status;
    use MatchReviewPackageWorkflowStep as Step;

    if status == Status::Superseded {
        return Vec::new();
    }

    let mut steps = vec![Step::ExportPackage];
    if previewed {
        steps.push(Step::CompleteExternalData);
    }
    if matches!(
        status,
        Status::PreviewValid
            | Status::Confirmed
            | Status::FactsCommitted
            | Status::ReviewCreated
            | Status::Settled
    ) {
        steps.push(Step::PreviewImport);
    }
    if matches!(
        status,
        Status::Confirmed | Status::FactsCommitted | Status::ReviewCreated | Status::Settled
    ) {
        steps.push(Step::ConfirmImport);
    }
    if matches!(
        status,
        Status::FactsCommitted | Status::ReviewCreated | Status::Settled
    ) {
        steps.push(Step::CommitFacts);
    }
    if matches!(status, Status::ReviewCreated | Status::Settled) {
        steps.push(Step::GenerateReview);
    }
    if status == Status::Settled {
        steps.push(Step::SettleReview);
    }
    steps
}

fn next_action(
    status: MatchReviewPackageWorkflowStatus,
) -> Option<MatchReviewPackageWorkflowAction> {
    use MatchReviewPackageWorkflowAction as Action;
    use MatchReviewPackageWorkflowStatus as Status;

    match status {
        Status::Exported | Status::PreviewBlocked => Some(Action::PreviewImport),
        Status::PreviewValid => Some(Action::ConfirmImport),
        Status::Confirmed => Some(Action::CommitFacts),
        Status::FactsCommitted => Some(Action::GenerateReview),
        Status::ReviewCreated => Some(Action::InspectSettlementReadiness),
        Status::Settled => Some(Action::OpenAnalytics),
        Status::Superseded => None,
    }
}

fn blocking_reason(
    status: MatchReviewPackageWorkflowStatus,
    action: MatchReviewPackageWorkflowAction,
) -> &'static str {
    use MatchReviewPackageWorkflowAction as Action;
    use MatchReviewPackageWorkflowStatus as Status;

    if status == Status::Superseded {
        return "该资料包已被新一轮导出替代";
    }

    match action {
        Action::ExportPackage => "当前资料包不能重新导出",
        Action::PreviewImport => match status {
            Status::Confirmed
            | Status::FactsCommitted
            | Status::ReviewCreated
            | Status::Settled => "资料包已人工确认，不能覆盖本轮预检结果",
            _ => "尚未导出可供预检的资料包",
        },
        Action::ConfirmImport => match status {
            Status::Exported => "尚未导入并预检填写后的资料包",
            Status::PreviewBlocked => "预检仍有阻断错误",
            Status::Confirmed
            | Status::FactsCommitted
            | Status::ReviewCreated
            | Status::Settled => "资料包已经人工确认",
            _ => "资料包尚未通过预检",
        },
        Action::CommitFacts => match status {
            Status::Exported | Status::PreviewBlocked | Status::PreviewValid => {
                "资料包尚未人工确认"
            }
            Status::FactsCommitted | Status::ReviewCreated | Status::Settled => {
                "真实赛后事实已经写入"
            }
            _ => "当前资料包不能写入赛后事实",
        },
        Action::GenerateReview => match status {
            Status::Exported
            | Status::PreviewBlocked
            | Status::PreviewValid
            | Status::Confirmed => "真实赛后事实尚未写入",
            Status::ReviewCreated | Status::Settled => "正式复盘已经生成",
            _ => "当前资料包不能生成正式复盘",
        },
        Action::InspectSettlementReadiness | Action::SettleReview => match status {
            Status::Settled => "正式结算已经完成",
            _ => "正式复盘尚未生成",
        },
        Action::OpenAnalytics => "正式结算尚未完成",
    }
}

#[cfg(test)]
mod workflow_tests {
    use super::*;

    #[test]
    fn workflow_state_exposes_one_authoritative_next_action() {
        use MatchReviewPackageWorkflowAction as Action;
        use MatchReviewPackageWorkflowStatus as Status;

        let cases = [
            (Status::Exported, Some(Action::PreviewImport)),
            (Status::PreviewBlocked, Some(Action::PreviewImport)),
            (Status::PreviewValid, Some(Action::ConfirmImport)),
            (Status::Confirmed, Some(Action::CommitFacts)),
            (Status::FactsCommitted, Some(Action::GenerateReview)),
            (
                Status::ReviewCreated,
                Some(Action::InspectSettlementReadiness),
            ),
            (Status::Settled, Some(Action::OpenAnalytics)),
            (Status::Superseded, None),
        ];

        for (status, expected) in cases {
            assert_eq!(next_action(status), expected);
        }
    }

    #[test]
    fn blocked_preview_does_not_unlock_confirmation() {
        use MatchReviewPackageWorkflowAction as Action;
        use MatchReviewPackageWorkflowStatus as Status;

        let actions = allowed_actions(Status::PreviewBlocked);
        assert!(actions.contains(&Action::PreviewImport));
        assert!(!actions.contains(&Action::ConfirmImport));
        assert_eq!(
            blocking_reason(Status::PreviewBlocked, Action::ConfirmImport),
            "预检仍有阻断错误"
        );
    }

    #[test]
    fn settled_workflow_only_opens_analysis_or_starts_new_export() {
        use MatchReviewPackageWorkflowAction as Action;
        use MatchReviewPackageWorkflowStatus as Status;

        let actions = allowed_actions(Status::Settled);
        assert_eq!(actions, vec![Action::ExportPackage, Action::OpenAnalytics]);
        assert!(completed_steps(Status::Settled, true)
            .contains(&MatchReviewPackageWorkflowStep::SettleReview));
    }

    #[test]
    fn persisted_status_round_trips_without_stringly_typed_logic() {
        use MatchReviewPackageWorkflowStatus as Status;

        for status in [
            Status::Exported,
            Status::PreviewBlocked,
            Status::PreviewValid,
            Status::Confirmed,
            Status::FactsCommitted,
            Status::ReviewCreated,
            Status::Settled,
            Status::Superseded,
        ] {
            assert_eq!(Status::parse(status.as_str()), Ok(status));
            assert_eq!(
                serde_json::to_string(&status).expect("序列化工作流状态"),
                format!("\"{}\"", status.as_str())
            );
        }
    }
}
