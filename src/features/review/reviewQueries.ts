import type { LineupRecord, MatchReviewDetail, MatchReviewPackageWorkflowRecord } from '../../types';
import type { ReviewApi } from './reviewPorts';
/** Preserve the existing directory ranking; manual submission separately requires two actual lineups. */
export async function readReviewLineups(api: ReviewApi, matchId: string): Promise<LineupRecord[]> {
    const summaries = await api.listLineups(matchId, 30);
    const rank: Record<LineupRecord['lineup_type'], number> = { actual: 0, confirmed: 1, expected: 2 };
    const selected = new Map<string, LineupRecord>();
    for (const row of summaries.filter(r => r.status === 'active').sort((a, b) => rank[a.lineup_type] - rank[b.lineup_type])) {
        if (row.match_id !== matchId)
            throw Error('阵容目录返回了其他比赛');
        if (!selected.has(row.team_id))
            selected.set(row.team_id, row);
    }
    const records = await Promise.all([...selected.values()].map(r => api.readLineup(r.id)));
    for (const record of records)
        if (record.match_id !== matchId || !selected.has(record.team_id) || selected.get(record.team_id)?.id !== record.id)
            throw Error('阵容详情与所选比赛不一致');
    return records;
}
export function validateWorkflow(workflow: MatchReviewPackageWorkflowRecord | null, matchId: string): void {
    if (!workflow)
        return;
    if (workflow.match_id !== matchId || !workflow.package_id)
        throw Error('工作流与当前比赛不一致');
    const p = workflow.preview;
    if (p && (p.match_id !== matchId || p.package_id !== workflow.package_id || p.source_sha256 !== workflow.import_sha256))
        throw Error('预检身份或SHA与当前工作流不一致');
}
export function validateReview(detail: MatchReviewDetail, id: string, matchId: string): void {
    if (detail.summary.id !== id || detail.summary.match_id !== matchId)
        throw Error('复盘结果与原比赛或复盘ID不一致');
}
export function workflowVersion(workflow: MatchReviewPackageWorkflowRecord | null): string {
    return JSON.stringify(workflow ? [workflow.match_id, workflow.package_id, workflow.import_sha256, workflow.export_sha256, workflow.status, workflow.updated_at, workflow.confirmed_at, workflow.facts_committed_at, workflow.review_id] : null);
}
