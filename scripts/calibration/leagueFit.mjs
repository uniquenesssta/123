import { digest, instant } from './artifacts.mjs';
import { leagueResearchPolicy } from './leagueProfile.mjs';

const names = ['attack_alpha', 'defence_alpha', 'environment_alpha', 'venue_alpha'];
const fixed = 2;
const estimated = [0, 1, 3];

export function leagueFitPlan(train, holdout, settings, description, call) {
    const capture = train[0].capture;
    const profile = capture.parameters?.profile;
    if (!profile?.calibration_research && profile?.calibration_status !== 'RESEARCH_BASELINE') return null;
    const profileHash = digest(profile);
    for (const { capture: sample } of [...train, ...holdout]) {
        const cutoff = instant(sample.input.snapshot.data_cutoff_time, '赛前特征截止时间');
        leagueResearchPolicy(sample.parameters.profile, sample.match, sample.competition_kind, sample.scope, cutoff);
        if (digest(sample.parameters.profile) !== profileHash ||
            !settings.target_competition_id || sample.match.competition_id !== settings.target_competition_id ||
            !settings.target_season_id || sample.match.season_id !== settings.target_season_id)
            throw Error('联赛拟合必须绑定同一数据库赛事、赛季和研究 Profile');
    }
    const firstHoldoutCutoff = Math.min(...holdout.map(({ capture: sample }) =>
        instant(sample.input.snapshot.data_cutoff_time, '留出赛前特征截止时间')));
    if (train.some(({ outcome }) => instant(outcome.confirmed_at, '训练赛果核验时间') >= firstHoldoutCutoff))
        throw Error('留出特征截止前尚不可知的训练赛果会造成时间泄漏');

    const baseline = names.map(name => capture.parameters.lambda?.[name]);
    const checkDescription = (result, samples) => {
        if (result.model_id !== capture.model_id || result.provider !== description.provider ||
            result.executable_sha256 !== description.executable_sha256 ||
            !Array.isArray(result.dimensions) || result.dimensions.length !== names.length ||
            !result.dimensions.every((d, i) => d.name === names[i] && Number.isFinite(baseline[i]) &&
                d.initial === baseline[i] && d.path?.length === 2 && d.path[0] === 'lambda' && d.path[1] === names[i]))
            throw Error('提供器必须返回同一基线的四类 α');
        if (!Array.isArray(result.feature_rows) || result.feature_rows.length !== samples.length * 2 ||
            !result.feature_rows.every(row => Array.isArray(row) && row.length === names.length &&
                row.every(Number.isFinite) && row[fixed] === 0))
            throw Error('训练和留出样本的环境项必须为零，才能固定该系数');
    };
    checkDescription(description, train);
    checkDescription(call({ action: 'describe', samples: holdout.map(({ capture }) => ({ capture })) }), holdout);
    return {
        dimensions: estimated.map(i => description.dimensions[i]),
        featureRows: description.feature_rows.map(row => estimated.map(i => row[i])),
        expand(values) {
            const coefficients = [...baseline];
            estimated.forEach((index, i) => { coefficients[index] = values[i]; });
            return coefficients;
        },
        report: {
            profile_sha256: profileHash,
            reference_sha256: profile.calibration_research.reference_sha256,
            target_season_id: settings.target_season_id,
            fixed_parameters: [{
                path: ['lambda', names[fixed]],
                value: baseline[fixed],
                reason: 'zero_environment_feature_in_training_and_holdout',
            }],
        },
    };
}
