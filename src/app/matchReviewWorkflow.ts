import type {
  MatchReviewPackageWorkflowAction,
  MatchReviewPackageWorkflowRecord,
  MatchReviewPackageWorkflowStep,
} from "../types";

export function matchReviewWorkflowAllows(
  workflow: MatchReviewPackageWorkflowRecord | null,
  action: MatchReviewPackageWorkflowAction,
): boolean {
  return workflow?.allowed_actions.includes(action) ?? false;
}

export function matchReviewWorkflowCompleted(
  workflow: MatchReviewPackageWorkflowRecord | null,
  step: MatchReviewPackageWorkflowStep,
): boolean {
  return workflow?.completed_steps.includes(step) ?? false;
}

export function matchReviewWorkflowBlocker(
  workflow: MatchReviewPackageWorkflowRecord | null,
  action: MatchReviewPackageWorkflowAction,
): string {
  return workflow?.blocking_reasons.find((item) => item.action === action)?.reason ?? "";
}
