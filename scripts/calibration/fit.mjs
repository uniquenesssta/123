import crypto from 'node:crypto';
import { digest } from './artifacts.mjs';
import { chronologicalSplit } from './dataset.mjs';
import { matrixRank, boundedSearch } from './optimizer.mjs';
const metrics = row => {
    if (!row || !['loss', 'log_loss', 'brier'].every(k => Number.isFinite(row[k]) && row[k] >= 0))
        throw Error('提供器评估指标不完整');
    return row;
};
export function fitInitial(rows, settings, call) {
    const scope = rows[0]?.capture.scope;
    if (!scope || settings.configuration_scope !== scope)
        throw Error('拟合设置与样本来源qa/real范围不匹配');
    const { train, holdout } = chronologicalSplit(rows, settings), description = call({ action: 'describe', samples: train }), dimensions = description.dimensions;
    if (!Array.isArray(dimensions) || !dimensions.length || new Set(dimensions.map(d => d.name)).size !== dimensions.length || dimensions.some(d => typeof d.name !== 'string' || !Array.isArray(d.path) || !d.path.length || d.path.some(p => typeof p !== 'string' || ['__proto__', 'constructor', 'prototype', 'profile', 'calibration_status', 'automatic_lambda_eligible'].includes(p))))
        throw Error('提供器参数维度不完整或包含启用状态');
    if (description.model_id !== rows[0].capture.model_id || train.length < Math.max(settings.minimum_training_matches, dimensions.length + 1))
        throw Error('模型身份不匹配或训练样本不足以识别参数');
    const rank = matrixRank(description.feature_rows, dimensions.length);
    if (rank !== dimensions.length)
        throw Error('特征不可识别：秩 ' + rank + ' / ' + dimensions.length + '；禁止用任意α补齐');
    const initial = dimensions.map(d => d.initial), baselineTrain = metrics(call({
        action: 'evaluate', samples: train, coefficients: initial
    })), baselineHoldout = metrics(call({
        action: 'evaluate', samples: holdout, coefficients: initial
    }));
    const solved = boundedSearch(initial, settings, c => metrics(call({
        action: 'evaluate', samples: train, coefficients: c
    })).loss);
    const candidateTrain = metrics(call({
        action: 'evaluate', samples: train, coefficients: solved.coefficients
    })), candidateHoldout = metrics(call({
        action: 'evaluate', samples: holdout, coefficients: solved.coefficients
    }));
    const validationPassed = solved.converged && ['loss', 'log_loss', 'brier'].every(k => candidateHoldout[k] <= baselineHoldout[k] + settings.tolerance);
    return {
        schema: 'football.initial-calibration.fit.v1', candidate_id: crypto.randomUUID(), status: !solved.converged ? 'NOT_CONVERGED' : !validationPassed ? 'VALIDATION_FAILED' : scope === 'qa' ? 'QA_ONLY' : 'REQUIRES_MANUAL_REVIEW', scope, alpha_calibrated: false, direct_activation_allowed: false, baseline_sha256: rows[0].capture.baseline_sha256, partition: {
            competition_id: rows[0].capture.match.competition_id, horizon: rows[0].capture.horizon, model_id: description.model_id
        }, provider: description.provider, provider_executable_sha256: description.executable_sha256 ?? null, dimension_names: dimensions.map(d => d.name), rank, training_matches: train.length, holdout_matches: holdout.length, training_capture_hashes: train.map(r => r.capture_sha256), training_outcome_hashes: train.map(r => r.outcome_sha256), holdout_capture_hashes: holdout.map(r => r.capture_sha256), holdout_outcome_hashes: holdout.map(r => r.outcome_sha256), settings, settings_sha256: digest(settings), optimization: solved, baseline_train: baselineTrain, candidate_train: candidateTrain, baseline_holdout: baselineHoldout, candidate_holdout: candidateHoldout, validation_passed: validationPassed, parameter_patch: dimensions.map((d, i) => ({
            path: d.path, baseline: d.initial, candidate: solved.coefficients[i]
        }))
    };
}
