import type { MatchReviewPackageWorkflowAction } from '../../types';
import type { ReviewFeature } from './reviewFeature';
import { matchReviewWorkflowAllows, matchReviewWorkflowBlocker, matchReviewWorkflowCompleted } from '../../app/matchReviewWorkflow';
export type WorkflowStepState = "done" | "current" | "locked" | "blocked";
export interface WorkflowStepView {
    no: number;
    title: string;
    purpose: string;
    status: string;
    completion: string;
    blocked: string;
    next: string;
    state: WorkflowStepState;
    action?: string;
}
export function workflowActionLabel(action: MatchReviewPackageWorkflowAction | null): string {
    if (!action)
        return "无后续动作";
    const labels: Record<MatchReviewPackageWorkflowAction, string> = {
        export_package: "重新导出资料包",
        preview_import: "导入并预检",
        confirm_import: "人工确认",
        commit_facts: "写入赛后事实",
        generate_review: "生成正式复盘",
        inspect_settlement_readiness: "检查结算门禁",
        settle_review: "正式结算",
        open_analytics: "进入分析与历史",
    };
    return labels[action];
}
function stepState(done: boolean, active: boolean, blocked = false): WorkflowStepState {
    if (done)
        return "done";
    if (blocked)
        return "blocked";
    return active ? "current" : "locked";
}
export function workflowStepStateLabel(state: WorkflowStepState): string {
    return ({ done: "已完成", current: "当前", locked: "锁定", blocked: "阻断" } as const)[state];
}
export function reviewSteps(feature: ReviewFeature): WorkflowStepView[] {
    const selected = feature.selected, workflow = feature.state.workflow, preview = feature.preview, detail = feature.state.detail, settlement = feature.settlement;
    const matchId = selected?.match_record.id ?? null;
    const activeWorkflow = workflow?.match_id === matchId ? workflow : null;
    const activePreview = preview?.match_id === matchId ? preview : null;
    const reviewId = activeWorkflow?.review_id ?? (detail?.summary.match_id === matchId ? detail.summary.id : null);
    const exported = matchReviewWorkflowCompleted(activeWorkflow, "export_package");
    const externalDataCompleted = matchReviewWorkflowCompleted(activeWorkflow, "complete_external_data");
    const previewValid = matchReviewWorkflowCompleted(activeWorkflow, "preview_import");
    const confirmed = matchReviewWorkflowCompleted(activeWorkflow, "confirm_import");
    const factsCommitted = matchReviewWorkflowCompleted(activeWorkflow, "commit_facts");
    const reviewCreated = matchReviewWorkflowCompleted(activeWorkflow, "generate_review");
    const settled = matchReviewWorkflowCompleted(activeWorkflow, "settle_review") || Boolean(settlement && settlement.match_id === matchId);
    const canPreview = matchReviewWorkflowAllows(activeWorkflow, "preview_import");
    const canConfirm = matchReviewWorkflowAllows(activeWorkflow, "confirm_import");
    const canCommitFacts = matchReviewWorkflowAllows(activeWorkflow, "commit_facts");
    const canGenerateReview = matchReviewWorkflowAllows(activeWorkflow, "generate_review");
    const canInspectSettlement = matchReviewWorkflowAllows(activeWorkflow, "inspect_settlement_readiness");
    const canSettle = matchReviewWorkflowAllows(activeWorkflow, "settle_review");
    const canOpenAnalytics = matchReviewWorkflowAllows(activeWorkflow, "open_analytics");
    const noMatchReason = selected ? "" : "暂无可复盘比赛或尚未载入比赛";
    const noExportReason = selected && !activeWorkflow ? "尚未导出本轮资料包" : "";
    const previewBlocked = Boolean(activePreview && !activePreview.ready);
    const steps: WorkflowStepView[] = [
        { no: 1, title: "选择比赛", purpose: "确定本次复盘唯一比赛身份，锁定主客队与开球时间。", status: selected ? "已选择" : "等待选择", completion: "比赛出现在可复盘列表中，并已载入当前页面。", blocked: noMatchReason, next: selected ? "导出本轮赛后复盘资料包。" : "在上方选择比赛并点击“载入比赛与阵容”。", state: stepState(Boolean(selected), !selected) },
        { no: 2, title: "导出赛后复盘资料包", purpose: "冻结赛前阵容、球员状态、模型身份、参数版本和预测输出。", status: exported ? "本轮资料包已导出" : "等待导出", completion: "后端登记 package_id、导出路径与原始 SHA256。", blocked: noMatchReason, next: exported ? "在外部补充真实比赛事实。" : "导出 .xlsx 资料包。", state: stepState(exported, Boolean(selected)) },
        { no: 3, title: "在外部补充真实比赛事实和球员量化数据", purpose: "补充比分、实际名单、换人、进球、助攻、牌、VAR、伤退、阵型变化与球员表现。", status: externalDataCompleted ? "已提交填写后的文件" : exported ? "等待外部补充" : "尚未开始", completion: "使用本轮导出的原文件完成填写，并保留 package_id。", blocked: noMatchReason || noExportReason, next: exported ? "选择填写后的文件并启动导入预检。" : "先完成资料包导出。", state: stepState(externalDataCompleted, exported && !externalDataCompleted, Boolean(noMatchReason || noExportReason)) },
        { no: 4, title: "导入并预检", purpose: "比较赛前值、当前数据库值与准备导入值，核验身份、差异、错误和警告。", status: previewBlocked ? "预检有阻断错误" : previewValid ? "预检通过" : canPreview ? "可以选择文件预检" : "等待资料包", completion: "本轮 package_id 匹配、后端复检通过且严重错误为零。", blocked: previewBlocked ? `${activePreview?.errors.length ?? 0} 条阻断错误需要修正` : matchReviewWorkflowBlocker(activeWorkflow, "preview_import") || noMatchReason || noExportReason, next: previewBlocked ? "修正文件后重新预检。" : previewValid ? "进行人工确认。" : "选择填写后的 .xlsx 文件。", state: stepState(previewValid, canPreview, previewBlocked) },
        { no: 5, title: "人工确认", purpose: "确认差异、来源和可信度；后端再次读取文件并校验 SHA256。", status: confirmed ? "已人工确认" : canConfirm ? "等待人工确认" : "尚未解锁", completion: "预检通过，并记录确认人、确认说明和确认时间。", blocked: confirmed ? "" : matchReviewWorkflowBlocker(activeWorkflow, "confirm_import") || noExportReason, next: confirmed ? "写入真实赛后事实。" : "核对预检结果并确认。", state: stepState(confirmed, canConfirm) },
        { no: 6, title: "写入真实赛后事实", purpose: "写入实际阵容、正式赛果、结构化比赛事件与球员观察，不覆盖赛前快照。", status: factsCommitted ? "赛后事实已写入" : canCommitFacts ? "等待写入" : "尚未解锁", completion: "实际阵容、赛果、换人、普通事件和球员观察均成功入库。", blocked: factsCommitted ? "" : matchReviewWorkflowBlocker(activeWorkflow, "commit_facts") || "资料包尚未人工确认", next: factsCommitted ? "生成正式复盘。" : "确认写入真实赛后事实。", state: stepState(factsCommitted, canCommitFacts) },
        { no: 7, title: "生成正式复盘", purpose: "基于已冻结的赛前预测与已写入的真实事实生成球队、球员和预测评价。", status: reviewCreated ? "正式复盘已生成" : canGenerateReview ? "等待生成" : "尚未解锁", completion: "生成 finalized 复盘记录并绑定本轮资料包。", blocked: reviewCreated ? "" : matchReviewWorkflowBlocker(activeWorkflow, "generate_review") || "真实赛后事实尚未写入", next: reviewCreated ? "检查正式结算门禁。" : "生成正式复盘。", state: stepState(reviewCreated, canGenerateReview) },
        { no: 8, title: "正式结算", purpose: "通过模型运行、冻结快照、赛事 Profile 与正式时点门禁，建立不可覆盖的结算样本。", status: settled ? "正式结算已完成" : canSettle ? "等待门禁检查与结算" : "尚未解锁", completion: "正式结算记录创建成功，并建立证据评分队列。", blocked: settled ? "" : matchReviewWorkflowBlocker(activeWorkflow, "settle_review") || (!reviewId ? "复盘记录尚未载入" : ""), next: settled ? "进入分析与历史。" : "检查结算门禁，通过后正式结算。", state: stepState(settled, canInspectSettlement || canSettle) },
        { no: 9, title: "进入分析与历史", purpose: "查看结算样本、证据评分、漂移监控、复盘历史和受控能力候选。", status: settled ? "可以进入" : "尚未解锁", completion: "正式结算已完成。", blocked: settled ? "" : matchReviewWorkflowBlocker(activeWorkflow, "open_analytics") || "正式结算尚未完成", next: settled ? "打开分析与历史页面。" : "先完成正式结算。", state: stepState(false, canOpenAnalytics) },
    ];
    return steps;
}
export function defaultReviewStep(feature: ReviewFeature): number {
    if (!feature.selected)
        return 1;
    if (feature.preview && !feature.preview.ready)
        return 4;
    const next = feature.state.workflow?.next_action;
    return next ? ({ export_package: 2, preview_import: 4, confirm_import: 5, commit_facts: 6, generate_review: 7, inspect_settlement_readiness: 8, settle_review: 8, open_analytics: 9 })[next] : feature.state.workflow ? 3 : 2;
}
