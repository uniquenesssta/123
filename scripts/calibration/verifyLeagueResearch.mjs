import assert from 'node:assert/strict';
import { leagueProfile } from './leagueProfile.mjs';
import { captureBody, outcomeBody, validateCapture } from './dataset.mjs';
import { seal } from './artifacts.mjs';
import { fitInitial } from './fit.mjs';

export function verifyLeagueResearch() {
    const reference = {
        schema: 'football.initial-calibration.league-reference.v1',
        scope: 'qa',
        profile_id: 'synthetic-league-test',
        competition_name: 'QA League',
        reference_season: '2023',
        target_season: '2025',
        target_from: '2025-01-01T00:00:00Z',
        target_until: '2026-01-01T00:00:00Z',
        observed_at: '2024-02-01T00:00:00Z',
        source_evidence: [{ url: 'https://example.invalid/a', sha256: 'a'.repeat(64) }, { url: 'https://example.invalid/b', sha256: 'b'.repeat(64) }],
        matches: [
            { id: 'old-1', date: '2023-01-01', home_goals_90: 2, away_goals_90: 1, cross_checked: true },
            { id: 'old-2', date: '2023-01-02', home_goals_90: 1, away_goals_90: 1, cross_checked: true },
        ],
    };
    const frozen = '2024-02-02T00:00:00Z';
    const profile = leagueProfile(reference, frozen);
    assert.equal(profile.base_goal_rate, 5 / 4);
    assert.throws(() => leagueProfile({ ...reference, matches: [reference.matches[0], reference.matches[0]] }, frozen), /重复/);
    assert.throws(() => leagueProfile({ ...reference, observed_at: '2025-12-01T00:00:00Z' }, frozen), /时间/);
    assert.throws(() => leagueProfile({ ...reference, matches: [{ ...reference.matches[0], date: '2025-03-01' }] }, frozen), /早于/);

    const names = ['attack_alpha', 'defence_alpha', 'environment_alpha', 'venue_alpha'];
    const baseline = [0, 0, 0.37, 0];
    const lambda = Object.fromEntries(names.map((name, i) => [name, baseline[i]]));
    const makeRow = (i, override = {}) => {
        const date = '2025-01-' + String(i + 1).padStart(2, '0');
        const kickoff = date + 'T12:00:00Z';
        const frozenAt = date + 'T11:00:00Z';
        const match = {
            id: 'qa-' + i, external_key: 'qa-' + i, competition_id: 'league-id',
            competition_name: 'QA League', season_id: 'season-id', status: 'scheduled', kickoff_time: kickoff,
        };
        const capture = seal(captureBody({
            schema: 'football.initial-calibration.producer.v1', producer: 'unit-qa', database_is_test: true,
            captured_at: frozenAt, completed_at: frozenAt,
            prepared: {
                competition_kind: override.kind ?? 'league', match_record: match,
                snapshot_type: 'T-1h', data_quality: {},
                match_input: {
                    database_match_id: match.id, match_id: match.external_key,
                    snapshot: { data_cutoff_time: override.cutoff ?? frozenAt, frozen_at: frozenAt },
                    data_quality: { run_mode: 'pre_match' },
                },
            },
        }, { lambda, profile }, 'qa', { matchId: match.id, horizon: 'T-1h' }, Date.parse(frozenAt)));
        const outcome = seal(outcomeBody(capture, {
            match_id: match.id, scope: 'qa', status: 'finished', home_goals_90: 2, away_goals_90: 1,
            confirmed_at: date + 'T15:00:00Z', source_url: 'qa://unit/league',
        }, Date.parse(date + 'T15:00:00Z')));
        return { capture: capture.body, capture_sha256: capture.sha256, outcome: outcome.body, outcome_sha256: outcome.sha256 };
    };
    assert.throws(() => makeRow(0, { kind: 'knockout_single_leg' }), /不匹配/);
    assert.throws(() => makeRow(0, { cutoff: '2024-02-01T12:00:00Z' }), /截止时间/);
    const rows = Array.from({ length: 10 }, (_, i) => makeRow(i));
    assert.throws(() => validateCapture(seal({ ...rows[0].capture, competition_kind: 'friendly' })), /不匹配/);
    const settings = {
        configuration_scope: 'qa', target_competition_id: 'league-id', target_season_id: 'season-id',
        minimum_training_matches: 5, holdout_matches: 3, lower_bounds: [-1, -1, -1], upper_bounds: [1, 1, 1],
        initial_steps: [0.2, 0.2, 0.2], tolerance: 1e-6, max_evaluations: 400,
    };
    const describe = samples => ({
        model_id: 'p4', provider: 'unit-league',
        dimensions: names.map((name, i) => ({ name, path: ['lambda', name], initial: baseline[i] })),
        feature_rows: Array.from({ length: samples.length * 2 }, (_, i) => [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 0, 1]][i % 3]),
    });
    const oracle = request => {
        if (request.action === 'describe') return describe(request.samples);
        const [attack, defence, environment, venue] = request.coefficients;
        assert.equal(environment, baseline[2]);
        const loss = 1 + (attack - 0.2) ** 2 + (defence + 0.2) ** 2 + (venue - 0.4) ** 2;
        return { loss, log_loss: loss, brier: loss / 2 };
    };
    const result = fitInitial(rows, settings, oracle);
    assert.equal(result.status, 'QA_ONLY');
    assert.equal(result.rank, 3);
    assert.deepEqual(result.parameter_patch.map(entry => entry.path[1]), ['attack_alpha', 'defence_alpha', 'venue_alpha']);
    assert.deepEqual(result.league_research.fixed_parameters.map(entry => [entry.path[1], entry.value]), [['environment_alpha', 0.37]]);
    assert.equal(result.alpha_calibrated, false);
    assert.equal(result.direct_activation_allowed, false);
    assert.throws(() => fitInitial(rows, { ...settings, target_season_id: 'wrong-season' }, oracle), /绑定/);
    const lateTrainingResult = structuredClone(rows);
    lateTrainingResult[7].capture.input.snapshot.data_cutoff_time = '2025-01-07T14:00:00Z';
    assert.throws(() => fitInitial(lateTrainingResult, settings, oracle), /截止前尚不可知/);
    assert.throws(() => fitInitial(rows, settings, request => {
        if (request.action !== 'describe' || request.samples.length !== 3) return oracle(request);
        const result = describe(request.samples);
        result.feature_rows[0][2] = 0.1;
        return result;
    }), /环境项必须为零/);
    assert.throws(() => fitInitial(rows, settings, request => {
        if (request.action !== 'describe') return oracle(request);
        const result = describe(request.samples);
        result.feature_rows.forEach(row => { row[3] = row[0]; });
        return result;
    }), /不可识别/);
    const degraded = fitInitial(rows, { ...settings, tolerance: 0.01 }, request => {
        const result = oracle(request);
        if (request.action === 'evaluate' && request.samples.length === 3)
            result.brier = 0.5 + Math.abs(request.coefficients[0]) * 0.005;
        return result;
    });
    assert.equal(degraded.status, 'VALIDATION_FAILED');
    console.log('League research: source, time, scope, fixed coefficient, rank and holdout checks passed');
}
