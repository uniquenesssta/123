import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { digest, instant, openArtifact } from './artifacts.mjs';
import { leagueResearchPolicy } from './leagueProfile.mjs';
export function captureBody(produced, parameters, scope, expected, clock = Date.now()) {
    if (!['qa', 'real'].includes(scope))
        throw Error('必须明确声明qa或real来源');
    const p = produced.prepared, m = p?.match_record, input = p?.match_input;
    if (produced.schema !== 'football.initial-calibration.producer.v1' || !m || !input || m.id !== expected.matchId || p.snapshot_type !== expected.horizon)
        throw Error('采样生产器返回的比赛或窗口不匹配');
    const at = instant(produced.captured_at, '采样时间'), end = instant(produced.completed_at, '采样结束时间'), kickoff = instant(m.kickoff_time, '开球时间');
    if (at > clock + 1000 || end > clock + 1000 || end < at || clock - at > 300000 || at >= kickoff || end >= kickoff)
        throw Error('只能在真实赛前采样，禁止回填历史快照');
    if (m.status !== 'scheduled')
        throw Error('仅允许尚未开赛的比赛采样');
    if (produced.database_is_test && scope !== 'qa')
        throw Error('测试库采样只能标记为qa');
    const cutoff = instant(input.snapshot?.data_cutoff_time, '特征截止时间'), frozen = instant(input.snapshot?.frozen_at, '特征冻结时间');
    if (cutoff > at || frozen !== at || cutoff >= kickoff || input.database_match_id !== m.id || input.match_id !== m.external_key)
        throw Error('特征冻结的时间或比赛身份不一致');
    if (input.data_quality?.run_mode !== 'pre_match')
        throw Error('历史重放不能标成实时赛前采样');
    if (!m.competition_id)
        throw Error('初始校准必须关联具体赛事');
    if (!['T-N', 'T-24h', 'T-6h', 'T-1h'].includes(p.snapshot_type))
        throw Error('采样窗口无效');
    const research = leagueResearchPolicy(parameters.profile, m, p.competition_kind, scope, cutoff);
    return {
        ...(research ? { competition_kind: p.competition_kind } : {}),
        schema: 'football.initial-calibration.capture.v1', capture_id: crypto.randomUUID(), scope, capture_kind: 'live_pre_match', captured_at: produced.captured_at, completed_at: produced.completed_at, match: m, horizon: p.snapshot_type, model_id: 'p4', input, data_quality: p.data_quality, parameters, baseline_sha256: digest(parameters), producer: produced.producer, database_is_test: !!produced.database_is_test
    };
}
export function validateCapture(row) {
    const c = row.body;
    if (c?.schema !== 'football.initial-calibration.capture.v1' || !['qa', 'real'].includes(c.scope) || c.capture_kind !== 'live_pre_match' || !(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(c.capture_id)) || !c.match?.id || !c.match.competition_id)
        throw Error('采样制品结构或范围不完整');
    if (c.model_id !== 'p4' || !['T-N', 'T-24h', 'T-6h', 'T-1h'].includes(c.horizon) || c.input?.data_quality?.run_mode !== 'pre_match')
        throw Error('采样模型、窗口或实时来源不符');
    if (c.database_is_test && c.scope !== 'qa')
        throw Error('测试库采样不能用于real拟合');
    if (digest(c.parameters) !== c.baseline_sha256 || c.input?.database_match_id !== c.match.id || c.input?.match_id !== c.match.external_key)
        throw Error('采样参数摘要或比赛身份不一致');
    const at = instant(c.captured_at, '采样时间'), end = instant(c.completed_at, '采样结束时间'), kickoff = instant(c.match.kickoff_time, '开球时间');
    if (end < at || end >= kickoff || instant(c.input.snapshot?.data_cutoff_time, '特征截止时间') > at || instant(c.input.snapshot?.frozen_at, '冻结时间') !== at)
        throw Error('采样制品违反赛前时间因果关系');
    leagueResearchPolicy(c.parameters.profile, c.match, c.competition_kind, c.scope,
        instant(c.input.snapshot?.data_cutoff_time, '特征截止时间'));
    return c;
}
export function outcomeBody(capture, result, clock = Date.now()) {
    const c = validateCapture(capture);
    if (result.match_id !== c.match.id || result.status !== 'finished' || result.scope !== c.scope)
        throw Error('赛果必须对应同一比赛、来源范围及完赛状态');
    for (const key of ['home_goals_90', 'away_goals_90'])
        if (!Number.isSafeInteger(result[key]) || result[key] < 0 || result[key] > 100)
            throw Error('90分钟赛果必须是有效非负整数');
    const known = instant(result.confirmed_at, '赛果核验时间');
    if (known < instant(c.match.kickoff_time, '开球时间') || known > clock + 1000)
        throw Error('赛果核验时间早于开球或位于未来');
    let url;
    try {
        url = new URL(result.source_url);
    }
    catch {
        throw Error('赛果必须保留明确来源');
    }
    if (!['https:', 'qa:'].includes(url.protocol) || c.scope === 'real' && url.protocol !== 'https:')
        throw Error('真实赛果必须保留HTTPS来源');
    if (url.username || url.password || [...url.searchParams.keys()].some(k => ['api_key', 'access_token', 'token', 'key'].includes(k.toLowerCase())))
        throw Error('赛果来源不得包含凭据');
    return {
        schema: 'football.initial-calibration.outcome.v1', capture_id: c.capture_id, capture_sha256: capture.sha256, match_id: c.match.id, scope: c.scope, status: 'finished', home_goals_90: result.home_goals_90, away_goals_90: result.away_goals_90, confirmed_at: result.confirmed_at, source_url: result.source_url, source_match_id: result.source_match_id ?? null
    };
}
export function readCohort(directory) {
    const files = fs.readdirSync(directory).filter(n => n.endsWith('.capture.json')).sort(), rows = [];
    if (!files.length)
        throw Error('没有赛前采样制品；历史赛果文件不能替代采样');
    const identities = new Set();
    let partition = null;
    for (const file of files) {
        const capture = openArtifact(path.join(directory, file)), c = validateCapture(capture);
        if (file !== c.capture_id + '.capture.json')
            throw Error('采样文件名与制品身份不匹配');
        const outcome = openArtifact(path.join(directory, c.capture_id + '.outcome.json')), l = outcome.body;
        if (l?.schema !== 'football.initial-calibration.outcome.v1' || l.capture_sha256 !== capture.sha256 || l.capture_id !== c.capture_id)
            throw Error('赛果与冻结采样摘要不匹配');
        const checked = outcomeBody(capture, l);
        if (digest(checked) !== digest(l))
            throw Error('赛果包含不符合契约的内容');
        const key = JSON.stringify([c.match.competition_id, c.horizon, c.model_id, c.baseline_sha256, c.scope]);
        if (partition !== null && key !== partition)
            throw Error('禁止混合赛事、窗口、参数基线或qa/real来源');
        partition = key;
        if (identities.has(c.match.id))
            throw Error('同一比赛不能重复进入训练或留出集');
        identities.add(c.match.id);
        rows.push({
            capture: c, capture_sha256: capture.sha256, outcome: l, outcome_sha256: outcome.sha256
        });
    }
    return rows.sort((a, b) => instant(a.capture.match.kickoff_time, '开球时间') - instant(b.capture.match.kickoff_time, '开球时间') || a.capture.match.id.localeCompare(b.capture.match.id));
}
export function chronologicalSplit(rows, settings) {
    if (!Number.isSafeInteger(settings.minimum_training_matches) || settings.minimum_training_matches < 1 || !Number.isSafeInteger(settings.holdout_matches) || settings.holdout_matches < 1)
        throw Error('必须明确训练和留出样本下限');
    let split = rows.length - settings.holdout_matches;
    if (split < settings.minimum_training_matches)
        throw Error('训练或留出样本不足');
    while (split > 0 && instant(rows[split - 1].capture.match.kickoff_time, '开球时间') === instant(rows[split].capture.match.kickoff_time, '开球时间'))
        split--;
    if (split < settings.minimum_training_matches)
        throw Error('相同开球时点不能跨训练与留出分割');
    const train = rows.slice(0, split), holdout = rows.slice(split), firstCapture = Math.min(...holdout.map(r => instant(r.capture.captured_at, '留出采样时间')));
    if (train.some(r => instant(r.outcome.confirmed_at, '训练赛果核验时间') >= firstCapture))
        throw Error('留出采样前尚不可知的训练赛果会造成时间泄漏');
    return { train, holdout };
}
