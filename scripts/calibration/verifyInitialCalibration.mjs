import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { seal, writeArtifact, openArtifact, digest } from './artifacts.mjs';
import { captureBody, validateCapture, outcomeBody, readCohort, chronologicalSplit } from './dataset.mjs';
import { boundedSearch, matrixRank } from './optimizer.mjs';
import { fitInitial } from './fit.mjs';
export function verifyInitialCalibration() {
    let count = 0;
    const test = (name, run) => { run(); count++; };
    const parameters = {
        lambda: { x: 0, y: 0 }, profile: { automatic_lambda_eligible: false, calibration_status: 'WAITING' }
    };
    const make = i => {
        const at = new Date(Date.UTC(2025, 0, i + 1, 10)).toISOString(), ko = new Date(Date.UTC(2025, 0, i + 1, 12)).toISOString(), id = 'qa-match-' + i;
        const produced = {
            schema: 'football.initial-calibration.producer.v1', captured_at: at, completed_at: at, database_is_test: true, producer: 'explicit-unit-boundary', prepared: {
                match_record: {
                    id, external_key: id, competition_id: 'qa-league', kickoff_time: ko, status: 'scheduled'
                }, snapshot_type: 'T-N', data_quality: { scope: 'explicit-unit-fixture' }, match_input: {
                    database_match_id: id, match_id: id, snapshot: { data_cutoff_time: at, frozen_at: at }, data_quality: { run_mode: 'pre_match' }
                }
            }
        };
        const c = seal(captureBody(produced, parameters, 'qa', { matchId: id, horizon: 'T-N' }, Date.parse(at)));
        const result = {
            match_id: id, status: 'finished', scope: 'qa', home_goals_90: 1, away_goals_90: 0, confirmed_at: new Date(Date.parse(ko) + 7200000).toISOString(), source_url: 'qa://declared-unit-input/' + i
        };
        const l = seal(outcomeBody(c, result, Date.parse(result.confirmed_at)));
        return {
            capture: c.body, capture_sha256: c.sha256, outcome: l.body, outcome_sha256: l.sha256, produced, result, sealed: c
        };
    };
    const rows = Array.from({ length: 10 }, (_, i) => make(i)), settings = {
        configuration_scope: 'qa', minimum_training_matches: 5, holdout_matches: 3, lower_bounds: [-1, -1], upper_bounds: [1, 1], initial_steps: [0.2, 0.2], tolerance: 1e-6, max_evaluations: 300
    };
    const descriptor = {
        model_id: 'p4', provider: 'explicit-quadratic-unit-oracle', dimensions: [{
                name: 'x', path: ['lambda', 'x'], initial: 0
            }, {
                name: 'y', path: ['lambda', 'y'], initial: 0
            }], feature_rows: [[1, 0], [0, 1], [1, 1]]
    };
    const call = r => r.action === 'describe' ? structuredClone(descriptor) : {
        loss: 1 + (r.coefficients[0] - 0.2) ** 2 + (r.coefficients[1] + 0.15) ** 2, log_loss: 0.5 + (r.coefficients[0] - 0.2) ** 2, brier: 0.1 + (r.coefficients[1] + 0.15) ** 2
    };
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'football-initial-calibration-'));
    test('canonical artifact identity ignores object insertion order', () => assert.equal(digest({ b: 2, a: 1 }), digest({ a: 1, b: 2 })));
    test('exclusive artifacts survive duplicate writes unchanged', () => { const file = path.join(directory, 'immutable.json'); writeArtifact(file, { x: 1 }); const before = fs.readFileSync(file); assert.throws(() => writeArtifact(file, { x: 2 })); assert.deepEqual(fs.readFileSync(file), before); });
    test('tampered captures are rejected', () => { const file = path.join(directory, 'tampered.json'); fs.writeFileSync(file, JSON.stringify({ body: { x: 2 }, sha256: digest({ x: 1 }) })); assert.throws(() => openArtifact(file), /摘要/); });
    test('nonfinite artifact fields fail before writing', () => assert.throws(() => writeArtifact(path.join(directory, 'invalid.json'), { x: NaN })));
    test('credential fields cannot be frozen into a calibration artifact', () => { const file = path.join(directory, 'credentials.json'); assert.throws(() => writeArtifact(file, { parameters: { api_key: 'explicit-fake-unit-key' } }), /凭据/); assert.equal(fs.existsSync(file), false); });
    for (const [name, mutate] of [
        ['late completed capture', p => { p.completed_at = p.prepared.match_record.kickoff_time; }],
        ['post-kickoff capture', p => { p.captured_at = p.prepared.match_record.kickoff_time; }],
        ['wrong match input', p => { p.prepared.match_input.database_match_id = 'other'; }],
        ['future source cutoff', p => { p.prepared.match_input.snapshot.data_cutoff_time = '2030-01-01T00:00:00Z'; }],
        ['changed freeze clock', p => { p.prepared.match_input.snapshot.frozen_at = '2024-01-01T00:00:00Z'; }],
        ['historical replay disguise', p => { p.prepared.match_input.data_quality.run_mode = 'historical_replay'; }],
        ['finished match capture', p => { p.prepared.match_record.status = 'finished'; }],
    ])
        test(name, () => { const f = make(0); mutate(f.produced); assert.throws(() => captureBody(f.produced, parameters, 'qa', { matchId: 'qa-match-0', horizon: 'T-N' }, Date.parse(f.capture.captured_at))); });
    test('QA producer cannot claim real data', () => { const f = make(0); assert.throws(() => captureBody(f.produced, parameters, 'real', { matchId: 'qa-match-0', horizon: 'T-N' }, Date.parse(f.capture.captured_at)), /测试库/); });
    for (const [name, mutate] of [
        ['wrong outcome identity', r => { r.match_id = 'other'; }], ['negative score', r => { r.home_goals_90 = -1; }],
        ['missing score', r => { delete r.away_goals_90; }], ['unknown match completion', r => { r.status = 'live'; }],
        ['pre-match outcome', r => { r.confirmed_at = '2024-01-01T00:00:00Z'; }], ['missing source', r => { r.source_url = ''; }],
        ['mixed evidence scope', r => { r.scope = 'real'; }],
        ['case-insensitive credential source', r => { r.source_url = 'https://example.invalid/result?API_KEY=explicit-fake-key'; }],
        ['credential-bearing source', r => { r.source_url = 'https://example.invalid/result?api_key=explicit-fake-key'; }],
    ])
        test(name, () => { const f = make(0); mutate(f.result); assert.throws(() => outcomeBody(f.sealed, f.result, Date.parse('2026-01-01T00:00:00Z'))); });
    for (const row of rows) {
        writeArtifact(path.join(directory, row.capture.capture_id + '.capture.json'), row.capture);
        writeArtifact(path.join(directory, row.capture.capture_id + '.outcome.json'), row.outcome);
    }
    test('saved capture/outcome cohort round trips production readers', () => assert.equal(readCohort(directory).length, 10));
    test('source corpus JSON is not a frozen feature cohort', () => { const empty = fs.mkdtempSync(path.join(os.tmpdir(), 'football-initial-calibration-empty-')); fs.writeFileSync(path.join(empty, 'matches.json'), '[]'); assert.throws(() => readCohort(empty), /没有赛前/); console.log('[retained test artifact] ' + empty); });
    test('duplicate match cannot contribute twice', () => { const c = { ...rows[0].capture, capture_id: '00000000-0000-4000-8000-000000000001' }; writeArtifact(path.join(directory, c.capture_id + '.capture.json'), c); writeArtifact(path.join(directory, c.capture_id + '.outcome.json'), outcomeBody(seal(c), rows[0].result)); assert.throws(() => readCohort(directory), /重复/); });
    test('capture identity cannot redirect output outside cohort', () => { const c = { ...rows[0].capture, capture_id: '../redirect' }; assert.throws(() => validateCapture(seal(c)), /结构/); });
    test('chronological split has explicit training and holdout counts', () => { const split = chronologicalSplit(rows, settings); assert.equal(split.train.length, 7); assert.equal(split.holdout.length, 3); });
    test('equal kickoff instants with different timezone spellings remain together', () => { const copy = structuredClone(rows); copy[6].capture.match.kickoff_time = '2025-01-08T13:00:00+01:00'; const split = chronologicalSplit(copy, settings); assert.equal(split.train.length, 6); assert.equal(split.holdout.length, 4); });
    test('training labels unknown at holdout capture are rejected', () => { const copy = structuredClone(rows); copy[0].outcome.confirmed_at = copy[9].capture.captured_at; assert.throws(() => chronologicalSplit(copy, settings), /泄漏/); });
    test('insufficient training data is not silently dropped', () => assert.throws(() => chronologicalSplit(rows.slice(0, 4), settings), /不足/));
    test('rank detects a missing coefficient dimension', () => assert.equal(matrixRank([[1, 0], [2, 0]], 2), 1));
    test('rank is invariant to uniform feature scaling', () => assert.equal(matrixRank([[1e-5, 0], [0, 1e-5]], 2), 2));
    test('bounded coordinate search improves a real coupled objective', () => { const result = boundedSearch([0, 0], settings, p => 1 + (p[0] - .2) ** 2 + (p[1] + .15) ** 2); assert.ok(result.converged); assert.ok(result.loss < result.initial_loss); assert.ok(Math.abs(result.coefficients[0] - .2) < .003); assert.ok(Math.abs(result.coefficients[1] + .15) < .003); });
    test('invalid optimization bounds cannot change coefficients', () => assert.throws(() => boundedSearch([0, 0], { ...settings, lower_bounds: [1, 1] }, () => 1)));
    test('invalid oracle losses abort fitting', () => assert.throws(() => boundedSearch([0, 0], settings, () => NaN), /损失/));
    test('production fitter creates QA-only patches with no activation', () => { const result = fitInitial(rows, settings, call); assert.equal(result.status, 'QA_ONLY'); assert.equal(result.alpha_calibrated, false); assert.equal(result.direct_activation_allowed, false); assert.equal(result.parameter_patch.length, 2); assert.ok(result.candidate_train.loss < result.baseline_train.loss); assert.equal(parameters.profile.automatic_lambda_eligible, false); });
    test('fitted result retains the exact outcome artifact identities', () => { const result = fitInitial(rows, settings, call); assert.deepEqual(result.training_outcome_hashes, rows.slice(0, 7).map(r => r.outcome_sha256)); assert.deepEqual(result.holdout_outcome_hashes, rows.slice(7).map(r => r.outcome_sha256)); });
    test('missing feature rank blocks rather than inventing alpha', () => assert.throws(() => fitInitial(rows, settings, r => r.action === 'describe' ? { ...descriptor, feature_rows: [[1, 0]] } : call(r)), /不可识别/));
    test('provider cannot patch Profile qualification', () => assert.throws(() => fitInitial(rows, settings, r => r.action === 'describe' ? {
        ...descriptor, dimensions: [{
                name: 'flag', path: ['profile', 'automatic_lambda_eligible'], initial: 0
            }]
    } : call(r)), /启用状态/));
    test('QA samples cannot use real fit settings', () => assert.throws(() => fitInitial(rows, { ...settings, configuration_scope: 'real' }, call), /范围/));
    test('holdout regression is reported as failed', () => { const result = fitInitial(rows, settings, r => { const value = call(r); return r.action === 'describe' || r.samples.length !== 3 ? value : { ...value, brier: 1 + Math.abs(r.coefficients[0]) }; }); assert.equal(result.status, 'VALIDATION_FAILED'); assert.equal(result.alpha_calibrated, false); });
    test('exhausted optimizer cannot claim convergence', () => { const result = boundedSearch([0, 0], { ...settings, max_evaluations: 5 }, p => 1 + (p[0] - .8) ** 2 + (p[1] - .8) ** 2); assert.equal(result.converged, false); });
    console.log('[retained test artifact] ' + directory);
    console.log('Initial calibration: ' + count + ' capture, causality, fitting and safety checks passed');
    return count;
}
