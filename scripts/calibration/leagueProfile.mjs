import { digest, instant, openArtifact, readJson } from './artifacts.mjs';

const REFERENCE_SCHEMA = 'football.initial-calibration.league-reference.v1';
const PROFILE_SCHEMA = 'football.initial-calibration.league-profile.v1';
const RESEARCH_STATUS = 'RESEARCH_BASELINE';
const sha256 = value => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
const text = value => typeof value === 'string' && value.trim() === value && value.length > 0;
const validGoals = value => Number.isSafeInteger(value) && value >= 0 && value <= 100;

export function leagueProfile(reference, now = new Date().toISOString()) {
    if (reference?.schema !== REFERENCE_SCHEMA || !['qa', 'real'].includes(reference.scope) ||
        !text(reference.profile_id) || !text(reference.competition_name) || !text(reference.reference_season) ||
        !text(reference.target_season) || reference.reference_season === reference.target_season)
        throw Error('联赛参考资料必须明确赛事、版本与两个赛季');
    const observed = instant(reference.observed_at, '参考资料观察时间');
    const frozen = instant(now, 'Profile 建立时间');
    const start = instant(reference.target_from, '目标时段开始');
    const end = instant(reference.target_until, '目标时段结束');
    if (observed > frozen || start >= end || frozen >= end)
        throw Error('联赛参考资料或目标时段的时间不一致');
    if (!Array.isArray(reference.source_evidence) || reference.source_evidence.length < 2 ||
        new Set(reference.source_evidence.map(source => source.url)).size !== reference.source_evidence.length ||
        new Set(reference.source_evidence.map(source => source.sha256)).size !== reference.source_evidence.length ||
        !reference.source_evidence.every(source => sha256(source.sha256) && validSource(source.url)))
        throw Error('联赛基率需要两份不同来源及其摘要');
    if (!Array.isArray(reference.matches) || reference.matches.length === 0)
        throw Error('联赛基率缺少已核对的参考赛果');

    const seen = new Set();
    let goals = 0;
    let latestDate = '';
    for (const match of reference.matches) {
        if (!text(match.id) || seen.has(match.id) || !validGoals(match.home_goals_90) || !validGoals(match.away_goals_90) ||
            match.cross_checked !== true || typeof match.date !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(match.date))
            throw Error('联赛参考赛果身份重复或未核对');
        const day = instant(match.date + 'T00:00:00Z', '参考赛果日期');
        if (new Date(day).toISOString().slice(0, 10) !== match.date ||
            day + 86400000 > Math.min(observed, start))
            throw Error('参考赛果必须早于观察日期和目标时段');
        seen.add(match.id);
        goals += match.home_goals_90 + match.away_goals_90;
        if (match.date > latestDate) latestDate = match.date;
    }
    if (goals <= 0) throw Error('联赛参考基率必须为正');
    return {
        profile_id: reference.profile_id,
        name: reference.competition_name + ' ' + reference.target_season + ' research baseline',
        competition_type: 'league',
        venue_type: 'home_away',
        normal_time_minutes: 90,
        base_goal_rate: goals / (2 * seen.size),
        goal_environment_score: 50,
        profile_confidence: 0,
        calibration_status: RESEARCH_STATUS,
        automatic_lambda_eligible: false,
        calibration_research: {
            schema: PROFILE_SCHEMA,
            scope: reference.scope,
            competition_name: reference.competition_name,
            reference_season: reference.reference_season,
            target_season: reference.target_season,
            target_from: reference.target_from,
            target_until: reference.target_until,
            reference_observed_at: reference.observed_at,
            frozen_at: now,
            latest_reference_date: latestDate,
            reference_sha256: digest(reference),
            source_evidence: reference.source_evidence,
            base_rate_method: 'mean_goals_per_team_match',
            reference_matches: seen.size,
            reference_goals: goals,
            environment_policy: 'fixed_neutral_reference',
        },
    };
}

function validSource(value) {
    try {
        const url = new URL(value);
        return url.protocol === 'https:' && !url.username && !url.password && !url.search && !url.hash;
    } catch {
        return false;
    }
}

export function loadCalibrationProfile(file) {
    const value = readJson(file);
    if (value?.body?.calibration_research) return openArtifact(file).body;
    if (value?.calibration_research || value?.calibration_status === RESEARCH_STATUS)
        throw Error('研究 Profile 必须从带摘要的制品读取');
    return value;
}

export function leagueResearchPolicy(profile, match, competitionKind, scope, cutoff) {
    const research = profile?.calibration_research;
    if (research === undefined && profile?.calibration_status !== RESEARCH_STATUS) return null;
    if (research?.schema !== PROFILE_SCHEMA || research.environment_policy !== 'fixed_neutral_reference' ||
        research.scope !== scope || profile.calibration_status !== RESEARCH_STATUS ||
        profile.automatic_lambda_eligible !== false || profile.competition_type !== 'league' ||
        profile.venue_type !== 'home_away' || profile.neutral_venue === true ||
        profile.normal_time_minutes !== 90 || profile.goal_environment_score !== 50 || profile.profile_confidence !== 0 ||
        !sha256(research.reference_sha256) || !Number.isSafeInteger(research.reference_matches) ||
        research.reference_matches < 1 || !Number.isSafeInteger(research.reference_goals) || research.reference_goals < 1 ||
        research.base_rate_method !== 'mean_goals_per_team_match' ||
        profile.base_goal_rate !== research.reference_goals / (2 * research.reference_matches))
        throw Error('联赛研究 Profile 基线或环境固定口径无效');
    if (competitionKind !== 'league' || !text(research.competition_name) ||
        match.competition_name !== research.competition_name || !match.season_id)
        throw Error('联赛研究 Profile 与比赛赛事或赛季不匹配');
    const kickoff = instant(match.kickoff_time, '开球时间');
    const from = instant(research.target_from, 'Profile 目标时段开始');
    const until = instant(research.target_until, 'Profile 目标时段结束');
    const observed = instant(research.reference_observed_at, '参考资料观察时间');
    const frozen = instant(research.frozen_at, 'Profile 建立时间');
    if (kickoff < from || kickoff >= until || observed > frozen || frozen > cutoff)
        throw Error('联赛研究 Profile 超出目标时段或晚于赛前特征截止时间');
    return research;
}
