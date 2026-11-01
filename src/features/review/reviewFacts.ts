import type { LineupRecord, MatchReviewDetail, MatchReviewDraft, PlayerPerformanceMetrics, ReviewableMatch } from '../../types';
import type { PreferenceStore } from '../../platform/storage';
export const reviewMetrics: ReadonlyArray<[
    keyof PlayerPerformanceMetrics,
    string,
    number
]> = [['goals', '进球', 1], ['assists', '助攻', 1], ['expected_goals', '预期进球', .01], ['expected_assists', '预期助攻', .01], ['shots', '射门', 1], ['shots_on_target', '射正', 1], ['key_passes', '关键传球', 1], ['progressive_actions', '推进动作', 1], ['tackles', '抢断', 1], ['interceptions', '拦截', 1], ['clearances', '解围', 1], ['blocks', '封堵', 1], ['duels_won', '对抗成功', 1], ['duels_total', '对抗总数', 1], ['fouls', '犯规', 1], ['yellow_cards', '黄牌', 1], ['red_cards', '红牌', 1], ['errors_leading_to_shot', '致险失误', 1]];
export interface ReviewFactsDraft {
    fields: Record<string, string>;
    players: Record<string, Record<string, string>>;
}
const empty = (): ReviewFactsDraft => ({ fields: {}, players: {} });
export function localReviewTime(value: string | null | undefined): string {
    if (!value || !Number.isFinite(Date.parse(value)))
        return '';
    const date = new Date(value);
    return new Date(date.getTime() - date.getTimezoneOffset() * 60000).toISOString().slice(0, 16);
}
/** Own raw manual controls by database+match. Workspace must opt this subtree out of generic form persistence. */
export class ReviewFacts {
    private revision = 0;
    constructor(private readonly store: PreferenceStore, private readonly key: string, private readonly database: () => string | null) { }
    get version() { return this.revision; }
    private identity(id: string): string { return JSON.stringify([this.database(), id]); }
    read(id: string): ReviewFactsDraft { return structuredClone(this.store.readObject<Record<string, ReviewFactsDraft>>(this.key)?.[this.identity(id)] ?? empty()); }
    capture(root: HTMLElement, id: string | null): void {
        if (!id || !this.database() || root.querySelector<HTMLElement>('[data-review-match-id]')?.dataset.reviewMatchId !== id)
            return;
        const draft = this.read(id);
        root.querySelectorAll<HTMLInputElement>('[data-review-fact]').forEach(input => { draft.fields[input.dataset.reviewFact!] = input.value; });
        root.querySelectorAll<HTMLElement>('[data-review-player]').forEach(row => {
            const player = row.dataset.playerId;
            if (!player)
                return;
            const values = { ...draft.players[player] };
            row.querySelectorAll<HTMLInputElement>('[data-field]').forEach(input => { values[input.dataset.field!] = input.value; });
            draft.players[player] = values;
            values._expanded = row.querySelector<HTMLDetailsElement>('details')?.open ? 'true' : 'false';
        });
        if (JSON.stringify(draft) !== JSON.stringify(this.read(id))) {
            const all = this.store.readObject<Record<string, ReviewFactsDraft>>(this.key) ?? {};
            this.store.writeJson(this.key, { ...all, [this.identity(id)]: draft });
            this.revision++;
        }
    }
    values(match: ReviewableMatch, lineups: LineupRecord[], detail: MatchReviewDetail | null = null): ReviewFactsDraft {
        const existing = detail?.summary.match_id === match.match_record.id ? detail : null;
        const saved = this.read(match.match_record.id), result = match.result ?? existing?.result;
        const fields: Record<string, string> = { home: result?.home_goals_90 === null || result?.home_goals_90 === undefined ? '' : String(result.home_goals_90), away: result?.away_goals_90 === null || result?.away_goals_90 === undefined ? '' : String(result.away_goals_90), coverage: existing ? String(existing.summary.data_coverage) : '', finalized: localReviewTime(result?.finalized_at), version: '', notes: '', confirmedBy: '', confirmationNote: '', ...saved.fields };
        const players: ReviewFactsDraft['players'] = {};
        for (const lineup of lineups)
            for (const player of lineup.players) {
                const recorded = existing?.player_reviews.find(row => row.player_id === player.player_id && row.team_id === lineup.team_id)?.metrics;
                const value = (key: string) => typeof recorded?.[key] === 'number' && Number.isFinite(recorded[key]) ? String(recorded[key]) : '';
                players[player.player_id] = { minutes: player.actual_minutes === null || player.actual_minutes === undefined ? '' : String(player.actual_minutes), rating: value('provider_rating'), ...Object.fromEntries(reviewMetrics.map(([key]) => [key, value(key)])), ...saved.players[player.player_id] };
            }
        return { fields, players };
    }
    validate(match: ReviewableMatch | null, lineups: LineupRecord[], detail: MatchReviewDetail | null = null): {
        draft: MatchReviewDraft | null;
        error: string | null;
    } {
        try {
            return { draft: this.parse(match, lineups, detail), error: null };
        }
        catch (error) {
            return { draft: null, error: error instanceof Error ? error.message : String(error) };
        }
    }
    parse(match: ReviewableMatch | null, lineups: LineupRecord[], detail: MatchReviewDetail | null = null): MatchReviewDraft {
        if (!match)
            throw Error('请先载入当前比赛');
        const teams = [match.match_record.home_team_id, match.match_record.away_team_id];
        if (teams.some(team => !lineups.some(row => row.team_id === team && row.lineup_type === 'actual' && row.status === 'active' && row.match_id === match.match_record.id)))
            throw Error('主客队实际阵容尚未完整；预计或确认阵容不能代替实际阵容');
        const values = this.values(match, lineups, detail);
        const number = (raw: string | undefined, label: string, max = Infinity, integer = false) => {
            if (raw === undefined || !raw.trim())
                throw Error(label + '尚未录入，空白不作为0');
            const n = Number(raw);
            if (!Number.isFinite(n) || n < 0 || n > max || (integer && !Number.isSafeInteger(n)))
                throw Error(label + '超出有效范围');
            return n;
        };
        const home = number(values.fields.home, '主队90分钟比分', Infinity, true), away = number(values.fields.away, '客队90分钟比分', Infinity, true), coverage = number(values.fields.coverage, '数据覆盖率', 1);
        const time = new Date(values.fields.finalized);
        if (!values.fields.finalized || !Number.isFinite(time.getTime()))
            throw Error('请填写真实赛果确认时间');
        const ids = new Set<string>();
        const observations = lineups.filter(row => row.lineup_type === 'actual' && teams.includes(row.team_id)).flatMap(lineup => lineup.players.map(player => {
            if (ids.has(player.player_id))
                throw Error('实际阵容中有重复球员，请先核对');
            ids.add(player.player_id);
            const raw = values.players[player.player_id], label = player.player_name;
            const minutes = number(raw.minutes, label + '分钟', 150);
            const rating = raw.rating?.trim() ? number(raw.rating, label + '评分', 10) : null;
            if (minutes > 0 && rating === null)
                throw Error(label + '评分尚未录入');
            const metricValues = Object.fromEntries(reviewMetrics.map(([key, name, step]) => [key, number(raw[key], label + ' · ' + name, Infinity, step === 1)]));
            const metrics = { ...metricValues, provider_rating: rating, extra: { entry_mode: 'desktop_review' } } as unknown as PlayerPerformanceMetrics;
            if (metrics.shots_on_target > metrics.shots || metrics.duels_won > metrics.duels_total)
                throw Error(label + '射正/对抗成功数不能超过对应总数');
            return { player_id: player.player_id, team_id: lineup.team_id, position_code: player.position_code, role_code: player.role_code, started: player.is_starter, minutes_played: minutes, performance_score: null, input_confidence: minutes > 0 ? .9 : .6, metrics, source_document_id: null };
        }));
        if (!observations.length)
            throw Error('实际阵容没有可记录的球员');
        const existing = detail?.summary.match_id === match.match_record.id ? detail : null;
        return { match_id: match.match_record.id, review_version: values.fields.version.trim() || null, data_coverage: coverage, source_run_id: null, result: { match_id: match.match_record.id, home_goals_90: home, away_goals_90: away, home_goals_extra_time: match.result?.home_goals_extra_time ?? null, away_goals_extra_time: match.result?.away_goals_extra_time ?? null, home_penalties: match.result?.home_penalties ?? null, away_penalties: match.result?.away_penalties ?? null, finalized_at: time.toISOString(), source_document_id: match.result?.source_document_id ?? null, metadata: { entry_mode: 'desktop_review' } }, substitutions: existing?.substitutions ?? [], events: existing?.events ?? [], player_observations: observations, notes: values.fields.notes.trim() || null };
    }
}
