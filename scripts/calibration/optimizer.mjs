export function matrixRank(matrix, dimension) {
    if (!matrix.length || matrix.some(r => r.length !== dimension || r.some(v => !Number.isFinite(v))))
        throw Error('提供器特征矩阵无效');
    const a = matrix.map(r => [...r]);
    let rank = 0;
    const scale = Math.max(...a.flat().map(Math.abs));
    if (!scale)
        return 0;
    for (let col = 0; col < dimension && rank < a.length; col++) {
        let pivot = rank;
        for (let i = rank + 1; i < a.length; i++)
            if (Math.abs(a[i][col]) > Math.abs(a[pivot][col]))
                pivot = i;
        if (Math.abs(a[pivot][col]) <= scale * 1e-10)
            continue;
        [a[rank], a[pivot]] = [a[pivot], a[rank]];
        const divisor = a[rank][col];
        for (let j = col; j < dimension; j++)
            a[rank][j] /= divisor;
        for (let i = rank + 1; i < a.length; i++) {
            const factor = a[i][col];
            for (let j = col; j < dimension; j++)
                a[i][j] -= factor * a[rank][j];
        }
        rank++;
    }
    return rank;
}
export function boundedSearch(initial, settings, evaluate) {
    const dimension = initial.length, lo = settings.lower_bounds, hi = settings.upper_bounds;
    let step = settings.initial_steps;
    if (!dimension || ![lo, hi, step].every(a => Array.isArray(a) && a.length === dimension) || initial.some((v, i) => ![v, lo[i], hi[i], step[i]].every(Number.isFinite) || lo[i] >= hi[i] || v < lo[i] || v > hi[i] || step[i] <= 0 || step[i] > hi[i] - lo[i]))
        throw Error('拟合边界、初值或步长无效');
    if (!Number.isFinite(settings.tolerance) || settings.tolerance <= 0 || settings.tolerance >= Math.min(...step) || !Number.isSafeInteger(settings.max_evaluations) || settings.max_evaluations < dimension * 2 + 1 || settings.max_evaluations > 10000)
        throw Error('拟合容差或评估预算无效');
    let point = [...initial], evaluations = 0;
    step = [...step];
    const cache = new Map();
    const score = v => {
        const key = JSON.stringify(v);
        if (cache.has(key))
            return cache.get(key);
        const value = evaluate(v);
        if (!Number.isFinite(value) || value < 0)
            throw Error('提供器返回无效目标损失');
        evaluations++;
        cache.set(key, value);
        return value;
    };
    let loss = score(point);
    const initialLoss = loss;
    while (evaluations < settings.max_evaluations && Math.max(...step) > settings.tolerance) {
        let improved = false;
        for (let i = 0; i < dimension && evaluations < settings.max_evaluations; i++) {
            let best = point, bestLoss = loss;
            for (const sign of [1, -1]) {
                if (evaluations >= settings.max_evaluations)
                    break;
                const candidate = [...point];
                candidate[i] = Math.max(lo[i], Math.min(hi[i], point[i] + sign * step[i]));
                if (candidate[i] === point[i])
                    continue;
                const value = score(candidate);
                if (value < bestLoss - settings.tolerance) {
                    best = candidate;
                    bestLoss = value;
                }
            }
            if (best !== point) {
                point = best;
                loss = bestLoss;
                improved = true;
            }
        }
        if (!improved)
            step = step.map(s => s / 2);
    }
    return {
        coefficients: point, loss, initial_loss: initialLoss, evaluations, converged: Math.max(...step) <= settings.tolerance
    };
}
